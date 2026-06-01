use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
}

#[derive(Debug)]
pub struct HeimdallLock {
    path: PathBuf,
    _file: File, // Hold file handle to maintain lock
}

impl HeimdallLock {
    /// Acquire an exclusive lock. Returns error if already locked.
    pub fn acquire() -> Result<Self> {
        let path = Self::lock_path()?;
        crate::utils::ensure_parent_exists(&path)?;

        // Check for existing lock
        if let Some(info) = Self::info()? {
            // Check if PID is still running
            if Self::is_process_running(info.pid) {
                anyhow::bail!(
                    "Heimdal is already running (PID {}, started {})",
                    info.pid,
                    info.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            // Stale lock, remove it
            std::fs::remove_file(&path)?;
        }

        let mut file = File::create(&path)?;
        let info = LockInfo {
            pid: std::process::id(),
            timestamp: chrono::Utc::now(),
            hostname: crate::utils::hostname(),
        };
        file.write_all(serde_json::to_string(&info)?.as_bytes())?;
        file.sync_all()?; // Ensure data is written to disk

        Ok(Self { path, _file: file })
    }

    /// Get info about current lock, if any.
    pub fn info() -> Result<Option<LockInfo>> {
        let path = Self::lock_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let mut content = String::new();
        match File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut content)?;
                if content.is_empty() {
                    return Ok(None);
                }
                Ok(Some(serde_json::from_str(&content)?))
            }
            Err(_) => Ok(None), // File was deleted between exists check and open
        }
    }

    /// Force remove a lock file (for `state unlock --force`).
    pub fn force_unlock() -> Result<()> {
        let path = Self::lock_path()?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn lock_path() -> Result<PathBuf> {
        // Lock goes in same dir as state.json (~/.heimdal/)
        let state_path = crate::utils::state_path()?;
        Ok(state_path.parent().unwrap().join("heimdal.lock"))
    }

    pub fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::kill(pid as i32, 0) == 0 }
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}

impl Drop for HeimdallLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
