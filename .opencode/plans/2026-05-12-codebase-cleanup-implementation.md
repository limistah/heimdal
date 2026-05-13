# Heimdal Codebase Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce code duplication, remove dead code, and implement stub features (locking, autosync)

**Architecture:** Incremental refactoring in dependency order - utilities first, then dead code removal, then new features

**Tech Stack:** Rust 2021, existing dependencies + libc for process checking

---

## Task 1: Add Utility Functions to utils.rs

**Files:**
- Modify: `src/utils.rs`
- Test: `tests/test_utils.rs` (create)

- [ ] **Step 1: Write tests for new utility functions**

Create `tests/test_utils.rs`:

```rust
use heimdal::utils::{atomic_write, ensure_parent_exists, hostname};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_atomic_write() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.txt");
    
    atomic_write(&path, b"hello world").unwrap();
    
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_atomic_write_overwrites() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.txt");
    
    fs::write(&path, "old").unwrap();
    atomic_write(&path, b"new").unwrap();
    
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "new");
}

#[test]
fn test_ensure_parent_exists() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("subdir").join("file.txt");
    
    ensure_parent_exists(&path).unwrap();
    
    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_ensure_parent_exists_noop_when_exists() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("file.txt");
    
    // Parent already exists (tmp dir)
    ensure_parent_exists(&path).unwrap();
    
    assert!(path.parent().unwrap().exists());
}

#[test]
fn test_hostname_returns_string() {
    let host = hostname();
    assert!(!host.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_utils --lib`
