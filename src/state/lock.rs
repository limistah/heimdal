//! Hybrid state locking system with local and remote coordination
//!
//! Provides multi-machine state locking using:
//! - Local file locks (fast, for same-machine operations)
//! - Git-based distributed locks (for multi-machine coordination)
//! - Conflict detection and resolution strategies

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// Lock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockConfig {
    /// Lock type (local, hybrid, disabled)
    pub lock_type: LockType,

    /// Lock timeout in seconds (0 = no timeout)
    pub timeout: u64,

    /// Enable automatic stale lock detection
    pub detect_stale: bool,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            lock_type: LockType::Hybrid,
            timeout: 300, // 5 minutes
            detect_stale: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockType {
    /// Local file locking only
    Local,

    /// Hybrid (local + remote via Git)
    Hybrid,

    /// No locking (dangerous!)
    Disabled,
}

/// Lock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateLock {
    /// Lock ID (unique per operation)
    pub id: String,

    /// Lock type that acquired this lock
    pub lock_type: String,

    /// Operation being performed
    pub operation: String,

    /// Machine that acquired the lock
    pub machine: LockMachine,

    /// Timestamp when lock was acquired
    pub created_at: DateTime<Utc>,

    /// Expected operation duration (for timeout calculation)
    pub expected_duration_seconds: Option<u64>,

    /// Reason/context for the lock
    pub reason: Option<String>,

    /// State lineage serial when lock was acquired
    pub state_serial: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockMachine {
    pub id: String,
    pub hostname: String,
    pub pid: u32,
    pub user: String,
}

impl LockMachine {
    pub fn current(machine_id: String) -> Result<Self> {
        Ok(Self {
            id: machine_id,
            hostname: hostname::get()?.to_string_lossy().to_string(),
            pid: process::id(),
            user: whoami::username(),
        })
    }
}

impl StateLock {
    pub fn new(
        operation: &str,
        machine_id: String,
        state_serial: u64,
        reason: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            lock_type: "hybrid".to_string(),
            operation: operation.to_string(),
            machine: LockMachine::current(machine_id)?,
            created_at: Utc::now(),
            expected_duration_seconds: None,
            reason,
            state_serial,
        })
    }

    /// Check if lock has expired based on timeout
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        if timeout_seconds == 0 {
            return false;
        }

        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.created_at);
        let elapsed_secs = elapsed.num_seconds();

        // Handle clock skew: if lock appears to be from the future, treat as not expired
        if elapsed_secs < 0 {
            return false;
        }

        elapsed_secs as u64 > timeout_seconds
    }

    /// Check if the lock is stale (process no longer exists)
    pub fn is_stale(&self) -> bool {
        // Only check if it's from the current machine
        let current_hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_default();

        if self.machine.hostname != current_hostname {
            // Can't check remote machine processes
            return false;
        }

        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .args(["-p", &self.machine.pid.to_string()])
                .output();

            if let Ok(output) = output {
                !output.status.success()
            } else {
                true
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, use tasklist
            false // TODO: Implement Windows process checking
        }
    }

    /// Get human-readable age of lock
    pub fn age(&self) -> String {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.created_at);

        let seconds = elapsed.num_seconds();
        if seconds < 60 {
            format!("{} seconds", seconds)
        } else if seconds < 3600 {
            format!("{} minutes", seconds / 60)
        } else {
            format!("{} hours", seconds / 3600)
        }
    }
}

/// Hybrid lock manager
pub struct LockManager {
    config: LockConfig,
}

impl LockManager {
    pub fn new(config: LockConfig) -> Self {
        Self { config }
    }

    /// Acquire a lock
    pub fn acquire(
        &self,
        operation: &str,
        machine_id: String,
        state_serial: u64,
        reason: Option<String>,
    ) -> Result<StateLock> {
        match self.config.lock_type {
            LockType::Disabled => {
                // No locking, just create a dummy lock
                eprintln!("⚠️  Warning: State locking is disabled!");
                StateLock::new(operation, machine_id, state_serial, reason)
            }
            LockType::Local => self.acquire_local(operation, machine_id, state_serial, reason),
            LockType::Hybrid => self.acquire_hybrid(operation, machine_id, state_serial, reason),
        }
    }

    /// Acquire local file lock
    fn acquire_local(
        &self,
        operation: &str,
        machine_id: String,
        state_serial: u64,
        reason: Option<String>,
    ) -> Result<StateLock> {
        let lock_path = Self::lock_path()?;

        // Check for existing lock
        if lock_path.exists() {
            let existing_lock = self.read_lock(&lock_path)?;

            if self.can_override_lock(&existing_lock)? {
                eprintln!("⚠️  Removing stale/expired lock...");
                fs::remove_file(&lock_path)?;
            } else {
                return Err(self.lock_error(&existing_lock));
            }
        }

        // Create new lock
        let lock = StateLock::new(operation, machine_id, state_serial, reason)?;
        self.write_lock(&lock_path, &lock)?;

        Ok(lock)
    }

