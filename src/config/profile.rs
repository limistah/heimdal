use anyhow::{Context, Result};

use super::schema::{
    DotfilesConfig, HeimdallConfig, Profile, ProfileHooks, ProfileSource, SourceOverride, Sources,
};

/// Resolve a profile by merging with its parents (additive)
pub fn resolve_profile(config: &HeimdallConfig, profile_name: &str) -> Result<ResolvedProfile> {
    let _profile = config
        .profiles
        .get(profile_name)
        .with_context(|| format!("Profile '{}' not found", profile_name))?;

    // Build inheritance chain from root to leaf
    let mut chain = vec![profile_name.to_string()];
    let mut current = profile_name;

    while let Some(p) = config.profiles.get(current) {
        if let Some(parent) = &p.extends {
            chain.insert(0, parent.clone());
            current = parent;
        } else {
            break;
        }
    }

    // Merge from base to specific (additive)
    let mut resolved = ResolvedProfile {
        name: profile_name.to_string(),
        sources: config.sources.clone(),
        dotfiles: DotfilesConfig::default(),
        hooks: ProfileHooks::default(),
    };

    // First, add root-level ignore patterns
    resolved.dotfiles.ignore.extend(config.ignore.clone());

    for profile_name in chain {
        let profile = config.profiles.get(&profile_name).unwrap();
        merge_profile(&mut resolved, profile)?;
    }

    Ok(resolved)
}

/// Resolved profile with all inheritance applied
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub name: String,
    pub sources: Sources,
    pub dotfiles: DotfilesConfig,
    pub hooks: ProfileHooks,
}

/// Merge profile into resolved profile (additive)
fn merge_profile(resolved: &mut ResolvedProfile, profile: &Profile) -> Result<()> {
    // Merge sources (additive)
    for source in &profile.sources {
        match source {
            ProfileSource::Name(_name) => {
                // Just marks that this source should be used
                // The actual sources come from the root config
            }
            ProfileSource::Override { name, config } => {
                // Apply overrides to sources
                match (name.as_str(), config) {
                    ("homebrew", SourceOverride::Homebrew { packages, casks }) => {
                        if let Some(brew) = &mut resolved.sources.homebrew {
                            if let Some(pkgs) = packages {
                                brew.packages.extend(pkgs.clone());
                            }
                            if let Some(csks) = casks {
                                brew.casks.extend(csks.clone());
                            }
                        }
                    }
                    ("apt", SourceOverride::Apt { packages }) => {
                        if let Some(apt) = &mut resolved.sources.apt {
                            if let Some(pkgs) = packages {
                                apt.packages.extend(pkgs.clone());
                            }
                        }
                    }
                    ("github", SourceOverride::Github { repos: Some(repos) }) => {
                        resolved.sources.github.extend(repos.clone());
                    }
                    ("custom", SourceOverride::Custom { items: Some(items) }) => {
                        resolved.sources.custom.extend(items.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    // Merge dotfiles config
    if profile.dotfiles.use_stowrc {
        resolved.dotfiles.use_stowrc = true;
    }
    if profile.dotfiles.symlink_all {
        resolved.dotfiles.symlink_all = true;
    }
    resolved
        .dotfiles
        .ignore
        .extend(profile.dotfiles.ignore.clone());
    resolved
        .dotfiles
        .files
        .extend(profile.dotfiles.files.clone());

    // Merge hooks (additive)
    resolved
        .hooks
        .post_apply
        .extend(profile.hooks.post_apply.clone());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;
    use std::collections::HashMap;

    #[test]
    fn test_resolve_simple_profile() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "base".to_string(),
            Profile {
                extends: None,
                sources: vec![ProfileSource::Name("packages".to_string())],
                dotfiles: DotfilesConfig::default(),
                hooks: ProfileHooks::default(),
                templates: ProfileTemplateConfig::default(),
            },
        );

        let config = HeimdallConfig {
            heimdal: HeimdallMeta {
                version: "1.0".to_string(),
                repo: Some("test".to_string()),
                stow_compat: true,
            },
            sources: Sources::default(),
            profiles,
            sync: SyncConfig::default(),
            ignore: vec![],
            mappings: HashMap::new(),
            hooks: GlobalHooks::default(),
            templates: TemplateConfig::default(),
        };

        let resolved = resolve_profile(&config, "base").unwrap();
        assert_eq!(resolved.name, "base");
    }
}