Expected: Compilation errors (functions don't exist yet)

- [ ] **Step 3: Implement utility functions in utils.rs**

Add to `src/utils.rs` after the existing imports:

```rust
use std::path::Path;

/// Atomically write content to a file using temp file + rename pattern.
/// Prevents partial writes and corruption.
pub fn atomic_write(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Ensure parent directory exists before writing a file.
pub fn ensure_parent_exists(path: &Path) -> anyhow::Result<()> {
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

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_utils`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/utils.rs tests/test_utils.rs
git commit -m "feat: add atomic_write, ensure_parent_exists, hostname utilities"
```

---

## Task 2: Update state.rs to Use New Utilities

**Files:**
- Modify: `src/state.rs:37-45`

- [ ] **Step 1: Update State::save() to use utilities**

In `src/state.rs`, replace lines 37-44:

```rust
pub fn save(&self) -> Result<()> {
    let path = Self::path()?;
    crate::utils::ensure_parent_exists(&path)?;
    let content = serde_json::to_string_pretty(self)?;
    crate::utils::atomic_write(&path, content.as_bytes())?;
    Ok(())
}
```

- [ ] **Step 2: Update State::create() to use hostname utility**

In `src/state.rs`, replace lines 56-59:

```rust
hostname: crate::utils::hostname(),
```

- [ ] **Step 3: Run tests to verify still passing**

Run: `cargo test test_init test_basic`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/state.rs
git commit -m "refactor: use atomic_write and hostname utilities in state.rs"
```

---

## Task 3: Update Other Files to Use Utilities

**Files:**
- Modify: `src/secrets.rs:53-66, 74-76`
- Modify: `src/config.rs:371-373`
- Modify: `src/symlink.rs:183-205`
- Modify: `src/templates.rs:44-48, 141-143`
- Modify: `src/history/store.rs:12-14`
- Modify: `src/commands/history/record.rs:46-48`
- Modify: `src/commands/history/rekey.rs:106-108`
- Modify: `src/commands/import.rs:90-92`

- [ ] **Step 1: Update src/secrets.rs**

Replace lines 53-55 with:
```rust
crate::utils::ensure_parent_exists(&cache_path)?;
```

Replace lines 64-66 with:
```rust
crate::utils::atomic_write(&cache_path, content.as_bytes())?;
```

Replace lines 74-76 with:
```rust
crate::utils::atomic_write(&cache_path, &updated_content)?;
```

- [ ] **Step 2: Update src/config.rs**

Replace lines 371-373 with:
```rust
crate::utils::ensure_parent_exists(path)?;
```

- [ ] **Step 3: Update src/symlink.rs**

Replace line 183 with:
```rust
crate::utils::ensure_parent_exists(&dest)?;
```

Replace line 186 with:
```rust
crate::utils::ensure_parent_exists(&dest)?;
```

Replace lines 203-205 with:
```rust
crate::utils::ensure_parent_exists(&dest)?;
```

- [ ] **Step 4: Update src/templates.rs**

Replace lines 44-48 with:
```rust
let hostname = crate::utils::hostname();
```

Replace lines 141-143 with:
```rust
crate::utils::ensure_parent_exists(&output_path)?;
```

- [ ] **Step 5: Update src/history/store.rs**

Replace lines 12-14 with:
```rust
crate::utils::ensure_parent_exists(&path)?;
```

- [ ] **Step 6: Update src/commands/history/record.rs**

Replace lines 46-48 with:
```rust
crate::utils::ensure_parent_exists(&cache_path)?;
```

- [ ] **Step 7: Update src/commands/history/rekey.rs**

Replace lines 106-108 with:
```rust
crate::utils::atomic_write(&path, &encrypted)?;
```

- [ ] **Step 8: Update src/commands/import.rs**

Replace lines 90-92 with:
```rust
crate::utils::ensure_parent_exists(&config_path)?;
```

- [ ] **Step 9: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 10: Commit**

```bash
git add src/secrets.rs src/config.rs src/symlink.rs src/templates.rs src/history/store.rs src/commands/history/record.rs src/commands/history/rekey.rs src/commands/import.rs
git commit -m "refactor: use utility functions across codebase"
```

---

## Task 4: Add write_config to config.rs

**Files:**
- Modify: `src/config.rs` (add function at end)
- Modify: `src/commands/profile.rs:224-230` (will update in next task)
- Modify: `src/commands/packages.rs:243-249` (will update in next task)

- [ ] **Step 1: Add write_config function to config.rs**

Add to end of `src/config.rs`:

```rust
/// Write HeimdalConfig to a YAML file atomically.
pub fn write_config(path: &Path, config: &HeimdalConfig) -> anyhow::Result<()> {
    let content = serde_yaml_ng::to_string(config)?;
    crate::utils::atomic_write(path, content.as_bytes())
}
```

- [ ] **Step 2: Update src/commands/profile.rs**

Remove the local `write_config` function (lines 224-230) and replace calls with `crate::config::write_config`.

Update line 232 from:
```rust
write_config(&config_path, &config)?;
```
to:
```rust
crate::config::write_config(&config_path, &config)?;
```

(Repeat for all other calls to `write_config` in this file)

- [ ] **Step 3: Update src/commands/packages.rs**

Remove the local `write_config` function (lines 243-249) and replace calls with `crate::config::write_config`.

Update all calls to `write_config` to use `crate::config::write_config`.

- [ ] **Step 4: Run tests**

Run: `cargo test test_profile test_packages`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/config.rs src/commands/profile.rs src/commands/packages.rs
git commit -m "refactor: consolidate write_config into config module"
```

---

## Task 5: Add Methods to DotfileEntry

**Files:**
- Modify: `src/config.rs:54-59` (add impl block)
- Modify: `src/commands/state.rs:34-37, 79-82` (use new methods)
- Modify: `src/commands/profile.rs:198-209` (use new methods)

- [ ] **Step 1: Write tests for DotfileEntry methods**

Add to `tests/test_config.rs`:

```rust
#[test]
fn test_dotfile_entry_simple_source() {
    let entry = heimdal::config::DotfileEntry::Simple(".bashrc".to_string());
    assert_eq!(entry.source(), ".bashrc");
}

#[test]
fn test_dotfile_entry_simple_target() {
    let entry = heimdal::config::DotfileEntry::Simple(".bashrc".to_string());
    assert_eq!(entry.target(), "~/.bashrc");
}

#[test]
fn test_dotfile_entry_mapped_source() {
    let entry = heimdal::config::DotfileEntry::Mapped(heimdal::config::DotfileMapping {
        source: "config/nvim".to_string(),
        target: "~/.config/nvim".to_string(),
        when: None,
    });
    assert_eq!(entry.source(), "config/nvim");
}

#[test]
fn test_dotfile_entry_mapped_target() {
    let entry = heimdal::config::DotfileEntry::Mapped(heimdal::config::DotfileMapping {
        source: "config/nvim".to_string(),
        target: "~/.config/nvim".to_string(),
        when: None,
    });
    assert_eq!(entry.target(), "~/.config/nvim");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_dotfile_entry`
Expected: Compilation errors (methods don't exist)

- [ ] **Step 3: Add methods to DotfileEntry**

Add after the `DotfileEntry` enum definition in `src/config.rs` (after line 59):

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

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_dotfile_entry`
Expected: All tests PASS

- [ ] **Step 5: Update src/commands/state.rs to use new methods**

Replace lines 34-37 with:
```rust
let src_rel = entry.source();
let target_str = entry.target();
```

Replace lines 79-82 with:
```rust
let target_str = entry.target();
```

- [ ] **Step 6: Update src/commands/profile.rs to use new methods**

Replace lines 198-201 with:
```rust
let src_rel = entry.source();
```

Replace lines 206-209 with:
```rust
let src_rel = entry.source();
```

- [ ] **Step 7: Update src/config.rs validate_config to use new method**

Replace lines 317-320 with:
```rust
let src = entry.source();
```

- [ ] **Step 8: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 9: Commit**

```bash
git add src/config.rs src/commands/state.rs src/commands/profile.rs tests/test_config.rs
git commit -m "feat: add source() and target() methods to DotfileEntry"
```

---

## Task 6: Add CommandContext Struct

**Files:**
- Modify: `src/config.rs` (add struct after imports)
- Modify: `src/commands/apply.rs:12-15`
- Modify: `src/commands/sync.rs:10-13`
- Modify: `src/commands/state.rs:27-30, 72-75`

- [ ] **Step 1: Add CommandContext to config.rs**

Add after the existing structs in `src/config.rs` (around line 135):

```rust
/// Common context loaded by most commands.
pub struct CommandContext {
    pub state: crate::state::State,
    pub config: HeimdalConfig,
    pub profile: Profile,
}

impl CommandContext {
    /// Load state, config, and resolved profile.
    pub fn load() -> anyhow::Result<Self> {
        let state = crate::state::State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, &state.active_profile)?;
        Ok(Self { state, config, profile })
    }

    /// Load with a specific profile override.
    pub fn load_with_profile(profile_name: &str) -> anyhow::Result<Self> {
        let state = crate::state::State::load()?;
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        let config = load_config(&config_path)?;
        let profile = resolve_profile(&config, profile_name)?;
        Ok(Self { state, config, profile })
    }
}
```

- [ ] **Step 2: Update src/commands/apply.rs**

Replace lines 12-15 with:
```rust
let ctx = crate::config::CommandContext::load()?;
let state = &ctx.state;
let config = &ctx.config;
let profile = &ctx.profile;
```

- [ ] **Step 3: Update src/commands/sync.rs**

Replace lines 10-13 with:
```rust
let ctx = crate::config::CommandContext::load()?;
let state = &ctx.state;
let config = &ctx.config;
let profile = &ctx.profile;
```

- [ ] **Step 4: Update src/commands/state.rs check_drift**

Replace lines 27-30 with:
```rust
let ctx = crate::config::CommandContext::load()?;
let state = &ctx.state;
let profile = &ctx.profile;
```

- [ ] **Step 5: Update src/commands/state.rs check_conflicts**

Replace lines 72-75 with:
```rust
let ctx = crate::config::CommandContext::load()?;
let state = &ctx.state;
let profile = &ctx.profile;
```

- [ ] **Step 6: Run tests**

Run: `cargo test test_apply test_basic`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/config.rs src/commands/apply.rs src/commands/sync.rs src/commands/state.rs
git commit -m "feat: add CommandContext to reduce boilerplate"
```

---

## Task 7: Remove Dead Code

**Files:**
- Delete: `src/profile.rs`
- Modify: `src/main.rs` (remove mod declaration)
- Modify: `src/lib.rs` (remove pub mod declaration)
- Modify: `src/packages.rs` (remove is_installed method)
- Modify: `src/utils.rs` (remove dotfiles_dir, remove #![allow(dead_code)])
- Modify: `src/error.rs` (remove Import variant)
- Modify: `src/git.rs` (remove current_commit)
- Modify: `src/import.rs` (remove tool field)

- [ ] **Step 1: Delete src/profile.rs**

```bash
rm src/profile.rs
```

- [ ] **Step 2: Remove profile mod from src/main.rs**

Find and remove line:
```rust
mod profile;
```

- [ ] **Step 3: Remove profile mod from src/lib.rs**

Find and remove line:
```rust
pub mod profile;
```

- [ ] **Step 4: Remove is_installed from PackageManager trait in packages.rs**

Remove the trait method definition (around line 15-16):
```rust
fn is_installed(&self, pkg: &str) -> bool;
```

Remove all 6 implementations from Homebrew, HomebrewCask, Apt, Dnf, Pacman, Apk structs.

- [ ] **Step 5: Remove dotfiles_dir from utils.rs**

Remove function (lines 120-122):
```rust
pub fn dotfiles_dir() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".dotfiles"))
}
```

Remove module-level attribute (line 1):
```rust
#![allow(dead_code)]
```

- [ ] **Step 6: Remove HeimdallError::Import from error.rs**

Find and remove the variant:
```rust
#[error("Import error: {0}")]
Import(String),
```

- [ ] **Step 7: Remove current_commit from git.rs**

Remove the method and its `#[allow(dead_code)]` annotation (around lines 246-257).

