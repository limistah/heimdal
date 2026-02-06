pub mod apt;
pub mod database;
pub mod dependencies;
pub mod dnf;
pub mod homebrew;
pub mod hooks;
pub mod manager;
pub mod mapper;
pub mod mas;
pub mod pacman;
pub mod profiles;

pub use database::PackageDatabase;
pub use dependencies::DependencyAnalyzer;
pub use hooks::{execute_hooks, HookResult};
pub use manager::{InstallResult, PackageManager};
pub use mapper::{map_package_name, PackageManagerType};
pub use profiles::{PackageProfile, ProfileSelector};

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::{PackageMapping, ResolvedProfile};
use crate::utils::{detect_os, header, info, warning, OperatingSystem};

/// Detect and return the appropriate package manager for the current OS
pub fn detect_package_manager() -> Option<Arc<dyn PackageManager>> {
    let os = detect_os();

    match os {
        OperatingSystem::MacOS => {
            if homebrew::HomebrewManager::new().is_available() {
                Some(Arc::new(homebrew::HomebrewManager::new()))
            } else {
                None
            }
        }
        OperatingSystem::Linux(distro) => {
            use crate::utils::LinuxDistro;
            match distro {
                LinuxDistro::Debian | LinuxDistro::Ubuntu => {
                    if apt::AptManager::new().is_available() {
                        Some(Arc::new(apt::AptManager::new()))
                    } else {
                        None
                    }
                }
                LinuxDistro::Fedora | LinuxDistro::RHEL | LinuxDistro::CentOS => {
                    // Use DNF for Fedora/RHEL/CentOS
                    use crate::config::DnfSource;
                    let dnf_source = DnfSource {
                        packages: vec![],
                        hooks: crate::config::Hooks::default(),
                    };
                    if dnf::Dnf::new(dnf_source.clone()).is_available() {
                        Some(Arc::new(dnf::Dnf::new(dnf_source)))
                    } else {
                        None
                    }
                }
                LinuxDistro::Arch | LinuxDistro::Manjaro => {
                    // Use Pacman for Arch/Manjaro
                    use crate::config::PacmanSource;
                    let pacman_source = PacmanSource {
                        packages: vec![],
                        hooks: crate::config::Hooks::default(),
                    };
                    if pacman::Pacman::new(pacman_source.clone()).is_available() {
                        Some(Arc::new(pacman::Pacman::new(pacman_source)))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        OperatingSystem::Unknown => None,
    }
}

/// Install packages from a resolved profile
pub fn install_packages(
    profile: &ResolvedProfile,
    custom_mappings: &HashMap<String, PackageMapping>,
    dry_run: bool,
) -> Result<InstallReport> {
    header(&format!(
        "Installing Packages for Profile: {}",
        profile.name
    ));

    let mut report = InstallReport::new();

    // Detect package manager
    let pm = detect_package_manager();

    if pm.is_none() {
        warning("No supported package manager found on this system");
        return Ok(report);
    }

    let pm = pm.unwrap();
    info(&format!("Using package manager: {}", pm.name()));

    // Determine package manager type
    let pm_type = match pm.name() {
        "apt" => PackageManagerType::Apt,
        "homebrew" => PackageManagerType::Homebrew,
        "dnf" => PackageManagerType::Dnf,
        "pacman" => PackageManagerType::Pacman,
        _ => {
            warning(&format!("Unknown package manager: {}", pm.name()));
            return Ok(report);
        }
    };

    // Install simple packages (auto-mapped)
    if !profile.sources.packages.is_empty() {
        info(&format!(
            "Installing {} simple packages...",
            profile.sources.packages.len()
        ));

        let mapped_packages: Vec<String> = profile
            .sources
            .packages
            .iter()
            .map(|tool| map_package_name(tool, pm_type, custom_mappings))
            .collect();

        let results = pm.install_many(&mapped_packages, dry_run)?;
        report.add_results(results);
    }

    // Install source-specific packages
    match pm_type {
        PackageManagerType::Homebrew => {
            if let Some(brew_source) = &profile.sources.homebrew {
                // Execute pre-install hooks
                if !brew_source.hooks.pre_install.is_empty() {
                    info("Running Homebrew pre-install hooks...");
                    let hook_results =
                        execute_hooks(&brew_source.hooks.pre_install, dry_run, "pre-install")?;
                    report.hook_results.extend(hook_results);
                }

                // Install formulae
                if !brew_source.packages.is_empty() {
                    info(&format!(
                        "Installing {} Homebrew formulae...",
                        brew_source.packages.len()
                    ));
                    let results = pm.install_many(&brew_source.packages, dry_run)?;
                    report.add_results(results);
                }

                // Install casks
                if !brew_source.casks.is_empty() {
                    info(&format!(
                        "Installing {} Homebrew casks...",
                        brew_source.casks.len()
                    ));
                    let cask_packages: Vec<String> = brew_source
                        .casks
                        .iter()
                        .map(|c| format!("--cask {}", c))
                        .collect();
                    let results = pm.install_many(&cask_packages, dry_run)?;
                    report.add_results(results);
                }

                // Execute post-install hooks
                if !brew_source.hooks.post_install.is_empty() {
                    info("Running Homebrew post-install hooks...");
                    let hook_results =
                        execute_hooks(&brew_source.hooks.post_install, dry_run, "post-install")?;
                    report.hook_results.extend(hook_results);
                }
            }

            // Install MAS apps if available
            if let Some(mas_source) = &profile.sources.mas {
                let mas_mgr = mas::MasManager::new();
                if mas_mgr.is_available() {
                    // Execute pre-install hooks
                    if !mas_source.hooks.pre_install.is_empty() {
                        info("Running MAS pre-install hooks...");
                        let hook_results =
                            execute_hooks(&mas_source.hooks.pre_install, dry_run, "pre-install")?;
                        report.hook_results.extend(hook_results);
                    }

                    // Install apps
                    if !mas_source.packages.is_empty() {
                        info(&format!(
                            "Installing {} Mac App Store apps...",
                            mas_source.packages.len()
                        ));
                        let app_ids: Vec<String> = mas_source
                            .packages
                            .iter()
                            .map(|p| p.id.to_string())
                            .collect();
                        let results = mas_mgr.install_many(&app_ids, dry_run)?;
                        report.add_results(results);
                    }

                    // Execute post-install hooks
                    if !mas_source.hooks.post_install.is_empty() {
                        info("Running MAS post-install hooks...");
                        let hook_results =
                            execute_hooks(&mas_source.hooks.post_install, dry_run, "post-install")?;
                        report.hook_results.extend(hook_results);
                    }
                } else {
                    warning("mas CLI not available, skipping Mac App Store apps");
                }
            }
        }
        PackageManagerType::Apt => {
            if let Some(apt_source) = &profile.sources.apt {
                // Execute pre-install hooks
                if !apt_source.hooks.pre_install.is_empty() {
                    info("Running APT pre-install hooks...");
                    let hook_results =
                        execute_hooks(&apt_source.hooks.pre_install, dry_run, "pre-install")?;
                    report.hook_results.extend(hook_results);
                }

                // Install packages
                if !apt_source.packages.is_empty() {
                    info(&format!(
                        "Installing {} APT packages...",
                        apt_source.packages.len()
                    ));
                    let results = pm.install_many(&apt_source.packages, dry_run)?;
                    report.add_results(results);
                }

                // Execute post-install hooks
                if !apt_source.hooks.post_install.is_empty() {
                    info("Running APT post-install hooks...");
                    let hook_results =
                        execute_hooks(&apt_source.hooks.post_install, dry_run, "post-install")?;
                    report.hook_results.extend(hook_results);
                }
            }
        }
        PackageManagerType::Dnf => {
            if let Some(dnf_source) = &profile.sources.dnf {
                // Execute pre-install hooks
                if !dnf_source.hooks.pre_install.is_empty() {
                    info("Running DNF pre-install hooks...");
                    let hook_results =
                        execute_hooks(&dnf_source.hooks.pre_install, dry_run, "pre-install")?;
                    report.hook_results.extend(hook_results);
                }

                // Install packages
                if !dnf_source.packages.is_empty() {
                    info(&format!(
                        "Installing {} DNF packages...",
                        dnf_source.packages.len()
                    ));
                    let results = pm.install_many(&dnf_source.packages, dry_run)?;
                    report.add_results(results);
                }

                // Execute post-install hooks
                if !dnf_source.hooks.post_install.is_empty() {
                    info("Running DNF post-install hooks...");
                    let hook_results =
                        execute_hooks(&dnf_source.hooks.post_install, dry_run, "post-install")?;
                    report.hook_results.extend(hook_results);
                }
            }
        }
        PackageManagerType::Pacman => {
            if let Some(pacman_source) = &profile.sources.pacman {
                // Execute pre-install hooks
                if !pacman_source.hooks.pre_install.is_empty() {
                    info("Running Pacman pre-install hooks...");
                    let hook_results =
                        execute_hooks(&pacman_source.hooks.pre_install, dry_run, "pre-install")?;
                    report.hook_results.extend(hook_results);
                }

                // Install packages
                if !pacman_source.packages.is_empty() {
                    info(&format!(
                        "Installing {} Pacman packages...",
                        pacman_source.packages.len()
                    ));
                    let results = pm.install_many(&pacman_source.packages, dry_run)?;
                    report.add_results(results);
                }

                // Execute post-install hooks
                if !pacman_source.hooks.post_install.is_empty() {
                    info("Running Pacman post-install hooks...");
                    let hook_results =
                        execute_hooks(&pacman_source.hooks.post_install, dry_run, "post-install")?;
                    report.hook_results.extend(hook_results);
                }
            }
        }
        _ => {}
    }

    Ok(report)
}

/// Report of package installation
#[derive(Debug)]
pub struct InstallReport {
    pub installed: Vec<String>,
    pub already_installed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub hook_results: Vec<HookResult>,
}

impl InstallReport {
    pub fn new() -> Self {
        Self {
            installed: Vec::new(),
            already_installed: Vec::new(),
            failed: Vec::new(),
            hook_results: Vec::new(),
        }
    }

    pub fn add_results(&mut self, results: Vec<InstallResult>) {
        for result in results {
            if result.already_installed {
                self.already_installed.push(result.package);
            } else if result.success {
                self.installed.push(result.package);
            } else {
                self.failed
                    .push((result.package, result.message.unwrap_or_default()));
            }
        }
    }

    pub fn print_summary(&self) {
        header("Installation Summary");

        if !self.installed.is_empty() {
            info(&format!("✓ Installed: {} packages", self.installed.len()));
        }

        if !self.already_installed.is_empty() {
            info(&format!(
                "○ Already installed: {} packages",
                self.already_installed.len()
            ));
        }

        if !self.failed.is_empty() {
            warning(&format!("✗ Failed: {} packages", self.failed.len()));
            for (pkg, err) in &self.failed {
                warning(&format!("  - {}: {}", pkg, err));
            }
        }

        let hook_failures = self
            .hook_results
            .iter()
            .filter(|h| !h.success && !h.skipped)
            .count();
        if hook_failures > 0 {
            warning(&format!("⚠ {} hooks failed", hook_failures));
        }
    }
}
