//! Privilege (sudo) handling for package installs.
//!
//! Package installs run in parallel behind a live progress display, with
//! their output captured. That combination used to break any install that
//! needed an administrator password: `sudo` writes its `Password:` prompt
//! straight to the terminal, the progress display redrew over it a few
//! milliseconds later, and several parallel installs could prompt at once
//! and fight over the keyboard — so the run appeared to hang.
//!
//! The fix is to authenticate **once, up front, in the foreground**:
//!
//! 1. Before any install runs, `install_for_profile` asks [`needs_auth`]
//!    whether the planned work will want sudo.
//! 2. If so, [`acquire`] pauses the progress display and runs `sudo -v`
//!    with the real terminal attached, so the prompt is shown normally.
//! 3. A [`SudoSession`] then refreshes the sudo timestamp in the background
//!    (`sudo -n -v`) until it is dropped, so long installs don't outlive it.
//! 4. System package managers are invoked as `sudo -n <cmd> ...` — `-n`
//!    means "never prompt", so if the credential is somehow gone the install
//!    fails immediately with a clear error instead of hanging invisibly.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// How often the background keep-alive refreshes the sudo timestamp.
/// sudo's default `timestamp_timeout` is 5 minutes (15 on some distros), so
/// once a minute leaves plenty of margin.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(60);

/// Whether heimdal is already running as root (no sudo needed).
pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        // SAFETY: geteuid has no preconditions and cannot fail.
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Package managers that must run as root to install anything.
///
/// Homebrew and mas run as the user (Homebrew refuses to run as root), so
/// they are never wrapped in sudo.
pub fn manager_needs_root(manager: &str) -> bool {
    matches!(manager, "apt" | "dnf" | "pacman" | "apk")
}

/// Package managers that run as the user but whose installs *may* call
/// `sudo` internally — e.g. Homebrew casks that ship a `.pkg` installer or
/// write to `/Library`. These aren't wrapped in sudo; we just make sure a
/// credential is cached first so the inner `sudo` call doesn't prompt.
pub fn manager_may_prompt(manager_field: &str) -> bool {
    matches!(manager_field, "homebrew_casks")
}

/// Who heimdal is running as, which decides how each install is launched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
    /// A normal user: root-only managers go through `sudo -n`.
    User,
    /// Root, started via `sudo heimdal ...` by this user (`$SUDO_USER`).
    /// User-level managers (Homebrew, mas, npm, cargo, ...) are dropped back
    /// to that user — Homebrew refuses to run as root.
    SudoFrom(String),
    /// Root with no invoking user (e.g. a root login or container).
    Root,
}

/// Detect the current [`Identity`].
pub fn current_identity() -> Identity {
    if !is_root() {
        return Identity::User;
    }
    match std::env::var("SUDO_USER") {
        Ok(u) if !u.is_empty() && u != "root" => Identity::SudoFrom(u),
        _ => Identity::Root,
    }
}

/// Whether any planned install — given as `(needs_root, manager_field)` —
/// will want a sudo credential when not running as root.
pub fn needs_auth<'a>(items: impl IntoIterator<Item = (bool, &'a str)>) -> bool {
    if is_root() {
        return false;
    }
    items
        .into_iter()
        .any(|(needs_root, field)| needs_root || manager_may_prompt(field))
}

/// Build the program + argv actually executed for an install.
///
/// - Root-only managers get `sudo -n` prepended when running as a user.
/// - User-level managers are dropped back to the invoking user when heimdal
///   itself was started with `sudo` (keeping `PATH`, so brew/npm/cargo are
///   still found).
pub fn build_invocation(
    cmd: &str,
    args: &[String],
    pkg: &str,
    needs_root: bool,
    identity: &Identity,
) -> (String, Vec<String>) {
    let mut argv: Vec<String> = args.to_vec();
    argv.push(pkg.to_string());
    match (needs_root, identity) {
        (true, Identity::User) => {
            let mut sudo_argv = vec!["-n".to_string(), cmd.to_string()];
            sudo_argv.extend(argv);
            ("sudo".to_string(), sudo_argv)
        }
        (false, Identity::SudoFrom(user)) => {
            let mut sudo_argv = vec![
                "-n".to_string(),
                "-u".to_string(),
                user.clone(),
                "-H".to_string(),
                "env".to_string(),
                format!("PATH={}", std::env::var("PATH").unwrap_or_default()),
                cmd.to_string(),
            ];
            sudo_argv.extend(argv);
            ("sudo".to_string(), sudo_argv)
        }
        _ => (cmd.to_string(), argv),
    }
}

