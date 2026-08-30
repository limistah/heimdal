//! Language-toolchain-level global package installs: npm, cargo, `go
//! install`, gem, and pip (via pipx).
//!
//! This is a genuinely different axis from `crate::packages` (the OS-level
//! homebrew/apt/dnf/pacman/apk/mas managers). Only one system package manager
//! is ever present on a given machine, so `packages` picks whichever one is
//! available for `common` entries. Language toolchains are not mutually
//! exclusive that way — a single machine can legitimately have npm AND cargo
//! AND go tools installed at once — so every toolchain manager present here
//! runs its own declared list independently, and there is no `common` /
//! auto-detect concept the way there is for OS packages.
//!
//! Execution reuses `crate::packages`' `WorkItem` and `run_parallel`
//! infrastructure directly rather than a separate pipeline.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::packages::{capture_stdout, check_command_available, maybe_skip_installed, WorkItem};

/// Parallel to `crate::packages::PackageManager`, but for language-toolchain
/// installs. Kept as a sibling trait rather than reusing `PackageManager`
/// directly: every impl here still fits `name`/`field_name`/`is_available`/
/// `installed` cleanly, but `installed()`'s contract is weaker for `Go` (see
/// its impl) than it is for `PackageManager`, where every implementor is a
/// genuine live system query.
pub trait ToolchainManager: Send + Sync {
    fn name(&self) -> &str;
    fn field_name(&self) -> &str; // matches ToolchainMap field: "npm", "cargo", "go", "gem", "pip"
    fn is_available(&self) -> bool;
    /// Query the system for identifiers this manager currently reports
    /// installed. Returns an empty set — never an error — when the manager
    /// itself is unavailable or the query fails to run. For `Go` this is
    /// *always* empty regardless of availability; see `Go::installed`.
    fn installed(&self) -> HashSet<String>;
}

// ── npm ───────────────────────────────────────────────────────────────────────

pub struct Npm;

/// Parse `npm ls -g --depth=0 --json` output: a `{"dependencies": {"<pkg>":
/// {...}, ...}}` object whose top-level keys are the installed package names.
fn parse_npm_installed(output: &str) -> HashSet<String> {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()
        .and_then(|v| v.get("dependencies").cloned())
        .and_then(|deps| deps.as_object().cloned())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

impl ToolchainManager for Npm {
    fn name(&self) -> &str {
        "npm"
    }
    fn field_name(&self) -> &str {
        "npm"
    }
    fn is_available(&self) -> bool {
        check_command_available("npm")
    }
    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("npm", &["ls", "-g", "--depth=0", "--json"])
            .map(|out| parse_npm_installed(&out))
            .unwrap_or_default()
    }
}

// ── cargo ─────────────────────────────────────────────────────────────────────

pub struct Cargo;

/// Parse `cargo install --list` output. Each installed crate starts a new,
/// non-indented line shaped `"<crate> v<version>:"`, followed by one or more
/// indented lines naming its installed binaries — only the crate name (a
/// header line's first whitespace-separated token) is an identifier here.
fn parse_cargo_installed(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with(char::is_whitespace))
        .filter_map(|l| l.split_whitespace().next())
        .map(String::from)
        .collect()
}

impl ToolchainManager for Cargo {
    fn name(&self) -> &str {
        "cargo"
    }
    fn field_name(&self) -> &str {
        "cargo"
    }
    fn is_available(&self) -> bool {
        check_command_available("cargo")
    }
    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("cargo", &["install", "--list"])
            .map(|out| parse_cargo_installed(&out))
            .unwrap_or_default()
    }
}

// ── gem ───────────────────────────────────────────────────────────────────────

pub struct Gem;

/// Parse `gem list --local` output: one gem per line, shaped
/// `"<name> (<version>[, <version>...])"` — only the leading name token is
/// an identifier here.
fn parse_gem_installed(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

impl ToolchainManager for Gem {
    fn name(&self) -> &str {
        "gem"
    }
    fn field_name(&self) -> &str {
        "gem"
    }
    fn is_available(&self) -> bool {
        check_command_available("gem")
    }
    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("gem", &["list", "--local"])
            .map(|out| parse_gem_installed(&out))
            .unwrap_or_default()
    }
}

// ── pip (via pipx) ────────────────────────────────────────────────────────────

pub struct Pip;

