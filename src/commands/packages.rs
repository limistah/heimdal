use crate::cli::PackagesCmd;
use crate::config::load_config;
use crate::state::State;
use crate::utils::{info, success, warning};
use anyhow::Result;

pub fn run(action: PackagesCmd) -> Result<()> {
    match action {
        PackagesCmd::List { installed: _ } => list(),
        PackagesCmd::Add {
            name,
            manager,
            no_install,
        } => add(&name, manager.as_deref(), no_install),
        PackagesCmd::Remove { name, no_uninstall } => remove(&name, no_uninstall),
        PackagesCmd::Search { query } => search(&query),
        PackagesCmd::Info { name } => pkg_info(&name),
        PackagesCmd::Groups => groups(),
    }
}

fn list() -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;
    let profile = config.profiles.get(&state.active_profile).ok_or_else(|| {
        crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        }
    })?;

    let pkgs = &profile.packages;
    let mut any = false;

    macro_rules! list_pm {
        ($field:expr, $label:expr) => {
            if !$field.is_empty() {
                println!("{}:", $label);
                for p in &$field {
                    println!("  - {}", p);
                }
                any = true;
            }
        };
    }

    list_pm!(pkgs.homebrew, "homebrew");
    list_pm!(pkgs.homebrew_casks, "homebrew-cask");
    list_pm!(pkgs.apt, "apt");
    list_pm!(pkgs.dnf, "dnf");
    list_pm!(pkgs.pacman, "pacman");
    list_pm!(pkgs.apk, "apk");

    if !any {
        info("No packages configured for this profile.");
    }
    Ok(())
}

fn add(pkg: &str, manager: Option<&str>, no_install: bool) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    // Determine which manager field to update
    let mgr = match manager {
        Some(m) => m.to_string(),
        None => {
            // Auto-detect
            crate::packages::detect_manager()
                .map(|m| m.field_name().to_string())
                .unwrap_or_else(|| {
                    warning("No package manager detected, defaulting to 'homebrew'");
                    "homebrew".to_string()
                })
        }
    };

    let profile = config
        .profiles
        .get_mut(&state.active_profile)
        .ok_or_else(|| crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        })?;

    let list = match mgr.as_str() {
        "homebrew" => &mut profile.packages.homebrew,
        "homebrew_casks" | "homebrew-cask" => &mut profile.packages.homebrew_casks,
        "apt" => &mut profile.packages.apt,
        "dnf" => &mut profile.packages.dnf,
        "pacman" => &mut profile.packages.pacman,
        "apk" => &mut profile.packages.apk,
        other => anyhow::bail!(
            "Unknown package manager '{}'. Valid: homebrew, apt, dnf, pacman, apk",
            other
        ),
    };

    if list.contains(&pkg.to_string()) {
        info(&format!(
            "'{}' is already in the {} package list.",
            pkg, mgr
        ));
        return Ok(());
    }

    list.push(pkg.to_string());
    write_config(&config_path, &config)?;
    success(&format!("Added '{}' to {} packages", pkg, mgr));

    if !no_install {
        info(&format!("Run 'heimdal apply' to install '{}'.", pkg));
    }
    Ok(())
}

fn remove(pkg: &str, no_uninstall: bool) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    let profile = config
        .profiles
        .get_mut(&state.active_profile)
        .ok_or_else(|| crate::error::HeimdallError::ProfileNotFound {
            name: state.active_profile.clone(),
        })?;

    let mut removed = false;
    macro_rules! remove_from {
        ($field:expr) => {
            let before = $field.len();
            $field.retain(|p| p != pkg);
            if $field.len() < before {
                removed = true;
            }
        };
    }

    remove_from!(profile.packages.homebrew);
    remove_from!(profile.packages.homebrew_casks);
    remove_from!(profile.packages.apt);
    remove_from!(profile.packages.dnf);
    remove_from!(profile.packages.pacman);
    remove_from!(profile.packages.apk);

    if removed {
        write_config(&config_path, &config)?;
        success(&format!("Removed '{}' from config", pkg));
        if !no_uninstall {
            warning(&format!(
                "'{}' was removed from config but NOT uninstalled from your system. \
                Run the appropriate uninstall command manually if needed.",
                pkg
            ));
        }
    } else {
        info(&format!("'{}' was not found in any package list.", pkg));
    }
    Ok(())
}

fn search(query: &str) -> Result<()> {
    let brew_args = ["search", query];
    let apt_args = ["search", query];
    let dnf_args = ["search", query];
    let pacman_args = ["-Ss", query];
    let apk_args = ["search", query];

    let managers: Vec<(&str, &[&str], &str)> = vec![
        ("brew", &brew_args[..], "macOS/Linux (Homebrew)"),
        ("apt-cache", &apt_args[..], "Debian/Ubuntu (apt)"),
        ("dnf", &dnf_args[..], "Fedora/RHEL (dnf)"),
        ("pacman", &pacman_args[..], "Arch Linux (pacman)"),
        ("apk", &apk_args[..], "Alpine (apk)"),
    ];

    for (cmd, args, label) in &managers {
        let available = std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if available {
            info(&format!("Searching {} packages for '{}'...", label, query));
            let status = std::process::Command::new(cmd)
                .args(*args)
                .status()
                .map_err(|e| anyhow::anyhow!("Cannot run {}: {}", cmd, e))?;
            if !status.success() {
                warning("Search returned no results or failed.");
            }
            return Ok(());
        }
    }

    info("No package manager detected. Search manually:");
    info(&format!("  Homebrew (macOS/Linux): brew search {}", query));
    info(&format!(
        "  APT (Debian/Ubuntu):    apt-cache search {}",
        query
    ));
    info(&format!("  DNF (Fedora/RHEL):      dnf search {}", query));
    info(&format!("  Pacman (Arch):          pacman -Ss {}", query));
    info(&format!("  APK (Alpine):           apk search {}", query));
    Ok(())
}

fn pkg_info(name: &str) -> Result<()> {
    let brew_args = ["info", name];
    let apt_args = ["show", name];
    let dnf_args = ["info", name];
    let pacman_args = ["-Si", name];
    let apk_args = ["info", name];

    let managers: Vec<(&str, &[&str], &str)> = vec![
        ("brew", &brew_args[..], "Homebrew"),
        ("apt-cache", &apt_args[..], "APT"),
        ("dnf", &dnf_args[..], "DNF"),
        ("pacman", &pacman_args[..], "Pacman"),
        ("apk", &apk_args[..], "APK"),
    ];

    for (cmd, args, label) in &managers {
        let available = std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if available {
            info(&format!("Package info from {} for '{}':", label, name));
            std::process::Command::new(cmd).args(*args).status().ok();
            return Ok(());
        }
    }

    info("No package manager detected. Get info manually:");
    info(&format!("  brew info {}", name));
    info(&format!("  apt-cache show {}", name));
    info(&format!("  dnf info {}", name));
    Ok(())
}

fn groups() -> Result<()> {
    info("Package groups are managed in heimdal.yaml under each profile.");
    info("Use 'heimdal profile show' to see the packages in the current profile.");
    Ok(())
}

fn write_config(path: &std::path::Path, config: &crate::config::HeimdalConfig) -> Result<()> {
    let content = serde_yaml_ng::to_string(config)?;
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
