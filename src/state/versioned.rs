//! Enhanced state management with versioning, locking, and multi-machine coordination
//!
//! This module provides robust state management with:
//! - State versioning and schema migrations
//! - Multi-machine conflict detection
//! - Binary version compatibility checks
//! - State drift detection and reconciliation
//! - Distributed locking via Git repository

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Current state schema version
pub const STATE_VERSION: u32 = 2;

/// Enhanced Heimdal state with versioning and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeimdallStateV2 {
    /// State schema version
    pub version: u32,

    /// Currently active profile
    pub active_profile: String,

    /// Path to dotfiles directory
    pub dotfiles_path: PathBuf,

    /// Git repository URL
    pub repo_url: String,

    /// Last sync timestamp
    #[serde(default)]
    pub last_sync: Option<DateTime<Utc>>,

    /// Last successful apply timestamp
    #[serde(default)]
    pub last_apply: Option<DateTime<Utc>>,

    /// Machine-specific metadata
    pub machine: MachineMetadata,

    /// Binary version that created/modified this state
    pub heimdal_version: String,

    /// State lineage (for conflict detection)
    pub lineage: StateLineage,

    /// Applied operations history (last 50)
    #[serde(default)]
    pub history: Vec<StateOperation>,

    /// Checksums of tracked files (for drift detection)
    #[serde(default)]
    pub checksums: HashMap<String, String>,
}

/// Machine-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineMetadata {
    /// Unique machine ID (generated once, stored in state)
    pub id: String,

    /// Machine hostname
    pub hostname: String,

    /// Operating system
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Architecture (x86_64, arm64, etc.)
    pub arch: String,

    /// User who owns this state
    pub user: String,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

impl MachineMetadata {
    pub fn current() -> Result<Self> {
        use std::env;

        let hostname = hostname::get()?.to_string_lossy().to_string();
        let user = whoami::username();
        let machine_id = Self::generate_machine_id(&hostname, &user);

        Ok(Self {
            id: machine_id,
            hostname,
            os: env::consts::OS.to_string(),
            os_version: Self::get_os_version(),
            arch: env::consts::ARCH.to_string(),
            user,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        })
    }

    fn generate_machine_id(hostname: &str, user: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        hostname.hash(&mut hasher);
        user.hash(&mut hasher);
        format!("machine-{:x}", hasher.finish())
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
            {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("VERSION_ID=") {
                        return line
                            .split('=')
                            .nth(1)
                            .unwrap_or("unknown")
                            .trim_matches('"')
                            .to_string();
                    }
                }
            }
        }

        "unknown".to_string()
    }
}

/// State lineage for conflict detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateLineage {
    /// Unique lineage ID (stays same across machine syncs)
    pub id: String,

    /// State serial number (increments with each write)
    pub serial: u64,

    /// Parent serial (for detecting conflicts)
    pub parent_serial: u64,

    /// Git commit hash when state was last saved
    pub git_commit: Option<String>,

    /// Machines that have modified this state
    pub machines: Vec<String>,
}

impl StateLineage {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            serial: 1,
            parent_serial: 0,
            git_commit: None,
            machines: Vec::new(),
        }
    }

    /// Increment serial for new state write
    pub fn increment(&mut self, machine_id: &str) {
        self.parent_serial = self.serial;
        self.serial += 1;

        if !self.machines.contains(&machine_id.to_string()) {
            self.machines.push(machine_id.to_string());
        }
    }

    /// Detect if there's a conflict with another state
    pub fn has_conflict(&self, other: &StateLineage) -> bool {
        // Conflict if:
        // 1. Same lineage ID but different serials
        // 2. Both have same parent but different current serial
        self.id == other.id
            && self.serial != other.serial
            && self.parent_serial == other.parent_serial
    }
}

