//! State conflict detection and resolution
//!
//! Handles conflicts when multiple machines modify state concurrently

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::versioned::HeimdallStateV2;

/// Conflict detection result
#[derive(Debug, Clone)]
pub struct ConflictDetection {
    pub has_conflict: bool,
    pub conflicts: Vec<StateConflict>,
}

/// Types of state conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateConflict {
    /// Serials diverged (both machines wrote from same parent)
    SerialDivergence {
        local_serial: u64,
        remote_serial: u64,
        common_parent: u64,
    },

    /// Different active profiles
    ProfileMismatch {
        local_profile: String,
        remote_profile: String,
    },

    /// Dotfiles path differs
    PathMismatch {
        local_path: String,
        remote_path: String,
    },

    /// File checksums differ (drift detected)
    FileDrift {
        file: String,
        local_checksum: String,
        remote_checksum: String,
    },
}

impl StateConflict {
    pub fn severity(&self) -> ConflictSeverity {
        match self {
            StateConflict::SerialDivergence { .. } => ConflictSeverity::High,
            StateConflict::ProfileMismatch { .. } => ConflictSeverity::Medium,
            StateConflict::PathMismatch { .. } => ConflictSeverity::High,
            StateConflict::FileDrift { .. } => ConflictSeverity::Low,
        }
    }

