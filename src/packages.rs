use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Check if a command is available on the system.
fn check_command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub struct InstallResult {
    #[allow(dead_code)]
    pub package: String,
    /// Human-readable name used in progress display (e.g. "Xcode" instead of "409183694" for mas).
    pub display_name: String,
    pub success: bool,
    #[allow(dead_code)]
    pub already_installed: bool,
    pub message: Option<String>,
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn field_name(&self) -> &str; // matches PackageMap field: "homebrew", "apt", etc.
    fn is_available(&self) -> bool;
}

// ── Homebrew ──────────────────────────────────────────────────────────────────

pub struct Homebrew;

impl PackageManager for Homebrew {
    fn name(&self) -> &str {
        "homebrew"
    }
    fn field_name(&self) -> &str {
        "homebrew"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// ── Homebrew Cask ─────────────────────────────────────────────────────────────

pub struct HomebrewCask;

impl PackageManager for HomebrewCask {
    fn name(&self) -> &str {
        "homebrew-cask"
    }
    fn field_name(&self) -> &str {
        "homebrew_casks"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// ── Apt ───────────────────────────────────────────────────────────────────────

pub struct Apt;

impl PackageManager for Apt {
    fn name(&self) -> &str {
        "apt"
    }
    fn field_name(&self) -> &str {
        "apt"
    }

    fn is_available(&self) -> bool {
        check_command_available("apt-get")
    }
}

// ── Dnf ───────────────────────────────────────────────────────────────────────

pub struct Dnf;

impl PackageManager for Dnf {
    fn name(&self) -> &str {
        "dnf"
    }
    fn field_name(&self) -> &str {
        "dnf"
    }

    fn is_available(&self) -> bool {
        check_command_available("dnf")
    }
}

// ── Pacman ────────────────────────────────────────────────────────────────────

pub struct Pacman;

impl PackageManager for Pacman {
    fn name(&self) -> &str {
        "pacman"
    }
    fn field_name(&self) -> &str {
        "pacman"
    }

    fn is_available(&self) -> bool {
        check_command_available("pacman")
    }
}

// ── Apk ───────────────────────────────────────────────────────────────────────

pub struct Apk;

impl PackageManager for Apk {
    fn name(&self) -> &str {
        "apk"
    }
    fn field_name(&self) -> &str {
        "apk"
    }

    fn is_available(&self) -> bool {
        check_command_available("apk")
    }
}

// ── Mac App Store (MAS) ───────────────────────────────────────────────────────

pub struct Mas;

impl PackageManager for Mas {
    fn name(&self) -> &str {
        "mas"
    }
    fn field_name(&self) -> &str {
        "mas"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("mas")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

// ── Public interface ──────────────────────────────────────────────────────────

/// A single unit of package installation work.
struct WorkItem {
    cmd: String,
    args: Vec<String>,
    /// CLI argument passed to the install command (e.g. the numeric ID for mas).
    pkg: String,
    /// Human-readable name shown in the progress bar (e.g. "Xcode" instead of "409183694").
    display_name: String,
    dry_run: bool,
}

/// Install one package synchronously; returns an `InstallResult`.
fn install_one(item: &WorkItem) -> InstallResult {
    if item.dry_run {
        return InstallResult {
            package: item.pkg.clone(),
            display_name: item.display_name.clone(),
            success: true,
            already_installed: false,
            message: Some(format!("[dry-run] Would install: {}", item.display_name)),
        };
    }
    let mut cmd = std::process::Command::new(&item.cmd);
    cmd.args(&item.args).arg(&item.pkg);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    match cmd.output() {
        Err(e) => InstallResult {
            package: item.pkg.clone(),
            display_name: item.display_name.clone(),
            success: false,
            already_installed: false,
            message: Some(format!("Cannot run {}: {}", item.cmd, e)),
        },
        Ok(out) => InstallResult {
            package: item.pkg.clone(),
            display_name: item.display_name.clone(),
            success: out.status.success(),
            already_installed: false,
            message: if out.status.success() {
                None
            } else {
                // Prefer stderr; fall back to stdout (some tools write errors there).
                // Normalize empty strings to None so unwrap_or("unknown error") fires.
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let msg = if !stderr.is_empty() { stderr } else { stdout };
                if msg.is_empty() {
                    None
                } else {
                    Some(msg)
                }
            },
        },
    }
}

/// Returns `(cmd, base_args)` for a package manager name.
fn manager_cmd(name: &str) -> (String, Vec<String>) {
    match name {
        "homebrew" => ("brew".to_string(), vec!["install".to_string()]),
        "homebrew-cask" => (
            "brew".to_string(),
            vec!["install".to_string(), "--cask".to_string()],
        ),
        "apt" => (
            "apt-get".to_string(),
            vec!["install".to_string(), "-y".to_string()],
        ),
        "dnf" => (
            "dnf".to_string(),
            vec!["install".to_string(), "-y".to_string()],
        ),
        "pacman" => (
            "pacman".to_string(),
            vec!["-S".to_string(), "--noconfirm".to_string()],
        ),
        "apk" => ("apk".to_string(), vec!["add".to_string()]),
        "mas" => ("mas".to_string(), vec!["install".to_string()]),
        _ => (name.to_string(), vec![]),
    }
}

/// Extract the package list for a given manager field name from a `PackageMap`.
/// Returns `(pkg, display_name)` pairs.  For most managers these are identical;
/// for `mas` the pkg is the numeric App Store ID and display_name is the app name.
fn get_manager_packages(field: &str, pkgs: &crate::config::PackageMap) -> Vec<(String, String)> {
    let ident = |p: &String| (p.clone(), p.clone());
    match field {
        "homebrew" => pkgs.homebrew.iter().map(ident).collect(),
        "homebrew_casks" => pkgs.homebrew_casks.iter().map(ident).collect(),
        "apt" => pkgs.apt.iter().map(ident).collect(),
        "dnf" => pkgs.dnf.iter().map(ident).collect(),
        "pacman" => pkgs.pacman.iter().map(ident).collect(),
        "apk" => pkgs.apk.iter().map(ident).collect(),
        "mas" => pkgs
            .mas
            .iter()
            .filter_map(|v| {
                let id = v.get("id").and_then(|id| id.as_u64())?;
                let id_str = id.to_string();
                let name = v
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or(&id_str)
                    .to_string();
                Some((id_str, name))
            })
            .collect(),
        _ => vec![],
    }
}

/// Run `work` items in parallel across `num_threads` workers.
/// Results are reported to `bar` as they arrive.
/// Returns `Vec<FailedPackage>`.
fn run_parallel(
    work: Vec<WorkItem>,
    bar: Arc<crate::progress::PackageBar>,
    num_threads: usize,
) -> Vec<crate::progress::FailedPackage> {
    if work.is_empty() {
        return vec![];
    }
    let queue = Arc::new(Mutex::new(VecDeque::from(work)));
    let (tx, rx) = mpsc::channel::<InstallResult>();

    let n_threads = num_threads.clamp(1, 16);
    let handles: Vec<_> = (0..n_threads)
        .map(|_| {
            let queue = Arc::clone(&queue);
            let tx = tx.clone();
            let bar = Arc::clone(&bar);
            std::thread::spawn(move || loop {
                let item = { queue.lock().unwrap().pop_front() };
                match item {
                    None => break,
                    Some(item) => {
                        bar.record_start(&item.display_name);
                        let result = install_one(&item);
                        let _ = tx.send(result);
                    }
                }
            })
        })
        .collect();

    drop(tx); // close sender so rx terminates when all workers finish

    for result in rx {
        if result.success {
            bar.record_success(&result.display_name);
        } else {
            bar.record_failure(
                &result.display_name,
                result.message.as_deref().unwrap_or("unknown error"),
            );
        }
    }

    for h in handles {
        let _ = h.join();
    }

    // Unwrap the Arc before calling into_failures (all workers have finished)
    Arc::try_unwrap(bar)
        .expect("PackageBar arc still referenced after all workers joined")
        .into_failures()
}

pub fn detect_manager() -> Option<Box<dyn PackageManager>> {
    let managers: Vec<Box<dyn PackageManager>> = vec![
        Box::new(Homebrew),
        Box::new(Apt),
        Box::new(Dnf),
        Box::new(Pacman),
        Box::new(Apk),
    ];
    managers.into_iter().find(|m| m.is_available())
}

/// Install packages for the active profile. Reports progress via `stage`.
/// Returns the list of packages that failed to install.
pub fn install_for_profile(
    profile: &crate::config::Profile,
    dry_run: bool,
    stage: &crate::progress::StageBar,
    parallel_jobs: usize,
) -> anyhow::Result<Vec<crate::progress::FailedPackage>> {
    let managers: Vec<Box<dyn PackageManager>> = vec![
        Box::new(Homebrew),
        Box::new(HomebrewCask),
        Box::new(Apt),
        Box::new(Dnf),
        Box::new(Pacman),
        Box::new(Apk),
        Box::new(Mas),
    ];

    let pkgs = &profile.packages;
    let mut work: Vec<WorkItem> = Vec::new();

    // Common packages → first available manager
    if !pkgs.common.is_empty() {
        match managers.iter().find(|m| m.is_available()) {
            Some(manager) => {
                let (cmd, args) = manager_cmd(manager.name());
                for pkg in &pkgs.common {
                    work.push(WorkItem {
                        cmd: cmd.clone(),
                        args: args.clone(),
                        pkg: pkg.clone(),
                        display_name: pkg.clone(),
                        dry_run,
                    });
                }
            }
            None => {
                crate::utils::warning(&format!(
                    "No package manager available. Skipping {} common package(s).",
                    pkgs.common.len()
                ));
            }
        }
    }

    // Manager-specific packages
    for manager in &managers {
        let to_install = get_manager_packages(manager.field_name(), pkgs);
        if to_install.is_empty() {
            continue;
        }
        if !manager.is_available() {
            crate::utils::warning(&format!(
                "Package manager '{}' not available. Skipping {} package(s).",
                manager.name(),
                to_install.len()
            ));
            continue;
        }
        let (cmd, args) = manager_cmd(manager.name());
        for (pkg, display_name) in to_install {
            work.push(WorkItem {
                cmd: cmd.clone(),
                args: args.clone(),
                pkg,
                display_name,
                dry_run,
            });
        }
    }

    if work.is_empty() {
        return Ok(vec![]);
    }

    let bar = Arc::new(stage.package_bar(work.len()));
    Ok(run_parallel(work, bar, parallel_jobs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PackageMap, Profile};
    use crate::progress::ApplyProgress;

    fn make_stage() -> (ApplyProgress, crate::progress::StageBar) {
        let p = ApplyProgress::new(5);
        let s = p.stage(2, "Installing packages");
        (p, s)
    }

    #[test]
    fn test_install_for_profile_dry_run_no_failures() {
        let (_p, stage) = make_stage();
        let profile = Profile {
            packages: PackageMap {
                common: vec!["git".to_string(), "curl".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = install_for_profile(&profile, true, &stage, 2);
        assert!(result.is_ok());
        let failures = result.unwrap();
        assert!(failures.is_empty(), "dry run should produce no failures");
    }

    #[test]
    fn test_install_for_profile_empty_profile() {
        let (_p, stage) = make_stage();
        let profile = Profile::default();
        let result = install_for_profile(&profile, false, &stage, 4);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // Bug 3: empty stderr should produce None message, not Some(""), so that
    // the "unknown error" fallback in run_parallel fires correctly.
    #[test]
    #[cfg(unix)]
    fn test_install_one_empty_stderr_yields_none_message() {
        // `false` exits with code 1 and writes nothing to stdout/stderr.
        let item = WorkItem {
            cmd: "false".to_string(),
            args: vec![],
            pkg: "testpkg".to_string(),
            display_name: "testpkg".to_string(),
            dry_run: false,
        };
        let result = install_one(&item);
        assert!(!result.success);
        assert!(
            result.message.is_none(),
            "empty stderr should produce None, not Some(\"\"), got: {:?}",
            result.message
        );
    }

    // Bug 2: mas entries have a "name" field that should be used as the
    // display name; the numeric ID is only the install-command argument.
    #[test]
    fn test_get_manager_packages_mas_uses_name_field() {
        let pkgs = PackageMap {
            mas: vec![serde_json::json!({"id": 409183694u64, "name": "Xcode"})],
            ..Default::default()
        };
        let items = get_manager_packages("mas", &pkgs);
        assert_eq!(items.len(), 1);
        let (pkg, display_name) = &items[0];
        assert_eq!(
            pkg, "409183694",
            "pkg should be the numeric ID for `mas install`"
        );
        assert_eq!(
            display_name, "Xcode",
            "display_name should come from the name field"
        );
    }

    #[test]
    fn test_get_manager_packages_mas_fallback_to_id_when_no_name() {
        let pkgs = PackageMap {
            mas: vec![serde_json::json!({"id": 409183694u64})],
            ..Default::default()
        };
        let items = get_manager_packages("mas", &pkgs);
        assert_eq!(items.len(), 1);
        let (pkg, display_name) = &items[0];
        assert_eq!(pkg, "409183694");
        assert_eq!(
            display_name, "409183694",
            "display_name should fall back to ID when name is absent"
        );
    }

    #[test]
    fn test_get_manager_packages_non_mas_uses_pkg_as_display_name() {
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string(), "curl".to_string()],
            ..Default::default()
        };
        let items = get_manager_packages("homebrew", &pkgs);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("git".to_string(), "git".to_string()));
        assert_eq!(items[1], ("curl".to_string(), "curl".to_string()));
    }
}
