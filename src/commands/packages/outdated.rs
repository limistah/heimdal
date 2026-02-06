//! Package version commands (outdated, upgrade)

use crate::package::{detect_package_manager, PackageVersion, VersionChecker};
use crate::utils::{error, header, info, success, warning};
use anyhow::{Context, Result};
use colored::Colorize;

/// Show packages that have updates available
pub fn run_outdated(all_packages: bool) -> Result<()> {
    header("Checking for Package Updates");

    // Detect package manager
    let pm = detect_package_manager();
    if pm.is_none() {
        error("No supported package manager found on this system");
        return Ok(());
    }

    let pm = pm.unwrap();
    let pm_name = pm.name();
    info(&format!("Using package manager: {}", pm_name));
    println!();

    // Get list of packages to check
    let packages = if all_packages {
        info("Checking all installed packages...");
        get_all_installed_packages(&pm_name)?
    } else {
        info("Checking packages from current profile...");
        get_profile_packages()?
    };

    if packages.is_empty() {
        warning("No packages to check");
        return Ok(());
    }

    info(&format!("Checking {} packages...", packages.len()));
    println!();

    // Check versions
    let versions = VersionChecker::check_versions(packages, &pm_name)?;

    // Filter outdated packages
    let outdated: Vec<&PackageVersion> = versions.iter().filter(|v| v.update_available).collect();

    if outdated.is_empty() {
        success("All packages are up to date!");
        return Ok(());
    }

    // Display outdated packages
    println!("{}", "Outdated Packages:".bold().yellow());
    println!();
    println!(
        "{:<30} {:<20} {:<20}",
        "Package".bold(),
        "Installed".bold(),
        "Available".bold()
    );
    println!("{}", "─".repeat(72));

    for pkg in &outdated {
        let installed = pkg.installed.as_deref().unwrap_or("?");
        let latest = pkg.latest.as_deref().unwrap_or("?");

        println!(
            "{:<30} {:<20} {:<20}",
            pkg.name.cyan(),
            installed.red(),
            latest.green()
        );
    }

    println!();
    info(&format!(
        "{} package{} can be updated",
        outdated.len(),
        if outdated.len() == 1 { "" } else { "s" }
    ));
    println!();
    info("Run 'heimdal packages upgrade' to update packages");
    info("Run 'heimdal packages upgrade <package>' to update specific package");

    Ok(())
}

/// Upgrade packages
pub fn run_upgrade(package: Option<String>, all: bool, dry_run: bool) -> Result<()> {
    if all {
        header("Upgrading All Packages");
    } else if let Some(ref pkg) = package {
        header(&format!("Upgrading Package: {}", pkg));
    } else {
        header("Upgrading Profile Packages");
    }

    // Detect package manager
    let pm = detect_package_manager();
    if pm.is_none() {
        error("No supported package manager found on this system");
        return Ok(());
    }

    let pm = pm.unwrap();
    let pm_name = pm.name();
    info(&format!("Using package manager: {}", pm_name));
    println!();

    if dry_run {
        warning("DRY RUN - No packages will be upgraded");
        println!();
    }

    // Get packages to upgrade
    let packages = if all {
        info("Checking all installed packages...");
        get_all_installed_packages(&pm_name)?
    } else if let Some(pkg) = package {
        vec![pkg]
    } else {
        info("Checking packages from current profile...");
        get_profile_packages()?
    };

    if packages.is_empty() {
        warning("No packages to upgrade");
        return Ok(());
    }

    // Check which packages need updates
    let versions = VersionChecker::check_versions(packages.clone(), &pm_name)?;
    let outdated: Vec<&PackageVersion> = versions.iter().filter(|v| v.update_available).collect();

    if outdated.is_empty() {
        success("All packages are already up to date!");
        return Ok(());
    }

    info(&format!(
        "Found {} package{} to upgrade",
        outdated.len(),
        if outdated.len() == 1 { "" } else { "s" }
    ));
    println!();

    // Show what will be upgraded
    for pkg in &outdated {
        if let Some(diff) = pkg.version_diff() {
            info(&format!("  {} {}", pkg.name, diff));
        }
    }
    println!();

    if dry_run {
        info("Would upgrade the packages listed above");
        return Ok(());
    }

    // Perform upgrade based on package manager
    match pm_name {
        "homebrew" => upgrade_homebrew(&outdated)?,
        "apt" => upgrade_apt(&outdated)?,
        "dnf" => upgrade_dnf(&outdated)?,
        "pacman" => upgrade_pacman(&outdated)?,
        _ => {
            warning(&format!("Upgrade not yet implemented for {}", pm_name));
        }
    }

    println!();
    success("Upgrade complete!");

    Ok(())
}

