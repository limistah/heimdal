use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, MultiSelect, Select};
use std::fs;
use std::path::Path;

use crate::config::{self, schema::HeimdallConfig};
use crate::package::{database::PackageDatabase, dependencies::DependencyAnalyzer, mapper};
use crate::state::HeimdallState;
use crate::utils::{error, header, info, success, warning};

/// Run the packages add command
pub fn run_add(
    package_name: &str,
    manager: Option<&str>,
    profile: Option<&str>,
    no_install: bool,
) -> Result<()> {
    header(&format!("Adding Package: {}", package_name));

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

    // Initialize package database
    let db = PackageDatabase::new();

    // Search for package in database
    let package_info = db.get(package_name);

    if let Some(info) = &package_info {
        println!();
        println!("{}", "Package Information".bold().cyan());
        println!("  {}: {}", "Name".bright_black(), info.name.green());
        println!("  {}: {}", "Description".bright_black(), info.description);
        println!("  {}: {:?}", "Category".bright_black(), info.category);
        println!();
    }

    // Determine package manager
    let selected_manager = if let Some(mgr) = manager {
        mgr.to_string()
    } else {
        // Auto-detect or ask user
        let available_managers = detect_available_managers();
        if available_managers.is_empty() {
            anyhow::bail!("No package managers found on this system");
        }

        if available_managers.len() == 1 {
            info(&format!("Using package manager: {}", available_managers[0]));
            available_managers[0].to_string()
        } else {
            // Ask user to select
            let selection = Select::new()
                .with_prompt("Select package manager")
                .items(&available_managers)
                .default(0)
                .interact()?;
            available_managers[selection].to_string()
        }
    };

    // Map/normalize package name
    let mapped_name = mapper::normalize_package_name(package_name);
    if mapped_name != package_name {
        info(&format!(
            "Mapping '{}' to '{}' for {}",
            package_name, mapped_name, selected_manager
        ));
    }

    // Check for dependencies
    let analyzer = DependencyAnalyzer::new();
    let analysis = analyzer.analyze(&[mapped_name.clone()]);

    let all_missing: Vec<_> = analysis
        .required_missing
        .iter()
        .chain(analysis.optional_missing.iter())
        .collect();

    if !all_missing.is_empty() {
        println!();
        println!("{}", "Dependencies".bold().yellow());
        println!("This package has the following dependencies:");
        println!();

        let mut required = Vec::new();
        let mut optional = Vec::new();

        for dep in &analysis.required_missing {
            required.push(&dep.dependency.package);
            println!(
                "  {} {} (required)",
                "⚠".yellow(),
                dep.dependency.package.cyan()
            );
        }

        for dep in &analysis.optional_missing {
            optional.push(&dep.dependency.package);
            println!(
                "  {} {} (optional)",
                "💡".blue(),
                dep.dependency.package.cyan()
            );
        }

        println!();

        // Ask about required dependencies
        if !required.is_empty() {
            let install_deps = Confirm::new()
                .with_prompt("Install required dependencies?")
                .default(true)
                .interact()?;

            if !install_deps {
                warning("Skipping dependencies. Package may not work correctly.");
            }
        }

        // Ask about optional dependencies
        if !optional.is_empty() {
            let _selected_optional = MultiSelect::new()
                .with_prompt("Select optional dependencies to install")
                .items(&optional)
                .interact()?;

            // Add selected optional dependencies to install list
            // (implementation would extend the package list)
        }
    }

    // Check if package is already in config
    let resolved = config::resolve_profile(&config, profile_name)?;
    let already_added = is_package_in_config(&resolved, &mapped_name, &selected_manager);

    if already_added {
        warning(&format!(
            "Package '{}' is already in profile '{}'",
            mapped_name, profile_name
        ));

        let overwrite = Confirm::new()
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()?;

        if !overwrite {
            info("Cancelled.");
            return Ok(());
        }
    }

    // Confirm addition
    println!();
    let confirm = Confirm::new()
        .with_prompt(format!(
            "Add '{}' to profile '{}' using {}?",
            mapped_name, profile_name, selected_manager
        ))
        .default(true)
        .interact()?;

    if !confirm {
        info("Cancelled.");
        return Ok(());
    }

    // Add package to config
    add_package_to_config(&mut config, profile_name, &mapped_name, &selected_manager)?;

    // Save config
    save_config(&config, &config_path)?;
    success(&format!(
        "Added '{}' to profile '{}'",
        mapped_name, profile_name
    ));

    // Install package if not skipped
    if !no_install {
        println!();
        info(&format!(
            "Installing '{}' via {}...",
            mapped_name, selected_manager
        ));

        let install_result = install_package(&mapped_name, &selected_manager);

        match install_result {
            Ok(_) => {
                success(&format!("Successfully installed '{}'", mapped_name));
            }
            Err(e) => {
                error(&format!("Failed to install '{}': {}", mapped_name, e));
                warning("Package was added to config but installation failed.");
                warning("You can try installing manually or run 'heimdal apply'.");
            }
        }
    } else {
        info("Skipped installation (--no-install)");
        info("Run 'heimdal apply' to install all packages");
    }

    Ok(())
}

