use crate::cli::StatusArgs;
use crate::config::{load_config, resolve_profile, Profile};
use crate::git::{GitRepo, GitStatus};
use crate::state::State;
use crate::utils::{info, success, warning};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub fn run(args: StatusArgs) -> Result<()> {
    let state = State::load()?;

    // Machine + profile info
    info(&format!("Profile:       {}", state.active_profile));
    info(&format!("Dotfiles:      {}", state.dotfiles_path.display()));
    info(&format!("Hostname:      {}", state.hostname));
    info(&format!("OS:            {}", state.os));

    if let Some(t) = state.last_apply {
        info(&format!(
            "Last apply:    {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        info("Last apply:    never");
    }
    if let Some(t) = state.last_sync {
        info(&format!(
            "Last sync:     {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        info("Last sync:     never");
    }

    // Git status
    let repo = GitRepo::open(&state.dotfiles_path);
    match repo.status() {
        Ok(files) if files.is_empty() => success("Working tree clean"),
        Ok(files) => {
            warning(&format!("{} uncommitted change(s):", files.len()));
            for f in &files {
                let marker = match f.status {
                    GitStatus::Modified => "M",
                    GitStatus::Added => "A",
                    GitStatus::Deleted => "D",
                    GitStatus::Untracked => "?",
                };
                info(&format!("  {} {}", marker, f.path));
            }
        }
        Err(e) => warning(&format!("Could not read git status: {}", e)),
    }

    if args.packages {
        println!();
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, &state.active_profile)?;
        print_package_drift(&profile);
    }

    Ok(())
}

/// Manager `(label, PackageManager::field_name())` pairs, in display order.
/// `label` matches the naming convention `commands::packages::list()` uses
/// (hyphenated for the cask field), extended to include `mas` — which that
/// command's sections currently omit.
const MANAGER_LABELS: [(&str, &str); 7] = [
    ("homebrew", "homebrew"),
    ("homebrew-cask", "homebrew_casks"),
    ("apt", "apt"),
    ("dnf", "dnf"),
    ("pacman", "pacman"),
    ("apk", "apk"),
    ("mas", "mas"),
];

/// One manager's package drift: identifiers declared in yaml but not
/// actually installed (`missing`), and identifiers actually installed but
/// not declared anywhere in yaml — neither the active profile nor the
/// top-level shared `packages` block (`untracked`). Both lists are sorted
/// for stable output.
pub struct ManagerDrift {
    pub label: &'static str,
    pub missing: Vec<String>,
    pub untracked: Vec<String>,
}

/// Compare `declared` (see `packages::declared_identifiers`) against
/// `installed` (see `packages::query_installed`) for every known manager,
/// returning only the managers that actually have drift.
pub fn compute_package_drift(
    declared: &HashMap<String, HashSet<String>>,
    installed: &HashMap<String, HashSet<String>>,
) -> Vec<ManagerDrift> {
    let empty: HashSet<String> = HashSet::new();
    MANAGER_LABELS
        .iter()
        .filter_map(|(label, field)| {
            let declared_set = declared.get(*field).unwrap_or(&empty);
            let installed_set = installed.get(*field).unwrap_or(&empty);

            let mut missing: Vec<String> =
                declared_set.difference(installed_set).cloned().collect();
            let mut untracked: Vec<String> =
                installed_set.difference(declared_set).cloned().collect();
            if missing.is_empty() && untracked.is_empty() {
                return None;
            }
            missing.sort();
            untracked.sort();
            Some(ManagerDrift {
                label,
                missing,
                untracked,
            })
        })
        .collect()
}

/// Query the machine's real package state and print drift against `profile`
/// (already resolved — extends chain and top-level shared `packages` block
/// merged in, per `config::resolve_profile`).
fn print_package_drift(profile: &Profile) {
    // Mirrors the manager `install_for_profile` would actually run `common`
    // packages through, so a `common` entry isn't reported as drift twice
    // (once bare, once resolved) or missed entirely.
    let common_field = crate::packages::detect_manager().map(|m| m.field_name().to_string());
    let declared =
        crate::packages::declared_identifiers(&profile.packages, common_field.as_deref());
    let installed = crate::packages::query_installed();
    let drift = compute_package_drift(&declared, &installed);

    if drift.is_empty() {
        success("No package drift detected.");
        return;
    }

    warning("Package drift detected:");
    for md in &drift {
        info(&format!("{}:", md.label));
        if !md.missing.is_empty() {
            info("  missing (declared, not installed):");
            for pkg in &md.missing {
                info(&format!("    - {}", pkg));
            }
        }
        if !md.untracked.is_empty() {
            info("  untracked (installed, not declared):");
            for pkg in &md.untracked {
                info(&format!("    - {}", pkg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_package_drift_no_drift_when_sets_match() {
        let mut declared = HashMap::new();
        declared.insert("homebrew".to_string(), HashSet::from(["git".to_string()]));
        let mut installed = HashMap::new();
        installed.insert("homebrew".to_string(), HashSet::from(["git".to_string()]));

        let drift = compute_package_drift(&declared, &installed);
        assert!(drift.is_empty(), "matching sets must report no drift");
    }

    #[test]
    fn test_compute_package_drift_missing_only() {
        let mut declared = HashMap::new();
        declared.insert(
            "homebrew".to_string(),
            HashSet::from(["git".to_string(), "fzf".to_string()]),
        );
        let mut installed = HashMap::new();
        installed.insert("homebrew".to_string(), HashSet::from(["git".to_string()]));

        let drift = compute_package_drift(&declared, &installed);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].label, "homebrew");
        assert_eq!(drift[0].missing, vec!["fzf".to_string()]);
        assert!(drift[0].untracked.is_empty());
    }

    #[test]
    fn test_compute_package_drift_untracked_only() {
        let declared = HashMap::new();
        let mut installed = HashMap::new();
        installed.insert("apt".to_string(), HashSet::from(["htop".to_string()]));

        let drift = compute_package_drift(&declared, &installed);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].label, "apt");
        assert!(drift[0].missing.is_empty());
        assert_eq!(drift[0].untracked, vec!["htop".to_string()]);
    }

    #[test]
    fn test_compute_package_drift_both_directions_simultaneously() {
        let mut declared = HashMap::new();
        declared.insert("homebrew".to_string(), HashSet::from(["fzf".to_string()]));
        let mut installed = HashMap::new();
        installed.insert("homebrew".to_string(), HashSet::from(["htop".to_string()]));

        let drift = compute_package_drift(&declared, &installed);
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].label, "homebrew");
        assert_eq!(drift[0].missing, vec!["fzf".to_string()]);
        assert_eq!(drift[0].untracked, vec!["htop".to_string()]);
    }

    #[test]
    fn test_compute_package_drift_skips_managers_with_nothing_on_either_side() {
        let declared = HashMap::new();
        let installed = HashMap::new();
        assert!(compute_package_drift(&declared, &installed).is_empty());
    }

    #[test]
    fn test_compute_package_drift_results_are_sorted() {
        let mut declared = HashMap::new();
        declared.insert(
            "apt".to_string(),
            HashSet::from(["zsh".to_string(), "atool".to_string(), "mtr".to_string()]),
        );
        let installed = HashMap::new();

        let drift = compute_package_drift(&declared, &installed);
        assert_eq!(drift.len(), 1);
        assert_eq!(
            drift[0].missing,
            vec!["atool".to_string(), "mtr".to_string(), "zsh".to_string()]
        );
    }
}
