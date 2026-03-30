use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub machine_id: String,
    pub hostname: String,
    pub username: String,
    pub os: String,
    pub active_profile: String,
    pub dotfiles_path: PathBuf,
    pub repo_url: String,
    pub last_apply: Option<DateTime<Utc>>,
    pub last_sync: Option<DateTime<Utc>>,
    pub heimdal_version: String,
}

impl State {
    pub fn path() -> Result<PathBuf> {
        crate::utils::state_path()
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Err(crate::error::HeimdallError::NotInitialized.into());
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content)
            .map_err(|e| crate::error::HeimdallError::State(e.to_string()).into())
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn create(
        active_profile: String,
        dotfiles_path: PathBuf,
        repo_url: String,
    ) -> Result<Self> {
        let state = Self {
            version: 1,
            machine_id: Uuid::new_v4().to_string(),
            hostname: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            username: whoami::username(),
            os: crate::utils::os_name().to_string(),
            active_profile,
            dotfiles_path,
            repo_url,
            last_apply: None,
            last_sync: None,
            heimdal_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        state.save()?;
        Ok(state)
    }
}