    pub fn description(&self) -> String {
        match self {
            StateConflict::SerialDivergence {
                local_serial,
                remote_serial,
                common_parent,
            } => {
                format!(
                    "State diverged: local serial {} vs remote serial {} (common parent: {})",
                    local_serial, remote_serial, common_parent
                )
            }
            StateConflict::ProfileMismatch {
                local_profile,
                remote_profile,
            } => {
                format!(
                    "Active profile differs: local '{}' vs remote '{}'",
                    local_profile, remote_profile
                )
            }
            StateConflict::PathMismatch {
                local_path,
                remote_path,
            } => {
                format!(
                    "Dotfiles path differs: local '{}' vs remote '{}'",
                    local_path, remote_path
                )
            }
            StateConflict::FileDrift { file, .. } => {
                format!("File '{}' has been modified", file)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Use local state (overwrite remote)
    UseLocal,

    /// Use remote state (overwrite local)
    UseRemote,

    /// Merge both states intelligently
    Merge,

    /// Manual resolution required
    Manual,
}

/// Conflict resolver
pub struct ConflictResolver;

impl ConflictResolver {
    /// Detect conflicts between local and remote state
    pub fn detect_conflicts(
        local: &HeimdallStateV2,
        remote: &HeimdallStateV2,
    ) -> ConflictDetection {
        let mut conflicts = Vec::new();

        // Check serial divergence
        if local.lineage.has_conflict(&remote.lineage) {
            conflicts.push(StateConflict::SerialDivergence {
                local_serial: local.lineage.serial,
                remote_serial: remote.lineage.serial,
                common_parent: local.lineage.parent_serial,
            });
        }

        // Check profile mismatch
        if local.active_profile != remote.active_profile {
            conflicts.push(StateConflict::ProfileMismatch {
                local_profile: local.active_profile.clone(),
                remote_profile: remote.active_profile.clone(),
            });
        }

        // Check path mismatch
        if local.dotfiles_path != remote.dotfiles_path {
            conflicts.push(StateConflict::PathMismatch {
                local_path: local.dotfiles_path.display().to_string(),
                remote_path: remote.dotfiles_path.display().to_string(),
            });
        }

        // Check file drift
        for (file, local_checksum) in &local.checksums {
            if let Some(remote_checksum) = remote.checksums.get(file) {
                if local_checksum != remote_checksum {
                    conflicts.push(StateConflict::FileDrift {
                        file: file.clone(),
                        local_checksum: local_checksum.clone(),
                        remote_checksum: remote_checksum.clone(),
                    });
                }
            }
        }

        ConflictDetection {
            has_conflict: !conflicts.is_empty(),
            conflicts,
        }
    }

    /// Display conflicts to user
    pub fn display_conflicts(detection: &ConflictDetection) {
        if !detection.has_conflict {
            println!("{}", "✓ No conflicts detected".green());
            return;
        }

        println!("{}", "⚠️  State Conflicts Detected".yellow().bold());
        println!();

        for (i, conflict) in detection.conflicts.iter().enumerate() {
            let severity_icon = match conflict.severity() {
                ConflictSeverity::High => "🔴",
                ConflictSeverity::Medium => "🟡",
                ConflictSeverity::Low => "🟢",
            };

            println!("{}. {} {}", i + 1, severity_icon, conflict.description());
        }

        println!();
        println!("Resolution options:");
        println!("  1. Use local state:  heimdal state resolve --use-local");
        println!("  2. Use remote state: heimdal state resolve --use-remote");
        println!("  3. Merge states:     heimdal state resolve --merge");
        println!("  4. Manual edit:      heimdal state resolve --manual");
    }

    /// Resolve conflicts using specified strategy
    pub fn resolve(
        local: &HeimdallStateV2,
        remote: &HeimdallStateV2,
        strategy: ResolutionStrategy,
    ) -> Result<HeimdallStateV2> {
        match strategy {
            ResolutionStrategy::UseLocal => {
                println!(
                    "{}",
                    "Using local state (remote will be overwritten)".yellow()
                );
                Ok(local.clone())
            }
            ResolutionStrategy::UseRemote => {
                println!(
                    "{}",
                    "Using remote state (local will be overwritten)".yellow()
                );
                Ok(remote.clone())
            }
            ResolutionStrategy::Merge => {
                println!("{}", "Merging states...".cyan());
                Self::merge_states(local, remote)
            }
            ResolutionStrategy::Manual => {
                anyhow::bail!(
                    "Manual resolution required.\n\
                    \n\
                    1. Edit state file manually: vim ~/.heimdal/state.json\n\
                    2. Verify changes: heimdal validate\n\
                    3. Commit and push: heimdal sync"
                );
            }
        }
    }

    /// Intelligent state merging
    fn merge_states(local: &HeimdallStateV2, remote: &HeimdallStateV2) -> Result<HeimdallStateV2> {
        let mut merged = local.clone();

        // Use higher serial number
        if remote.lineage.serial > local.lineage.serial {
            merged.lineage.serial = remote.lineage.serial;
        }
        merged.lineage.parent_serial = merged.lineage.serial;

        // Merge machine list
        for machine in &remote.lineage.machines {
            if !merged.lineage.machines.contains(machine) {
                merged.lineage.machines.push(machine.clone());
            }
        }

        // Use most recent timestamps
        if let Some(remote_sync) = remote.last_sync {
            if local
                .last_sync
                .map_or(true, |local_sync| remote_sync > local_sync)
            {
                merged.last_sync = Some(remote_sync);
            }
        }

        if let Some(remote_apply) = remote.last_apply {
            if local
                .last_apply
                .map_or(true, |local_apply| remote_apply > local_apply)
            {
                merged.last_apply = Some(remote_apply);
            }
        }

        // Merge history (keep unique operations)
        let mut merged_history = local.history.clone();
        for remote_op in &remote.history {
            if !merged_history
                .iter()
                .any(|op| op.serial == remote_op.serial)
            {
                merged_history.push(remote_op.clone());
            }
        }
        // Sort by timestamp and keep last 50
        merged_history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if merged_history.len() > 50 {
            merged_history = merged_history.split_off(merged_history.len() - 50);
        }
        merged.history = merged_history;

        // Merge checksums (use remote if different)
        for (file, checksum) in &remote.checksums {
            if !merged.checksums.contains_key(file) {
                merged.checksums.insert(file.clone(), checksum.clone());
            }
        }

        println!("{}", "✓ States merged successfully".green());
        println!("  - Combined {} machine(s)", merged.lineage.machines.len());
        println!("  - Merged {} history entries", merged.history.len());

        Ok(merged)
    }

    /// Check for state drift (file changes)
    pub fn check_drift(state: &HeimdallStateV2) -> Result<Vec<FileDrift>> {
        let mut drifts = Vec::new();

        for (file, stored_checksum) in &state.checksums {
            let file_path = state.dotfiles_path.join(file);

            if !file_path.exists() {
                drifts.push(FileDrift {
                    file: file.clone(),
                    kind: DriftKind::Missing,
                    stored_checksum: stored_checksum.clone(),
                    current_checksum: None,
                });
                continue;
            }

            let current_checksum = Self::compute_checksum(&file_path)?;

            if &current_checksum != stored_checksum {
                drifts.push(FileDrift {
                    file: file.clone(),
                    kind: DriftKind::Modified,
                    stored_checksum: stored_checksum.clone(),
                    current_checksum: Some(current_checksum),
                });
            }
        }

        Ok(drifts)
    }

    /// Compute MD5 checksum of file
    fn compute_checksum(path: &std::path::Path) -> Result<String> {
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Using MD5 for file checksums
        Ok(format!("{:x}", md5::compute(&buffer)))
    }

    /// Update checksums for tracked files
    #[allow(dead_code)]
    pub fn update_checksums(state: &mut HeimdallStateV2) -> Result<()> {
        state.checksums.clear();

        // TODO: Get list of tracked files from config
        // For now, this is a placeholder

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileDrift {
    pub file: String,
    pub kind: DriftKind,
    #[allow(dead_code)]
    pub stored_checksum: String,
    #[allow(dead_code)]
    pub current_checksum: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftKind {
    Modified,
    Missing,
    #[allow(dead_code)]
    Added,
}

impl FileDrift {
    pub fn display(&self) {
        match self.kind {
            DriftKind::Modified => {
                println!("  {} {}", "⚠️  Modified:".yellow(), self.file);
            }
            DriftKind::Missing => {
                println!("  {} {}", "✗ Missing:".red(), self.file);
            }
            DriftKind::Added => {
                println!("  {} {}", "✓ Added:".green(), self.file);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_state(serial: u64, profile: &str) -> HeimdallStateV2 {
        let mut state = HeimdallStateV2::new(
            profile.to_string(),
            PathBuf::from("/tmp/dotfiles"),
            "https://github.com/user/dotfiles.git".to_string(),
        )
        .unwrap();
        state.lineage.serial = serial;
        state
    }

    #[test]
    fn test_no_conflict() {
        let local = create_test_state(1, "personal");
        let remote = create_test_state(1, "personal");

        let detection = ConflictResolver::detect_conflicts(&local, &remote);
        assert!(!detection.has_conflict);
    }

    #[test]
    fn test_serial_divergence() {
        let mut local = create_test_state(2, "personal");
        let mut remote = create_test_state(3, "personal");

        // Make them part of same lineage (same ID)
        let lineage_id = local.lineage.id.clone();
        remote.lineage.id = lineage_id;

        // Same parent, different current = conflict
        local.lineage.parent_serial = 1;
        remote.lineage.parent_serial = 1;

        let detection = ConflictResolver::detect_conflicts(&local, &remote);
        assert!(detection.has_conflict);
        assert!(matches!(
            detection.conflicts[0],
            StateConflict::SerialDivergence { .. }
        ));
    }

    #[test]
    fn test_profile_mismatch() {
        let local = create_test_state(1, "personal");
        let remote = create_test_state(1, "work");

        let detection = ConflictResolver::detect_conflicts(&local, &remote);
        assert!(detection.has_conflict);
        assert!(matches!(
            detection.conflicts[0],
            StateConflict::ProfileMismatch { .. }
        ));
    }
}
