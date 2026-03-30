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
        PackagesCmd::Suggest { dir } => suggest(dir.as_deref()),
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

fn suggest(dir: Option<&str>) -> Result<()> {
    let scan_dir = match dir {
        Some(d) => std::path::PathBuf::from(d),
        None => std::env::current_dir()?,
    };

    struct Suggestion {
        file: &'static str,
        packages: &'static [&'static str],
        label: &'static str,
    }

    let rules: &[Suggestion] = &[
        Suggestion {
            file: "Cargo.toml",
            packages: &["rust", "cargo"],
            label: "Rust/Cargo",
        },
        Suggestion {
            file: "package.json",
            packages: &["node", "nvm"],
            label: "Node.js",
        },
        Suggestion {
            file: ".nvmrc",
            packages: &["node", "nvm"],
            label: "Node.js (nvm)",
        },
        Suggestion {
            file: "Gemfile",
            packages: &["ruby", "rbenv"],
            label: "Ruby",
        },
        Suggestion {
            file: "requirements.txt",
            packages: &["python", "pyenv"],
            label: "Python (pip)",
        },
        Suggestion {
            file: "Pipfile",
            packages: &["python", "pyenv"],
            label: "Python (pipenv)",
        },
        Suggestion {
            file: "pyproject.toml",
            packages: &["python", "pyenv"],
            label: "Python",
        },
        Suggestion {
            file: "go.mod",
            packages: &["go"],
            label: "Go",
        },
        Suggestion {
            file: "pom.xml",
            packages: &["java", "maven"],
            label: "Java (Maven)",
        },
        Suggestion {
            file: "build.gradle",
            packages: &["java", "gradle"],
            label: "Java (Gradle)",
        },
        Suggestion {
            file: "composer.json",
            packages: &["php", "composer"],
            label: "PHP",
        },
        Suggestion {
            file: "CMakeLists.txt",
            packages: &["cmake", "gcc"],
            label: "C/C++ (CMake)",
        },
    ];

    let mut found_any = false;
    for rule in rules {
        if scan_dir.join(rule.file).exists() {
            println!(
                "Detected {} ({}): suggest {}",
                rule.label,
                rule.file,
                rule.packages.join(", ")
            );
            found_any = true;
        }
    }

    if !found_any {
        info("No known project files detected in this directory.");
    }
    Ok(())
}

fn search(query: &str) -> Result<()> {
    // No central database — just inform the user
    info(&format!(
        "Search is not available offline. To find packages for '{}', visit:",
        query
    ));
    info("  Homebrew: https://formulae.brew.sh/");
    info("  Apt:      https://packages.ubuntu.com/");
    info("  Pacman:   https://archlinux.org/packages/");
    Ok(())
}

fn pkg_info(name: &str) -> Result<()> {
    info(&format!(
        "Package info for '{}' is not available offline.",
        name
    ));
    info("Use your package manager directly: brew info <pkg>, apt show <pkg>, etc.");
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
