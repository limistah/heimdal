use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use uuid::Uuid;

/// What heimdal itself has installed via one package manager, and when it
/// last recorded a change. Keyed in `State::package_inventory` by
/// `PackageManager::field_name()` (e.g. "homebrew", "apt", "mas").
///
/// This is heimdal's own install ledger, not a live query of the system —
/// see `crate::packages::query_installed` for that.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageInventoryEntry {
    /// Identifiers heimdal has successfully installed for this manager
    /// (package names, or numeric App Store ids for `mas`).
    #[serde(default)]
    pub identifiers: BTreeSet<String>,
    /// When this entry was last updated by a successful install.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Packages heimdal itself has installed, per manager. Additive field —
    /// absent in state.json files written before this existed, so it
    /// defaults to empty rather than failing to parse.
    #[serde(default)]
    pub package_inventory: BTreeMap<String, PackageInventoryEntry>,
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
        crate::utils::ensure_parent_exists(&path)?;
        let content = serde_json::to_string_pretty(self)?;
        crate::utils::atomic_write(&path, content.as_bytes())?;
        Ok(())
    }

    pub fn create(
        active_profile: String,
        dotfiles_path: PathBuf,
        repo_url: String,
    ) -> Result<Self> {
        let state = Self {
            version: 1,
            machine_id: Uuid::new_v4().to_string(),
            hostname: crate::utils::hostname(),
            username: whoami::username(),
            os: crate::utils::os_name().to_string(),
            active_profile,
            dotfiles_path,
            repo_url,
            last_apply: None,
            last_sync: None,
            heimdal_version: env!("CARGO_PKG_VERSION").to_string(),
            package_inventory: BTreeMap::new(),
        };
        state.save()?;
        Ok(state)
    }

    /// Record that `identifiers` were successfully installed via the package
    /// manager identified by `manager_field` (a `PackageManager::field_name()`,
    /// e.g. "homebrew", "apt", "mas"), merging them into any existing
    /// inventory for that manager and stamping the current time. Does not
    /// save to disk — callers persist via `save()` once done recording.
    pub fn record_installed<I>(&mut self, manager_field: &str, identifiers: I)
    where
        I: IntoIterator<Item = String>,
    {
        let entry = self
            .package_inventory
            .entry(manager_field.to_string())
            .or_default();
        entry.identifiers.extend(identifiers);
        entry.updated_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_old_state_json_without_package_inventory_still_parses() {
        // Simulates a state.json written before `package_inventory` existed.
        let json = serde_json::json!({
            "version": 1, "machine_id": "x", "hostname": "h", "username": "u",
            "os": "linux", "active_profile": "default",
            "dotfiles_path": "/home/u/.dotfiles",
            "repo_url": "", "last_apply": null, "last_sync": null,
            "heimdal_version": "3.0.0"
        })
        .to_string();

        let state: State = serde_json::from_str(&json).expect("old state.json must still parse");
        assert!(
            state.package_inventory.is_empty(),
            "missing field should default to an empty inventory"
        );
    }

    #[test]
    fn test_record_installed_creates_entry_with_identifiers() {
        let mut state = State::default();
        state.record_installed("homebrew", vec!["git".to_string(), "curl".to_string()]);

        let entry = state.package_inventory.get("homebrew").unwrap();
        assert!(entry.identifiers.contains("git"));
        assert!(entry.identifiers.contains("curl"));
        assert!(entry.updated_at.is_some());
    }

    #[test]
    fn test_record_installed_merges_into_existing_entry() {
        let mut state = State::default();
        state.record_installed("apt", vec!["git".to_string()]);
        state.record_installed("apt", vec!["vim".to_string()]);

        let entry = state.package_inventory.get("apt").unwrap();
        assert_eq!(entry.identifiers.len(), 2);
        assert!(entry.identifiers.contains("git"));
        assert!(entry.identifiers.contains("vim"));
    }

    #[test]
    fn test_record_installed_keeps_managers_separate() {
        let mut state = State::default();
        state.record_installed("homebrew", vec!["git".to_string()]);
        state.record_installed("apt", vec!["git".to_string()]);

        assert_eq!(state.package_inventory.len(), 2);
        assert!(state.package_inventory.contains_key("homebrew"));
        assert!(state.package_inventory.contains_key("apt"));
    }

    #[test]
    fn test_package_inventory_roundtrips_through_json() {
        let mut state = State::default();
        state.record_installed("mas", vec!["409183694".to_string()]);

        let json = serde_json::to_string(&state).unwrap();
        let restored: State = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.package_inventory.get("mas").unwrap().identifiers,
            state.package_inventory.get("mas").unwrap().identifiers
        );
    }
}
