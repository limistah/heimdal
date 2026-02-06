use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use crate::utils::{error, info, package_error, step, success, PackageErrorType};

pub struct HomebrewManager;

impl HomebrewManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for HomebrewManager {
    fn name(&self) -> &str {
        "homebrew"
    }

    fn is_available(&self) -> bool {
        command_exists("brew")
    }

    fn is_installed(&self, package: &str) -> bool {
        // Handle cask packages (e.g., "--cask firefox")
        let (is_cask, pkg_name) = if package.starts_with("--cask ") {
            (true, package.trim_start_matches("--cask ").trim())
        } else {
            (false, package)
        };

        let output = if is_cask {
            Command::new("brew")
                .args(["list", "--cask", pkg_name])
                .output()
        } else {
            Command::new("brew").args(["list", pkg_name]).output()
        };

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    fn install(&self, package: &str, dry_run: bool) -> Result<()> {
        if !dry_run && self.is_installed(package) {
            info(&format!("Package already installed: {}", package));
            return Ok(());
        }

        step(&format!("Installing {} via brew...", package));

        if dry_run {
            info(&format!("Would run: brew install {}", package));
            return Ok(());
        }

        // Parse package (might be "--cask app" or just "package")
        let args: Vec<&str> = package.split_whitespace().collect();

        let output = Command::new("brew")
            .arg("install")
            .args(&args)
            .output()
            .with_context(|| format!("Failed to install {}", package))?;

        if output.status.success() {
            success(&format!("Installed {}", package));
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            let error_type = if err.contains("No available formula") || err.contains("No cask") {
                PackageErrorType::PackageNotFound
            } else {
                PackageErrorType::InstallationFailed(err.to_string())
            };

            let error_msg = package_error(package, "brew", error_type);
            eprintln!("{}", error_msg);
            anyhow::bail!("Failed to install {}: {}", package, err);
        }
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        let mut results = Vec::new();

        // Homebrew doesn't batch install casks and formulae well, so install one by one
        for package in packages {
            // Skip installed check in dry-run mode
            if !dry_run && self.is_installed(package) {
                results.push(InstallResult::already_installed(package.clone()));
                continue;
            }

            step(&format!("Installing {} via brew...", package));

            if dry_run {
                info(&format!("Would run: brew install {}", package));
                results.push(InstallResult::success(package.clone()));
                continue;
            }

            let args: Vec<&str> = package.split_whitespace().collect();

            let output = Command::new("brew").arg("install").args(&args).output();

            match output {
                Ok(out) if out.status.success() => {
                    success(&format!("Installed {}", package));
                    results.push(InstallResult::success(package.clone()));
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);

                    let error_type =
                        if err.contains("No available formula") || err.contains("No cask") {
                            PackageErrorType::PackageNotFound
                        } else {
                            PackageErrorType::InstallationFailed(err.to_string())
                        };

                    let error_msg = package_error(package, "brew", error_type);
                    eprintln!("{}", error_msg);
                    error(&format!("Failed to install {}: {}", package, err));
                    results.push(InstallResult::failed(package.clone(), err.to_string()));
                }
                Err(e) => {
                    let error_msg =
                        package_error(package, "brew", PackageErrorType::ManagerNotFound);
                    eprintln!("{}", error_msg);
                    error(&format!("Failed to install {}: {}", package, e));
                    results.push(InstallResult::failed(package.clone(), e.to_string()));
                }
            }
        }

        Ok(results)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        step("Updating Homebrew...");

        if dry_run {
            info("Would run: brew update");
            return Ok(());
        }

        let output = Command::new("brew")
            .arg("update")
            .output()
            .context("Failed to update Homebrew")?;

        if output.status.success() {
            success("Updated Homebrew");
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to update Homebrew: {}", err);
        }
    }
}
