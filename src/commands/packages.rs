use crate::cli::PackagesCmd;
use crate::config::{load_config, write_config, PackageMap};
use crate::state::State;
use crate::utils::{info, success, warning};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub fn run(action: PackagesCmd) -> Result<()> {
    match action {
        PackagesCmd::List { installed } => list(installed),
        PackagesCmd::Add {
            name,
            manager,
            no_install,
            id,
        } => add(&name, manager.as_deref(), no_install, id),
        PackagesCmd::Remove { name, no_uninstall } => remove(&name, no_uninstall),
        PackagesCmd::Search { query } => search(&query),
        PackagesCmd::Info { name } => pkg_info(&name),
    }
}

fn list(installed: bool) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;
    let profile = config.profiles.get(&state.active_profile).ok_or_else(|| {
        crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        }
    })?;

    // `--installed` cross-references declared packages against a real query
    // of the system (see `packages::query_installed`); without it, packages
    // are listed exactly as declared, matching prior behavior.
    let installed_sets = if installed {
        Some(crate::packages::query_installed())
    } else {
        None
    };

    let sections = build_package_sections(&profile.packages, installed_sets.as_ref());
    if sections.is_empty() {
        info("No packages configured for this profile.");
        return Ok(());
    }
    for (label, lines) in sections {
        println!("{}:", label);
        for line in lines {
            println!("{}", line);
        }
    }
    Ok(())
}

/// Build the `label -> display lines` sections for `packages list`.
///
/// When `installed_sets` is `Some` (i.e. `--installed` was passed), each
/// declared package is annotated `(installed)` / `(missing)` by
/// cross-referencing it against that manager's real installed-identifier
/// set. When `None`, packages are listed as declared with no annotation,
/// matching the format `packages list` has always used.
pub fn build_package_sections(
    pkgs: &PackageMap,
    installed_sets: Option<&HashMap<String, HashSet<String>>>,
) -> Vec<(&'static str, Vec<String>)> {
    let manager_sections: [(&'static str, &Vec<String>, &str); 6] = [
        ("homebrew", &pkgs.homebrew, "homebrew"),
        ("homebrew-cask", &pkgs.homebrew_casks, "homebrew_casks"),
        ("apt", &pkgs.apt, "apt"),
        ("dnf", &pkgs.dnf, "dnf"),
        ("pacman", &pkgs.pacman, "pacman"),
        ("apk", &pkgs.apk, "apk"),
    ];

    manager_sections
        .into_iter()
        .filter(|(_, list, _)| !list.is_empty())
        .map(|(label, list, field)| {
            let lines = list
                .iter()
                .map(|p| format_package_line(p, field, installed_sets))
                .collect();
            (label, lines)
        })
        .collect()
}

/// Format one `packages list` line for `pkg`, declared under the manager
/// identified by `manager_field` (a `PackageManager::field_name()`).
fn format_package_line(
    pkg: &str,
    manager_field: &str,
    installed_sets: Option<&HashMap<String, HashSet<String>>>,
) -> String {
    match installed_sets {
        None => format!("  - {}", pkg),
        Some(sets) => {
            let is_installed = sets
                .get(manager_field)
                .map(|ids| ids.contains(pkg))
                .unwrap_or(false);
            format!(
                "  - {} ({})",
                pkg,
                if is_installed { "installed" } else { "missing" }
            )
        }
    }
}

fn add(pkg: &str, manager: Option<&str>, no_install: bool, id: Option<u64>) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    // Determine which manager field to update
    let mgr = match manager {
        Some(m) => m.to_string(),
        None => {
            // Auto-detect
            crate::packages::detect_manager()
                .map(|m| m.field_name().to_string())
                .unwrap_or_else(|| {
                    warning("No package manager detected, defaulting to 'homebrew'");
                    "homebrew".to_string()
                })
        }
    };

    let profile = config
        .profiles
        .get_mut(&state.active_profile)
        .ok_or_else(|| crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        })?;

    if mgr == "mas" {
        // mas entries are objects ({ id, name }), not plain strings, so they
        // need their own add path with a required App Store ID.
        let app_id = id.ok_or_else(|| {
            anyhow::anyhow!(
                "Missing required '--id <APP_STORE_ID>' for package manager 'mas'. \
                Example: heimdal packages add \"Slack\" --manager mas --id 803453959"
            )
        })?;

        let already = profile
            .packages
            .mas
            .iter()
            .any(|v| v.get("id").and_then(|i| i.as_u64()) == Some(app_id));
        if already {
            info(&format!(
                "App Store id '{}' is already in the mas package list.",
                app_id
            ));
            return Ok(());
        }

        profile
            .packages
            .mas
            .push(serde_json::json!({ "id": app_id, "name": pkg }));
        write_config(&config_path, &config)?;
        success(&format!("Added '{}' (id {}) to mas packages", pkg, app_id));

        if !no_install {
            info(&format!("Run 'heimdal apply' to install '{}'.", pkg));
        }
        return Ok(());
    }

    let list = match mgr.as_str() {
        "homebrew" => &mut profile.packages.homebrew,
        "homebrew_casks" | "homebrew-cask" => &mut profile.packages.homebrew_casks,
        "apt" => &mut profile.packages.apt,
        "dnf" => &mut profile.packages.dnf,
        "pacman" => &mut profile.packages.pacman,
        "apk" => &mut profile.packages.apk,
        other => anyhow::bail!(
            "Unknown package manager '{}'. Valid: homebrew, apt, dnf, pacman, apk, mas",
            other
        ),
    };

    if list.contains(&pkg.to_string()) {
        info(&format!(
            "'{}' is already in the {} package list.",
            pkg, mgr
        ));
        return Ok(());
    }

    list.push(pkg.to_string());
    write_config(&config_path, &config)?;
    success(&format!("Added '{}' to {} packages", pkg, mgr));

    if !no_install {
        info(&format!("Run 'heimdal apply' to install '{}'.", pkg));
    }
    Ok(())
}

