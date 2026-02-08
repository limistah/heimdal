// Integration tests for heimdal status command
//
// Tests cover:
// - Help output
// - Status without initialization
// - Status after initialization
// - Status with verbose flag

use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_status_help() {
    cargo_bin_cmd!()
        
        .arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Show current status"))
        .stdout(predicates::str::contains("--verbose"));
}

#[test]
#[serial]
fn test_status_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Status command fails when not initialized
    cargo_bin_cmd!()
        
        .arg("status")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_status_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check status after initialization
    cargo_bin_cmd!()
        
        .arg("status")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Heimdal Status"))
        .stdout(predicates::str::contains("Profile"));
}

#[test]
#[serial]
fn test_status_shows_profile_info() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize with test profile
    cargo_bin_cmd!()
        
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Status should show the active profile
    cargo_bin_cmd!()
        
        .arg("status")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("test"));
}

#[test]
#[serial]
fn test_status_verbose_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Test verbose flag
    let output = cargo_bin_cmd!()
        
        .args(["status", "--verbose"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose should show more detailed information
    assert!(
        stdout.contains("Heimdal Status"),
        "Verbose status should show status header"
    );
}

#[test]
#[serial]
fn test_status_shows_dotfiles_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Status should show dotfiles directory path
    cargo_bin_cmd!()
        
        .arg("status")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Dotfiles"));
}
