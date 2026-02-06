use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use crate::utils::{error, info, step, success};

pub struct MasManager;

impl MasManager {
    pub fn new() -> Self {
        Self
    }
}

impl PackageManager for MasManager {
    fn name(&self) -> &str {
        "mas"
    }

    fn is_available(&self) -> bool {
        command_exists("mas")
    }

    fn is_installed(&self, package: &str) -> bool {
        // Package format is the App Store ID (e.g., "497799835")
        let output = Command::new("mas").args(&["list"]).output();

        if let Ok(out) = output {
            if out.status.success() {
                let list = String::from_utf8_lossy(&out.stdout);
                return list.lines().any(|line| line.starts_with(package));
            }
        }

        false
    }

    fn install(&self, package: &str, dry_run: bool) -> Result<()> {
        if !dry_run && self.is_installed(package) {
            info(&format!("App already installed: {}", package));
            return Ok(());
        }

        step(&format!("Installing app {} from App Store...", package));

        if dry_run {
            info(&format!("Would run: mas install {}", package));
            return Ok(());
        }

        let output = Command::new("mas")
            .args(&["install", package])
            .output()
            .with_context(|| format!("Failed to install app {}", package))?;

        if output.status.success() {
            success(&format!("Installed app {}", package));
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install app {}: {}", package, err);
        }
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        let mut results = Vec::new();

        // Install apps one by one
        for package in packages {
            if !dry_run && self.is_installed(package) {
                results.push(InstallResult::already_installed(package.clone()));
                continue;
            }

            step(&format!("Installing app {} from App Store...", package));

            if dry_run {
                info(&format!("Would run: mas install {}", package));
                results.push(InstallResult::success(package.clone()));
                continue;
            }

            let output = Command::new("mas").args(&["install", package]).output();

            match output {
                Ok(out) if out.status.success() => {
                    success(&format!("Installed app {}", package));
                    results.push(InstallResult::success(package.clone()));
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    error(&format!("Failed to install app {}: {}", package, err));
                    results.push(InstallResult::failed(package.clone(), err.to_string()));
                }
                Err(e) => {
                    error(&format!("Failed to install app {}: {}", package, e));
                    results.push(InstallResult::failed(package.clone(), e.to_string()));
                }
            }
        }

        Ok(results)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        step("Checking for App Store updates...");

        if dry_run {
            info("Would run: mas upgrade");
            return Ok(());
        }

        let output = Command::new("mas")
            .arg("upgrade")
            .output()
            .context("Failed to check for App Store updates")?;

        if output.status.success() {
            success("Checked for App Store updates");
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to check for App Store updates: {}", err);
        }
    }
}
