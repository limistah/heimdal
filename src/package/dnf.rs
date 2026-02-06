use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{InstallResult, PackageManager};
use crate::config::DnfSource;
use crate::utils::{error, info, step, success};

/// DNF package manager (Fedora, RHEL, CentOS)
pub struct Dnf {
    source: DnfSource,
}

impl Dnf {
    pub fn new(source: DnfSource) -> Self {
        Self { source }
    }
}

impl PackageManager for Dnf {
    fn name(&self) -> &str {
        "dnf"
    }

    fn is_available(&self) -> bool {
        Command::new("dnf")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("dnf")
            .arg("list")
            .arg("installed")
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

        step(&format!("Installing {} via dnf...", package));

        if dry_run {
            info(&format!("Would run: sudo dnf install -y {}", package));
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg("dnf")
            .arg("install")
            .arg("-y")
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

            step(&format!("Installing {} via dnf...", package));

            if dry_run {
                info(&format!("Would run: sudo dnf install -y {}", package));
                results.push(InstallResult::success(package.clone()));
                continue;
            }

            let status = Command::new("sudo")
                .arg("dnf")
                .arg("install")
                .arg("-y")
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
        step("Updating DNF...");

        if dry_run {
            info("Would run: sudo dnf check-update");
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg("dnf")
            .arg("check-update")
            .status()
            .context("Failed to update DNF")?;

        if status.success() {
            success("Updated DNF");
        }

        Ok(())
    }
}
