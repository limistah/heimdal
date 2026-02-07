use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;
use std::fs;
use std::path::Path;

use crate::config::{self, schema::HeimdallConfig};
use crate::package::dependencies::DependencyAnalyzer;
use crate::state::HeimdallState;
use crate::utils::{error, header, info, success, warning};

/// Run the packages remove command
pub fn run_remove(
    package_name: &str,
    profile: Option<&str>,
    force: bool,
    no_uninstall: bool,
) -> Result<()> {
    header(&format!("Removing Package: {}", package_name));

    // Load state to get dotfiles path and current profile
    let state = HeimdallState::load()?;
    let profile_name = profile.unwrap_or(&state.active_profile);

    info(&format!("Profile: {}", profile_name));

    // Load config
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found: {}\nRun 'heimdal init' first.",
            config_path.display()
        );
    }

    let mut config = config::load_config(&config_path)?;

    // Verify profile exists
    if !config.profiles.contains_key(profile_name) {
        anyhow::bail!("Profile '{}' not found in config", profile_name);
    }

    // Find package in config
    let resolved = config::resolve_profile(&config, profile_name)?;
    let (found, manager) = find_package_in_config(&resolved, package_name);

    if !found {
        error(&format!(
            "Package '{}' not found in profile '{}'",
            package_name, profile_name
        ));
        info("Use 'heimdal packages list' to see all packages in this profile");
        return Ok(());
    }

    let manager_name = manager.as_deref().unwrap_or("unknown");
    info(&format!(
        "Found '{}' in {} packages",
        package_name, manager_name
    ));

    // Check for dependents (packages that depend on this one)
    if !force {
        let analyzer = DependencyAnalyzer::new();
        let dependents = find_dependents(package_name, &resolved, &analyzer);

        if !dependents.is_empty() {
            println!();
            warning("The following packages depend on this package:");
            println!();
            for dep in &dependents {
                println!("  {} {}", "→".yellow(), dep.cyan());
            }
            println!();

            if !force {
                let confirm = Confirm::new()
                    .with_prompt("Remove anyway? (may break dependent packages)")
                    .default(false)
                    .interact()?;

                if !confirm {
                    info("Cancelled.");
                    return Ok(());
                }
            }
        }
    }

    // Confirm removal
    println!();
    let confirm = Confirm::new()
        .with_prompt(format!(
            "Remove '{}' from profile '{}'?",
            package_name, profile_name
        ))
        .default(true)
        .interact()?;

    if !confirm {
        info("Cancelled.");
        return Ok(());
    }

    // Remove package from config
    remove_package_from_config(&mut config, profile_name, package_name, manager_name)?;

    // Save config
    save_config(&config, &config_path)?;
    success(&format!(
        "Removed '{}' from profile '{}'",
        package_name, profile_name
    ));

    // Uninstall package if not skipped
    if !no_uninstall {
        println!();
        let uninstall_confirm = Confirm::new()
            .with_prompt(format!("Uninstall '{}' from system?", package_name))
            .default(false)
            .interact()?;

        if uninstall_confirm {
            info(&format!(
                "Uninstalling '{}' via {}...",
                package_name, manager_name
            ));

            let uninstall_result = uninstall_package(package_name, manager_name);

            match uninstall_result {
                Ok(_) => {
                    success(&format!("Successfully uninstalled '{}'", package_name));
                }
                Err(e) => {
                    error(&format!("Failed to uninstall '{}': {}", package_name, e));
                    warning("Package was removed from config but uninstallation failed.");
                    warning("You may need to uninstall manually.");
                }
            }
        } else {
            info("Package remains installed on system");
        }
    } else {
        info("Skipped uninstallation (--no-uninstall)");
    }

    Ok(())
}

/// Find package in configuration and return (found, manager)
fn find_package_in_config(
    resolved: &config::profile::ResolvedProfile,
    package_name: &str,
) -> (bool, Option<String>) {
    // Check homebrew
    if let Some(homebrew) = &resolved.sources.homebrew {
        if homebrew.packages.contains(&package_name.to_string()) {
            return (true, Some("homebrew".to_string()));
        }
        if homebrew.casks.contains(&package_name.to_string()) {
            return (true, Some("homebrew-cask".to_string()));
        }
    }

    // Check apt
    if let Some(apt) = &resolved.sources.apt {
        if apt.packages.contains(&package_name.to_string()) {
            return (true, Some("apt".to_string()));
        }
    }

    // Check dnf
    if let Some(dnf) = &resolved.sources.dnf {
        if dnf.packages.contains(&package_name.to_string()) {
            return (true, Some("dnf".to_string()));
        }
    }

    // Check pacman
    if let Some(pacman) = &resolved.sources.pacman {
        if pacman.packages.contains(&package_name.to_string()) {
            return (true, Some("pacman".to_string()));
        }
    }

    (false, None)
}

