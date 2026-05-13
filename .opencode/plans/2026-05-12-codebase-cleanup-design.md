# Heimdal Codebase Cleanup Design

**Date:** 2026-05-12  
**Goal:** Reduce code through consolidation, dead code removal, and implementing stub features  
**Estimated Impact:** ~200 lines removed, ~210 lines added for new features, net improvement in maintainability

---

## 1. Utility Function Extraction

### 1.1 New Utilities in `src/utils.rs`

```rust
/// Atomically write content to a file using temp file + rename pattern.
/// Prevents partial writes and corruption.
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Ensure parent directory exists before writing a file.
pub fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Get the system hostname as a String.
pub fn hostname() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
```

**Files to update:** `state.rs`, `secrets.rs`, `commands/profile.rs`, `commands/packages.rs`, `commands/history/rekey.rs`, `config.rs`, `symlink.rs`, `templates.rs`, `history/store.rs`, `commands/history/record.rs`, `commands/import.rs`

### 1.2 Move `write_config` to `src/config.rs`

```rust
/// Write HeimdalConfig to a YAML file atomically.
pub fn write_config(path: &Path, config: &HeimdalConfig) -> Result<()> {
    let content = serde_yaml_ng::to_string(config)?;
    crate::utils::atomic_write(path, content.as_bytes())
}
```

**Files to update:** Remove duplicate from `commands/profile.rs` and `commands/packages.rs`

### 1.3 Add Methods to `DotfileEntry`

In `src/config.rs`:

```rust
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
```

**Files to update:** `commands/state.rs`, `commands/profile.rs`, `config.rs` (validate_config)

### 1.4 Add `CommandContext` Struct

In `src/config.rs`:

```rust
/// Common context loaded by most commands.
pub struct CommandContext {
    pub state: State,
    pub config: HeimdalConfig,
    pub profile: Profile,
}

impl CommandContext {
    /// Load state, config, and resolved profile.
    pub fn load() -> Result<Self> {
        let state = State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, &state.active_profile)?;
        Ok(Self { state, config, profile })
    }

    /// Load with a specific profile override.
    pub fn load_with_profile(profile_name: &str) -> Result<Self> {
        let state = State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, profile_name)?;
        Ok(Self { state, config, profile })
    }
}
```

**Files to update:** `commands/apply.rs`, `commands/sync.rs`, `commands/state.rs`, `commands/status.rs`

---

## 2. Dead Code Removal

### 2.1 Files to Delete

| File | Reason |
|------|--------|
| `src/profile.rs` | Empty placeholder (1-line comment only) |

### 2.2 Code to Remove

| Location | Item | Reason |
|----------|------|--------|
| `src/packages.rs` | `is_installed()` trait method and all 6 implementations | Never called |
| `src/utils.rs` | `dotfiles_dir()` function | Never called externally |
| `src/error.rs` | `HeimdallError::Import` variant | Never constructed |
| `src/git.rs` | `GitRepo::current_commit()` method | Never called |
| `src/import.rs` | `ImportResult.tool` field | Set but never read |
| `src/main.rs` | `mod profile;` declaration | Module being deleted |
| `src/lib.rs` | `pub mod profile;` declaration | Module being deleted |

### 2.3 Remove `#[allow(dead_code)]` Annotations

After removing dead code, remove these annotations:
- `src/utils.rs` line 1: `#![allow(dead_code)]`
- `src/packages.rs` lines 3-4: annotation on `InstallResult`
- `src/git.rs` lines 246-247: annotation on `current_commit`
- `src/config.rs` lines 344-345: annotation on `create_minimal_config` (keep function, it's used)

---

## 3. Implement Process-Level Locking

### 3.1 New File: `src/lock.rs`

```rust
use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LockInfo {
    pub pid: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
}

pub struct HeimdallLock {
    path: PathBuf,
    _file: File,  // Hold file handle to maintain lock
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
        
        Ok(Self { path, _file: file })
    }

    /// Get info about current lock, if any.
    pub fn info() -> Result<Option<LockInfo>> {
        let path = Self::lock_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let mut content = String::new();
        File::open(&path)?.read_to_string(&mut content)?;
        Ok(Some(serde_json::from_str(&content)?))
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
```

### 3.2 Update `commands/state.rs`

Replace `lock_info()` and `unlock()` stubs with real implementations that use `HeimdallLock`.

### 3.3 Wrap Critical Operations

Update `commands/apply.rs` and `commands/sync.rs` to acquire lock at start.

---

## 4. Implement Native Autosync

### 4.1 Platform Support

| Platform | Method |
|----------|--------|
| macOS | launchd plist in `~/Library/LaunchAgents/com.heimdal.autosync.plist` |
| Linux (systemd) | User timer in `~/.config/systemd/user/heimdal-autosync.{service,timer}` |
| Fallback | crontab manipulation |

### 4.2 Commands

- `autosync enable --interval 1h` - Install appropriate scheduler
- `autosync disable` - Remove scheduler entry  
- `autosync status` - Check if installed and show details

### 4.3 Implementation

Replace current stub in `commands/autosync.rs` with full implementation:
- `parse_interval()` - Parse "1h", "30m", "3600s" formats
- Platform-specific enable/disable/status functions
- Auto-detect best method per platform

---

## 5. Code Simplification

### 5.1 Package Manager Helper in `src/packages.rs`

```rust
fn check_command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

### 5.2 Merge Packages Macro in `src/config.rs`

```rust
macro_rules! merge_vec {
    ($base:expr, $child:expr) => {{
        let mut v = $base;
        v.extend($child);
        v
    }};
}
```

### 5.3 Linux Distro Matching in `src/utils.rs`

Extract duplicated match block into `fn match_distro_id(id: &str) -> Option<LinuxDistro>`.

---

## 6. Update Example Files

### 6.1 `examples/minimal.yaml`

```yaml
heimdal:
  version: "1"

profiles:
  default:
    dotfiles:
      - .bashrc
      - .vimrc
    packages:
      homebrew: [git, vim, tmux]
      apt: [git, vim, tmux]
```

### 6.2 `examples/full.yaml`

Full example showing:
- Global packages
- Profile inheritance with `extends`
- Conditional dotfiles with `when`
- Hooks
- Templates
- History config

### 6.3 `examples/multi-platform.yaml`

Example showing platform-specific packages and conditional dotfiles.

---

## 7. Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
libc = "0.2"  # For process checking in lock implementation
```

---

## 8. Summary of Changes

| Category | Files Added | Files Modified | Files Deleted | Net Effect |
|----------|-------------|----------------|---------------|------------|
| Utilities | 0 | 6 | 0 | ~30 lines saved |
| Dead code | 0 | 5 | 1 | ~100 lines removed |
| Lock system | 1 | 2 | 0 | ~80 lines added |
| Autosync | 0 | 1 | 0 | ~170 lines added (replacing ~30 stub) |
| Simplification | 0 | 3 | 0 | ~30 lines saved |
| Examples | 0 | 3 | 0 | Corrected schema |

**New file:** `src/lock.rs`  
**Deleted file:** `src/profile.rs`

---

## 9. Implementation Order

1. Add utility functions (foundation for other changes)
2. Remove dead code (clean slate)
3. Add `DotfileEntry` methods and `CommandContext`
4. Implement lock system
5. Implement autosync
6. Apply code simplifications
7. Update example files
8. Run tests and verify
