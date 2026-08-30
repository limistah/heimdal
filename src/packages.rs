use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Check if a command is available on the system.
pub(crate) fn check_command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run `cmd args...` and return its captured stdout on success.
/// Returns `None` (never an error) if the command can't be spawned or exits
/// unsuccessfully — callers treat that the same as "nothing installed".
pub(crate) fn capture_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Parse output where each installed package name appears alone on its own
/// line — the shape shared by `brew list --formula`/`--cask`,
/// `dpkg-query -W -f='${Package}\n'`, `rpm -qa --qf '%{NAME}\n'`,
/// `pacman -Qq`, and `apk info`.
fn parse_one_per_line(output: &str) -> HashSet<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Parse `brew list --formula` / `brew list --cask` output: one package name
/// per line.
fn parse_brew_installed(output: &str) -> HashSet<String> {
    parse_one_per_line(output)
}

/// Parse `dpkg-query -W -f='${Package}\n'` output: one package name per line.
/// Chosen over `apt list --installed` because dpkg-query's output is a
/// stable, script-friendly format (no header/footer, no "[installed]"
/// suffix noise on stderr).
fn parse_dpkg_installed(output: &str) -> HashSet<String> {
    parse_one_per_line(output)
}

/// Parse `rpm -qa --qf '%{NAME}\n'` output (used to query dnf-managed
/// systems, since dnf sits on top of rpm): one package name per line, with
/// no version/arch/repo suffix to strip, unlike `dnf list installed`.
fn parse_rpm_installed(output: &str) -> HashSet<String> {
    parse_one_per_line(output)
}

/// Parse `pacman -Qq` output: one package name per line.
fn parse_pacman_installed(output: &str) -> HashSet<String> {
    parse_one_per_line(output)
}

/// Parse `apk info` output: one package name per line.
fn parse_apk_installed(output: &str) -> HashSet<String> {
    parse_one_per_line(output)
}

