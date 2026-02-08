/// Common test helpers for heimdal CLI testing
///
/// This module provides utilities for testing heimdal CLI commands
/// across different platforms and scenarios.
use assert_cmd::Command;
use assert_fs::TempDir;
use std::path::PathBuf;

/// Test repository URL for consistent testing
pub const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

/// Test repository URL with drift (for sync/pull tests)
pub const TEST_REPO_DRIFT: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";
pub const TEST_REPO_DRIFT_BRANCH: &str = "drift-test";

/// Helper to create a heimdal command with proper environment
pub fn heimdal_cmd() -> Command {
    Command::cargo_bin("heimdal").expect("Failed to find heimdal binary")
}

/// Setup a test environment with temp directories
pub struct TestEnv {
    #[allow(dead_code)] // Needed for RAII cleanup on drop
    pub temp_dir: TempDir,
    pub home_path: PathBuf,
}

impl TestEnv {
    /// Create a new test environment
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let home_path = temp_dir.path().join("home");

        std::fs::create_dir_all(&home_path).expect("Failed to create home dir");

        Self {
            temp_dir,
            home_path,
        }
    }

    /// Get the path to the fake home directory
    pub fn home_dir(&self) -> PathBuf {
        self.home_path.clone()
    }

    /// Create a heimdal command with this environment
    pub fn heimdal_cmd(&self) -> Command {
        let mut cmd = heimdal_cmd();
        cmd.env("HOME", self.home_dir());
        cmd
    }
}
