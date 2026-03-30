use crate::cli::ProfileCmd;
use crate::config::{load_config, resolve_profile, DotfileEntry, HeimdalConfig, Profile};
use crate::error::HeimdallError;
use crate::state::State;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(action: ProfileCmd) -> Result<()> {
    match action {
        ProfileCmd::List => list(),
        ProfileCmd::Current => current(),
        ProfileCmd::Switch { name, no_apply } => switch(&name, no_apply),
        ProfileCmd::Show { name, resolved } => show(name.as_deref(), resolved),
        ProfileCmd::Create { name, extends } => create(&name, extends.as_deref()),
        ProfileCmd::Clone { source, dest } => clone_profile(&source, &dest),
        ProfileCmd::Diff { profile1, profile2 } => diff_profiles(profile1.as_deref(), &profile2),
    }
}

fn list() -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;

    let mut names: Vec<_> = config.profiles.keys().cloned().collect();
    names.sort();

    for name in &names {
        if name == &state.active_profile {
            println!("* {}", name); // active marker
        } else {
            println!("  {}", name);
        }
    }
    Ok(())
}

fn current() -> Result<()> {
    let state = State::load()?;
    println!("{}", state.active_profile);
    Ok(())
}

fn switch(name: &str, no_apply: bool) -> Result<()> {
    let mut state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;

    if !config.profiles.contains_key(name) {
        let mut available: Vec<_> = config.profiles.keys().cloned().collect();
        available.sort();
        eprintln!("Available profiles: {}", available.join(", "));
        return Err(HeimdallError::ProfileNotFound {
            name: name.to_string(),
        }
        .into());
    }

    if state.active_profile == name {
        info(&format!("Already on profile '{}'", name));
        return Ok(());
    }

    state.active_profile = name.to_string();
    state.save()?;
    success(&format!("Switched to profile '{}'", name));

    if !no_apply {
        info("Run 'heimdal apply' to apply the new profile.");
    }
    Ok(())
}

fn show(name: Option<&str>, resolved: bool) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;

    let profile_name = name.unwrap_or(&state.active_profile);

    let profile = if resolved {
        resolve_profile(&config, profile_name)?
    } else {
        config
            .profiles
            .get(profile_name)
            .ok_or_else(|| HeimdallError::ProfileNotFound {
                name: profile_name.to_string(),
            })?
            .clone()
    };

    println!("Profile: {}", profile_name);
    if let Some(ext) = &profile.extends {
        println!("Extends: {}", ext);
    }

    if !profile.dotfiles.is_empty() {
        println!("\nDotfiles:");
        for entry in &profile.dotfiles {
            match entry {
                DotfileEntry::Simple(s) => println!("  - {}", s),
                DotfileEntry::Mapped(m) => println!("  - {} → {}", m.source, m.target),
            }
        }
    } else {
        println!("\nDotfiles: (stow walk — all top-level files)");
    }

    println!("\nPackages:");
    let pkgs = &profile.packages;
    if !pkgs.homebrew.is_empty() {
        println!("  homebrew: {}", pkgs.homebrew.join(", "));
    }
    if !pkgs.apt.is_empty() {
        println!("  apt: {}", pkgs.apt.join(", "));
    }
    if !pkgs.dnf.is_empty() {
        println!("  dnf: {}", pkgs.dnf.join(", "));
    }
    if !pkgs.pacman.is_empty() {
        println!("  pacman: {}", pkgs.pacman.join(", "));
    }

    Ok(())
}

fn create(name: &str, extends: Option<&str>) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    if config.profiles.contains_key(name) {
        anyhow::bail!(
            "Profile '{}' already exists. Use 'heimdal profile list' to see all profiles.",
            name
        );
    }

    if let Some(parent) = extends {
        if !config.profiles.contains_key(parent) {
            return Err(HeimdallError::ProfileNotFound {
                name: parent.to_string(),
            }
            .into());
        }
    }

    let new_profile = Profile {
        extends: extends.map(|s| s.to_string()),
        ..Default::default()
    };

    config.profiles.insert(name.to_string(), new_profile);
    write_config(&config_path, &config)?;
    success(&format!("Created profile '{}'", name));
    Ok(())
}

fn clone_profile(source: &str, dest: &str) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let mut config = load_config(&config_path)?;

    let source_profile = config
        .profiles
        .get(source)
        .ok_or_else(|| HeimdallError::ProfileNotFound {
            name: source.to_string(),
        })?
        .clone();

    if config.profiles.contains_key(dest) {
        anyhow::bail!("Profile '{}' already exists.", dest);
    }

    config.profiles.insert(dest.to_string(), source_profile);
    write_config(&config_path, &config)?;
    success(&format!("Cloned '{}' → '{}'", source, dest));
    Ok(())
}

fn diff_profiles(profile1: Option<&str>, profile2: &str) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;

    let name1 = profile1.unwrap_or(&state.active_profile);

    let p1 = resolve_profile(&config, name1)?;
    let p2 = resolve_profile(&config, profile2)?;

    println!("Diff: {} vs {}", name1, profile2);
    println!("\nDotfiles in {} but not {}:", name1, profile2);
    let p1_sources: std::collections::HashSet<String> = p1
        .dotfiles
        .iter()
        .map(|e| match e {
            DotfileEntry::Simple(s) => s.clone(),
            DotfileEntry::Mapped(m) => m.source.clone(),
        })
        .collect();
    let p2_sources: std::collections::HashSet<String> = p2
        .dotfiles
        .iter()
        .map(|e| match e {
            DotfileEntry::Simple(s) => s.clone(),
            DotfileEntry::Mapped(m) => m.source.clone(),
        })
        .collect();

    for s in p1_sources.difference(&p2_sources) {
        println!("  - {}", s);
    }
    println!("Dotfiles in {} but not {}:", profile2, name1);
    for s in p2_sources.difference(&p1_sources) {
        println!("  + {}", s);
    }

    Ok(())
}

/// Serialize config back to YAML and write atomically.
fn write_config(path: &std::path::Path, config: &HeimdalConfig) -> Result<()> {
    let content = serde_yaml_ng::to_string(config)?;
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
