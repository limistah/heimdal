//! # Test Helpers for Heimdal CLI
//!
//! This module provides common utilities for testing the Heimdal CLI application.
//!
//! ## Modern Testing Pattern (assert_cmd 2.0+)
//!
//! We use the `cargo_bin_cmd!()` macro from `assert_cmd` 2.0+ for running
//! CLI commands in tests. This is the modern replacement for the deprecated
//! `Command::cargo_bin()` API.
//!
//! ### Why cargo_bin_cmd!()?
//!
//! - **Compatible with custom build directories**: Works with `cargo build --target-dir`
//! - **No unwrap needed**: The macro returns `Command` directly
//! - **More robust**: Better error handling and panic messages
//! - **Officially recommended**: This is the preferred API since assert_cmd 2.0
//!
//! ### Example Usage:
//!
//! ```rust,no_run
//! use assert_cmd::cargo::cargo_bin_cmd;
//!
//! // Simple command test
//! cargo_bin_cmd!()
//!     .arg("init")
//!     .arg("--profile")
//!     .arg("test")
//!     .assert()
//!     .success();
//! ```
//!
//! ### Test Environment Setup
//!
//! Use `TestEnv` to create isolated test environments with fake home directories:
//!
//! ```rust,no_run
//! use crate::helpers::TestEnv;
//!
//! let env = TestEnv::new();
//! env.heimdal_cmd()
//!     .arg("init")
//!     .arg("--repo")
//!     .arg("https://github.com/user/dotfiles.git")
//!     .assert()
//!     .success();
//! ```
//!
//! ### Migration from Old API
//!
//! If you see deprecated warnings, update your tests:
//!
//! ```rust,ignore
//! // OLD (deprecated):
//! Command::cargo_bin("heimdal")
//!     .unwrap()
//!     .arg("init")
//!
//! // NEW (modern):
//! cargo_bin_cmd!()
//!     .arg("init")
//! ```

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use assert_fs::TempDir;
use std::path::PathBuf;

/// Test repository URL for consistent testing
pub const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

/// Test repository URL with drift (for sync/pull tests)
pub const TEST_REPO_DRIFT: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

/// Test repository branch with drift for sync/pull tests
pub const TEST_REPO_DRIFT_BRANCH: &str = "drift-test";

/// Helper to create a heimdal command with proper environment.
///
/// This uses the modern `cargo_bin_cmd!()` macro which is compatible
/// with custom build directories and doesn't require `.unwrap()`.
///
/// # Example
///
/// ```rust,no_run
/// use crate::helpers::heimdal_cmd;
///
/// heimdal_cmd()
///     .arg("--version")
///     .assert()
///     .success();
/// ```
pub fn heimdal_cmd() -> Command {
    cargo_bin_cmd!()
}

/// Setup a test environment with temp directories.
///
/// This struct provides an isolated testing environment with:
/// - A temporary directory that auto-cleans on drop
/// - A fake HOME directory for testing
/// - Helper methods to run commands in this environment
///
/// # Example
///
/// ```rust,no_run
/// use crate::helpers::TestEnv;
///
/// let env = TestEnv::new();
///
/// // Run command with fake HOME
/// env.heimdal_cmd()
///     .arg("init")
///     .arg("--repo")
///     .arg("https://github.com/user/dotfiles.git")
///     .assert()
///     .success();
///
/// // Access the fake home directory
/// let home = env.home_dir();
/// assert!(home.join(".heimdal").exists());
/// ```
pub struct TestEnv {
    #[allow(dead_code)] // Needed for RAII cleanup on drop
    pub temp_dir: TempDir,
    pub home_path: PathBuf,
}

impl TestEnv {
    /// Create a new test environment with isolated temp directories.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let home_path = temp_dir.path().join("home");

        std::fs::create_dir_all(&home_path).expect("Failed to create home dir");

        Self {
            temp_dir,
            home_path,
        }
    }

    /// Get the path to the fake home directory.
    pub fn home_dir(&self) -> PathBuf {
        self.home_path.clone()
    }

    /// Create a heimdal command with this test environment's HOME.
    ///
    /// This automatically sets the HOME environment variable to the
    /// fake home directory created by this TestEnv.
    pub fn heimdal_cmd(&self) -> Command {
        let mut cmd = heimdal_cmd();
        cmd.env("HOME", self.home_dir());
        cmd
    }
}
