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

        let info = LockInfo {
            pid: std::process::id(),
            timestamp: chrono::Utc::now(),
            hostname: crate::utils::hostname(),
        };
        let payload = serde_json::to_string(&info)?;

        // `create_new` fails atomically at the OS level if the file already
        // exists, so two processes racing here can never both succeed —
        // unlike the previous check-then-`File::create` (which truncates)
        // pattern, which left a window where both would pass the "no lock"
        // check and overwrite each other's lock.
        for _ in 0..2 {
            match File::options().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    file.write_all(payload.as_bytes())?;
                    file.sync_all()?; // Ensure data is written to disk
                    return Ok(Self { path, _file: file });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if let Some(existing) = Self::info()? {
                        if Self::is_process_running(existing.pid) {
                            anyhow::bail!(
                                "Heimdal is already running (PID {}, started {})",
                                existing.pid,
                                existing.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                            );
                        }
                        // Stale lock from a dead process — remove it and retry once.
                        std::fs::remove_file(&path)?;
                        continue;
                    }
                    // Lock file vanished between the AlreadyExists error and
                    // reading it back (e.g. another process just released
                    // it) — retry.
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        anyhow::bail!("Could not acquire heimdal lock: contended by another process")
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
        let parent = state_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("state path has no parent directory"))?;
        Ok(parent.join("heimdal.lock"))
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
