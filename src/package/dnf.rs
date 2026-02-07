use anyhow::Result;
use std::process::Command;

use super::manager::{InstallResult, PackageManager};
use super::manager_base::{dnf_config, BaseManager};
use crate::config::DnfSource;

/// DNF package manager (Fedora, RHEL, CentOS)
pub struct Dnf {
    source: DnfSource,
    base: BaseManager,
}

impl Dnf {
    pub fn new(source: DnfSource) -> Self {
        Self {
            source,
            base: BaseManager::new(dnf_config()),
        }
    }
}

impl PackageManager for Dnf {
    fn name(&self) -> &str {
        self.base.config().name
    }

    fn is_available(&self) -> bool {
        Command::new("dnf")
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