/// When heimdal runs under `sudo`, files it creates in the user's home
/// (state.json, install logs) would end up owned by root, breaking the next
/// non-sudo run. Hand them back to the invoking user. No-op otherwise.
pub fn give_back_to_invoking_user(path: &std::path::Path) {
    #[cfg(unix)]
    {
        if !is_root() {
            return;
        }
        let (Ok(uid), Ok(gid)) = (std::env::var("SUDO_UID"), std::env::var("SUDO_GID")) else {
            return;
        };
        let (Ok(uid), Ok(gid)) = (uid.parse::<u32>(), gid.parse::<u32>()) else {
            return;
        };
        use std::os::unix::ffi::OsStrExt;
        if let Ok(c) = std::ffi::CString::new(path.as_os_str().as_bytes()) {
            // SAFETY: valid NUL-terminated path; failure is ignored.
            unsafe {
                libc::chown(c.as_ptr(), uid, gid);
            }
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Keeps a sudo credential fresh for as long as it is alive.
pub struct SudoSession {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl SudoSession {
    fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            // Sleep in short slices so dropping the session is prompt.
            let slice = Duration::from_millis(250);
            let mut waited = Duration::ZERO;
            while !flag.load(Ordering::Relaxed) {
                std::thread::sleep(slice);
                waited += slice;
                if waited >= KEEPALIVE_INTERVAL {
                    waited = Duration::ZERO;
                    let _ = std::process::Command::new("sudo")
                        .args(["-n", "-v"])
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                }
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for SudoSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Result of trying to obtain a sudo credential.
pub enum Auth {
    /// Running as root — nothing to do.
    Root,
    /// Credential cached; keep the session alive until installs finish.
    Session(SudoSession),
    /// Couldn't authenticate. The string explains why, in a form suitable
    /// for showing as the failure reason of every root-only install.
    Failed(String),
}

/// Make sure a sudo credential is cached, prompting for the password if
/// needed. The stage's live progress display is paused while the prompt is
/// up, so the prompt and the typed password aren't drawn over.
pub fn acquire(stage: &crate::progress::StageBar) -> Auth {
    if is_root() {
        return Auth::Root;
    }

    // Already have a cached credential (e.g. user ran sudo recently)?
    if sudo_quiet(&["-n", "-v"]) {
        return Auth::Session(SudoSession::start());
    }

    if !std::io::stdin().is_terminal() {
        return Auth::Failed(
            "needs administrator rights, but heimdal isn't running in an interactive \
             terminal to ask for a password. Run `sudo -v` first, or run `heimdal apply` \
             from a terminal."
                .to_string(),
        );
    }

    let ok = stage.suspend(|| {
        eprintln!();
        eprintln!("  🔐 Some packages need administrator rights to install.");
        std::process::Command::new("sudo")
            .args(["-v", "-p", "     Password for %u: "])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    });

    if ok {
        Auth::Session(SudoSession::start())
    } else {
        Auth::Failed(
            "needs administrator rights, but sudo authentication failed or was cancelled"
                .to_string(),
        )
    }
}

fn sudo_quiet(args: &[&str]) -> bool {
    std::process::Command::new("sudo")
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_managers_need_root() {
        for m in ["apt", "dnf", "pacman", "apk"] {
            assert!(manager_needs_root(m), "{m} should need root");
        }
        for m in ["homebrew", "homebrew-cask", "mas", "npm", "cargo"] {
            assert!(!manager_needs_root(m), "{m} should not need root");
        }
    }

    #[test]
    fn test_casks_may_prompt() {
        assert!(manager_may_prompt("homebrew_casks"));
        assert!(!manager_may_prompt("homebrew"));
    }

    #[test]
    fn test_build_invocation_wraps_root_managers_in_non_interactive_sudo() {
        let (prog, argv) = build_invocation(
            "apt-get",
            &["install".into(), "-y".into()],
            "curl",
            true,
            &Identity::User,
        );
        assert_eq!(prog, "sudo");
        assert_eq!(argv, vec!["-n", "apt-get", "install", "-y", "curl"]);
    }

    #[test]
    fn test_build_invocation_no_sudo_when_already_root() {
        let (prog, argv) = build_invocation(
            "apt-get",
            &["install".into(), "-y".into()],
            "curl",
            true,
            &Identity::Root,
        );
        assert_eq!(prog, "apt-get");
        assert_eq!(argv, vec!["install", "-y", "curl"]);
    }

    #[test]
    fn test_build_invocation_user_managers_untouched() {
        let (prog, argv) =
            build_invocation("brew", &["install".into()], "git", false, &Identity::User);
        assert_eq!(prog, "brew");
        assert_eq!(argv, vec!["install", "git"]);
    }

    #[test]
    fn test_needs_auth_detects_root_and_cask_work() {
        if is_root() {
            // Root never needs to authenticate.
            assert!(!needs_auth([(true, "apt")]));
            return;
        }
        assert!(needs_auth([(false, "homebrew"), (true, "apt")]));
        assert!(needs_auth([(false, "homebrew_casks")]));
        assert!(!needs_auth([(false, "homebrew"), (false, "mas")]));
        assert!(!needs_auth(std::iter::empty::<(bool, &str)>()));
    }

    #[test]
    fn test_build_invocation_drops_brew_back_to_sudo_user() {
        let (prog, argv) = build_invocation(
            "brew",
            &["install".into()],
            "git",
            false,
            &Identity::SudoFrom("aleem".into()),
        );
        assert_eq!(prog, "sudo");
        assert_eq!(&argv[..5], &["-n", "-u", "aleem", "-H", "env"]);
        assert!(argv[5].starts_with("PATH="));
        assert_eq!(&argv[6..], &["brew", "install", "git"]);
    }

    #[test]
    fn test_build_invocation_root_manager_under_sudo_runs_directly() {
        let (prog, argv) = build_invocation(
            "apt-get",
            &["install".into()],
            "curl",
            true,
            &Identity::SudoFrom("aleem".into()),
        );
        assert_eq!(prog, "apt-get");
        assert_eq!(argv, vec!["install", "curl"]);
    }
}
