use anyhow::Result;
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use super::manager_base::{apk_config, BaseManager};

pub struct ApkManager {
    base: BaseManager,
}

impl ApkManager {
    pub fn new() -> Self {
        Self {
            base: BaseManager::new(apk_config()),
        }
    }
}

impl PackageManager for ApkManager {
    fn name(&self) -> &str {
        self.base.config().name
    }

    fn is_available(&self) -> bool {
        command_exists("apk")
    }

    fn is_installed(&self, package: &str) -> bool {
        // APK uses 'apk info' for checking installations
        Command::new("apk")
            .args(["info", "-e", package])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn install(&self, package: &str, dry_run: bool) -> Result<()> {
        self.base.install_default(package, dry_run)
    }

    fn install_many(&self, packages: &[String], dry_run: bool) -> Result<Vec<InstallResult>> {
        self.base.install_many_default(packages, dry_run)
    }

    fn update(&self, dry_run: bool) -> Result<()> {
        self.base.update_default(dry_run)
    }
}
