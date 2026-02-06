use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeimdallConfig {
    pub heimdal: HeimdallMeta,
    #[serde(default)]
    pub sources: Sources,
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub mappings: HashMap<String, PackageMapping>,
}

/// Metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeimdallMeta {
    pub version: String,
    pub repo: String,
    #[serde(default = "default_stow_compat")]
    pub stow_compat: bool,
}

fn default_stow_compat() -> bool {
    true
}

/// Package sources configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sources {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub homebrew: Option<HomebrewSource>,
    #[serde(default)]
    pub mas: Option<MasSource>,
    #[serde(default)]
    pub apt: Option<AptSource>,
    #[serde(default)]
    pub dnf: Option<DnfSource>,
    #[serde(default)]
    pub pacman: Option<PacmanSource>,
    #[serde(default)]
    pub github: Vec<GitHubRepo>,
    #[serde(default)]
    pub custom: Vec<CustomInstall>,
}

/// Homebrew source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomebrewSource {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub casks: Vec<String>,
    #[serde(default)]
    pub hooks: Hooks,
}

/// Mac App Store source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasSource {
    pub packages: Vec<MasPackage>,
    #[serde(default)]
    pub hooks: Hooks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasPackage {
    pub id: u64,
    pub name: String,
}

/// APT source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptSource {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub hooks: Hooks,
}

/// DNF source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnfSource {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub hooks: Hooks,
}

/// Pacman source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacmanSource {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub hooks: Hooks,
}

/// GitHub repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub url: String,
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub dest: Option<String>, // Alias for destination
    #[serde(default = "default_true")]
    pub submodule: bool,
    #[serde(default)]
    pub hooks: RepoHooks,
}

/// Custom installation script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomInstall {
    pub name: String,
    #[serde(default)]
    pub hooks: CustomHooks,
}

/// Hook configuration for custom installs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomHooks {
    #[serde(default)]
    pub check: Vec<HookCommand>,
    #[serde(default)]
    pub install: Vec<HookCommand>,
    #[serde(default)]
    pub post_install: Vec<HookCommand>,
}

/// Hooks for package managers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(default)]
    pub pre_install: Vec<HookCommand>,
    #[serde(default)]
    pub post_install: Vec<HookCommand>,
}

/// Hooks for git repos
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoHooks {
    #[serde(default)]
    pub post_clone: Vec<HookCommand>,
}

/// A single hook command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookCommand {
    Simple(String),
    Detailed {
        command: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        os: Vec<String>,
        #[serde(default)]
        shell: Vec<String>,
        #[serde(default)]
        when: Option<String>,
        #[serde(default = "default_true")]
        fail_on_error: bool,
    },
}

/// Machine profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub sources: Vec<ProfileSource>,
    #[serde(default)]
    pub dotfiles: DotfilesConfig,
    #[serde(default)]
    pub hooks: ProfileHooks,
}

/// Profile-specific source (for overrides)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProfileSource {
    Name(String),
    Override {
        name: String,
        #[serde(flatten)]
        config: SourceOverride,
    },
}

/// Source override in profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceOverride {
    Homebrew {
        packages: Option<Vec<String>>,
        casks: Option<Vec<String>>,
    },
    Apt {
        packages: Option<Vec<String>>,
    },
    Github {
        repos: Option<Vec<GitHubRepo>>,
    },
    Custom {
        items: Option<Vec<CustomInstall>>,
    },
}

/// Dotfiles symlinking configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DotfilesConfig {
    #[serde(default = "default_true")]
    pub use_stowrc: bool,
    #[serde(default)]
    pub symlink_all: bool,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub files: Vec<DotfileMapping>,
}

/// Individual dotfile mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfileMapping {
    pub source: String,
    pub target: String,
}

/// Profile-level hooks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileHooks {
    #[serde(default)]
    pub post_apply: Vec<HookCommand>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub interval: String,
    #[serde(default = "default_true")]
    pub auto_apply: bool,
    #[serde(default)]
    pub notify: NotifyConfig,
    #[serde(default = "default_true")]
    pub rollback_on_error: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: "1h".to_string(),
            auto_apply: true,
            notify: NotifyConfig::default(),
            rollback_on_error: true,
        }
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyConfig {
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default = "default_true")]
    pub log: bool,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            log: true,
        }
    }
}

/// Package name mapping for different package managers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMapping {
    pub apt: Option<String>,
    pub brew: Option<String>,
    pub dnf: Option<String>,
    pub pacman: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> String {
    "1h".to_string()
}
