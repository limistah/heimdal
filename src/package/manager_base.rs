use anyhow::{Context, Result};
use std::process::Command;

use super::manager::{command_exists, InstallResult, PackageManager};
use crate::utils::{error, info, package_error, step, success, PackageErrorType};

/// Configuration for a package manager
pub struct ManagerConfig {
    pub name: &'static str,
    pub command: &'static str,
    pub check_command: Option<&'static str>,
    pub install_cmd: Vec<&'static str>,
    pub list_cmd: Vec<&'static str>,
    pub update_cmd: Vec<&'static str>,
    pub use_sudo: bool,
    pub supports_batch: bool,
}

/// Base package manager implementation with shared logic
pub struct BaseManager {
    config: ManagerConfig,
}

impl BaseManager {
    pub fn new(config: ManagerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ManagerConfig {
        &self.config
    }

    /// Execute a command with optional sudo
    fn execute_command(&self, args: &[&str], dry_run: bool) -> Result<std::process::Output> {
        if dry_run {
            let cmd_str = if self.config.use_sudo {
                format!("sudo {} {}", self.config.command, args.join(" "))
            } else {
                format!("{} {}", self.config.command, args.join(" "))
            };
            info(&format!("Would run: {}", cmd_str));
            return Ok(std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }

        let mut cmd = if self.config.use_sudo {
            let mut c = Command::new("sudo");
            c.arg(self.config.command);
            c
        } else {
            Command::new(self.config.command)
        };

        cmd.args(args).output().with_context(|| {
            format!(
                "Failed to execute {} {}",
                self.config.command,
                args.join(" ")
            )
        })
    }

    /// Default implementation of is_installed
    pub fn is_installed_default(&self, package: &str) -> bool {
        let mut args = self.config.list_cmd.clone();
        args.push(package);

        let output = if self.config.use_sudo {
            Command::new("sudo")
                .arg(self.config.command)
                .args(&args)
                .output()
        } else {
            Command::new(self.config.command).args(&args).output()
        };

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    /// Default implementation of install
    pub fn install_default(&self, package: &str, dry_run: bool) -> Result<()> {
        if !dry_run && self.is_installed_default(package) {
            info(&format!("Package already installed: {}", package));
            return Ok(());
        }

        step(&format!(
            "Installing {} via {}...",
            package, self.config.name
        ));

        let mut args = self.config.install_cmd.clone();

        // Handle special package formats (like homebrew's "--cask app")
        let pkg_args: Vec<&str> = package.split_whitespace().collect();
        args.extend(pkg_args);

        let output = self.execute_command(&args, dry_run)?;

        if dry_run || output.status.success() {
            if !dry_run {
                success(&format!("Installed {}", package));
            }
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            let error_type = if err.contains("not found") || err.contains("No such") {
                PackageErrorType::PackageNotFound
            } else {
                PackageErrorType::InstallationFailed(err.to_string())
            };

            let error_msg = package_error(package, self.config.name, error_type);
            error(&error_msg);
            anyhow::bail!("Failed to install {}: {}", package, err);
        }
    }

    /// Default implementation of install_many (batch if supported)
    pub fn install_many_default(
        &self,
        packages: &[String],
        dry_run: bool,
    ) -> Result<Vec<InstallResult>> {
        let mut results = Vec::new();

        if self.config.supports_batch {
            // Batch installation
            let mut to_install = Vec::new();
            for package in packages {
                if !dry_run && self.is_installed_default(package) {
                    results.push(InstallResult::already_installed(package.clone()));
                } else {
                    to_install.push(package.as_str());
                }
            }

            if !to_install.is_empty() {
                step(&format!(
                    "Installing {} packages via {}...",
                    to_install.len(),
                    self.config.name
                ));

                let mut args = self.config.install_cmd.clone();
                args.extend(to_install.iter().copied());

                let output = self.execute_command(&args, dry_run)?;

                if dry_run || output.status.success() {
                    for pkg in to_install {
                        results.push(InstallResult::success(pkg.to_string()));
                    }
                    if !dry_run {
                        success(&format!("Installed {} packages", results.len()));
                    }
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    for pkg in to_install {
                        results.push(InstallResult::failed(pkg.to_string(), err.to_string()));
                    }
                }
            }
        } else {
            // Install one by one
            for package in packages {
                if !dry_run && self.is_installed_default(package) {
                    results.push(InstallResult::already_installed(package.clone()));
                    continue;
                }

                match self.install_default(package, dry_run) {
                    Ok(_) => results.push(InstallResult::success(package.clone())),
                    Err(e) => results.push(InstallResult::failed(package.clone(), e.to_string())),
                }
            }
        }

        Ok(results)
    }

    /// Default implementation of update
    pub fn update_default(&self, dry_run: bool) -> Result<()> {
        step(&format!("Updating {} package list...", self.config.name));

        let output = self.execute_command(&self.config.update_cmd, dry_run)?;

        if dry_run || output.status.success() {
            if !dry_run {
                success(&format!("Updated {} package list", self.config.name));
            }
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to update {}: {}", self.config.name, err);
        }
    }
}

/// Create homebrew configuration
pub fn homebrew_config() -> ManagerConfig {
    ManagerConfig {
        name: "homebrew",
        command: "brew",
        check_command: Some("brew"),
        install_cmd: vec!["install"],
        list_cmd: vec!["list"],
        update_cmd: vec!["update"],
        use_sudo: false,
        supports_batch: false, // Homebrew doesn't batch well with casks
    }
}

/// Create apt configuration
pub fn apt_config() -> ManagerConfig {
    ManagerConfig {
        name: "apt",
        command: "apt-get",
        check_command: Some("apt-get"),
        install_cmd: vec!["install", "-y"],
        list_cmd: vec![], // apt uses dpkg for checking
        update_cmd: vec!["update"],
        use_sudo: true,
        supports_batch: true,
    }
}

/// Create dnf configuration
pub fn dnf_config() -> ManagerConfig {
    ManagerConfig {
        name: "dnf",
        command: "dnf",
        check_command: Some("dnf"),
        install_cmd: vec!["install", "-y"],
        list_cmd: vec!["list", "installed"],
        update_cmd: vec!["check-update"],
        use_sudo: true,
        supports_batch: true,
    }
}

/// Create pacman configuration
pub fn pacman_config() -> ManagerConfig {
    ManagerConfig {
        name: "pacman",
        command: "pacman",
        check_command: Some("pacman"),
        install_cmd: vec!["-S", "--noconfirm"],
        list_cmd: vec!["-Q"],
        update_cmd: vec!["-Sy"],
        use_sudo: true,
        supports_batch: true,
    }
}
