use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use crate::utils::{error, info, step, success};

pub struct AptManager {
    use_sudo: bool,
}

impl AptManager {
    pub fn new() -> Self {
        Self { use_sudo: true }
    }
}

impl PackageManager for AptManager {
    fn name(&self) -> &str {
        "apt"
    }

    fn is_available(&self) -> bool {
        command_exists("apt-get") || command_exists("apt")
    }

    fn is_installed(&self, package: &str) -> bool {
        Command::new("dpkg")
            .args(["-s", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn install(&self, package: &str, dry_run: bool) -> Result<()> {
        if !dry_run && self.is_installed(package) {
            info(&format!("Package already installed: {}", package));
            return Ok(());
        }

        step(&format!("Installing {} via apt...", package));

        if dry_run {
            info(&format!("Would run: sudo apt-get install -y {}", package));
            return Ok(());
        }

        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", package])
            .output()
            .with_context(|| format!("Failed to install {}", package))?;

        if output.status.success() {
            success(&format!("Installed {}", package));
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install {}: {}", package, err);
        }
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        let mut results = Vec::new();

        // Filter out already installed packages (skip in dry-run)
        let mut to_install = Vec::new();
        for package in packages {
            if !dry_run && self.is_installed(package) {
                results.push(InstallResult::already_installed(package.clone()));
            } else {
                to_install.push(package.as_str());
            }
        }

        if to_install.is_empty() {
            return Ok(results);
        }

        step(&format!(
            "Installing {} packages via apt...",
            to_install.len()
        ));

        if dry_run {
            info(&format!(
                "Would run: sudo apt-get install -y {}",
                to_install.join(" ")
            ));
            for pkg in to_install {
                results.push(InstallResult::success(pkg.to_string()));
            }
            return Ok(results);
        }

        // Install all at once (more efficient)
        let mut args = vec!["apt-get", "install", "-y"];
        args.extend(&to_install);

        let output = Command::new("sudo")
            .args(&args)
            .output()
            .context("Failed to run apt-get")?;

        if output.status.success() {
            for pkg in to_install {
                results.push(InstallResult::success(pkg.to_string()));
                success(&format!("Installed {}", pkg));
            }
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            error(&format!("apt-get failed: {}", err));

            // Mark all as failed
            for pkg in to_install {
                results.push(InstallResult::failed(pkg.to_string(), err.to_string()));
            }
        }

        Ok(results)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        step("Updating apt package lists...");

        if dry_run {
            info("Would run: sudo apt-get update");
            return Ok(());
        }

        let output = Command::new("sudo")
            .args(["apt-get", "update"])
            .output()
            .context("Failed to update apt")?;

        if output.status.success() {
            success("Updated apt package lists");
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to update apt: {}", err);
        }
    }
}