fn remove(pkg: &str, no_uninstall: bool) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    let profile = config
        .profiles
        .get_mut(&state.active_profile)
        .ok_or_else(|| crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        })?;

    let mut removed = false;
    macro_rules! remove_from {
        ($field:expr) => {
            let before = $field.len();
            $field.retain(|p| p != pkg);
            if $field.len() < before {
                removed = true;
            }
        };
    }

    remove_from!(profile.packages.homebrew);
    remove_from!(profile.packages.homebrew_casks);
    remove_from!(profile.packages.apt);
    remove_from!(profile.packages.dnf);
    remove_from!(profile.packages.pacman);
    remove_from!(profile.packages.apk);

    // mas entries are objects ({ id, name }) rather than plain strings, so
    // match against either the app name or the numeric App Store id.
    let mas_before = profile.packages.mas.len();
    profile.packages.mas.retain(|v| {
        let name_matches = v.get("name").and_then(|n| n.as_str()) == Some(pkg);
        let id_matches = v
            .get("id")
            .and_then(|i| i.as_u64())
            .map(|i| i.to_string() == pkg)
            .unwrap_or(false);
        !(name_matches || id_matches)
    });
    if profile.packages.mas.len() < mas_before {
        removed = true;
    }

    if removed {
        write_config(&config_path, &config)?;
        success(&format!("Removed '{}' from config", pkg));
        if !no_uninstall {
            warning(&format!(
                "'{}' was removed from config but NOT uninstalled from your system. \
                Run the appropriate uninstall command manually if needed.",
                pkg
            ));
        }
    } else {
        info(&format!("'{}' was not found in any package list.", pkg));
    }
    Ok(())
}