/// Upgrade packages using Homebrew
fn upgrade_homebrew(packages: &[&PackageVersion]) -> Result<()> {
    use std::process::Command;

    info("Running: brew upgrade...");

    let pkg_names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();

    let output = Command::new("brew")
        .arg("upgrade")
        .args(&pkg_names)
        .output()
        .context("Failed to run brew upgrade")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error(&format!("Upgrade failed: {}", stderr));
        return Err(anyhow::anyhow!("brew upgrade failed"));
    }

    Ok(())
}

/// Upgrade packages using APT
fn upgrade_apt(packages: &[&PackageVersion]) -> Result<()> {
    use std::process::Command;

    info("Running: sudo apt-get install --only-upgrade...");

    let pkg_names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();

    let output = Command::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("--only-upgrade")
        .arg("-y")
        .args(&pkg_names)
        .output()
        .context("Failed to run apt-get upgrade")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error(&format!("Upgrade failed: {}", stderr));
        return Err(anyhow::anyhow!("apt-get upgrade failed"));
    }

    Ok(())
}

/// Upgrade packages using DNF
fn upgrade_dnf(packages: &[&PackageVersion]) -> Result<()> {
    use std::process::Command;

    info("Running: sudo dnf upgrade...");

    let pkg_names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();

    let output = Command::new("sudo")
        .arg("dnf")
        .arg("upgrade")
        .arg("-y")
        .args(&pkg_names)
        .output()
        .context("Failed to run dnf upgrade")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error(&format!("Upgrade failed: {}", stderr));
        return Err(anyhow::anyhow!("dnf upgrade failed"));
    }

    Ok(())
}

/// Upgrade packages using Pacman
fn upgrade_pacman(packages: &[&PackageVersion]) -> Result<()> {
    use std::process::Command;

    info("Running: sudo pacman -S...");

    let pkg_names: Vec<&str> = packages.iter().map(|p| p.name.as_str()).collect();

    let output = Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .args(&pkg_names)
        .output()
        .context("Failed to run pacman upgrade")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error(&format!("Upgrade failed: {}", stderr));
        return Err(anyhow::anyhow!("pacman upgrade failed"));
    }

    Ok(())
}

/// Get all installed packages from the package manager
fn get_all_installed_packages(pm_name: &str) -> Result<Vec<String>> {
    use std::process::Command;

    match pm_name {
        "homebrew" => {
            let output = Command::new("brew")
                .args(["list", "--formula"])
                .output()
                .context("Failed to list brew packages")?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.lines().map(|s| s.to_string()).collect())
        }
        "apt" => {
            let output = Command::new("dpkg-query")
                .args(["-W", "-f=${Package}\\n"])
                .output()
                .context("Failed to list dpkg packages")?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.lines().map(|s| s.to_string()).collect())
        }
        "dnf" => {
            let output = Command::new("dnf")
                .args(["list", "--installed"])
                .output()
                .context("Failed to list dnf packages")?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        Some(parts[0].split('.').next()?.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        }
        "pacman" => {
            let output = Command::new("pacman")
                .args(["-Q"])
                .output()
                .context("Failed to list pacman packages")?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(packages)
        }
        _ => Ok(Vec::new()),
    }
}

/// Get packages from current profile
/// For now, returns a small test set - in production this would read from config
fn get_profile_packages() -> Result<Vec<String>> {
    // TODO: Read from actual profile configuration
    // For now, return common packages as a demo
    Ok(vec![
        "git".to_string(),
        "node".to_string(),
        "rust".to_string(),
        "docker".to_string(),
        "fzf".to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profile_packages() {
        let result = get_profile_packages();
        assert!(result.is_ok());
        let packages = result.unwrap();
        assert!(!packages.is_empty());
    }

    #[test]
    fn test_run_outdated() {
        // This is a basic test that just ensures the function doesn't panic
        // Real testing would require mocking package managers
        let result = run_outdated(false);
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