- [ ] **Step 8: Remove tool field from ImportResult in import.rs**

Remove from struct (around line 41-42):
```rust
pub tool: SourceTool,
```

Remove from all struct instantiations in the file (remove `tool: SourceTool::X,` lines).

- [ ] **Step 9: Remove remaining #[allow(dead_code)] annotations**

In `src/packages.rs`, remove annotation from `InstallResult` (lines 3-4).

In `src/config.rs`, remove annotation from `create_minimal_config` (lines 344-345) - keep the function, just remove the attribute.

- [ ] **Step 10: Build to verify no compilation errors**

Run: `cargo build`
Expected: Successful build with no errors

- [ ] **Step 11: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 12: Commit**

```bash
git rm src/profile.rs
git add src/main.rs src/lib.rs src/packages.rs src/utils.rs src/error.rs src/git.rs src/import.rs src/config.rs
git commit -m "refactor: remove dead code"
```

---

## Task 8: Code Simplification - Package Manager Helper

**Files:**
- Modify: `src/packages.rs` (add helper, update 6 is_available methods)

- [ ] **Step 1: Add check_command_available helper**

Add near top of `src/packages.rs` after trait definition:

```rust
/// Check if a command is available on the system.
fn check_command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

- [ ] **Step 2: Update Homebrew::is_available**

Replace implementation (lines ~35-38) with:
```rust
fn is_available(&self) -> bool {
    check_command_available("brew")
}
```

- [ ] **Step 3: Update HomebrewCask::is_available**

Replace implementation with:
```rust
fn is_available(&self) -> bool {
    check_command_available("brew")
}
```

- [ ] **Step 4: Update Apt::is_available**

Replace implementation with:
```rust
fn is_available(&self) -> bool {
    check_command_available("apt-get")
}
```

- [ ] **Step 5: Update Dnf::is_available**

Replace implementation with:
```rust
fn is_available(&self) -> bool {
    check_command_available("dnf")
}
```

- [ ] **Step 6: Update Pacman::is_available**

Replace implementation with:
```rust
fn is_available(&self) -> bool {
    check_command_available("pacman")
}
```

- [ ] **Step 7: Update Apk::is_available**

Replace implementation with:
```rust
fn is_available(&self) -> bool {
    check_command_available("apk")
}
```

- [ ] **Step 8: Run tests**

Run: `cargo test test_packages`
Expected: All tests PASS

- [ ] **Step 9: Commit**

```bash
git add src/packages.rs
git commit -m "refactor: consolidate package manager availability checks"
```

---

## Task 9: Code Simplification - Merge Packages Macro

**Files:**
- Modify: `src/config.rs:212-254` (add macro, simplify function)

- [ ] **Step 1: Add merge_vec macro before merge_packages function**

Add before line 212 in `src/config.rs`:

```rust
macro_rules! merge_vec {
    ($base:expr, $child:expr) => {{
        let mut v = $base;
        v.extend($child);
        v
    }};
}
```

- [ ] **Step 2: Simplify merge_packages function**

Replace the entire `merge_packages` function (lines 212-254) with:

```rust
fn merge_packages(base: PackageMap, child: PackageMap) -> PackageMap {
    PackageMap {
        common: merge_vec!(base.common, child.common),
        homebrew: merge_vec!(base.homebrew, child.homebrew),
        homebrew_casks: merge_vec!(base.homebrew_casks, child.homebrew_casks),
        apt: merge_vec!(base.apt, child.apt),
        dnf: merge_vec!(base.dnf, child.dnf),
        pacman: merge_vec!(base.pacman, child.pacman),
        apk: merge_vec!(base.apk, child.apk),
        mas: merge_vec!(base.mas, child.mas),
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test test_config`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/config.rs
git commit -m "refactor: use macro to simplify merge_packages"
```

---

## Task 10: Code Simplification - Linux Distro Matching

**Files:**
- Modify: `src/utils.rs` (add helper, update detect_os)

- [ ] **Step 1: Add match_distro_id helper**

Add in `src/utils.rs` before the `detect_os` function:

```rust
fn match_distro_id(id: &str) -> Option<LinuxDistro> {
    match id {
        "debian" => Some(LinuxDistro::Debian),
        "ubuntu" => Some(LinuxDistro::Ubuntu),
        "fedora" => Some(LinuxDistro::Fedora),
        "rhel" | "centos" | "rocky" | "almalinux" => Some(LinuxDistro::Rhel),
        "arch" | "manjaro" | "endeavouros" => Some(LinuxDistro::Arch),
        "alpine" => Some(LinuxDistro::Alpine),
        _ => None,
    }
}
```

- [ ] **Step 2: Update detect_os to use helper**

Replace the first match block (lines ~53-63) with:
```rust
if let Some(id_value) = id {
    if let Some(distro) = match_distro_id(&id_value) {
        return Some(Os::Linux(distro));
    }
}
```

Replace the second match block (lines ~81-89) with:
```rust
for id_like_val in id_like.split_whitespace() {
    if let Some(distro) = match_distro_id(id_like_val) {
        return Some(Os::Linux(distro));
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/utils.rs
git commit -m "refactor: consolidate Linux distro matching logic"
```

---

## Task 11: Add libc Dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add libc to dependencies**

Add to `[dependencies]` section in `Cargo.toml`:

```toml
libc = "0.2"
```

- [ ] **Step 2: Update Cargo.lock**

Run: `cargo build`
Expected: Dependencies updated, successful build

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "deps: add libc for process checking"
```

---

## Task 12: Implement Lock System - Create lock.rs

**Files:**
- Create: `src/lock.rs`
- Modify: `src/main.rs` (add mod declaration)
- Modify: `src/lib.rs` (add pub mod declaration)
- Test: `tests/test_lock.rs` (create)

- [ ] **Step 1: Write tests for lock system**

Create `tests/test_lock.rs`:

```rust
use heimdal::lock::{HeimdallLock, LockInfo};
use serial_test::serial;

#[test]
#[serial]
fn test_lock_acquire() {
    // Clean up any existing lock
    let _ = HeimdallLock::force_unlock();
    
    let _lock = HeimdallLock::acquire().unwrap();
    let info = HeimdallLock::info().unwrap();
    
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.pid, std::process::id());
}

#[test]
#[serial]
fn test_lock_prevents_concurrent() {
    let _ = HeimdallLock::force_unlock();
    
    let _lock1 = HeimdallLock::acquire().unwrap();
    let result = HeimdallLock::acquire();
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already running"));
}

#[test]
#[serial]
fn test_lock_released_on_drop() {
    let _ = HeimdallLock::force_unlock();
    
    {
        let _lock = HeimdallLock::acquire().unwrap();
        assert!(HeimdallLock::info().unwrap().is_some());
    }
    
    // Lock should be released
    assert!(HeimdallLock::info().unwrap().is_none());
}

#[test]
#[serial]
fn test_force_unlock() {
    let _ = HeimdallLock::force_unlock();
    
    let _lock = HeimdallLock::acquire().unwrap();
    assert!(HeimdallLock::info().unwrap().is_some());
    
    HeimdallLock::force_unlock().unwrap();
    assert!(HeimdallLock::info().unwrap().is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_lock`
Expected: Compilation errors (module doesn't exist)

- [ ] **Step 3: Create src/lock.rs**

```rust
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

pub struct HeimdallLock {
    path: PathBuf,
    _file: File,
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

- [ ] **Step 4: Add mod declaration to src/main.rs**

Add after other mod declarations:
```rust
mod lock;
```

- [ ] **Step 5: Add pub mod declaration to src/lib.rs**

Add after other pub mod declarations:
```rust
pub mod lock;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test test_lock`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/lock.rs src/main.rs src/lib.rs tests/test_lock.rs
git commit -m "feat: implement process-level locking system"
```

---

## Task 13: Update state.rs Commands to Use Lock

**Files:**
- Modify: `src/commands/state.rs:16-24`

- [ ] **Step 1: Update lock_info function**

Replace function (lines 16-19) with:

```rust
fn lock_info() -> Result<()> {
    match crate::lock::HeimdallLock::info()? {
        Some(info) => {
            let running = crate::lock::HeimdallLock::is_process_running(info.pid);
            crate::utils::info(&format!("Lock held by PID {} on {}", info.pid, info.hostname));
            crate::utils::info(&format!("Started: {}", info.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
            crate::utils::info(&format!("Status: {}", if running { "active" } else { "stale" }));
        }
        None => {
            crate::utils::info("No active lock.");
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Update unlock function**

Replace function (lines 21-24) with:

```rust
fn unlock(force: bool) -> Result<()> {
    if !force {
        crate::utils::info("Use --force to remove a lock file.");
        return Ok(());
    }
    crate::lock::HeimdallLock::force_unlock()?;
    crate::utils::success("Lock removed.");
    Ok(())
}
```

- [ ] **Step 3: Update StateCmd enum to include force flag**

In `src/cli.rs`, find the `StateCmd` enum and update:

```rust
Unlock {
    #[arg(long)]
    force: bool,
},
```

- [ ] **Step 4: Update state.rs run function**

Update line 9 to pass the force flag:

```rust
StateCmd::Unlock { force } => unlock(force),
```

- [ ] **Step 5: Run manual test**

Run: `cargo build && ./target/debug/heimdal state lock-info`
Expected: "No active lock."

- [ ] **Step 6: Commit**

```bash
git add src/commands/state.rs src/cli.rs
git commit -m "feat: implement lock-info and unlock commands"
```

---

## Task 14: Add Lock to apply and sync Commands

**Files:**
- Modify: `src/commands/apply.rs:10-11`
- Modify: `src/commands/sync.rs:8-9`

- [ ] **Step 1: Add lock to apply command**

In `src/commands/apply.rs`, add at the beginning of the `run` function (after line 10):

```rust
let _lock = crate::lock::HeimdallLock::acquire()?;
```

- [ ] **Step 2: Add lock to sync command**

In `src/commands/sync.rs`, add at the beginning of the `run` function (after line 8):

```rust
let _lock = crate::lock::HeimdallLock::acquire()?;
```

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 4: Commit**

```bash
git add src/commands/apply.rs src/commands/sync.rs
git commit -m "feat: add locking to apply and sync commands"
```

---

## Task 15: Implement Autosync - Basic Structure

**Files:**
- Modify: `src/commands/autosync.rs` (complete rewrite)

- [ ] **Step 1: Replace autosync.rs with new implementation - Part 1 (helpers)**

Replace entire contents of `src/commands/autosync.rs`:

```rust
use crate::cli::AutoSyncCmd;
use crate::utils::{info, success};
use anyhow::Result;
use std::path::PathBuf;

pub fn run(action: AutoSyncCmd) -> Result<()> {
    match action {
        AutoSyncCmd::Enable { interval } => enable(&interval),
        AutoSyncCmd::Disable => disable(),
        AutoSyncCmd::Status => status(),
    }
}

fn enable(interval: &str) -> Result<()> {
    let interval_secs = parse_interval(interval)?;

    #[cfg(target_os = "macos")]
    {
        enable_launchd(interval_secs)?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            enable_systemd(interval_secs)?;
        } else {
            enable_cron(interval_secs)?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        enable_cron(interval_secs)?;
    }

    Ok(())
}

fn disable() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        disable_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            disable_systemd()?;
        } else {
            disable_cron()?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        disable_cron()?;
    }

    Ok(())
}

fn status() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        status_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            status_systemd()?;
        } else {
            status_cron()?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        status_cron()?;
    }

    Ok(())
}

fn parse_interval(s: &str) -> Result<u64> {
    let s = s.trim().to_lowercase();
    if let Some(n) = s.strip_suffix('h') {
        Ok(n.parse::<u64>()? * 3600)
    } else if let Some(n) = s.strip_suffix('m') {
        Ok(n.parse::<u64>()? * 60)
    } else if let Some(n) = s.strip_suffix('s') {
        Ok(n.parse::<u64>()?)
    } else {
        // Assume minutes if no suffix
        Ok(s.parse::<u64>()? * 60)
    }
}

// Placeholder functions - will implement in next steps
#[cfg(target_os = "macos")]
fn enable_launchd(_interval_secs: u64) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_launchd() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn status_launchd() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn has_systemd() -> bool {
    std::path::Path::new("/run/systemd/system").exists()
}

#[cfg(target_os = "linux")]
fn enable_systemd(_interval_secs: u64) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_systemd() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
fn status_systemd() -> Result<()> {
    Ok(())
}

fn enable_cron(_interval_secs: u64) -> Result<()> {
    Ok(())
}

fn disable_cron() -> Result<()> {
    Ok(())
}

fn status_cron() -> Result<()> {
    Ok(())
}
```

- [ ] **Step 2: Build to verify structure**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 3: Commit**

```bash
git add src/commands/autosync.rs
git commit -m "refactor: restructure autosync with platform detection"
```

---

## Task 16: Implement Autosync - macOS launchd

**Files:**
- Modify: `src/commands/autosync.rs` (implement launchd functions)

- [ ] **Step 1: Implement macOS launchd functions**

Replace the placeholder launchd functions in `src/commands/autosync.rs`:

```rust
#[cfg(target_os = "macos")]
const LAUNCHD_LABEL: &str = "com.heimdal.autosync";

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LAUNCHD_LABEL))
}

#[cfg(target_os = "macos")]
fn enable_launchd(interval_secs: u64) -> Result<()> {
    let plist_path = launchd_plist_path();
    let heimdal_path = std::env::current_exe()?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>sync</string>
    </array>
    <key>StartInterval</key>
    <integer>{}</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/heimdal-autosync.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/heimdal-autosync.log</string>
</dict>
</plist>"#,
        LAUNCHD_LABEL,
        heimdal_path.display(),
        interval_secs
    );

    crate::utils::ensure_parent_exists(&plist_path)?;
    std::fs::write(&plist_path, plist)?;

    // Load the agent
    std::process::Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .status()?;

    success(&format!(
        "AutoSync enabled (every {} seconds)",
        interval_secs
    ));
    info(&format!("Plist: {}", plist_path.display()));
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_launchd() -> Result<()> {
    let plist_path = launchd_plist_path();

    if plist_path.exists() {
        std::process::Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap()])
            .status()?;
        std::fs::remove_file(&plist_path)?;
        success("AutoSync disabled.");
    } else {
        info("AutoSync was not enabled.");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn status_launchd() -> Result<()> {
    let output = std::process::Command::new("launchctl")
        .args(["list", LAUNCHD_LABEL])
        .output()?;

    if output.status.success() {
        success("AutoSync is enabled (launchd).");
        let plist_path = launchd_plist_path();
        info(&format!("Plist: {}", plist_path.display()));
    } else {
        info("AutoSync is not enabled.");
    }
    Ok(())
}
```

- [ ] **Step 2: Build**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 3: Commit**

```bash
git add src/commands/autosync.rs
git commit -m "feat: implement macOS launchd autosync"
```

---

## Task 17: Implement Autosync - Linux systemd

**Files:**
- Modify: `src/commands/autosync.rs` (implement systemd functions)

- [ ] **Step 1: Implement Linux systemd functions**

Replace the placeholder systemd functions in `src/commands/autosync.rs`:

```rust
#[cfg(target_os = "linux")]
fn systemd_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join(".config/systemd/user")
}

#[cfg(target_os = "linux")]
fn enable_systemd(interval_secs: u64) -> Result<()> {
    let dir = systemd_dir();
    crate::utils::ensure_parent_exists(&dir.join("dummy"))?;

    let heimdal_path = std::env::current_exe()?;

    // Service file
    let service = format!(
        r#"[Unit]
Description=Heimdal dotfile sync

[Service]
Type=oneshot
ExecStart={} sync
"#,
        heimdal_path.display()
    );

    // Timer file
    let timer = format!(
        r#"[Unit]
Description=Heimdal autosync timer

[Timer]
OnBootSec=60
OnUnitActiveSec={}s
Unit=heimdal-autosync.service

[Install]
WantedBy=timers.target
"#,
        interval_secs
    );

    std::fs::write(dir.join("heimdal-autosync.service"), service)?;
    std::fs::write(dir.join("heimdal-autosync.timer"), timer)?;

    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;
    std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", "heimdal-autosync.timer"])
        .status()?;

    success(&format!(
        "AutoSync enabled (every {} seconds)",
        interval_secs
    ));
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_systemd() -> Result<()> {
    std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .status()?;

    let dir = systemd_dir();
    let _ = std::fs::remove_file(dir.join("heimdal-autosync.service"));
    let _ = std::fs::remove_file(dir.join("heimdal-autosync.timer"));

    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    success("AutoSync disabled.");
    Ok(())
}

#[cfg(target_os = "linux")]
fn status_systemd() -> Result<()> {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-active", "heimdal-autosync.timer"])
        .output()?;

    if output.status.success() {
        success("AutoSync is enabled (systemd timer).");
        // Show next run time
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "list-timers", "heimdal-autosync.timer"])
            .status();
    } else {
        info("AutoSync is not enabled.");
    }
    Ok(())
}
```

- [ ] **Step 2: Build**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 3: Commit**

```bash
git add src/commands/autosync.rs
git commit -m "feat: implement Linux systemd autosync"
```

---

## Task 18: Implement Autosync - Cron Fallback

**Files:**
- Modify: `src/commands/autosync.rs` (implement cron functions)

- [ ] **Step 1: Implement cron functions**

Replace the placeholder cron functions in `src/commands/autosync.rs`:

```rust
fn enable_cron(interval_secs: u64) -> Result<()> {
    let heimdal_path = std::env::current_exe()?;
    let minutes = (interval_secs / 60).max(1);

    // Get existing crontab
    let output = std::process::Command::new("crontab").arg("-l").output();

    let mut crontab = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    };

    // Remove any existing heimdal entries
    crontab = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    // Add new entry
    let entry = format!("*/{} * * * * {} sync", minutes, heimdal_path.display());
    if !crontab.is_empty() && !crontab.ends_with('\n') {
        crontab.push('\n');
    }
    crontab.push_str(&entry);
    crontab.push('\n');

    // Write new crontab
    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(crontab.as_bytes())?;
    }
    child.wait()?;

    success(&format!(
        "AutoSync enabled (cron, every {} minutes)",
        minutes
    ));
    Ok(())
}

fn disable_cron() -> Result<()> {
    let output = std::process::Command::new("crontab").arg("-l").output()?;

    if !output.status.success() {
        info("No crontab found.");
        return Ok(());
    }

    let crontab: String = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(crontab.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    child.wait()?;

    success("AutoSync disabled.");
    Ok(())
}

fn status_cron() -> Result<()> {
    let output = std::process::Command::new("crontab").arg("-l").output();

    match output {
        Ok(o) if o.status.success() => {
            let crontab = String::from_utf8_lossy(&o.stdout);
            if crontab.contains("heimdal sync") {
                success("AutoSync is enabled (cron).");
                for line in crontab.lines() {
                    if line.contains("heimdal sync") {
                        info(&format!("  {}", line));
                    }
                }
            } else {
                info("AutoSync is not enabled.");
            }
        }
        _ => {
            info("AutoSync is not enabled (no crontab).");
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Build**

Run: `cargo build`
Expected: Successful build

- [ ] **Step 3: Test parse_interval**

Create quick test in `tests/test_basic.rs` or manually:

```bash
# Manual test:
./target/debug/heimdal autosync enable --interval 1h
./target/debug/heimdal autosync status
./target/debug/heimdal autosync disable
```

- [ ] **Step 4: Commit**

```bash
git add src/commands/autosync.rs
git commit -m "feat: implement cron fallback for autosync"
```

---

## Task 19: Update Example Files

**Files:**
- Modify: `examples/minimal.yaml`
- Modify: `examples/full.yaml`
- Modify: `examples/multi-platform.yaml`

- [ ] **Step 1: Update examples/minimal.yaml**

Replace entire file:

```yaml
# Minimal Heimdal Configuration
# A simple setup with basic packages and dotfiles

heimdal:
  version: "1"

profiles:
  default:
    dotfiles:
      - .bashrc
      - .vimrc
    packages:
      homebrew:
        - git
        - vim
        - tmux
      apt:
        - git
        - vim
        - tmux
```

- [ ] **Step 2: Update examples/full.yaml**

Replace entire file:

```yaml
# Full-Featured Heimdal Configuration
# Demonstrates all available features

heimdal:
  version: "1"
  repo: "git@github.com:yourusername/dotfiles.git"

# Global packages (inherited by all profiles)
packages:
  common:
    - git
    - curl
    - wget
  homebrew:
    - ripgrep
    - fd
    - bat
  homebrew_casks:
    - iterm2
    - visual-studio-code
  apt:
    - ripgrep
    - fd-find
    - bat

# Global ignore patterns
ignore:
  - "*.swp"
  - ".DS_Store"
  - "*.bak"

# Shell history sync
history:
  enabled: true
  sync: true
  max_age_days: 90

profiles:
  # Base profile with common dotfiles
  base:
    dotfiles:
      - .bashrc
      - .zshrc
      - .vimrc
      - .gitconfig
      - source: config/nvim
        target: ~/.config/nvim

  # Work profile extends base
  work:
    extends: base
    dotfiles:
      - .work-secrets
    packages:
      homebrew:
        - awscli
        - kubectl
    hooks:
      post_apply:
        - command: "echo 'Work profile applied!'"
          description: "Notify work setup complete"

  # Personal profile extends base
  personal:
    extends: base
    dotfiles:
      - source: config/personal-git
        target: ~/.gitconfig
        when:
          hostname: "macbook-*"
    packages:
      homebrew_casks:
        - spotify
        - discord
    templates:
      - src: templates/gitconfig.tpl
        dest: ~/.gitconfig
        vars:
          email: "personal@example.com"

  # Server profile - minimal, no GUI
  server:
    dotfiles:
      - .bashrc
      - .vimrc
    packages:
      apt:
        - htop
        - ncdu
    ignore:
      - "*.local"
```

- [ ] **Step 3: Update examples/multi-platform.yaml**

Replace entire file:

```yaml
# Multi-Platform Heimdal Configuration
# Same dotfiles, platform-specific packages

heimdal:
  version: "1"

packages:
  homebrew:
    - git
    - neovim
    - tmux
    - ripgrep
  apt:
    - git
    - neovim
    - tmux
    - ripgrep
  dnf:
    - git
    - neovim
    - tmux
    - ripgrep
  pacman:
    - git
    - neovim
    - tmux
    - ripgrep
  apk:
    - git
    - neovim
    - tmux
    - ripgrep

profiles:
  default:
    dotfiles:
      - .bashrc
      - .zshrc
      - .vimrc
      - source: config/nvim
        target: ~/.config/nvim
      # macOS-specific
      - source: macos/Brewfile
        target: ~/.Brewfile
        when:
          os: [macos]
      # Linux-specific
      - source: linux/xinitrc
        target: ~/.xinitrc
        when:
          os: [linux]

  # macOS-specific profile
  macos:
    extends: default
    packages:
      homebrew_casks:
        - iterm2
        - rectangle
        - raycast
      mas:
        - [497799835, "Xcode"]

  # Linux desktop profile
  linux-desktop:
    extends: default
    packages:
      apt:
        - i3
        - rofi
        - feh
```

- [ ] **Step 4: Validate example files**

Run validation on each:

```bash
cargo build
./target/debug/heimdal validate --config examples/minimal.yaml
./target/debug/heimdal validate --config examples/full.yaml
./target/debug/heimdal validate --config examples/multi-platform.yaml
```

Expected: All validate successfully

- [ ] **Step 5: Commit**

```bash
git add examples/minimal.yaml examples/full.yaml examples/multi-platform.yaml
git commit -m "docs: update example configs to match actual schema"
```

---

## Task 20: Final Testing and Documentation

**Files:**
- Create: `CHANGELOG.md` entry

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

- [ ] **Step 2: Build release binary**

Run: `cargo build --release`
Expected: Successful build

- [ ] **Step 3: Manual integration test**

```bash
# Test lock system
./target/release/heimdal state lock-info

# Test autosync (if on macOS or Linux)
./target/release/heimdal autosync enable --interval 1h
./target/release/heimdal autosync status
./target/release/heimdal autosync disable
```

- [ ] **Step 4: Update CHANGELOG.md**

Add entry at top:

```markdown
## [Unreleased]

### Added
- Process-level locking to prevent concurrent operations
- Native autosync implementation (launchd/systemd/cron)
- Utility functions for common operations (atomic_write, ensure_parent_exists, hostname)
- DotfileEntry methods (source(), target())
- CommandContext struct for reduced boilerplate

### Changed
- Updated example configs to match actual schema
- Consolidated package manager availability checks
- Simplified merge_packages with macro
- Consolidated Linux distro matching

### Removed
- Dead code: empty profile.rs, unused is_installed(), dotfiles_dir(), etc.
- Duplicate write_config implementations

### Fixed
- Example YAML files now use correct schema
```

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: update changelog for cleanup refactor"
```

- [ ] **Step 6: Final verification**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

Run: `cargo fmt -- --check`
Expected: All files formatted

---

## Summary

**Total Tasks:** 20  
**Estimated Time:** 4-6 hours  
**Lines Removed:** ~300  
**Lines Added:** ~430 (new features)  
**Net Effect:** Improved maintainability, reduced duplication, fully functional stub features

**Key Improvements:**
1. Utility consolidation saves ~100 lines across codebase
2. Dead code removal cleans up ~100 lines
3. Lock system adds robust concurrency control
4. Autosync provides real platform-native scheduling
5. Example files now actually work