/// Parse `pipx list --json` output: a `{"venvs": {"<pkg>": {...}, ...}}`
/// object whose top-level keys under `venvs` are the installed tool names.
fn parse_pipx_installed(output: &str) -> HashSet<String> {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()
        .and_then(|v| v.get("venvs").cloned())
        .and_then(|venvs| venvs.as_object().cloned())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

impl ToolchainManager for Pip {
    fn name(&self) -> &str {
        "pip"
    }
    fn field_name(&self) -> &str {
        "pip"
    }
    fn is_available(&self) -> bool {
        // Installs go through pipx rather than raw `pip install --user` —
        // pipx isolates each CLI tool in its own venv, avoiding the classic
        // "installing tool B breaks tool A" problem with global pip installs.
        check_command_available("pipx")
    }
    fn installed(&self) -> HashSet<String> {
        if !self.is_available() {
            return HashSet::new();
        }
        capture_stdout("pipx", &["list", "--json"])
            .map(|out| parse_pipx_installed(&out))
            .unwrap_or_default()
    }
}

// ── go ────────────────────────────────────────────────────────────────────────

pub struct Go;

impl ToolchainManager for Go {
    fn name(&self) -> &str {
        "go"
    }
    fn field_name(&self) -> &str {
        "go"
    }
    fn is_available(&self) -> bool {
        // `go` doesn't support `--version` the way most other CLIs here do
        // (check_command_available assumes that flag) — it uses a `version`
        // subcommand instead.
        std::process::Command::new("go")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    /// Always empty. This is a real, permanent constraint of the Go
    /// toolchain, not a TODO to resolve later: `go install` retains no
    /// system-level record of what it has installed, unlike npm/cargo/gem/
    /// pipx (each has its own "list installed" command this module parses
    /// above). There is no directory or registry heimdal can enumerate to
    /// answer "what has `go install` put on this machine" — a Go binary on
    /// `$GOBIN` doesn't even record the module path it came from anywhere
    /// heimdal could recover without shelling out per-binary and hoping
    /// `go version -m` covers every case.
    ///
    /// Because of this, `plan_toolchain_work` never calls this method for
    /// the "go" field — the go install decision is made purely from
    /// heimdal's own `state.package_inventory["go"]` (see
    /// `maybe_skip_go_installed`), which is why the earlier state-tracking
    /// PR's `package_inventory` field is not optional for Go the way it's
    /// merely a nice-to-have accuracy improvement for every other manager.
    fn installed(&self) -> HashSet<String> {
        HashSet::new()
    }
}

// ── Public interface ──────────────────────────────────────────────────────────

/// Returns `(cmd, args)` for a toolchain manager field name.
fn toolchain_manager_cmd(field: &str) -> (String, Vec<String>) {
    match field {
        "npm" => (
            "npm".to_string(),
            vec!["install".to_string(), "-g".to_string()],
        ),
        "cargo" => ("cargo".to_string(), vec!["install".to_string()]),
        "go" => ("go".to_string(), vec!["install".to_string()]),
        "gem" => ("gem".to_string(), vec!["install".to_string()]),
        "pip" => ("pipx".to_string(), vec!["install".to_string()]),
        _ => (field.to_string(), vec![]),
    }
}

/// Extract the declared package list for a given toolchain field from a
/// `ToolchainMap`. Unlike `packages::get_manager_packages`, every entry here
/// (across all five managers) is used verbatim as both the install-command
/// argument and the display name — there's no mas-style id/name split.
fn get_toolchain_packages(field: &str, tc: &crate::config::ToolchainMap) -> Vec<String> {
    match field {
        "npm" => tc.npm.clone(),
        "cargo" => tc.cargo.clone(),
        "go" => tc.go.clone(),
        "gem" => tc.gem.clone(),
        "pip" => tc.pip.clone(),
        _ => vec![],
    }
}

/// Go-specific skip check. Unlike every other toolchain manager, Go has no
/// live "what's installed" query (see `Go::installed`), so whether a
/// go-install entry is already present is decided purely from heimdal's own
/// `state.package_inventory["go"]` — never a system query. `force` bypasses
/// this entirely, matching every other manager's `--force` behavior.
fn maybe_skip_go_installed(
    entry: &str,
    force: bool,
    state: &crate::state::State,
    stage: &crate::progress::StageBar,
) -> bool {
    if force {
        return false;
    }
    let already_tracked = state
        .package_inventory
        .get("go")
        .map(|inv| inv.identifiers.contains(entry))
        .unwrap_or(false);
    if already_tracked {
        stage.println(format!(
            "         · skip       {} — already installed (per heimdal's own record; \
            `go install` has no live \"what's installed\" check)",
            entry
        ));
    }
    already_tracked
}

/// Decide which declared toolchain packages still need their install command
/// to run, mirroring `packages::plan_work` — with one deliberate divergence:
/// the "go" field's skip decision consults `state.package_inventory["go"]`
/// directly (via `maybe_skip_go_installed`) instead of a live system query,
/// since none exists for Go.
///
/// Every declared `toolchains.go` entry is validated up front: `go install`
/// requires an explicit `@version` suffix, and a missing one must fail
/// clearly here rather than mis-invoking `go install <bare-path>`.
#[allow(clippy::type_complexity)]
fn plan_toolchain_work(
    toolchains: &crate::config::ToolchainMap,
    managers: &[Box<dyn ToolchainManager>],
    force: bool,
    state: &crate::state::State,
    stage: &crate::progress::StageBar,
    dry_run: bool,
) -> anyhow::Result<(Vec<WorkItem>, Vec<(String, String)>)> {
    for entry in &toolchains.go {
        crate::config::validate_go_module_entry(entry).map_err(|e| anyhow::anyhow!(e))?;
    }

    // Skip the live query entirely when forcing: every package installs
    // regardless of what's already present, so there's nothing to check.
    // (Also skipped for "go" always, since Go never has a live query.)
    let installed_sets: HashMap<String, HashSet<String>> = if force {
        HashMap::new()
    } else {
        managers
            .iter()
            .filter(|m| m.field_name() != "go" && m.is_available())
            .map(|m| (m.field_name().to_string(), m.installed()))
            .collect()
    };

    let mut work: Vec<WorkItem> = Vec::new();
    let mut newly_tracked: Vec<(String, String)> = Vec::new();

    for manager in managers {
        let field = manager.field_name();
        let declared = get_toolchain_packages(field, toolchains);
        if declared.is_empty() {
            continue;
        }
        if !manager.is_available() {
            crate::utils::warning(&format!(
                "Toolchain manager '{}' not available. Skipping {} package(s).",
                manager.name(),
                declared.len()
            ));
            continue;
        }
        let (cmd, args) = toolchain_manager_cmd(field);
        for pkg in declared {
            let skip = if field == "go" {
                maybe_skip_go_installed(&pkg, force, state, stage)
            } else {
                maybe_skip_installed(
                    field,
                    &pkg,
                    &pkg,
                    force,
                    &installed_sets,
                    state,
                    stage,
                    &mut newly_tracked,
                )
            };
            if skip {
                continue;
            }
            work.push(WorkItem {
                cmd: cmd.clone(),
                args: args.clone(),
                pkg: pkg.clone(),
                manager_field: field.to_string(),
                display_name: pkg,
                dry_run,
            });
        }
    }

    Ok((work, newly_tracked))
}

/// Install every declared language-toolchain package for the active profile
/// — across all of npm, cargo, go, gem, and pip that are actually available
/// on this machine, not just the first one found (see the module docs for
/// why that's correct here but wrong for OS-level `packages`).
///
/// Mirrors `packages::install_for_profile`: every package that installs
/// successfully (outside of `dry_run`) is recorded into `state`'s package
/// inventory, keyed the same way ("npm", "cargo", "go", "gem", "pip") —
/// callers are responsible for calling `state.save()` afterwards.
///
/// Returns an error immediately if any declared `toolchains.go` entry is
/// missing its required `@version` suffix, before anything is installed.
pub fn install_toolchains_for_profile(
    profile: &crate::config::Profile,
    dry_run: bool,
    stage: &crate::progress::StageBar,
    parallel_jobs: usize,
    force: bool,
    state: &mut crate::state::State,
) -> anyhow::Result<Vec<crate::progress::FailedPackage>> {
    let managers: Vec<Box<dyn ToolchainManager>> = vec![
        Box::new(Npm),
        Box::new(Cargo),
        Box::new(Go),
        Box::new(Gem),
        Box::new(Pip),
    ];

    let (work, newly_tracked) =
        plan_toolchain_work(&profile.toolchains, &managers, force, state, stage, dry_run)?;

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
    let (failures, successes) = crate::packages::run_parallel(work, bar, parallel_jobs);

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
    use crate::config::{Profile, ToolchainMap};
    use crate::progress::ApplyProgress;

    fn make_stage() -> (ApplyProgress, crate::progress::StageBar) {
        let p = ApplyProgress::new(5);
        let s = p.stage(2, "Installing packages");
        (p, s)
    }

    // ── Parsing ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_npm_installed_extracts_dependency_keys() {
        let output =
            r#"{"dependencies":{"typescript":{"version":"5.4.0"},"eslint":{"version":"8.57.0"}}}"#;
        let installed = parse_npm_installed(output);
        assert_eq!(installed.len(), 2);
        assert!(installed.contains("typescript"));
        assert!(installed.contains("eslint"));
    }

    #[test]
    fn test_parse_npm_installed_empty_dependencies() {
        let output = r#"{"dependencies":{}}"#;
        assert!(parse_npm_installed(output).is_empty());
    }

    #[test]
    fn test_parse_npm_installed_malformed_json_yields_empty() {
        assert!(parse_npm_installed("not json").is_empty());
    }

    #[test]
    fn test_parse_cargo_installed_takes_crate_name_from_header_line() {
        let output = "cargo-audit v0.17.6:\n    cargo-audit\nripgrep v13.0.0:\n    rg\n";
        let installed = parse_cargo_installed(output);
        assert_eq!(installed.len(), 2);
        assert!(installed.contains("cargo-audit"));
        assert!(installed.contains("ripgrep"));
        // Indented binary names must not leak in as crate identifiers.
        assert!(!installed.contains("rg"));
    }

    #[test]
    fn test_parse_gem_installed_one_name_per_line() {
        let output = "bundler (2.4.10)\nrake (13.0.6, 13.0.3)\n";
        let installed = parse_gem_installed(output);
        assert_eq!(installed.len(), 2);
        assert!(installed.contains("bundler"));
        assert!(installed.contains("rake"));
    }

    #[test]
    fn test_parse_pipx_installed_extracts_venv_keys() {
        let output = r#"{"venvs":{"black":{},"httpie":{}}}"#;
        let installed = parse_pipx_installed(output);
        assert_eq!(installed.len(), 2);
        assert!(installed.contains("black"));
        assert!(installed.contains("httpie"));
    }

    // ── Install/skip logic ───────────────────────────────────────────────────

    /// A `ToolchainManager` stand-in for tests: no real subprocess is ever
    /// spawned — `installed()` returns whatever set the test configures.
    struct MockToolchainManager {
        name: &'static str,
        field: &'static str,
        available: bool,
        installed_set: HashSet<String>,
    }

    impl ToolchainManager for MockToolchainManager {
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

    fn mock_npm(installed: &[&str]) -> Box<dyn ToolchainManager> {
        Box::new(MockToolchainManager {
            name: "npm",
            field: "npm",
            available: true,
            installed_set: installed.iter().map(|s| s.to_string()).collect(),
        })
    }

    fn mock_cargo(installed: &[&str]) -> Box<dyn ToolchainManager> {
        Box::new(MockToolchainManager {
            name: "cargo",
            field: "cargo",
            available: true,
            installed_set: installed.iter().map(|s| s.to_string()).collect(),
        })
    }

    #[test]
    fn test_plan_toolchain_work_skips_already_installed_npm_package() {
        let (_p, stage) = make_stage();
        let toolchains = ToolchainMap {
            npm: vec!["typescript".to_string()],
            ..Default::default()
        };
        let managers = vec![mock_npm(&["typescript"])];
        let state = crate::state::State::default();
        let (work, newly_tracked) =
            plan_toolchain_work(&toolchains, &managers, false, &state, &stage, false).unwrap();
        assert!(
            work.is_empty(),
            "an already-installed npm package must not enqueue an install command"
        );
        assert_eq!(
            newly_tracked,
            vec![("npm".to_string(), "typescript".to_string())]
        );
    }

    #[test]
    fn test_plan_toolchain_work_installs_not_yet_installed_cargo_package() {
        let (_p, stage) = make_stage();
        let toolchains = ToolchainMap {
            cargo: vec!["ripgrep".to_string()],
            ..Default::default()
        };
        let managers = vec![mock_cargo(&[])];
        let state = crate::state::State::default();
        let (work, newly_tracked) =
            plan_toolchain_work(&toolchains, &managers, false, &state, &stage, false).unwrap();
        assert_eq!(work.len(), 1);
        assert_eq!(work[0].pkg, "ripgrep");
        assert_eq!(work[0].manager_field, "cargo");
        assert!(newly_tracked.is_empty());
    }

    #[test]
    fn test_plan_toolchain_work_force_bypasses_skip() {
        let (_p, stage) = make_stage();
        let toolchains = ToolchainMap {
            npm: vec!["typescript".to_string()],
            ..Default::default()
        };
        let managers = vec![mock_npm(&["typescript"])];
        let state = crate::state::State::default();
        let (work, newly_tracked) =
            plan_toolchain_work(&toolchains, &managers, true, &state, &stage, false).unwrap();
        assert_eq!(
            work.len(),
            1,
            "--force must reinstall an already-present package"
        );
        assert!(newly_tracked.is_empty());
    }

    #[test]
    fn test_plan_toolchain_work_unavailable_manager_skips_without_panic() {
        let (_p, stage) = make_stage();
        let toolchains = ToolchainMap {
            npm: vec!["typescript".to_string()],
            ..Default::default()
        };
        let managers: Vec<Box<dyn ToolchainManager>> = vec![Box::new(MockToolchainManager {
            name: "npm",
            field: "npm",
            available: false,
            installed_set: HashSet::new(),
        })];
        let state = crate::state::State::default();
        let (work, _) =
            plan_toolchain_work(&toolchains, &managers, false, &state, &stage, false).unwrap();
        assert!(work.is_empty());
    }

    // ── Go: state-based skip, not a live query ───────────────────────────────

    #[test]
    fn test_go_installed_always_returns_empty_set() {
        // Regardless of whether `go` is even on this machine, `installed()`
        // must never claim to know what's installed — see the doc comment.
        assert!(Go.installed().is_empty());
    }

    #[test]
    fn test_maybe_skip_go_installed_uses_state_not_live_query() {
        let (_p, stage) = make_stage();
        let mut state = crate::state::State::default();
        state.record_installed(
            "go",
            ["golang.org/x/tools/cmd/goimports@latest".to_string()],
        );

        assert!(maybe_skip_go_installed(
            "golang.org/x/tools/cmd/goimports@latest",
            false,
            &state,
            &stage
        ));
        assert!(!maybe_skip_go_installed(
            "github.com/bufbuild/buf/cmd/buf@v1.32.0",
            false,
            &state,
            &stage
        ));
    }

    #[test]
    fn test_maybe_skip_go_installed_force_bypasses_tracked_state() {
        let (_p, stage) = make_stage();
        let mut state = crate::state::State::default();
        state.record_installed(
            "go",
            ["golang.org/x/tools/cmd/goimports@latest".to_string()],
        );
        assert!(!maybe_skip_go_installed(
            "golang.org/x/tools/cmd/goimports@latest",
            true,
            &state,
            &stage
        ));
    }

    #[test]
    fn test_plan_toolchain_work_rejects_go_entry_missing_version() {
        let (_p, stage) = make_stage();
        let toolchains = ToolchainMap {
            go: vec!["golang.org/x/tools/cmd/goimports".to_string()],
            ..Default::default()
        };
        let managers: Vec<Box<dyn ToolchainManager>> = vec![Box::new(Go)];
        let state = crate::state::State::default();
        let result = plan_toolchain_work(&toolchains, &managers, false, &state, &stage, false);
        match result {
            Ok(_) => panic!("a go entry missing @version must be rejected up front"),
            Err(e) => assert!(e.to_string().contains("@version")),
        }
    }

    #[test]
    fn test_install_toolchains_for_profile_dry_run_no_failures() {
        let (_p, stage) = make_stage();
        let profile = Profile {
            toolchains: ToolchainMap {
                npm: vec!["typescript".to_string()],
                cargo: vec!["ripgrep".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let mut state = crate::state::State::default();
        let result = install_toolchains_for_profile(&profile, true, &stage, 2, false, &mut state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        assert!(
            state.package_inventory.is_empty(),
            "dry run must not record anything as actually installed"
        );
    }

    #[test]
    fn test_install_toolchains_for_profile_empty_toolchains_is_a_noop() {
        let (_p, stage) = make_stage();
        let profile = Profile::default();
        let mut state = crate::state::State::default();
        let result = install_toolchains_for_profile(&profile, false, &stage, 4, false, &mut state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
