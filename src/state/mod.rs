pub mod conflict;
pub mod lock;
pub mod versioned;

// Re-export for backwards compatibility
pub use versioned::HeimdallStateV2;

// Original V1 state (kept for migration)
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Heimdal state stored in ~/.heimdal/ (V1 - Legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeimdallState {
    /// Currently active profile
    pub active_profile: String,

    /// Path to dotfiles directory
    pub dotfiles_path: PathBuf,

    /// Git repository URL
    pub repo_url: String,

    /// Last sync timestamp
    #[serde(default)]
    pub last_sync: Option<String>,

    /// Last successful apply timestamp
    #[serde(default)]
    pub last_apply: Option<String>,
}

impl HeimdallState {
    /// Create a new state
    pub fn new(profile: String, dotfiles_path: PathBuf, repo_url: String) -> Self {
        Self {
            active_profile: profile,
            dotfiles_path,
            repo_url,
            last_sync: None,
            last_apply: None,
        }
    }

    /// Load state from disk
    pub fn load() -> Result<Self> {
        let state_path = Self::state_path()?;

        if !state_path.exists() {
            anyhow::bail!(
                "Heimdal not initialized. Run 'heimdal init' first.\nState file not found: {}",
                state_path.display()
            );
        }

        let content = fs::read_to_string(&state_path)
            .with_context(|| format!("Failed to read state file: {}", state_path.display()))?;

        let state: HeimdallState =
            serde_json::from_str(&content).with_context(|| "Failed to parse state file")?;

        Ok(state)
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let state_dir = Self::state_dir()?;
        fs::create_dir_all(&state_dir).with_context(|| {
            format!("Failed to create state directory: {}", state_dir.display())
        })?;

        let state_path = Self::state_path()?;
        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize state")?;

        fs::write(&state_path, content)
            .with_context(|| format!("Failed to write state file: {}", state_path.display()))?;

        Ok(())
    }

    /// Get the state directory path (~/.heimdal)
    pub fn state_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;
        Ok(home.join(".heimdal"))
    }

    /// Get the state file path (~/.heimdal/heimdal.state.json)
    /// Also checks old state.json for backward compatibility
    pub fn state_path() -> Result<PathBuf> {
        let state_dir = Self::state_dir()?;

        // Check new naming first
        let new_path = state_dir.join("heimdal.state.json");
        if new_path.exists() {
            return Ok(new_path);
        }

        // Check old naming for backward compatibility
        let old_path = state_dir.join("state.json");
        if old_path.exists() {
            return Ok(old_path);
        }

        // Return new naming for new installations
        Ok(new_path)
    }

    /// Get the backup directory path (~/.heimdal/backups)
    pub fn backup_dir() -> Result<PathBuf> {
        Ok(Self::state_dir()?.join("backups"))
    }

    /// Update last sync timestamp
    pub fn update_sync_time(&mut self) {
        self.last_sync = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Update last apply timestamp
    pub fn update_apply_time(&mut self) {
        self.last_apply = Some(chrono::Utc::now().to_rfc3339());
    }
}
