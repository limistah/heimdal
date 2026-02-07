//! Package version management
//!
//! This module handles version checking, comparison, and upgrade detection
//! for installed packages across different package managers.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// Information about a package's version status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageVersion {
    /// Package name
    pub name: String,

    /// Currently installed version (None if not installed)
    pub installed: Option<String>,

    /// Latest available version
    pub latest: Option<String>,

    /// Whether an update is available
    pub update_available: bool,

    /// Package manager that manages this package
    pub manager: String,
}

impl PackageVersion {
    /// Creates a new package version info
    pub fn new(name: String, manager: String) -> Self {
        Self {
            name,
            installed: None,
            latest: None,
            update_available: false,
            manager,
        }
    }

    /// Check if package is installed
    #[allow(dead_code)]
    pub fn is_installed(&self) -> bool {
        self.installed.is_some()
    }

    /// Get version difference description
    pub fn version_diff(&self) -> Option<String> {
        match (&self.installed, &self.latest) {
            (Some(current), Some(latest)) if current != latest => {
                Some(format!("{} → {}", current, latest))
            }
            _ => None,
        }
    }
}

/// Version checker for different package managers
pub struct VersionChecker;

impl VersionChecker {
    /// Check versions for a list of packages using the appropriate package manager
    pub fn check_versions(packages: Vec<String>, manager: &str) -> Result<Vec<PackageVersion>> {
        match manager {
            "homebrew" => Self::check_homebrew_versions(packages),
            "apt" => Self::check_apt_versions(packages),
            "dnf" => Self::check_dnf_versions(packages),
            "pacman" => Self::check_pacman_versions(packages),
            _ => Ok(packages
                .into_iter()
                .map(|name| PackageVersion::new(name, manager.to_string()))
                .collect()),
        }
    }

    /// Check Homebrew package versions
    fn check_homebrew_versions(packages: Vec<String>) -> Result<Vec<PackageVersion>> {
        let mut results = Vec::new();

        // Get installed versions
        let installed_map = Self::get_brew_installed()?;

        // Get outdated packages
        let outdated_map = Self::get_brew_outdated()?;

        for pkg_name in packages {
            let mut version = PackageVersion::new(pkg_name.clone(), "homebrew".to_string());

            if let Some(installed_ver) = installed_map.get(&pkg_name) {
                version.installed = Some(installed_ver.clone());

                if let Some(latest_ver) = outdated_map.get(&pkg_name) {
                    version.latest = Some(latest_ver.clone());
                    version.update_available = true;
                } else {
                    // Already up to date
                    version.latest = Some(installed_ver.clone());
                    version.update_available = false;
                }
            }

            results.push(version);
        }

        Ok(results)
    }

    /// Get installed Homebrew packages and their versions
    fn get_brew_installed() -> Result<HashMap<String, String>> {
        let output = Command::new("brew")
            .args(["list", "--versions"])
            .output()
            .context("Failed to run 'brew list --versions'")?;

        if !output.status.success() {
            return Ok(HashMap::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut map = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();
                map.insert(name, version);
            }
        }

        Ok(map)
    }