    /// Acquire hybrid lock (local + remote coordination)
    fn acquire_hybrid(
        &self,
        operation: &str,
        machine_id: String,
        state_serial: u64,
        reason: Option<String>,
    ) -> Result<StateLock> {
        // Step 1: Acquire local lock first
        let lock =
            self.acquire_local(operation, machine_id.clone(), state_serial, reason.clone())?;

        // Step 2: Try to coordinate with remote (Git)
        match self.coordinate_remote(&lock) {
            Ok(()) => Ok(lock),
            Err(e) => {
                // Failed to coordinate with remote, release local lock
                let _ = self.release(&lock);
                Err(e)
            }
        }
    }

    /// Coordinate with remote Git repository
    fn coordinate_remote(&self, lock: &StateLock) -> Result<()> {
        // Pull latest state from remote
        let pull_result = self.pull_remote_state();

        match pull_result {
            Ok(RemotePullResult::UpToDate) => {
                // We're in sync, proceed
                Ok(())
            }
            Ok(RemotePullResult::Updated) => {
                // Remote had changes, need to check for conflicts
                self.check_remote_conflicts(lock)
            }
            Ok(RemotePullResult::Conflicts) => {
                anyhow::bail!(
                    "Cannot acquire lock: Remote state has conflicts.\n\
                    \n\
                    This usually means:\n\
                    - Another machine made changes\n\
                    - You have local uncommitted changes\n\
                    \n\
                    Resolve with:\n\
                    1. Run 'heimdal sync' to pull and merge remote changes\n\
                    2. Run 'heimdal state resolve' to resolve conflicts\n\
                    3. Try your operation again"
                );
            }
            Err(e) => {
                // Can't reach remote, warn but allow operation
                eprintln!(
                    "⚠️  Warning: Cannot reach remote repository: {}\n\
                    Proceeding with local-only lock.\n\
                    Remember to sync when connection is restored.",
                    e
                );
                Ok(())
            }
        }
    }

    /// Pull remote state
    fn pull_remote_state(&self) -> Result<RemotePullResult> {
        let state = crate::state::versioned::HeimdallStateV2::load()?;
        let repo_path = &state.dotfiles_path;

        // Check if we're in a git repo
        if !repo_path.join(".git").exists() {
            return Ok(RemotePullResult::UpToDate);
        }

        // Fetch from remote
        let fetch_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["fetch", "origin"])
            .output()?;

        if !fetch_output.status.success() {
            anyhow::bail!(
                "Failed to fetch from remote: {}",
                String::from_utf8_lossy(&fetch_output.stderr)
            );
        }

