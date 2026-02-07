use anyhow::Result;
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use super::manager_base::{homebrew_config, BaseManager};

pub struct HomebrewManager {
    base: BaseManager,
}

impl HomebrewManager {
    pub fn new() -> Self {
        Self {
            base: BaseManager::new(homebrew_config()),
        }
    }
}

impl PackageManager for HomebrewManager {
    fn name(&self) -> &str {
        self.base.config().name
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
        // Use base implementation which handles the common pattern
        self.base.install_default(package, dry_run)
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        // Use base implementation (which handles one-by-one for homebrew)
        self.base.install_many_default(packages, dry_run)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        // Use base implementation
        self.base.update_default(dry_run)
    }
}
