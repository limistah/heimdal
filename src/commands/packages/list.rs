use anyhow::{Context, Result};
use colored::*;

use crate::config;
use crate::state::HeimdallState;
use crate::utils::header;

/// Run the packages list command
pub fn run_list(installed_only: bool, profile: Option<&str>) -> Result<()> {
    // Load state to get current profile
    let state = HeimdallState::load()?;
    let profile_name = profile.unwrap_or(&state.active_profile);

    header(&format!("Packages in Profile: {}", profile_name));

    // Load config
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found: {}\nRun 'heimdal init' first.",
            config_path.display()
        );
    }

    let config = config::load_config(&config_path)?;

    // Verify profile exists
    if !config.profiles.contains_key(profile_name) {
        anyhow::bail!("Profile '{}' not found in config", profile_name);
    }

    // Resolve profile
    let resolved = config::resolve_profile(&config, profile_name)?;

    let mut total_count = 0;

    // List Homebrew packages
    if let Some(homebrew) = &resolved.sources.homebrew {
        if !homebrew.packages.is_empty() {
            println!();
            println!("{}", "Homebrew Packages".bold().cyan());
            for pkg in &homebrew.packages {
                if installed_only {
                    if is_homebrew_installed(pkg) {
                        println!("  {} {} {}", "✓".green(), pkg, "(installed)".bright_black());
                        total_count += 1;
                    }
                } else {
                    let installed = is_homebrew_installed(pkg);
                    let icon = if installed { "✓" } else { "✗" };
                    let status = if installed {
                        "(installed)".green()
                    } else {
                        "(not installed)".red()
                    };
                    println!("  {} {} {}", icon, pkg, status);
                    total_count += 1;
                }
            }
        }

        if !homebrew.casks.is_empty() {
            println!();
            println!("{}", "Homebrew Casks".bold().cyan());
            for pkg in &homebrew.casks {
                if installed_only {
                    if is_homebrew_cask_installed(pkg) {
                        println!("  {} {} {}", "✓".green(), pkg, "(installed)".bright_black());
                        total_count += 1;
                    }
                } else {
                    let installed = is_homebrew_cask_installed(pkg);
                    let icon = if installed { "✓" } else { "✗" };
                    let status = if installed {
                        "(installed)".green()
                    } else {
                        "(not installed)".red()
                    };
                    println!("  {} {} {}", icon, pkg, status);
                    total_count += 1;
                }
            }
        }
    }

    // List APT packages
    if let Some(apt) = &resolved.sources.apt {
        if !apt.packages.is_empty() {
            println!();
            println!("{}", "APT Packages".bold().cyan());
            for pkg in &apt.packages {
                if installed_only {
                    if is_apt_installed(pkg) {
                        println!("  {} {} {}", "✓".green(), pkg, "(installed)".bright_black());
                        total_count += 1;
                    }
                } else {
                    let installed = is_apt_installed(pkg);
                    let icon = if installed { "✓" } else { "✗" };
                    let status = if installed {
                        "(installed)".green()
                    } else {
                        "(not installed)".red()
                    };
                    println!("  {} {} {}", icon, pkg, status);
                    total_count += 1;
                }
            }
        }
    }

    // List DNF packages
    if let Some(dnf) = &resolved.sources.dnf {
        if !dnf.packages.is_empty() {
            println!();
            println!("{}", "DNF Packages".bold().cyan());
            for pkg in &dnf.packages {
                if installed_only {
                    if is_dnf_installed(pkg) {
                        println!("  {} {} {}", "✓".green(), pkg, "(installed)".bright_black());
                        total_count += 1;
                    }
                } else {
                    let installed = is_dnf_installed(pkg);
                    let icon = if installed { "✓" } else { "✗" };
                    let status = if installed {
                        "(installed)".green()
                    } else {
                        "(not installed)".red()
                    };
                    println!("  {} {} {}", icon, pkg, status);
                    total_count += 1;
                }
            }
        }
    }

    // List Pacman packages
    if let Some(pacman) = &resolved.sources.pacman {
        if !pacman.packages.is_empty() {
            println!();
            println!("{}", "Pacman Packages".bold().cyan());
            for pkg in &pacman.packages {
                if installed_only {
                    if is_pacman_installed(pkg) {
                        println!("  {} {} {}", "✓".green(), pkg, "(installed)".bright_black());
                        total_count += 1;
                    }
                } else {
                    let installed = is_pacman_installed(pkg);
                    let icon = if installed { "✓" } else { "✗" };
                    let status = if installed {
                        "(installed)".green()
                    } else {
                        "(not installed)".red()
                    };
                    println!("  {} {} {}", icon, pkg, status);
                    total_count += 1;
                }
            }
        }
    }

    println!();
    let filter_text = if installed_only { " installed" } else { "" };
    println!(
        "{}",
        format!("Total:{} packages", total_count).bright_black()
    );
    println!();

    Ok(())
}

/// Check if a Homebrew package is installed
fn is_homebrew_installed(package: &str) -> bool {
    std::process::Command::new("brew")
        .arg("list")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a Homebrew cask is installed
fn is_homebrew_cask_installed(package: &str) -> bool {
    std::process::Command::new("brew")
        .arg("list")
        .arg("--cask")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if an APT package is installed
fn is_apt_installed(package: &str) -> bool {
    std::process::Command::new("dpkg")
        .arg("-s")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a DNF package is installed
fn is_dnf_installed(package: &str) -> bool {
    std::process::Command::new("dnf")
        .arg("list")
        .arg("--installed")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a Pacman package is installed
fn is_pacman_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .arg("-Q")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_check_functions() {
        // These functions should not panic
        let _ = is_homebrew_installed("git");
        let _ = is_homebrew_cask_installed("firefox");
        let _ = is_apt_installed("git");
        let _ = is_dnf_installed("git");
        let _ = is_pacman_installed("git");
    }
}
