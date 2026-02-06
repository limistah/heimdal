use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{InstallResult, PackageManager};
use crate::config::PacmanSource;
use crate::utils::{error, info, step, success};

/// Pacman package manager (Arch Linux, Manjaro)
pub struct Pacman {
    source: PacmanSource,
}

impl Pacman {
    pub fn new(source: PacmanSource) -> Self {
        Self { source }
    }
}

impl PackageManager for Pacman {
    fn name(&self) -> &str {
        "pacman"
    }

    fn is_available(&self) -> bool {
        Command::new("pacman")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("pacman")
            .arg("-Q")
            .arg(package)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn install(&self, package: &str, dry_run: bool) -> Result<()> {
        if !dry_run && self.is_installed(package) {
            info(&format!("Package already installed: {}", package));
            return Ok(());
        }

        step(&format!("Installing {} via pacman...", package));

        if dry_run {
            info(&format!(
                "Would run: sudo pacman -S --noconfirm {}",
                package
            ));
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg("pacman")
            .arg("-S")
            .arg("--noconfirm")
            .arg(package)
            .status()
            .with_context(|| format!("Failed to install {}", package))?;

        if status.success() {
            success(&format!("Installed {}", package));
            Ok(())
        } else {
            anyhow::bail!("Failed to install {}", package)
        }
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        let mut results = Vec::new();

        for package in packages {
            // Skip installed check in dry-run mode
            if !dry_run && self.is_installed(package) {
                results.push(InstallResult::already_installed(package.clone()));
                continue;
            }

            step(&format!("Installing {} via pacman...", package));

            if dry_run {
                info(&format!(
                    "Would run: sudo pacman -S --noconfirm {}",
                    package
                ));
                results.push(InstallResult::success(package.clone()));
                continue;
            }

            let status = Command::new("sudo")
                .arg("pacman")
                .arg("-S")
                .arg("--noconfirm")
                .arg(package)
                .status();

            match status {
                Ok(s) if s.success() => {
                    success(&format!("Installed {}", package));
                    results.push(InstallResult::success(package.clone()));
                }
                Ok(_) => {
                    error(&format!("Failed to install {}", package));
                    results.push(InstallResult::failed(
                        package.clone(),
                        "Installation failed".to_string(),
                    ));
                }
                Err(e) => {
                    error(&format!("Failed to install {}: {}", package, e));
                    results.push(InstallResult::failed(package.clone(), e.to_string()));
                }
            }
        }

        Ok(results)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        step("Updating Pacman...");

        if dry_run {
            info("Would run: sudo pacman -Sy");
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg("pacman")
            .arg("-Sy")
            .status()
            .context("Failed to update Pacman")?;

        if status.success() {
            success("Updated Pacman");
        }

        Ok(())
    }
}