fn search(query: &str) -> Result<()> {
    let brew_args = ["search", query];
    let apt_args = ["search", query];
    let dnf_args = ["search", query];
    let pacman_args = ["-Ss", query];
    let apk_args = ["search", query];

    let managers: Vec<(&str, &[&str], &str)> = vec![
        ("brew", &brew_args[..], "macOS/Linux (Homebrew)"),
        ("apt-cache", &apt_args[..], "Debian/Ubuntu (apt)"),
        ("dnf", &dnf_args[..], "Fedora/RHEL (dnf)"),
        ("pacman", &pacman_args[..], "Arch Linux (pacman)"),
        ("apk", &apk_args[..], "Alpine (apk)"),
    ];

    for (cmd, args, label) in &managers {
        let available = std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if available {
            info(&format!("Searching {} packages for '{}'...", label, query));
            let status = std::process::Command::new(cmd)
                .args(*args)
                .status()
                .map_err(|e| anyhow::anyhow!("Cannot run {}: {}", cmd, e))?;
            if !status.success() {
                warning("Search returned no results or failed.");
            }
            return Ok(());
        }
    }

    info("No package manager detected. Search manually:");
    info(&format!("  Homebrew (macOS/Linux): brew search {}", query));
    info(&format!(
        "  APT (Debian/Ubuntu):    apt-cache search {}",
        query
    ));
    info(&format!("  DNF (Fedora/RHEL):      dnf search {}", query));
    info(&format!("  Pacman (Arch):          pacman -Ss {}", query));
    info(&format!("  APK (Alpine):           apk search {}", query));
    Ok(())
}

fn pkg_info(name: &str) -> Result<()> {
    let brew_args = ["info", name];
    let apt_args = ["show", name];
    let dnf_args = ["info", name];
    let pacman_args = ["-Si", name];
    let apk_args = ["info", name];

    let managers: Vec<(&str, &[&str], &str)> = vec![
        ("brew", &brew_args[..], "Homebrew"),
        ("apt-cache", &apt_args[..], "APT"),
        ("dnf", &dnf_args[..], "DNF"),
        ("pacman", &pacman_args[..], "Pacman"),
        ("apk", &apk_args[..], "APK"),
    ];

    for (cmd, args, label) in &managers {
        let available = std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if available {
            info(&format!("Package info from {} for '{}':", label, name));
            std::process::Command::new(cmd).args(*args).status().ok();
            return Ok(());
        }
    }

    info("No package manager detected. Get info manually:");
    info(&format!("  brew info {}", name));
    info(&format!("  apt-cache show {}", name));
    info(&format!("  dnf info {}", name));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pkgs() -> PackageMap {
        PackageMap {
            homebrew: vec!["git".to_string(), "curl".to_string()],
            apt: vec!["vim".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn test_build_package_sections_without_installed_flag_lists_plain() {
        let pkgs = sample_pkgs();
        let sections = build_package_sections(&pkgs, None);

        let homebrew = sections
            .iter()
            .find(|(label, _)| *label == "homebrew")
            .unwrap();
        assert_eq!(
            homebrew.1,
            vec!["  - git".to_string(), "  - curl".to_string()]
        );
    }

    #[test]
    fn test_build_package_sections_skips_empty_managers() {
        let pkgs = sample_pkgs();
        let sections = build_package_sections(&pkgs, None);
        // pacman/apk/dnf/homebrew_casks were never declared, so they must
        // not appear at all — matching the pre-existing list() behavior.
        assert!(!sections.iter().any(|(label, _)| *label == "pacman"));
        assert!(!sections.iter().any(|(label, _)| *label == "apk"));
        assert!(!sections.iter().any(|(label, _)| *label == "dnf"));
    }

    // Covers "packages list --installed" against a mocked/injected installed
    // set, so this test never has to shell out to a real brew/apt/etc.
    #[test]
    fn test_build_package_sections_with_installed_flag_marks_installed_and_missing() {
        let pkgs = sample_pkgs();

        let mut installed_sets: HashMap<String, HashSet<String>> = HashMap::new();
        // Pretend the system genuinely has `git` (via homebrew) but not
        // `curl`, and doesn't have apt available at all.
        installed_sets.insert("homebrew".to_string(), HashSet::from(["git".to_string()]));

        let sections = build_package_sections(&pkgs, Some(&installed_sets));

        let homebrew = sections
            .iter()
            .find(|(label, _)| *label == "homebrew")
            .unwrap();
        assert_eq!(
            homebrew.1,
            vec![
                "  - git (installed)".to_string(),
                "  - curl (missing)".to_string(),
            ]
        );

        // apt wasn't in `installed_sets` at all (manager unavailable) — its
        // declared package must still show up, just as missing.
        let apt = sections.iter().find(|(label, _)| *label == "apt").unwrap();
        assert_eq!(apt.1, vec!["  - vim (missing)".to_string()]);
    }
}