/// Parse `mas list` output. Each line looks like:
///   409183694 Xcode (15.0)
/// yaml `mas` entries key on the numeric App Store id (see
/// `get_manager_packages`), so extract just the leading id token.
fn parse_mas_installed(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

pub struct InstallResult {
    pub package: String,
    /// `PackageManager::field_name()` this package was installed through
    /// (e.g. "homebrew", "apt", "mas") — used to attribute a successful
    /// install to the right manager in `State::package_inventory`.
    pub manager_field: String,
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
    /// Query the system for identifiers this manager currently has
    /// installed. Returns an empty set — never an error — when the manager
    /// itself is unavailable on this machine or the query fails to run.
    fn installed(&self) -> HashSet<String>;
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("brew", &["list", "--formula"])
            .map(|out| parse_brew_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("brew", &["list", "--cask"])
            .map(|out| parse_brew_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("dpkg-query", &["-W", "-f=${Package}\n"])
            .map(|out| parse_dpkg_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        // `rpm -qa` parses more reliably than `dnf list installed`, which
        // prepends a header line and suffixes each entry with arch/repo.
        capture_stdout("rpm", &["-qa", "--qf", "%{NAME}\n"])
            .map(|out| parse_rpm_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("pacman", &["-Qq"])
            .map(|out| parse_pacman_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("apk", &["info"])
            .map(|out| parse_apk_installed(&out))
            .unwrap_or_default()
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

    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("mas", &["list"])
            .map(|out| parse_mas_installed(&out))
            .unwrap_or_default()
    }
}

// ── Public interface ──────────────────────────────────────────────────────────

/// A single unit of package installation work.
///
/// Shared with `crate::toolchains`, which builds these for npm/cargo/go/gem/
/// pip installs and runs them through the same `run_parallel` used here —
/// there is one execution path for every kind of install heimdal does.
pub(crate) struct WorkItem {
    pub(crate) cmd: String,
    pub(crate) args: Vec<String>,
    /// CLI argument passed to the install command (e.g. the numeric ID for mas).
    pub(crate) pkg: String,
    /// `PackageManager::field_name()` this install runs through.
    pub(crate) manager_field: String,
    /// Human-readable name shown in the progress bar (e.g. "Xcode" instead of "409183694").
    pub(crate) display_name: String,
    pub(crate) dry_run: bool,
}

/// Install one package synchronously; returns an `InstallResult`.
fn install_one(item: &WorkItem) -> InstallResult {
    if item.dry_run {
        return InstallResult {
            package: item.pkg.clone(),
            manager_field: item.manager_field.clone(),
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
            manager_field: item.manager_field.clone(),
            display_name: item.display_name.clone(),
            success: false,
            already_installed: false,
            message: Some(format!("Cannot run {}: {}", item.cmd, e)),
        },
        Ok(out) => InstallResult {
            package: item.pkg.clone(),
            manager_field: item.manager_field.clone(),
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
/// Returns `(failures, successes)`, where `successes` is a list of
/// `(manager_field, pkg)` pairs for every install that succeeded — including
/// dry-run "installs", which callers should filter out before recording
/// anything as actually installed.
pub(crate) fn run_parallel(
    work: Vec<WorkItem>,
    bar: Arc<crate::progress::PackageBar>,
    num_threads: usize,
) -> (Vec<crate::progress::FailedPackage>, Vec<(String, String)>) {
    if work.is_empty() {
        return (vec![], vec![]);
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

    let mut successes: Vec<(String, String)> = Vec::new();
    for result in rx {
        if result.success {
            bar.record_success(&result.display_name);
            successes.push((result.manager_field.clone(), result.package.clone()));
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
    let failures = Arc::try_unwrap(bar)
        .expect("PackageBar arc still referenced after all workers joined")
        .into_failures();
    (failures, successes)
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

/// Query every package manager for what's actually installed on this
/// machine, restricted to managers whose `is_available()` reports true here
/// — this is a live system query, distinct from `State::package_inventory`
/// (heimdal's own install ledger).
///
/// Returns a map from `field_name` ("homebrew", "apt", "mas", ...) to the
/// set of identifiers that manager reports as installed. A manager that
/// isn't present on this machine is simply absent from the map.
pub fn query_installed() -> HashMap<String, HashSet<String>> {
    let managers: Vec<Box<dyn PackageManager>> = vec![
        Box::new(Homebrew),
        Box::new(HomebrewCask),
        Box::new(Apt),
        Box::new(Dnf),
        Box::new(Pacman),
        Box::new(Apk),
        Box::new(Mas),
    ];
    managers
        .into_iter()
        .filter(|m| m.is_available())
        .map(|m| {
            let field = m.field_name().to_string();
            let installed = m.installed();
            (field, installed)
        })
        .collect()
}

/// Compute the fully-resolved set of identifiers `pkgs` declares per package
/// manager, keyed by `PackageManager::field_name()` — the "what should be
/// installed" counterpart to `query_installed`'s "what actually is".
///
/// `pkgs` is expected to already have any top-level shared packages merged
/// in (see `config::resolve_profile`), so this only needs to expand the
/// manager-specific fields plus `common`.
///
/// `common_manager_field` is the field name of whichever manager `common`
/// packages would actually install through — the same manager
/// `install_for_profile` picks (the first available of Homebrew/Apt/Dnf/
/// Pacman/Apk, mirrored by `detect_manager()`). Pass `None` when no manager
/// is available on this machine, matching `install_for_profile`'s "skip
/// common, nothing to resolve against" behavior.
pub fn declared_identifiers(
    pkgs: &crate::config::PackageMap,
    common_manager_field: Option<&str>,
) -> HashMap<String, HashSet<String>> {
    const FIELDS: [&str; 7] = [
        "homebrew",
        "homebrew_casks",
        "apt",
        "dnf",
        "pacman",
        "apk",
        "mas",
    ];

    let mut declared: HashMap<String, HashSet<String>> = HashMap::new();
    for field in FIELDS {
        let ids: HashSet<String> = get_manager_packages(field, pkgs)
            .into_iter()
            .map(|(pkg, _display_name)| pkg)
            .collect();
        if !ids.is_empty() {
            declared.insert(field.to_string(), ids);
        }
    }

    if let Some(field) = common_manager_field {
        let entry = declared.entry(field.to_string()).or_default();
        for pkg in &pkgs.common {
            entry.insert(pkg.resolve(field));
        }
    }

    declared
}

/// If `pkg` is already installed for `field` (per `installed_sets`) and
/// `force` isn't set, report it as a skip and return `true` — the caller
/// must not enqueue a `WorkItem` for it. A skip that heimdal's own
/// `state.package_inventory` doesn't yet know about is appended to
/// `newly_tracked` so the caller can record it without running the install
/// command.
#[allow(clippy::too_many_arguments)]
pub(crate) fn maybe_skip_installed(
    field: &str,
    pkg: &str,
    display_name: &str,
    force: bool,
    installed_sets: &HashMap<String, HashSet<String>>,
    state: &crate::state::State,
    stage: &crate::progress::StageBar,
    newly_tracked: &mut Vec<(String, String)>,
) -> bool {
    if force {
        return false;
    }
    let already_installed = installed_sets
        .get(field)
        .map(|set| set.contains(pkg))
        .unwrap_or(false);
    if !already_installed {
        return false;
    }
    let already_tracked = state
        .package_inventory
        .get(field)
        .map(|entry| entry.identifiers.contains(pkg))
        .unwrap_or(false);
    if !already_tracked {
        newly_tracked.push((field.to_string(), pkg.to_string()));
    }
    stage.println(format!(
        "         · skip       {} — already installed",
        display_name
    ));
    true
}

/// Decide which declared packages still need their install command to run,
/// and which are already installed on the system but not yet reflected in
/// heimdal's own inventory.
///
/// `force` bypasses the "already installed" check entirely, matching apply's
/// `--force`: every declared package still gets a `WorkItem`.
///
/// Returns `(work, newly_tracked)` — `work` is what `run_parallel` should
/// actually run; `newly_tracked` is `(manager_field, pkg)` pairs the caller
/// should record directly into `state`, without running an install command.
fn plan_work(
    pkgs: &crate::config::PackageMap,
    managers: &[Box<dyn PackageManager>],
    force: bool,
    state: &crate::state::State,
    stage: &crate::progress::StageBar,
    dry_run: bool,
) -> (Vec<WorkItem>, Vec<(String, String)>) {
    // Skip the live query entirely when forcing: every package installs
    // regardless of what's already present, so there's nothing to check.
    let installed_sets: HashMap<String, HashSet<String>> = if force {
        HashMap::new()
    } else {
        managers
            .iter()
            .filter(|m| m.is_available())
            .map(|m| (m.field_name().to_string(), m.installed()))
            .collect()
    };

    let mut work: Vec<WorkItem> = Vec::new();
    let mut newly_tracked: Vec<(String, String)> = Vec::new();

    // Common packages → first available manager
    if !pkgs.common.is_empty() {
        match managers.iter().find(|m| m.is_available()) {
            Some(manager) => {
                let (cmd, args) = manager_cmd(manager.name());
                let field = manager.field_name();
                for pkg in &pkgs.common {
                    let name = pkg.resolve(field);
                    if maybe_skip_installed(
                        field,
                        &name,
                        &name,
                        force,
                        &installed_sets,
                        state,
                        stage,
                        &mut newly_tracked,
                    ) {
                        continue;
                    }
                    work.push(WorkItem {
                        cmd: cmd.clone(),
                        args: args.clone(),
                        pkg: name.clone(),
                        manager_field: field.to_string(),
                        display_name: name,
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
    for manager in managers {
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
        let field = manager.field_name();
        for (pkg, display_name) in to_install {
            if maybe_skip_installed(
                field,
                &pkg,
                &display_name,
                force,
                &installed_sets,
                state,
                stage,
                &mut newly_tracked,
            ) {
                continue;
            }
            work.push(WorkItem {
                cmd: cmd.clone(),
                args: args.clone(),
                pkg,
                manager_field: field.to_string(),
                display_name,
                dry_run,
            });
        }
    }

    (work, newly_tracked)
}

/// Install packages for the active profile. Reports progress via `stage`.
/// Every package that installs successfully (outside of `dry_run`) is
/// recorded into `state`'s package inventory — callers are responsible for
/// calling `state.save()` afterwards.
///
/// A package already present on the system (per the live `installed()`
/// query) is skipped without running its install command, unless `force` is
/// set — but if heimdal's own inventory didn't already know about it, it is
/// still recorded, so "installed outside heimdal" and "installed by heimdal
/// before" both end up accurately tracked.
///
/// Returns the list of packages that failed to install.
pub fn install_for_profile(
    profile: &crate::config::Profile,
    dry_run: bool,
    stage: &crate::progress::StageBar,
    parallel_jobs: usize,
    force: bool,
    state: &mut crate::state::State,
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

    let (work, newly_tracked) =
        plan_work(&profile.packages, &managers, force, state, stage, dry_run);

    // Packages the system already has but heimdal didn't know about: record
    // them now. Skipped in dry-run — dry-run must never modify state.
    if !dry_run {
        for (manager_field, pkg) in newly_tracked {
            state.record_installed(&manager_field, [pkg]);
        }
    }

    if work.is_empty() {
        return Ok(vec![]);
    }

    let bar = Arc::new(stage.package_bar(work.len()));
    let (failures, successes) = run_parallel(work, bar, parallel_jobs);

    // Dry runs report every item as a "success" purely for progress display;
    // nothing was actually installed, so nothing should be recorded.
    if !dry_run {
        for (manager_field, pkg) in successes {
            state.record_installed(&manager_field, [pkg]);
        }
    }

    Ok(failures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CommonPackage, CommonPackageAliases, PackageMap, Profile};
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
                common: vec![
                    CommonPackage::Simple("git".to_string()),
                    CommonPackage::Simple("curl".to_string()),
                ],
                ..Default::default()
            },
            ..Default::default()
        };
        let mut state = crate::state::State::default();
        let result = install_for_profile(&profile, true, &stage, 2, false, &mut state);
        assert!(result.is_ok());
        let failures = result.unwrap();
        assert!(failures.is_empty(), "dry run should produce no failures");
        assert!(
            state.package_inventory.is_empty(),
            "dry run must not record anything as actually installed"
        );
    }

    #[test]
    fn test_install_for_profile_empty_profile() {
        let (_p, stage) = make_stage();
        let profile = Profile::default();
        let mut state = crate::state::State::default();
        let result = install_for_profile(&profile, false, &stage, 4, false, &mut state);
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
            manager_field: "homebrew".to_string(),
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

    // A plain-string `common` entry must resolve to the exact same name no
    // matter which package manager actually ran (pre-existing behavior).
    #[test]
    fn test_common_package_simple_resolves_same_name_for_every_manager() {
        let pkg = CommonPackage::Simple("zsh".to_string());
        for field in [
            "homebrew",
            "homebrew_casks",
            "apt",
            "dnf",
            "pacman",
            "apk",
            "mas",
        ] {
            assert_eq!(pkg.resolve(field), "zsh");
        }
    }

    // An aliased `common` entry should resolve to the manager-specific
    // override when present, and fall back to `default` otherwise.
    #[test]
    fn test_common_package_aliased_resolves_per_manager() {
        let pkg = CommonPackage::Aliased(CommonPackageAliases {
            default: "docker-desktop".to_string(),
            homebrew: None,
            homebrew_casks: Some("docker-desktop".to_string()),
            apt: Some("docker-ce".to_string()),
            dnf: Some("docker-ce".to_string()),
            pacman: Some("docker".to_string()),
            apk: Some("docker".to_string()),
            mas: None,
        });

        assert_eq!(pkg.resolve("homebrew_casks"), "docker-desktop");
        assert_eq!(pkg.resolve("apt"), "docker-ce");
        assert_eq!(pkg.resolve("dnf"), "docker-ce");
        assert_eq!(pkg.resolve("pacman"), "docker");
        assert_eq!(pkg.resolve("apk"), "docker");
        // "homebrew" and "mas" have no override → fall back to `default`.
        assert_eq!(pkg.resolve("homebrew"), "docker-desktop");
        assert_eq!(pkg.resolve("mas"), "docker-desktop");
    }

    // install_for_profile should resolve an aliased common package using
    // the field name of whichever manager ran, not the literal string.
    #[test]
    fn test_install_for_profile_resolves_aliased_common_package_dry_run() {
        let (_p, stage) = make_stage();
        let profile = Profile {
            packages: PackageMap {
                common: vec![CommonPackage::Aliased(CommonPackageAliases {
                    default: "docker-desktop".to_string(),
                    homebrew: None,
                    homebrew_casks: Some("docker-desktop".to_string()),
                    apt: Some("docker-ce".to_string()),
                    dnf: Some("docker-ce".to_string()),
                    pacman: Some("docker".to_string()),
                    apk: Some("docker".to_string()),
                    mas: None,
                })],
                ..Default::default()
            },
            ..Default::default()
        };
        // dry_run: install_for_profile only needs the entry to resolve
        // without panicking and to report success either way.
        let mut state = crate::state::State::default();
        let result = install_for_profile(&profile, true, &stage, 2, false, &mut state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ── Idempotent install (skip already-installed packages) ────────────────

    /// A `PackageManager` stand-in for tests: no real subprocess is ever
    /// spawned — `installed()` returns whatever set the test configures.
    struct MockManager {
        name: &'static str,
        field: &'static str,
        available: bool,
        installed_set: HashSet<String>,
    }

    impl PackageManager for MockManager {
        fn name(&self) -> &str {
            self.name
        }
        fn field_name(&self) -> &str {
            self.field
        }
        fn is_available(&self) -> bool {
            self.available
        }
        fn installed(&self) -> HashSet<String> {
            self.installed_set.clone()
        }
    }

    fn mock_homebrew(installed: &[&str]) -> Vec<Box<dyn PackageManager>> {
        vec![Box::new(MockManager {
            name: "homebrew",
            field: "homebrew",
            available: true,
            installed_set: installed.iter().map(|s| s.to_string()).collect(),
        })]
    }

    #[test]
    fn test_plan_work_skips_already_installed_package() {
        let (_p, stage) = make_stage();
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string()],
            ..Default::default()
        };
        let managers = mock_homebrew(&["git"]);
        let state = crate::state::State::default();
        let (work, newly_tracked) = plan_work(&pkgs, &managers, false, &state, &stage, false);
        assert!(
            work.is_empty(),
            "an already-installed package must not enqueue an install command"
        );
        assert_eq!(
            newly_tracked,
            vec![("homebrew".to_string(), "git".to_string())],
            "an untracked-but-installed package should still be recorded"
        );
    }

    #[test]
    fn test_plan_work_installs_not_yet_installed_package() {
        let (_p, stage) = make_stage();
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string()],
            ..Default::default()
        };
        let managers = mock_homebrew(&[]);
        let state = crate::state::State::default();
        let (work, newly_tracked) = plan_work(&pkgs, &managers, false, &state, &stage, false);
        assert_eq!(
            work.len(),
            1,
            "a not-yet-installed package must still install"
        );
        assert_eq!(work[0].pkg, "git");
        assert!(newly_tracked.is_empty());
    }

    #[test]
    fn test_plan_work_force_bypasses_skip() {
        let (_p, stage) = make_stage();
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string()],
            ..Default::default()
        };
        let managers = mock_homebrew(&["git"]);
        let state = crate::state::State::default();
        let (work, newly_tracked) = plan_work(&pkgs, &managers, true, &state, &stage, false);
        assert_eq!(
            work.len(),
            1,
            "--force must reinstall even an already-installed package"
        );
        assert!(
            newly_tracked.is_empty(),
            "force reinstalls via the command instead of a tracking-only record"
        );
    }

    #[test]
    fn test_plan_work_does_not_re_track_already_tracked_package() {
        let (_p, stage) = make_stage();
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string()],
            ..Default::default()
        };
        let managers = mock_homebrew(&["git"]);
        let mut state = crate::state::State::default();
        state.record_installed("homebrew", vec!["git".to_string()]);
        let (work, newly_tracked) = plan_work(&pkgs, &managers, false, &state, &stage, false);
        assert!(work.is_empty());
        assert!(
            newly_tracked.is_empty(),
            "a package heimdal already tracks must not be re-recorded on every apply"
        );
    }

    // ── Installed-set parsing ────────────────────────────────────────────────

    #[test]
    fn test_parse_brew_installed_one_name_per_line() {
        let output = "git\nvim\ncurl\n";
        let installed = parse_brew_installed(output);
        assert_eq!(installed.len(), 3);
        assert!(installed.contains("git"));
        assert!(installed.contains("vim"));
        assert!(installed.contains("curl"));
    }

    #[test]
    fn test_parse_brew_installed_ignores_blank_lines() {
        let output = "git\n\nvim\n\n";
        let installed = parse_brew_installed(output);
        assert_eq!(installed.len(), 2);
    }

    #[test]
    fn test_parse_dpkg_installed_one_name_per_line() {
        // Realistic `dpkg-query -W -f='${Package}\n'` output.
        let output = "git\nvim\nlibc6\n";
        let installed = parse_dpkg_installed(output);
        assert_eq!(installed.len(), 3);
        assert!(installed.contains("git"));
        assert!(installed.contains("libc6"));
    }

    #[test]
    fn test_parse_rpm_installed_one_name_per_line() {
        // Realistic `rpm -qa --qf '%{NAME}\n'` output.
        let output = "git\nvim-enhanced\nglibc\n";
        let installed = parse_rpm_installed(output);
        assert_eq!(installed.len(), 3);
        assert!(installed.contains("vim-enhanced"));
    }

    #[test]
    fn test_parse_pacman_installed_one_name_per_line() {
        // Realistic `pacman -Qq` output.
        let output = "git\nvim\nlinux\n";
        let installed = parse_pacman_installed(output);
        assert_eq!(installed.len(), 3);
        assert!(installed.contains("linux"));
    }

    #[test]
    fn test_parse_apk_installed_one_name_per_line() {
        // Realistic `apk info` output.
        let output = "musl\nbusybox\ngit\n";
        let installed = parse_apk_installed(output);
        assert_eq!(installed.len(), 3);
        assert!(installed.contains("busybox"));
    }

    #[test]
    fn test_parse_mas_installed_extracts_leading_id() {
        // Realistic `mas list` output: "<id> <name> (<version>)".
        let output = "409183694 Xcode (15.0)\n803453959 Slack (4.36.140)\n";
        let installed = parse_mas_installed(output);
        assert_eq!(installed.len(), 2);
        assert!(installed.contains("409183694"));
        assert!(installed.contains("803453959"));
        // The name/version must not leak into the identifier set.
        assert!(!installed.contains("Xcode"));
    }

    #[test]
    fn test_parse_mas_installed_ignores_blank_lines() {
        let output = "409183694 Xcode (15.0)\n\n803453959 Slack (4.36.140)\n";
        let installed = parse_mas_installed(output);
        assert_eq!(installed.len(), 2);
    }

    // ── declared_identifiers ─────────────────────────────────────────────────

    #[test]
    fn test_declared_identifiers_expands_manager_specific_fields() {
        let pkgs = PackageMap {
            homebrew: vec!["git".to_string(), "curl".to_string()],
            apt: vec!["vim".to_string()],
            ..Default::default()
        };
        let declared = declared_identifiers(&pkgs, None);
        assert_eq!(
            declared.get("homebrew").unwrap(),
            &HashSet::from(["git".to_string(), "curl".to_string()])
        );
        assert_eq!(
            declared.get("apt").unwrap(),
            &HashSet::from(["vim".to_string()])
        );
        // Managers with nothing declared should not appear at all.
        assert!(!declared.contains_key("dnf"));
        assert!(!declared.contains_key("pacman"));
    }

    #[test]
    fn test_declared_identifiers_resolves_common_into_given_manager() {
        let pkgs = PackageMap {
            common: vec![
                CommonPackage::Simple("zsh".to_string()),
                CommonPackage::Aliased(CommonPackageAliases {
                    default: "docker-desktop".to_string(),
                    homebrew: None,
                    homebrew_casks: Some("docker-desktop".to_string()),
                    apt: Some("docker-ce".to_string()),
                    dnf: None,
                    pacman: None,
                    apk: None,
                    mas: None,
                }),
            ],
            ..Default::default()
        };

        let declared = declared_identifiers(&pkgs, Some("apt"));
        assert_eq!(
            declared.get("apt").unwrap(),
            &HashSet::from(["zsh".to_string(), "docker-ce".to_string()])
        );
    }

    #[test]
    fn test_declared_identifiers_skips_common_when_no_manager_available() {
        let pkgs = PackageMap {
            common: vec![CommonPackage::Simple("zsh".to_string())],
            ..Default::default()
        };
        let declared = declared_identifiers(&pkgs, None);
        assert!(
            declared.is_empty(),
            "no manager-specific packages and no common_manager_field should yield nothing"
        );
    }
}
