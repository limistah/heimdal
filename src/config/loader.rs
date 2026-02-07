use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::schema::HeimdallConfig;

/// Load configuration from YAML file
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<HeimdallConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: HeimdallConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML config: {}", path.display()))?;

    Ok(config)
}

/// Validate configuration
pub fn validate_config(config: &HeimdallConfig) -> Result<()> {
    // Check version format
    if config.heimdal.version.is_empty() {
        anyhow::bail!("heimdal.version cannot be empty");
    }

    // Repo is optional - it can be specified in config or stored in state
    // No validation needed here

    // Check that at least one profile exists
    if config.profiles.is_empty() {
        anyhow::bail!("At least one profile must be defined");
    }

    // Validate profile inheritance
    for (profile_name, profile) in &config.profiles {
        if let Some(parent) = &profile.extends {
            if !config.profiles.contains_key(parent) {
                anyhow::bail!(
                    "Profile '{}' extends '{}' which does not exist",
                    profile_name,
                    parent
                );
            }

            // Check for circular inheritance
            let mut visited = vec![profile_name.clone()];
            let mut current = parent.clone();
            while let Some(p) = config.profiles.get(&current) {
                if visited.contains(&current) {
                    anyhow::bail!("Circular inheritance detected: {}", visited.join(" -> "));
                }
                visited.push(current.clone());
                if let Some(next) = &p.extends {
                    current = next.clone();
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_version() {
        use crate::config::schema::*;
        use std::collections::HashMap;

        let config = HeimdallConfig {
            heimdal: HeimdallMeta {
                version: "".to_string(),
                repo: Some("test".to_string()),
                stow_compat: true,
            },
            sources: Sources::default(),
            profiles: {
                let mut map = HashMap::new();
                map.insert(
                    "test".to_string(),
                    Profile {
                        extends: None,
                        sources: vec![],
                        dotfiles: DotfilesConfig::default(),
                        hooks: ProfileHooks::default(),
                        templates: ProfileTemplateConfig::default(),
                    },
                );
                map
            },
            sync: SyncConfig::default(),
            ignore: vec![],
            mappings: HashMap::new(),
            hooks: GlobalHooks::default(),
            templates: TemplateConfig::default(),
        };

        assert!(validate_config(&config).is_err());
    }
}
