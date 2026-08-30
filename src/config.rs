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
    #[serde(default)]
    pub history: Option<HistoryConfig>,
    #[serde(default)]
    pub hooks: ProfileHooks,
    #[serde(default)]
    pub defaults: Option<DefaultsConfig>,
    #[serde(default = "default_parallel_jobs")]
    pub parallel_jobs: usize,
}

fn default_parallel_jobs() -> usize {
    4
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeimdalMeta {
    pub version: String,
    #[serde(default)]
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub sync: bool,
    #[serde(default = "max_age_days_default")]
    pub max_age_days: u32,
}

fn max_age_days_default() -> u32 {
    90
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "defaults_path_default")]
    pub path: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        }
    }
}

fn defaults_path_default() -> String {
    "macos-defaults".to_string()
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

impl DotfileEntry {
    /// Get the source path (relative to dotfiles directory).
    pub fn source(&self) -> &str {
        match self {
            DotfileEntry::Simple(s) => s,
            DotfileEntry::Mapped(m) => &m.source,
        }
    }

    /// Get the target path (with ~ prefix for home directory).
    pub fn target(&self) -> String {
        match self {
            DotfileEntry::Simple(s) => format!("~/{}", s),
            DotfileEntry::Mapped(m) => m.target.clone(),
        }
    }
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

/// An entry in `packages.common`.
///
/// Most packages have the same name across every package manager (e.g.
/// `zsh`), so a plain string is enough. Some packages diverge by ecosystem
/// (VS Code is `visual-studio-code` on a Homebrew cask but `code` on
/// apt/dnf/pacman) and need a per-manager name override instead.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommonPackage {
    /// A package name that is identical across every package manager.
    Simple(String),
    /// A package whose install name differs per package manager.
    Aliased(CommonPackageAliases),
}

/// Per-manager name overrides for a `common` package entry. `default` is
/// used whenever the manager that actually ran has no override of its own.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CommonPackageAliases {
    pub default: String,
    #[serde(default)]
    pub homebrew: Option<String>,
    #[serde(default)]
    pub homebrew_casks: Option<String>,
    #[serde(default)]
    pub apt: Option<String>,
    #[serde(default)]
    pub dnf: Option<String>,
    #[serde(default)]
    pub pacman: Option<String>,
    #[serde(default)]
    pub apk: Option<String>,
    #[serde(default)]
    pub mas: Option<String>,
}

impl CommonPackage {
    /// Resolve the package name to use for the given manager field name
    /// (a `PackageManager::field_name()`, e.g. "homebrew", "apt",
    /// "homebrew_casks"). Falls back to the aliased entry's `default`, or
    /// returns the plain string unchanged for a `Simple` entry.
    pub fn resolve(&self, field: &str) -> String {
        match self {
            CommonPackage::Simple(name) => name.clone(),
            CommonPackage::Aliased(aliases) => aliases
                .for_field(field)
                .cloned()
                .unwrap_or_else(|| aliases.default.clone()),
        }
    }
}

