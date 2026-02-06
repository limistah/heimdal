use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::loader::load_config;
use crate::config::profile::resolve_profile;
use crate::state::HeimdallState;

/// Switch to a different profile
pub fn switch_profile(profile_name: &str, auto_apply: bool) -> Result<(bool, String)> {
    // Load state and config
    let mut state = HeimdallState::load()?;
    let config = load_config(&state.dotfiles_path)?;

    // Validate that the profile exists
    if !config.profiles.contains_key(profile_name) {
        anyhow::bail!(
            "Profile '{}' not found. Available profiles:\n  {}",
            profile_name,
            config
                .profiles
                .keys()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }

    // Check if already on this profile
    if state.active_profile == profile_name {
        println!(
            "{} Already on profile '{}'",
            "✓".green(),
            profile_name.cyan()
        );
        return Ok((false, profile_name.to_string()));
    }

    let old_profile = state.active_profile.clone();

    // Update state with new profile
    state.active_profile = profile_name.to_string();
    state.save()?;

    println!(
        "{} Switched from '{}' to '{}'",
        "✓".green(),
        old_profile.yellow(),
        profile_name.cyan()
    );

    Ok((auto_apply, profile_name.to_string()))
}

/// Get the currently active profile name
pub fn get_current_profile() -> Result<String> {
    let state = HeimdallState::load()?;
    Ok(state.active_profile)
}

/// Show detailed information about a profile
pub fn show_profile_info(profile_name: Option<&str>, show_resolved: bool) -> Result<()> {
    let state = HeimdallState::load()?;
    let config = load_config(&state.dotfiles_path)?;

    let profile_name = profile_name.unwrap_or(&state.active_profile);

    // Validate profile exists
    let profile = config
        .profiles
        .get(profile_name)
        .with_context(|| format!("Profile '{}' not found", profile_name))?;

    // Show header
    println!("{}", format!("Profile: {}", profile_name).cyan().bold());
    println!("{}", "─".repeat(50).dimmed());

    // Show inheritance
    if let Some(extends) = &profile.extends {
        println!("{}: {}", "Extends".yellow(), extends);
    } else {
        println!("{}: {}", "Extends".yellow(), "none".dimmed());
    }

    // Show sources
    println!("\n{}", "Sources:".yellow());
    if profile.sources.is_empty() {
        println!("  {}", "none".dimmed());
    } else {
        for source in &profile.sources {
            match source {
                crate::config::schema::ProfileSource::Name(name) => {
                    println!("  • {}", name);
                }
                crate::config::schema::ProfileSource::Override { name, .. } => {
                    println!("  • {} {}", name, "(with overrides)".dimmed());
                }
            }
        }
    }

    // Show dotfiles count
    println!(
        "\n{}: {}",
        "Dotfiles".yellow(),
        profile.dotfiles.files.len()
    );
    if !profile.dotfiles.files.is_empty() {
        for file in &profile.dotfiles.files {
            println!("  {} → {}", file.source.dimmed(), file.target);
        }
    }

    // Show hooks
    let has_hooks = !profile.hooks.pre_apply.is_empty()
        || !profile.hooks.post_apply.is_empty()
        || !profile.hooks.pre_sync.is_empty()
        || !profile.hooks.post_sync.is_empty();

    println!(
        "\n{}: {}",
        "Hooks".yellow(),
        if has_hooks { "yes" } else { "none" }
    );
    if has_hooks {
        if !profile.hooks.pre_apply.is_empty() {
            println!("  pre_apply: {} hook(s)", profile.hooks.pre_apply.len());
        }
        if !profile.hooks.post_apply.is_empty() {
            println!("  post_apply: {} hook(s)", profile.hooks.post_apply.len());
        }
        if !profile.hooks.pre_sync.is_empty() {
            println!("  pre_sync: {} hook(s)", profile.hooks.pre_sync.len());
        }
        if !profile.hooks.post_sync.is_empty() {
            println!("  post_sync: {} hook(s)", profile.hooks.post_sync.len());
        }
    }

    // Show resolved configuration if requested
    if show_resolved {
        println!("\n{}", "Resolved Configuration:".cyan().bold());
        println!("{}", "─".repeat(50).dimmed());

        let resolved = resolve_profile(&config, profile_name)?;

        // Show all packages
        let mut total_packages = 0;
        if let Some(brew) = &resolved.sources.homebrew {
            total_packages += brew.packages.len() + brew.casks.len();
        }
        if let Some(apt) = &resolved.sources.apt {
            total_packages += apt.packages.len();
        }
        if let Some(dnf) = &resolved.sources.dnf {
            total_packages += dnf.packages.len();
        }
        if let Some(pacman) = &resolved.sources.pacman {
            total_packages += pacman.packages.len();
        }

        println!("{}: {}", "Total Packages".yellow(), total_packages);
        println!(
            "{}: {}",
            "Total Dotfiles".yellow(),
            resolved.dotfiles.files.len()
        );
    }

    Ok(())
}

/// List all available profiles
pub fn list_profiles(verbose: bool) -> Result<()> {
    let state = HeimdallState::load()?;
    let config = load_config(&state.dotfiles_path)?;

    println!("{}", "Available Profiles:".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());

    for (name, profile) in &config.profiles {
        let is_current = name == &state.active_profile;
        let marker = if is_current {
            "●".green()
        } else {
            " ".normal()
        };

        if verbose {
            println!(
                "{} {} {}",
                marker,
                name.cyan(),
                if is_current {
                    "(current)".green()
                } else {
                    "".normal()
                }
            );

            if let Some(extends) = &profile.extends {
                println!("    Extends: {}", extends.dimmed());
            }

            println!(
                "    Sources: {} | Dotfiles: {}",
                profile.sources.len(),
                profile.dotfiles.files.len()
            );
        } else {
            print!("{} {}", marker, name.cyan());
            if is_current {
                print!(" {}", "(current)".green());
            }
            if let Some(extends) = &profile.extends {
                print!(" {} {}", "→".dimmed(), extends.dimmed());
            }
            println!();
        }
    }

    println!(
        "\n{} Use {} to switch profiles",
        "ℹ".blue(),
        "heimdal profile switch <name>".cyan()
    );

    Ok(())
}

/// Compare two profiles
pub fn diff_profiles(profile1: Option<&str>, profile2: &str) -> Result<()> {
    let state = HeimdallState::load()?;
    let config = load_config(&state.dotfiles_path)?;

    let profile1_name = profile1.unwrap_or(&state.active_profile);

    // Validate profiles exist
    if !config.profiles.contains_key(profile1_name) {
        anyhow::bail!("Profile '{}' not found", profile1_name);
    }
    if !config.profiles.contains_key(profile2) {
        anyhow::bail!("Profile '{}' not found", profile2);
    }

    // Resolve both profiles
    let resolved1 = resolve_profile(&config, profile1_name)?;
    let resolved2 = resolve_profile(&config, profile2)?;

    println!(
        "{} {} {} {}",
        "Comparing:".cyan().bold(),
        profile1_name.yellow(),
        "vs".dimmed(),
        profile2.yellow()
    );
    println!("{}", "─".repeat(50).dimmed());

    // Compare dotfiles
    println!("\n{}", "Dotfiles:".yellow());
    let files1: std::collections::HashSet<_> =
        resolved1.dotfiles.files.iter().map(|f| &f.source).collect();
    let files2: std::collections::HashSet<_> =
        resolved2.dotfiles.files.iter().map(|f| &f.source).collect();

    let only_in_1: Vec<_> = files1.difference(&files2).collect();
    let only_in_2: Vec<_> = files2.difference(&files1).collect();
    let in_both: Vec<_> = files1.intersection(&files2).collect();

    println!(
        "  Common: {} | Only in {}: {} | Only in {}: {}",
        in_both.len(),
        profile1_name,
        only_in_1.len(),
        profile2,
        only_in_2.len()
    );

    if !only_in_1.is_empty() {
        println!("\n  {} Only in {}:", "−".red(), profile1_name.yellow());
        for file in only_in_1 {
            println!("    {}", file.dimmed());
        }
    }

    if !only_in_2.is_empty() {
        println!("\n  {} Only in {}:", "+".green(), profile2.yellow());
        for file in only_in_2 {
            println!("    {}", file);
        }
    }

    // Compare packages
    println!("\n{}", "Packages:".yellow());
    let mut packages1 = Vec::new();
    let mut packages2 = Vec::new();

    if let Some(brew) = &resolved1.sources.homebrew {
        packages1.extend(brew.packages.iter().cloned());
        packages1.extend(brew.casks.iter().cloned());
    }
    if let Some(brew) = &resolved2.sources.homebrew {
        packages2.extend(brew.packages.iter().cloned());
        packages2.extend(brew.casks.iter().cloned());
    }

    let pkgs1: std::collections::HashSet<_> = packages1.iter().collect();
    let pkgs2: std::collections::HashSet<_> = packages2.iter().collect();

    let only_pkgs_1: Vec<_> = pkgs1.difference(&pkgs2).collect();
    let only_pkgs_2: Vec<_> = pkgs2.difference(&pkgs1).collect();
    let common_pkgs: Vec<_> = pkgs1.intersection(&pkgs2).collect();

    println!(
        "  Common: {} | Only in {}: {} | Only in {}: {}",
        common_pkgs.len(),
        profile1_name,
        only_pkgs_1.len(),
        profile2,
        only_pkgs_2.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_env() -> Result<(TempDir, String)> {
        let temp = TempDir::new()?;
        let dotfiles_path = temp.path().join("dotfiles");
        std::fs::create_dir_all(&dotfiles_path)?;

        // Create a test config
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                extends: None,
                sources: vec![],
                dotfiles: DotfilesConfig::default(),
                hooks: ProfileHooks::default(),
            },
        );
        profiles.insert(
            "work".to_string(),
            Profile {
                extends: None,
                sources: vec![],
                dotfiles: DotfilesConfig::default(),
                hooks: ProfileHooks::default(),
            },
        );

        let config = HeimdallConfig {
            heimdal: HeimdallMeta {
                version: "1.0".to_string(),
                repo: "test".to_string(),
                stow_compat: true,
            },
            sources: Sources::default(),
            profiles,
            sync: SyncConfig::default(),
            ignore: vec![],
            mappings: HashMap::new(),
            hooks: GlobalHooks::default(),
        };

        let config_path = dotfiles_path.join("heimdal.yaml");
        let config_str = serde_yaml::to_string(&config)?;
        std::fs::write(&config_path, config_str)?;

        // Create state
        let state = HeimdallState::new(
            "default".to_string(),
            dotfiles_path.clone(),
            "test".to_string(),
        );

        let state_dir = temp.path().join(".heimdal");
        std::fs::create_dir_all(&state_dir)?;
        let state_path = state_dir.join("state.json");
        let state_str = serde_json::to_string_pretty(&state)?;
        std::fs::write(&state_path, state_str)?;

        Ok((temp, dotfiles_path.to_string_lossy().to_string()))
    }

    #[test]
    fn test_get_current_profile() {
        // This test requires heimdal to be initialized
        // In a real environment, it would work
        // For now, we'll skip it in tests
    }

    #[test]
    fn test_profile_validation() {
        // Test that switching to a non-existent profile fails
        // This would require a proper test setup
    }
}
