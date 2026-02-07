use anyhow::Result;
use std::process::Command;

use super::manager::{InstallResult, PackageManager};
use super::manager_base::{pacman_config, BaseManager};
use crate::config::PacmanSource;

/// Pacman package manager (Arch Linux, Manjaro)
pub struct Pacman {
    #[allow(dead_code)]
    source: PacmanSource,
    base: BaseManager,
}

impl Pacman {
    pub fn new(source: PacmanSource) -> Self {
        Self {
            source,
            base: BaseManager::new(pacman_config()),
        }
    }
}

impl PackageManager for Pacman {
    fn name(&self) -> &str {
        self.base.config().name
    }

    fn is_available(&self) -> bool {
        Command::new("pacman")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_installed(&self, package: &str) -> bool {
        self.base.is_installed_default(package)
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