impl CommonPackageAliases {
    fn for_field(&self, field: &str) -> Option<&String> {
        match field {
            "homebrew" => self.homebrew.as_ref(),
            "homebrew_casks" => self.homebrew_casks.as_ref(),
            "apt" => self.apt.as_ref(),
            "dnf" => self.dnf.as_ref(),
            "pacman" => self.pacman.as_ref(),
            "apk" => self.apk.as_ref(),
            "mas" => self.mas.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PackageMap {
    #[serde(default)]
    pub common: Vec<CommonPackage>,
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

/// Common context loaded by most commands.
pub struct CommandContext {
    pub state: crate::state::State,
    pub config: HeimdalConfig,
    pub profile: Profile,
}

impl CommandContext {
    /// Load state, config, and resolved profile.
    pub fn load() -> anyhow::Result<Self> {
        let state = crate::state::State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, &state.active_profile)?;
        Ok(Self {
            state,
            config,
            profile,
        })
    }

    /// Load with a specific profile override.
    #[allow(dead_code)]
    pub fn load_with_profile(profile_name: &str) -> anyhow::Result<Self> {
        let state = crate::state::State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, profile_name)?;
        Ok(Self {
            state,
            config,
            profile,
        })
    }
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
    // Prepend top-level ignore so profile-specific ones take effect after
    let mut combined_ignore = config.ignore.clone();
    combined_ignore.extend(profile.ignore);
    profile.ignore = combined_ignore;
    // Prepend global hooks so profile-specific ones run after
    profile.hooks = merge_hooks(config.hooks.clone(), profile.hooks);
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

macro_rules! merge_vec {
    ($base:expr, $child:expr) => {{
        let mut v = $base;
        v.extend($child);
        v
    }};
}

fn merge_packages(base: PackageMap, child: PackageMap) -> PackageMap {
    PackageMap {
        common: merge_vec!(base.common, child.common),
        homebrew: merge_vec!(base.homebrew, child.homebrew),
        homebrew_casks: merge_vec!(base.homebrew_casks, child.homebrew_casks),
        apt: merge_vec!(base.apt, child.apt),
        dnf: merge_vec!(base.dnf, child.dnf),
        pacman: merge_vec!(base.pacman, child.pacman),
        apk: merge_vec!(base.apk, child.apk),
        mas: merge_vec!(base.mas, child.mas),
    }
}

fn merge_hooks(base: ProfileHooks, child: ProfileHooks) -> ProfileHooks {
    ProfileHooks {
        pre_apply: {
            let mut v = base.pre_apply;
            v.extend(child.pre_apply);
            v
        },
        post_apply: {
            let mut v = base.post_apply;
            v.extend(child.post_apply);
            v
        },
        pre_sync: {
            let mut v = base.pre_sync;
            v.extend(child.pre_sync);
            v
        },
        post_sync: {
            let mut v = base.post_sync;
            v.extend(child.post_sync);
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
            let src = entry.source();
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
        history: None,
        hooks: ProfileHooks::default(),
        defaults: None,
        parallel_jobs: default_parallel_jobs(),
    };

    crate::utils::ensure_parent_exists(path)?;
    let content = serde_yaml_ng::to_string(&config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Write HeimdalConfig to a YAML file atomically.
pub fn write_config(path: &Path, config: &HeimdalConfig) -> anyhow::Result<()> {
    let content = serde_yaml_ng::to_string(config)?;
    crate::utils::atomic_write(path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_profile_merges_top_level_ignore() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            Profile {
                ignore: vec!["profile.txt".to_string()],
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
            ignore: vec![".git".to_string(), "*.md".to_string()],
            history: None,
            hooks: ProfileHooks::default(),
            defaults: None,
            parallel_jobs: 4,
        };

        let resolved = resolve_profile(&config, "test").unwrap();

        // Verify: top-level ignore prepended to profile ignore
        assert_eq!(resolved.ignore.len(), 3);
        assert_eq!(resolved.ignore[0], ".git");
        assert_eq!(resolved.ignore[1], "*.md");
        assert_eq!(resolved.ignore[2], "profile.txt");
    }

    #[test]
    fn resolve_profile_merges_top_level_packages() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            Profile {
                packages: PackageMap {
                    common: vec![CommonPackage::Simple("profile-pkg".to_string())],
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
            packages: PackageMap {
                common: vec![CommonPackage::Simple("top-pkg".to_string())],
                ..Default::default()
            },
            ignore: vec![],
            history: None,
            hooks: ProfileHooks::default(),
            defaults: None,
            parallel_jobs: 4,
        };

        let resolved = resolve_profile(&config, "test").unwrap();

        // Verify: top-level packages prepended to profile packages
        assert_eq!(resolved.packages.common.len(), 2);
        assert_eq!(
            resolved.packages.common[0],
            CommonPackage::Simple("top-pkg".to_string())
        );
        assert_eq!(
            resolved.packages.common[1],
            CommonPackage::Simple("profile-pkg".to_string())
        );
    }

    #[test]
    fn resolve_profile_merges_global_hooks() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "test".to_string(),
            Profile {
                hooks: ProfileHooks {
                    post_apply: vec![HookEntry::Simple("profile-hook".to_string())],
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
            history: None,
            hooks: ProfileHooks {
                post_apply: vec![HookEntry::Simple("global-hook".to_string())],
                ..Default::default()
            },
            defaults: None,
            parallel_jobs: 4,
        };

        let resolved = resolve_profile(&config, "test").unwrap();

        // Verify: global hooks prepended to profile hooks
        assert_eq!(resolved.hooks.post_apply.len(), 2);
        match &resolved.hooks.post_apply[0] {
            HookEntry::Simple(cmd) => assert_eq!(cmd, "global-hook"),
            _ => panic!("Expected Simple hook"),
        }
        match &resolved.hooks.post_apply[1] {
            HookEntry::Simple(cmd) => assert_eq!(cmd, "profile-hook"),
            _ => panic!("Expected Simple hook"),
        }
    }

    #[test]
    fn test_defaults_config_parses() {
        let yaml = r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
defaults:
  enabled: true
  include:
    - com.apple.dock
    - com.apple.finder
  exclude:
    - com.apple.Safari.SandboxBroker
"#;
        let config: HeimdalConfig = serde_yaml_ng::from_str(yaml).unwrap();
        let defaults = config.defaults.unwrap();
        assert!(defaults.enabled);
        assert_eq!(defaults.include, vec!["com.apple.dock", "com.apple.finder"]);
        assert_eq!(defaults.exclude, vec!["com.apple.Safari.SandboxBroker"]);
    }

    #[test]
    fn test_defaults_config_yaml_defaults() {
        let yaml = r#"
defaults: {}
"#;
        let config: DefaultsConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.path, "macos-defaults");
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_parallel_jobs_default() {
        let yaml = "heimdal:\n  version: \"1\"\nprofiles:\n  default: {}\n";
        let config: HeimdalConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.parallel_jobs, 4);
    }

    #[test]
    fn test_parallel_jobs_explicit() {
        let yaml = "heimdal:\n  version: \"1\"\nprofiles:\n  default: {}\nparallel_jobs: 8\n";
        let config: HeimdalConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.parallel_jobs, 8);
    }

    #[test]
    fn test_defaults_config_defaults_to_none() {
        let yaml = r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
"#;
        let config: HeimdalConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(config.defaults.is_none());
    }

    #[test]
    fn common_package_plain_string_parses_and_resolves_unchanged() {
        let yaml = "common:\n  - zsh\n";
        let pkgs: PackageMap = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(pkgs.common, vec![CommonPackage::Simple("zsh".to_string())]);
        // A plain-string entry resolves to the same name regardless of
        // which manager actually ran.
        assert_eq!(pkgs.common[0].resolve("homebrew"), "zsh");
        assert_eq!(pkgs.common[0].resolve("apt"), "zsh");
        assert_eq!(pkgs.common[0].resolve("anything-unknown"), "zsh");
    }

    #[test]
    fn common_package_aliased_resolves_per_manager_with_default_fallback() {
        let yaml = r#"
common:
  - default: docker-desktop
    homebrew_casks: docker-desktop
    apt: docker-ce
    dnf: docker-ce
    pacman: docker
    apk: docker
"#;
        let pkgs: PackageMap = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(pkgs.common.len(), 1);
        let entry = &pkgs.common[0];

        assert_eq!(entry.resolve("homebrew_casks"), "docker-desktop");
        assert_eq!(entry.resolve("apt"), "docker-ce");
        assert_eq!(entry.resolve("dnf"), "docker-ce");
        assert_eq!(entry.resolve("pacman"), "docker");
        assert_eq!(entry.resolve("apk"), "docker");
        // No "mas" override was given, so it falls back to `default`.
        assert_eq!(entry.resolve("mas"), "docker-desktop");
        // A manager with no matching field at all also falls back to `default`.
        assert_eq!(entry.resolve("homebrew"), "docker-desktop");
    }

    #[test]
    fn common_package_yaml_round_trip_plain_and_aliased() {
        let original = vec![
            CommonPackage::Simple("zsh".to_string()),
            CommonPackage::Aliased(CommonPackageAliases {
                default: "docker-desktop".to_string(),
                homebrew: None,
                homebrew_casks: Some("docker-desktop".to_string()),
                apt: Some("docker-ce".to_string()),
                dnf: Some("docker-ce".to_string()),
                pacman: Some("docker".to_string()),
                apk: Some("docker".to_string()),
                mas: None,
            }),
        ];

        let yaml = serde_yaml_ng::to_string(&original).unwrap();
        let round_tripped: Vec<CommonPackage> = serde_yaml_ng::from_str(&yaml).unwrap();

        assert_eq!(round_tripped, original);
    }
}