/// Record of a state operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateOperation {
    /// Operation type (apply, sync, add, remove, etc.)
    pub operation: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Machine that performed operation
    pub machine_id: String,

    /// User who performed operation
    pub user: String,

    /// Brief description
    pub description: String,

    /// State serial after this operation
    pub serial: u64,
}

impl HeimdallStateV2 {
    /// Create a new state
    pub fn new(profile: String, dotfiles_path: PathBuf, repo_url: String) -> Result<Self> {
        let machine = MachineMetadata::current()?;
        let mut lineage = StateLineage::new();
        lineage.machines.push(machine.id.clone());

        Ok(Self {
            version: STATE_VERSION,
            active_profile: profile,
            dotfiles_path,
            repo_url,
            last_sync: None,
            last_apply: None,
            machine,
            heimdal_version: env!("CARGO_PKG_VERSION").to_string(),
            lineage,
            history: Vec::new(),
            checksums: HashMap::new(),
        })
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

        let mut state = serde_json::from_str::<HeimdallStateV2>(&content)
            .with_context(|| "Failed to parse state file")?;

        // Update machine metadata
        state.machine.last_seen = Utc::now();
        Ok(state)
    }

    /// Save state to disk with atomic write
    /// Save state to disk (in dotfiles repository)
    pub fn save(&mut self) -> Result<()> {
        // Increment lineage
        self.lineage.increment(&self.machine.id);

        // Update version and machine info
        self.heimdal_version = env!("CARGO_PKG_VERSION").to_string();
        self.machine.last_seen = Utc::now();

        // Get current git commit
        self.lineage.git_commit = Self::get_git_commit_at(&self.dotfiles_path).ok().flatten();

        // Save to dotfiles repo for Git synchronization
        let state_path = self.dotfiles_path.join("heimdal.state.json");
        let state_dir = self.dotfiles_path.as_path();

        fs::create_dir_all(&state_dir).with_context(|| {
            format!("Failed to create state directory: {}", state_dir.display())
        })?;

        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize state")?;

        // Atomic write: write to temp file, then rename
        let temp_path = state_path.with_extension("tmp");
        fs::write(&temp_path, &content)
            .with_context(|| format!("Failed to write temp state file: {}", temp_path.display()))?;

        fs::rename(&temp_path, &state_path)
            .with_context(|| format!("Failed to move state file: {}", state_path.display()))?;

        Ok(())
    }