/// Find packages that depend on the given package
fn find_dependents(
    package_name: &str,
    resolved: &config::profile::ResolvedProfile,
    analyzer: &DependencyAnalyzer,
) -> Vec<String> {
    let mut dependents = Vec::new();

    // Collect all packages in the profile
    let mut all_packages = Vec::new();

    if let Some(homebrew) = &resolved.sources.homebrew {
        all_packages.extend(homebrew.packages.clone());
        all_packages.extend(homebrew.casks.clone());
    }
    if let Some(apt) = &resolved.sources.apt {
        all_packages.extend(apt.packages.clone());
    }
    if let Some(dnf) = &resolved.sources.dnf {
        all_packages.extend(dnf.packages.clone());
    }
    if let Some(pacman) = &resolved.sources.pacman {
        all_packages.extend(pacman.packages.clone());
    }

    // Check each package to see if it depends on the package being removed
    for pkg in all_packages {
        if pkg == package_name {
            continue;
        }

        let analysis = analyzer.analyze(std::slice::from_ref(&pkg));
        for dep in &analysis.required_missing {
            if dep.dependency.package == package_name {
                dependents.push(pkg.clone());
                break;
            }
        }
        for dep in &analysis.optional_missing {
            if dep.dependency.package == package_name {
                dependents.push(pkg.clone());
                break;
            }
        }
    }

    dependents
}

/// Remove package from configuration
fn remove_package_from_config(
    config: &mut HeimdallConfig,
    _profile_name: &str,
    package_name: &str,
    manager: &str,
) -> Result<()> {
    // Remove from root sources
    match manager {
        "homebrew" | "homebrew-cask" => {
            if let Some(homebrew) = &mut config.sources.homebrew {
                homebrew.packages.retain(|p| p != package_name);
                homebrew.casks.retain(|p| p != package_name);
            }
        }
        "apt" => {
            if let Some(apt) = &mut config.sources.apt {
                apt.packages.retain(|p| p != package_name);
            }
        }
        "dnf" => {
            if let Some(dnf) = &mut config.sources.dnf {
                dnf.packages.retain(|p| p != package_name);
            }
        }
        "pacman" => {
            if let Some(pacman) = &mut config.sources.pacman {
                pacman.packages.retain(|p| p != package_name);
            }
        }
        _ => {}
    }

    Ok(())
}

/// Save configuration to file
fn save_config(config: &HeimdallConfig, path: &Path) -> Result<()> {
    let yaml = serde_yaml::to_string(config).context("Failed to serialize config")?;
    fs::write(path, yaml).context("Failed to write config file")?;
    Ok(())
}

/// Uninstall a package using the specified manager
fn uninstall_package(package_name: &str, manager: &str) -> Result<()> {
    let status = match manager {
        "homebrew" | "homebrew-cask" => std::process::Command::new("brew")
            .arg("uninstall")
            .arg(package_name)
            .status()
            .context("Failed to execute brew uninstall")?,
        "apt" => std::process::Command::new("sudo")
            .arg("apt")
            .arg("remove")
            .arg("-y")
            .arg(package_name)
            .status()
            .context("Failed to execute apt remove")?,
        "dnf" => std::process::Command::new("sudo")
            .arg("dnf")
            .arg("remove")
            .arg("-y")
            .arg(package_name)
            .status()
            .context("Failed to execute dnf remove")?,
        "pacman" => std::process::Command::new("sudo")
            .arg("pacman")
            .arg("-R")
            .arg("--noconfirm")
            .arg(package_name)
            .status()
            .context("Failed to execute pacman remove")?,
        _ => anyhow::bail!("Unsupported package manager: {}", manager),
    };

    if !status.success() {
        anyhow::bail!("Package uninstallation failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_package_in_config() {
        use crate::config::schema::*;

        let resolved = config::profile::ResolvedProfile {
            name: "test".to_string(),
            sources: Sources {
                homebrew: Some(HomebrewSource {
                    packages: vec!["git".to_string(), "vim".to_string()],
                    casks: vec![],
                    hooks: Default::default(),
                }),
                apt: None,
                dnf: None,
                pacman: None,
                mas: None,
                packages: vec![],
                github: vec![],
                custom: vec![],
            },
            dotfiles: DotfilesConfig::default(),
            hooks: ProfileHooks::default(),
        };

        let (found, manager) = find_package_in_config(&resolved, "git");
        assert!(found);
        assert_eq!(manager, Some("homebrew".to_string()));

        let (found, _) = find_package_in_config(&resolved, "nonexistent");
        assert!(!found);
    }
}