    /// Get outdated Homebrew packages
    fn get_brew_outdated() -> Result<HashMap<String, String>> {
        let output = Command::new("brew")
            .args(["outdated", "--json"])
            .output()
            .context("Failed to run 'brew outdated --json'")?;

        if !output.status.success() {
            return Ok(HashMap::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut map = HashMap::new();

        // Parse JSON output
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(formulae) = json.get("formulae").and_then(|f| f.as_array()) {
                for formula in formulae {
                    if let (Some(name), Some(latest)) = (
                        formula.get("name").and_then(|n| n.as_str()),
                        formula.get("current_version").and_then(|v| v.as_str()),
                    ) {
                        map.insert(name.to_string(), latest.to_string());
                    }
                }
            }

            if let Some(casks) = json.get("casks").and_then(|c| c.as_array()) {
                for cask in casks {
                    if let (Some(name), Some(latest)) = (
                        cask.get("name").and_then(|n| n.as_str()),
                        cask.get("current_version").and_then(|v| v.as_str()),
                    ) {
                        map.insert(name.to_string(), latest.to_string());
                    }
                }
            }
        }

        Ok(map)
    }

    /// Check APT package versions
    fn check_apt_versions(packages: Vec<String>) -> Result<Vec<PackageVersion>> {
        let mut results = Vec::new();

        for pkg_name in packages {
            let mut version = PackageVersion::new(pkg_name.clone(), "apt".to_string());

            // Check installed version
            let installed_output = Command::new("dpkg-query")
                .args(["-W", "-f=${Version}", &pkg_name])
                .output();

            if let Ok(output) = installed_output {
                if output.status.success() {
                    let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !ver.is_empty() && !ver.contains("no packages found") {
                        version.installed = Some(ver.clone());

                        // Check available version
                        let available_output = Command::new("apt-cache")
                            .args(["policy", &pkg_name])
                            .output();

                        if let Ok(avail_out) = available_output {
                            let stdout = String::from_utf8_lossy(&avail_out.stdout);
                            for line in stdout.lines() {
                                if line.trim().starts_with("Candidate:") {
                                    let latest =
                                        line.split(':').nth(1).map(|s| s.trim().to_string());
                                    if let Some(latest_ver) = latest {
                                        version.latest = Some(latest_ver.clone());
                                        version.update_available = latest_ver != ver;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            results.push(version);
        }

        Ok(results)
    }

    /// Check DNF package versions
    fn check_dnf_versions(packages: Vec<String>) -> Result<Vec<PackageVersion>> {
        let mut results = Vec::new();

        for pkg_name in packages {
            let mut version = PackageVersion::new(pkg_name.clone(), "dnf".to_string());

            // Check installed and available versions
            let output = Command::new("dnf")
                .args(["list", "--installed", "--available", &pkg_name])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let mut installed_ver = None;
                    let mut available_ver = None;
                    let mut in_installed_section = false;
                    let mut in_available_section = false;

                    for line in stdout.lines() {
                        if line.contains("Installed Packages") {
                            in_installed_section = true;
                            in_available_section = false;
                            continue;
                        } else if line.contains("Available Packages") {
                            in_installed_section = false;
                            in_available_section = true;
                            continue;
                        }

                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 && parts[0].starts_with(&pkg_name) {
                            let ver = parts[1].to_string();
                            if in_installed_section {
                                installed_ver = Some(ver);
                            } else if in_available_section {
                                available_ver = Some(ver);
                            }
                        }
                    }

                    version.installed = installed_ver.clone();
                    version.latest = available_ver.or(installed_ver);
                    version.update_available = matches!((&version.installed, &version.latest),
                        (Some(i), Some(l)) if i != l);
                }
            }

            results.push(version);
        }

        Ok(results)
    }

    /// Check Pacman package versions
    fn check_pacman_versions(packages: Vec<String>) -> Result<Vec<PackageVersion>> {
        let mut results = Vec::new();

        for pkg_name in packages {
            let mut version = PackageVersion::new(pkg_name.clone(), "pacman".to_string());

            // Check installed version
            let installed_output = Command::new("pacman").args(["-Q", &pkg_name]).output();

            if let Ok(output) = installed_output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = stdout.split_whitespace().collect();
                    if parts.len() >= 2 {
                        version.installed = Some(parts[1].to_string());
                    }
                }
            }

            // Check available version
            let available_output = Command::new("pacman").args(["-Si", &pkg_name]).output();

            if let Ok(output) = available_output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.starts_with("Version") {
                            if let Some(ver) = line.split(':').nth(1) {
                                let latest_ver = ver.trim().to_string();
                                version.latest = Some(latest_ver.clone());
                                if let Some(installed) = &version.installed {
                                    version.update_available = installed != &latest_ver;
                                }
                                break;
                            }
                        }
                    }
                }
            }

            results.push(version);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_version_creation() {
        let version = PackageVersion::new("test-pkg".to_string(), "homebrew".to_string());
        assert_eq!(version.name, "test-pkg");
        assert_eq!(version.manager, "homebrew");
        assert_eq!(version.installed, None);
        assert_eq!(version.latest, None);
        assert!(!version.update_available);
    }

    #[test]
    fn test_is_installed() {
        let mut version = PackageVersion::new("test-pkg".to_string(), "homebrew".to_string());
        assert!(!version.is_installed());

        version.installed = Some("1.0.0".to_string());
        assert!(version.is_installed());
    }

    #[test]
    fn test_version_diff() {
        let mut version = PackageVersion::new("test-pkg".to_string(), "homebrew".to_string());

        // No diff when nothing installed
        assert_eq!(version.version_diff(), None);

        // No diff when same version
        version.installed = Some("1.0.0".to_string());
        version.latest = Some("1.0.0".to_string());
        assert_eq!(version.version_diff(), None);

        // Diff when different versions
        version.latest = Some("1.1.0".to_string());
        assert_eq!(version.version_diff(), Some("1.0.0 → 1.1.0".to_string()));
    }

    #[test]
    fn test_update_available() {
        let mut version = PackageVersion::new("test-pkg".to_string(), "homebrew".to_string());
        version.installed = Some("1.0.0".to_string());
        version.latest = Some("1.1.0".to_string());
        version.update_available = true;

        assert!(version.update_available);
        assert_eq!(version.version_diff(), Some("1.0.0 → 1.1.0".to_string()));
    }

    #[test]
    fn test_check_versions_unsupported_manager() {
        let packages = vec!["test-pkg".to_string()];
        let result = VersionChecker::check_versions(packages, "unsupported");

        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].name, "test-pkg");
        assert_eq!(versions[0].manager, "unsupported");
    }
}
