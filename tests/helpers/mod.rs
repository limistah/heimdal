pub mod fixtures;
/// Common test helpers for heimdal CLI testing
///
/// This module provides utilities for testing heimdal CLI commands
/// across different platforms and scenarios.
pub mod git;

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
    pub temp_dir: TempDir,
    pub dotfiles_path: PathBuf,
    pub home_path: PathBuf,
}

impl TestEnv {
    /// Create a new test environment
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dotfiles_path = temp_dir.path().join(".dotfiles");
        let home_path = temp_dir.path().join("home");

        std::fs::create_dir_all(&home_path).expect("Failed to create home dir");

        Self {
            temp_dir,
            dotfiles_path,
            home_path,
        }
    }

    /// Get the path to the dotfiles directory
    pub fn dotfiles_dir(&self) -> PathBuf {
        self.dotfiles_path.clone()
    }

    /// Get the path to the fake home directory
    pub fn home_dir(&self) -> PathBuf {
        self.home_path.clone()
    }

    /// Get the state file path
    pub fn state_file(&self) -> PathBuf {
        self.home_path.join(".heimdal").join("heimdal.state.json")
    }

    /// Create a heimdal command with this environment
    pub fn heimdal_cmd(&self) -> Command {
        let mut cmd = heimdal_cmd();
        cmd.env("HOME", self.home_dir());
        cmd
    }

    /// Initialize heimdal in this environment
    pub fn init_heimdal(&self, repo: &str, profile: &str) -> assert_cmd::assert::Assert {
        self.heimdal_cmd()
            .arg("init")
            .arg("--repo")
            .arg(repo)
            .arg("--profile")
            .arg(profile)
            .current_dir(self.temp_dir.path())
            .assert()
    }
}

/// Platform detection helpers
pub mod platform {
    use std::env;

    pub fn is_linux() -> bool {
        env::consts::OS == "linux"
    }

    pub fn is_macos() -> bool {
        env::consts::OS == "macos"
    }

    pub fn is_windows() -> bool {
        env::consts::OS == "windows"
    }

    /// Get the package manager for the current platform
    pub fn package_manager() -> Option<&'static str> {
        if is_macos() {
            Some("brew")
        } else if is_linux() {
            // Try to detect Linux package manager
            if std::process::Command::new("apt").output().is_ok() {
                Some("apt")
            } else if std::process::Command::new("pacman").output().is_ok() {
                Some("pacman")
            } else if std::process::Command::new("dnf").output().is_ok() {
                Some("dnf")
            } else if std::process::Command::new("apk").output().is_ok() {
                Some("apk")
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Assertion helpers
pub mod assertions {
    use std::path::Path;

    /// Check if a symlink exists and points to the expected target
    pub fn assert_symlink_exists(link: &Path, target: &Path) -> bool {
        if !link.exists() {
            return false;
        }

        if !link.is_symlink() {
            return false;
        }

        if let Ok(link_target) = std::fs::read_link(link) {
            link_target == target
        } else {
            false
        }
    }

    /// Check if a file contains a specific string
    pub fn file_contains(path: &Path, needle: &str) -> bool {
        if let Ok(content) = std::fs::read_to_string(path) {
            content.contains(needle)
        } else {
            false
        }
    }
}