        // Check if remote has changes
        let diff_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["diff", "HEAD", "origin/main", "--", "state.json"])
            .output()?;

        if diff_output.stdout.is_empty() {
            return Ok(RemotePullResult::UpToDate);
        }

        // Try to pull with rebase
        let pull_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["pull", "--rebase", "origin", "main"])
            .output()?;

        if !pull_output.status.success() {
            let stderr = String::from_utf8_lossy(&pull_output.stderr);
            if stderr.contains("conflict") {
                return Ok(RemotePullResult::Conflicts);
            }
            anyhow::bail!("Failed to pull from remote: {}", stderr);
        }

        Ok(RemotePullResult::Updated)
    }

    /// Check for state conflicts after remote update
    fn check_remote_conflicts(&self, _lock: &StateLock) -> Result<()> {
        // Load both local and remote state
        let local_state = crate::state::versioned::HeimdallStateV2::load()?;

        // Check if state was modified by another machine
        let last_machine = local_state.lineage.machines.last();
        if let Some(machine) = last_machine {
            if machine != &local_state.machine.id {
                eprintln!(
                    "⚠️  Warning: State was recently modified by another machine ({})",
                    machine
                );
                eprintln!("   Current machine: {}", local_state.machine.id);
                eprintln!("   Proceeding carefully...");
            }
        }

        Ok(())
    }

    /// Release a lock
    pub fn release(&self, lock: &StateLock) -> Result<()> {
        let lock_path = Self::lock_path()?;

        if !lock_path.exists() {
            // Already released
            return Ok(());
        }

        // Verify we own the lock
        let existing_lock = self.read_lock(&lock_path)?;
        if existing_lock.id != lock.id {
            anyhow::bail!(
                "Cannot release lock: owned by different operation (ID: {})",
                existing_lock.id
            );
        }

        // Remove lock file
        fs::remove_file(&lock_path)?;

        // Push state to remote if hybrid
        if matches!(self.config.lock_type, LockType::Hybrid) {
            let _ = self.push_state_to_remote(); // Best effort
        }

        Ok(())
    }

    /// Push state to remote repository
    fn push_state_to_remote(&self) -> Result<()> {
        let state = crate::state::versioned::HeimdallStateV2::load()?;
        let repo_path = &state.dotfiles_path;

        if !repo_path.join(".git").exists() {
            return Ok(());
        }

        // Add and commit state file
        let _add_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["add", "state.json"])
            .output()?;

        let commit_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args([
                "commit",
                "-m",
                &format!("Update state: {}", state.machine.hostname),
            ])
            .output()?;

        // It's OK if there's nothing to commit
        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            if !stderr.contains("nothing to commit") {
                eprintln!("⚠️  Could not commit state: {}", stderr);
            }
        }

        // Push to remote
        let push_output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["push", "origin", "main"])
            .output()?;

        if !push_output.status.success() {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            eprintln!("⚠️  Could not push to remote: {}", stderr);
            eprintln!("   Remember to push manually later with: git push");
        }

        Ok(())
    }

    /// Check if we can override an existing lock
    fn can_override_lock(&self, lock: &StateLock) -> Result<bool> {
        // Check if lock is stale
        if self.config.detect_stale && lock.is_stale() {
            return Ok(true);
        }

        // Check if lock has expired
        if lock.is_expired(self.config.timeout) {
            return Ok(true);
        }

        Ok(false)
    }

    /// Create lock error message
    fn lock_error(&self, lock: &StateLock) -> anyhow::Error {
        anyhow::anyhow!(
            "State is locked by another operation!\n\
            \n\
            Lock Information:\n\
            - Operation: {}\n\
            - Machine: {} ({})\n\
            - User: {}\n\
            - PID: {}\n\
            - Acquired: {} ago\n\
            - Lock ID: {}\n\
            \n\
            This usually means:\n\
            - Another heimdal command is running\n\
            - Operation is running on another machine\n\
            \n\
            Possible actions:\n\
            1. Wait for operation to complete\n\
            2. Check running processes: ps aux | grep heimdal\n\
            3. If process died, force unlock: heimdal state unlock --force\n\
            4. View lock details: heimdal state lock-info",
            lock.operation,
            lock.machine.hostname,
            lock.machine.id,
            lock.machine.user,
            lock.machine.pid,
            lock.age(),
            lock.id
        )
    }

    /// Read lock from file
    fn read_lock(&self, path: &Path) -> Result<StateLock> {
        let content = fs::read_to_string(path)?;
        let lock: StateLock = serde_json::from_str(&content)?;
        Ok(lock)
    }

    /// Write lock to file
    fn write_lock(&self, path: &Path, lock: &StateLock) -> Result<()> {
        let content = serde_json::to_string_pretty(lock)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Force unlock (dangerous!)
    pub fn force_unlock() -> Result<()> {
        let lock_path = Self::lock_path()?;

        if !lock_path.exists() {
            println!("No lock file found");
            return Ok(());
        }

        fs::remove_file(&lock_path)?;
        println!("✓ Lock removed");
        Ok(())
    }

    /// Show lock information
    pub fn show_lock() -> Result<()> {
        let lock_path = Self::lock_path()?;

        if !lock_path.exists() {
            println!("State is not locked");
            return Ok(());
        }

        let content = fs::read_to_string(&lock_path)?;
        let lock: StateLock = serde_json::from_str(&content)?;

        println!("State Lock Information:");
        println!("  Operation: {}", lock.operation);
        println!("  Machine: {} ({})", lock.machine.hostname, lock.machine.id);
        println!("  User: {}", lock.machine.user);
        println!("  PID: {}", lock.machine.pid);
        println!("  Acquired: {} ago", lock.age());
        println!("  Lock ID: {}", lock.id);

        if let Some(reason) = &lock.reason {
            println!("  Reason: {}", reason);
        }

        if lock.is_stale() {
            println!("\n⚠️  Lock appears to be stale (process no longer exists)");
            println!("   Run 'heimdal state unlock --force' to remove");
        }

        Ok(())
    }

    pub fn lock_path() -> Result<PathBuf> {
        let state_dir = crate::state::versioned::HeimdallStateV2::state_dir()?;
        Ok(state_dir.join("heimdal.state.lock"))
    }
}

/// RAII guard for automatic lock release
pub struct StateGuard {
    lock: Option<StateLock>,
    manager: LockManager,
}

impl StateGuard {
    pub fn acquire(
        operation: &str,
        machine_id: String,
        state_serial: u64,
        reason: Option<String>,
        config: LockConfig,
    ) -> Result<Self> {
        let manager = LockManager::new(config);
        let lock = manager.acquire(operation, machine_id, state_serial, reason)?;
        Ok(Self {
            lock: Some(lock),
            manager,
        })
    }

    #[allow(dead_code)]
    pub fn lock(&self) -> Result<&StateLock> {
        self.lock
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Lock not acquired"))
    }
}

impl Drop for StateGuard {
    fn drop(&mut self) {
        if let Some(lock) = self.lock.take() {
            if let Err(e) = self.manager.release(&lock) {
                eprintln!("⚠️  Failed to release state lock: {}", e);
            }
        }
    }
}

enum RemotePullResult {
    UpToDate,
    Updated,
    Conflicts,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_creation() {
        let lock = StateLock::new(
            "test",
            "machine-123".to_string(),
            1,
            Some("Testing".to_string()),
        )
        .unwrap();

        assert_eq!(lock.operation, "test");
        assert_eq!(lock.machine.id, "machine-123");
        assert!(!lock.is_expired(300));
    }

    #[test]
    fn test_lock_config_default() {
        let config = LockConfig::default();
        assert!(matches!(config.lock_type, LockType::Hybrid));
        assert_eq!(config.timeout, 300);
        assert!(config.detect_stale);
    }
}
