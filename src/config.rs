use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeimdalConfig {
    pub heimdal: HeimdalMeta,
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub packages: PackageMap,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeimdalMeta {
    pub version: String,
    #[serde(default)]
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub dotfiles: Vec<DotfileEntry>,
    #[serde(default)]
    pub packages: PackageMap,
    #[serde(default)]
    pub hooks: ProfileHooks,
    #[serde(default)]
    pub templates: Vec<TemplateEntry>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DotfileEntry {
    Simple(String),
    Mapped(DotfileMapping),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DotfileMapping {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub when: Option<DotfileCondition>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DotfileCondition {
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub profile: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PackageMap {
    #[serde(default)]
    pub common: Vec<String>,
    #[serde(default)]
    pub homebrew: Vec<String>,
    #[serde(default)]
    pub homebrew_casks: Vec<String>,
    #[serde(default)]
    pub apt: Vec<String>,
    #[serde(default)]
    pub dnf: Vec<String>,
    #[serde(default)]
    pub pacman: Vec<String>,
    #[serde(default)]
    pub apk: Vec<String>,
    #[serde(default)]
    pub mas: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProfileHooks {
    #[serde(default)]
    pub pre_apply: Vec<HookEntry>,
    #[serde(default)]
    pub post_apply: Vec<HookEntry>,
    #[serde(default)]
    pub pre_sync: Vec<HookEntry>,
    #[serde(default)]
    pub post_sync: Vec<HookEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum HookEntry {
    Simple(String),
    Full {
        command: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        os: Vec<String>,
        #[serde(default = "default_true")]
        fail_on_error: bool,
    },
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TemplateEntry {
    pub src: String,
    pub dest: String,
    #[serde(default)]
    pub vars: HashMap<String, String>,
}

pub fn load_config(path: &Path) -> anyhow::Result<HeimdalConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        crate::error::HeimdallError::Config(format!(
            "Cannot read {}: {}. Run: heimdal init",
            path.display(),
            e
        ))
    })?;
    serde_yaml_ng::from_str(&content)
        .map_err(|e| crate::error::HeimdallError::Config(e.to_string()).into())
}

pub fn resolve_profile(config: &HeimdalConfig, name: &str) -> anyhow::Result<Profile> {
    let mut profile = resolve_recursive(config, name, &mut Vec::new())?;
    // Prepend top-level packages so profile-specific ones take effect after
    profile.packages = merge_packages(config.packages.clone(), profile.packages);
    Ok(profile)
}

fn resolve_recursive(
    config: &HeimdalConfig,
    name: &str,
    chain: &mut Vec<String>,
) -> anyhow::Result<Profile> {
    if chain.contains(&name.to_string()) {
        return Err(anyhow::anyhow!(
            "Circular extends detected: {} -> {}",
            chain.join(" -> "),
            name
        ));
    }
    let profile = config
        .profiles
        .get(name)
        .ok_or_else(|| crate::error::HeimdallError::ProfileNotFound {
            name: name.to_string(),
        })?
        .clone();

    match &profile.extends.clone() {
        None => Ok(profile),
        Some(parent_name) => {
            chain.push(name.to_string());
            let parent = resolve_recursive(config, parent_name, chain)?;
            Ok(merge_profiles(parent, profile))
        }
    }
}

fn merge_profiles(base: Profile, child: Profile) -> Profile {
    Profile {
        extends: None,
        dotfiles: {
            let mut d = base.dotfiles;
            d.extend(child.dotfiles);
            d
        },
        packages: merge_packages(base.packages, child.packages),
        // Hooks: child completely replaces parent hooks (not merged).
        // A child profile that wants parent hooks must explicitly repeat them.
        // This is intentional — lifecycle hooks are profile-specific scripts.
        hooks: child.hooks,
        templates: {
            let mut t = base.templates;
            t.extend(child.templates);
            t
        },
        ignore: {
            let mut i = base.ignore;
            i.extend(child.ignore);
            i
        },
    }
}

fn merge_packages(base: PackageMap, child: PackageMap) -> PackageMap {
    PackageMap {
        common: {
            let mut v = base.common;
            v.extend(child.common);
            v
        },
        homebrew: {
            let mut v = base.homebrew;
            v.extend(child.homebrew);
            v
        },
        homebrew_casks: {
            let mut v = base.homebrew_casks;
            v.extend(child.homebrew_casks);
            v
        },
        apt: {
            let mut v = base.apt;
            v.extend(child.apt);
            v
        },
        dnf: {
            let mut v = base.dnf;
            v.extend(child.dnf);
            v
        },
        pacman: {
            let mut v = base.pacman;
            v.extend(child.pacman);
            v
        },
        apk: {
            let mut v = base.apk;
            v.extend(child.apk);
            v
        },
        mas: {
            let mut v = base.mas;
            v.extend(child.mas);
            v
        },
    }
}

/// Validate config for logical errors (after YAML parse succeeds).
/// Returns a list of human-readable error strings (empty = valid).
pub fn validate_config(config: &HeimdalConfig) -> Vec<String> {
    let mut errors = Vec::new();

    // Check extends targets exist
    for (name, profile) in &config.profiles {
        if let Some(parent) = &profile.extends {
            if !config.profiles.contains_key(parent.as_str()) {
                errors.push(format!(
                    "Profile '{}' extends '{}' which does not exist",
                    name, parent
                ));
            }
        }
    }

    // Check for circular extends — report each cycle only once
    let mut reported_cycles: std::collections::HashSet<String> = std::collections::HashSet::new();
    for name in config.profiles.keys() {
        let mut chain: Vec<&str> = vec![];
        let mut current = name.as_str();
        loop {
            if let Some(pos) = chain.iter().position(|&n| n == current) {
                // Build canonical cycle key (sort the cycle nodes to deduplicate)
                let cycle_nodes = &chain[pos..];
                let mut sorted_key: Vec<&str> = cycle_nodes.to_vec();
                sorted_key.sort_unstable();
                let key = sorted_key.join(",");
                if reported_cycles.insert(key) {
                    // Show full cycle with closing node
                    let mut display = chain[pos..].to_vec();
                    display.push(current);
                    errors.push(format!(
                        "Circular extends detected: {}",
                        display.join(" → ")
                    ));
                }
                break;
            }
            chain.push(current);
            match config
                .profiles
                .get(current)
                .and_then(|p| p.extends.as_deref())
            {
                None => break,
                Some(next) => {
                    if !config.profiles.contains_key(next) {
                        break; // Unknown extends already reported in previous loop
                    }
                    current = next;
                }
            }
        }
    }

    // Check dotfile source paths are relative and don't traverse outside dotfiles dir
    for (prof_name, profile) in &config.profiles {
        for entry in &profile.dotfiles {
            let src = match entry {
                DotfileEntry::Simple(s) => s.as_str(),
                DotfileEntry::Mapped(m) => m.source.as_str(),
            };
            if std::path::Path::new(src).is_absolute() {
                errors.push(format!(
                    "Profile '{}': dotfile source '{}' must be a relative path",
                    prof_name, src
                ));
            }
            // Check for path traversal attempts using proper component inspection
            let has_parent_dir = std::path::Path::new(src)
                .components()
                .any(|c| c == std::path::Component::ParentDir);
            if has_parent_dir {
                errors.push(format!(
                    "Profile '{}': dotfile source '{}' must not contain '..' components",
                    prof_name, src
                ));
            }
        }
    }

    errors
}

/// Write a minimal valid heimdal.yaml to `path` for the given profile name.
#[allow(dead_code)]
pub fn create_minimal_config(path: &std::path::Path, profile_name: &str) -> anyhow::Result<()> {
    let mut profiles = HashMap::new();
    profiles.insert(
        profile_name.to_string(),
        Profile {
            dotfiles: vec![],
            packages: PackageMap {
                homebrew: vec![],
                apt: vec![],
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let config = HeimdalConfig {
        heimdal: HeimdalMeta {
            version: "1".to_string(),
            repo: None,
        },
        profiles,
        packages: PackageMap::default(),
        ignore: vec![],
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_yaml_ng::to_string(&config)?;
    std::fs::write(path, content)?;
    Ok(())
}