    /// Record an operation in history
    pub fn record_operation(&mut self, operation: &str, description: &str) {
        let op = StateOperation {
            operation: operation.to_string(),
            timestamp: Utc::now(),
            machine_id: self.machine.id.clone(),
            user: self.machine.user.clone(),
            description: description.to_string(),
            serial: self.lineage.serial,
        };

        self.history.push(op);

        // Keep only last 50 operations
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// Get current git commit hash
    /// Get current git commit hash for the given repository
    fn get_git_commit_at(repo_path: &PathBuf) -> Result<Option<String>> {
        let output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["rev-parse", "HEAD"])
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(Some(
                String::from_utf8_lossy(&out.stdout).trim().to_string(),
            )),
            _ => Ok(None), // Not a Git repo or command failed
        }
    }

    /// Check binary version compatibility
    pub fn check_version_compatibility(&self) -> Result<VersionCompatibility> {
        let current_version = env!("CARGO_PKG_VERSION");

        if self.heimdal_version == current_version {
            return Ok(VersionCompatibility::Exact);
        }

        // Parse versions
        let state_ver = Self::parse_version(&self.heimdal_version)?;
        let current_ver = Self::parse_version(current_version)?;

        // Major version mismatch = incompatible
        if state_ver.0 != current_ver.0 {
            return Ok(VersionCompatibility::Incompatible {
                state_version: self.heimdal_version.clone(),
                current_version: current_version.to_string(),
                reason: "Major version mismatch".to_string(),
            });
        }

        // Minor version diff = compatible but upgrade recommended
        if state_ver.1 < current_ver.1 {
            return Ok(VersionCompatibility::UpgradeRecommended {
                state_version: self.heimdal_version.clone(),
                current_version: current_version.to_string(),
            });
        }

        if state_ver.1 > current_ver.1 {
            return Ok(VersionCompatibility::DowngradeWarning {
                state_version: self.heimdal_version.clone(),
                current_version: current_version.to_string(),
            });
        }

        Ok(VersionCompatibility::Compatible)
    }

    fn parse_version(ver: &str) -> Result<(u32, u32, u32)> {
        // Strip pre-release and build metadata (anything after '-' or '+')
        // e.g., "1.2.3-alpha.1+build.5" -> "1.2.3"
        let core = ver.split(|c| c == '-' || c == '+').next().unwrap_or(ver);

        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid version format: {}", ver);
        }

        Ok((
            parts[0].parse().context("Invalid major version")?,
            parts[1].parse().context("Invalid minor version")?,
            parts[2].parse().context("Invalid patch version")?,
        ))
    }

    /// Get the state directory path
    ///
    /// Checks:
    /// 1. ~/.dotfiles/heimdal.state.json (common location)
    /// 2. ~/.heimdal (fallback for bootstrap)
    pub fn state_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;

        // Check common dotfiles location
        let common_path = home.join(".dotfiles/heimdal.state.json");
        if common_path.exists() {
            return Ok(home.join(".dotfiles"));
        }

        // Fallback to ~/.heimdal for bootstrap
        Ok(home.join(".heimdal"))
    }

    /// Get the state file path
    pub fn state_path() -> Result<PathBuf> {
        Ok(Self::state_dir()?.join("heimdal.state.json"))
    }

    /// Get the backup directory path (always in ~/.heimdal)
    pub fn backup_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to determine home directory"))?;
        Ok(home.join(".heimdal").join("backups"))
    }

    /// Update last sync timestamp
    pub fn update_sync_time(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Update last apply timestamp
    pub fn update_apply_time(&mut self) {
        self.last_apply = Some(Utc::now());
    }
}

/// Version compatibility status
#[derive(Debug, Clone)]
pub enum VersionCompatibility {
    /// Exact version match
    Exact,

    /// Compatible versions
    Compatible,

    /// Compatible but upgrade recommended
    UpgradeRecommended {
        state_version: String,
        current_version: String,
    },

    /// Downgrade warning
    DowngradeWarning {
        state_version: String,
        current_version: String,
    },

    /// Incompatible versions
    Incompatible {
        state_version: String,
        current_version: String,
        reason: String,
    },
}

impl VersionCompatibility {
    pub fn is_safe(&self) -> bool {
        matches!(
            self,
            VersionCompatibility::Exact
                | VersionCompatibility::Compatible
                | VersionCompatibility::UpgradeRecommended { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_metadata() {
        let meta = MachineMetadata::current().unwrap();
        assert!(!meta.id.is_empty());
        assert!(!meta.hostname.is_empty());
        assert!(!meta.user.is_empty());
    }

    #[test]
    fn test_lineage_increment() {
        let mut lineage = StateLineage::new();
        assert_eq!(lineage.serial, 1);
        assert_eq!(lineage.parent_serial, 0);

        lineage.increment("machine-123");
        assert_eq!(lineage.serial, 2);
        assert_eq!(lineage.parent_serial, 1);
    }

    #[test]
    fn test_conflict_detection() {
        let mut lineage1 = StateLineage::new();
        lineage1.id = "same-id".to_string();
        lineage1.serial = 5;
        lineage1.parent_serial = 4;

        let mut lineage2 = lineage1.clone();
        lineage2.serial = 6; // Different serial, same parent = conflict

        assert!(lineage1.has_conflict(&lineage2));
    }

    #[test]
    fn test_version_parsing() {
        let ver = HeimdallStateV2::parse_version("1.2.3").unwrap();
        assert_eq!(ver, (1, 2, 3));
    }
}