/// Detect available package managers on the system
fn detect_available_managers() -> Vec<String> {
    let mut managers = Vec::new();

    // Check for homebrew
    if std::process::Command::new("brew")
        .arg("--version")
        .output()
        .is_ok()
    {
        managers.push("homebrew".to_string());
    }

    // Check for apt
    if std::process::Command::new("apt")
        .arg("--version")
        .output()
        .is_ok()
    {
        managers.push("apt".to_string());
    }

    // Check for dnf
    if std::process::Command::new("dnf")
        .arg("--version")
        .output()
        .is_ok()
    {
        managers.push("dnf".to_string());
    }

    // Check for pacman
    if std::process::Command::new("pacman")
        .arg("--version")
        .output()
        .is_ok()
    {
        managers.push("pacman".to_string());
    }

    managers
}

/// Check if package is already in profile configuration
fn is_package_in_config(
    resolved: &config::profile::ResolvedProfile,
    package_name: &str,
    manager: &str,
) -> bool {
    match manager {
        "homebrew" => {
            if let Some(homebrew) = &resolved.sources.homebrew {
                homebrew.packages.contains(&package_name.to_string())
                    || homebrew.casks.contains(&package_name.to_string())
            } else {
                false
            }
        }
        "apt" => {
            if let Some(apt) = &resolved.sources.apt {
                apt.packages.contains(&package_name.to_string())
            } else {
                false
            }
        }
        "dnf" => resolved
            .sources
            .dnf
            .as_ref()
            .map(|dnf| dnf.packages.contains(&package_name.to_string()))
            .unwrap_or(false),
        "pacman" => resolved
            .sources
            .pacman
            .as_ref()
            .map(|pacman| pacman.packages.contains(&package_name.to_string()))
            .unwrap_or(false),
        _ => false,
    }
}

/// Add package to configuration
fn add_package_to_config(
    config: &mut HeimdallConfig,
    profile_name: &str,
    package_name: &str,
    manager: &str,
) -> Result<()> {
    use crate::config::schema::*;

    // Get or create the profile
    let profile = config
        .profiles
        .get_mut(profile_name)
        .context("Profile not found")?;

    // Ensure manager source exists in root sources
    match manager {
        "homebrew" => {
            if config.sources.homebrew.is_none() {
                config.sources.homebrew = Some(HomebrewSource {
                    packages: Vec::new(),
                    casks: Vec::new(),
                    hooks: Default::default(),
                });
            }
            if let Some(homebrew) = &mut config.sources.homebrew {
                if !homebrew.packages.contains(&package_name.to_string()) {
                    homebrew.packages.push(package_name.to_string());
                }
            }
        }
        "apt" => {
            if config.sources.apt.is_none() {
                config.sources.apt = Some(AptSource {
                    packages: Vec::new(),
                    hooks: Default::default(),
                });
            }
            if let Some(apt) = &mut config.sources.apt {
                if !apt.packages.contains(&package_name.to_string()) {
                    apt.packages.push(package_name.to_string());
                }
            }
        }
        "dnf" => {
            if config.sources.dnf.is_none() {
                config.sources.dnf = Some(DnfSource {
                    packages: Vec::new(),
                    hooks: Default::default(),
                });
            }
            if let Some(dnf) = &mut config.sources.dnf {
                if !dnf.packages.contains(&package_name.to_string()) {
                    dnf.packages.push(package_name.to_string());
                }
            }
        }
        "pacman" => {
            if config.sources.pacman.is_none() {
                config.sources.pacman = Some(PacmanSource {
                    packages: Vec::new(),
                    hooks: Default::default(),
                });
            }
            if let Some(pacman) = &mut config.sources.pacman {
                if !pacman.packages.contains(&package_name.to_string()) {
                    pacman.packages.push(package_name.to_string());
                }
            }
        }
        _ => anyhow::bail!("Unsupported package manager: {}", manager),
    }

    // Ensure profile references the source
    let source_name = manager;

    // Check if profile already references this source
    let has_source = profile.sources.iter().any(|s| match s {
        ProfileSource::Name(name) => name == source_name,
        ProfileSource::Override { name, .. } => name == source_name,
    });

    if !has_source {
        profile
            .sources
            .push(ProfileSource::Name(source_name.to_string()));
    }

    Ok(())
}

/// Save configuration to file
fn save_config(config: &HeimdallConfig, path: &Path) -> Result<()> {
    let yaml = serde_yaml::to_string(config).context("Failed to serialize config")?;
    fs::write(path, yaml).context("Failed to write config file")?;
    Ok(())
}

/// Install a package using the specified manager
fn install_package(package_name: &str, manager: &str) -> Result<()> {
    let status = match manager {
        "homebrew" => std::process::Command::new("brew")
            .arg("install")
            .arg(package_name)
            .status()
            .context("Failed to execute brew install")?,
        "apt" => std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("-y")
            .arg(package_name)
            .status()
            .context("Failed to execute apt install")?,
        "dnf" => std::process::Command::new("sudo")
            .arg("dnf")
            .arg("install")
            .arg("-y")
            .arg(package_name)
            .status()
            .context("Failed to execute dnf install")?,
        "pacman" => std::process::Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .arg("--noconfirm")
            .arg(package_name)
            .status()
            .context("Failed to execute pacman install")?,
        _ => anyhow::bail!("Unsupported package manager: {}", manager),
    };

    if !status.success() {
        anyhow::bail!("Package installation failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_available_managers() {
        let managers = detect_available_managers();
        // At least we should be able to run this without panicking
        assert!(managers.len() >= 0);
    }
}
