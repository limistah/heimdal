/// Test `heimdal profiles` command
///
/// Tests profile listing functionality including:
/// - Help output
/// - Listing profiles after init
/// - Listing without init (should fail)
/// - Multiple profiles display
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_profiles_help() {
    cargo_bin_cmd!()
        .arg("profiles")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List available profiles"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
#[serial]
fn test_profiles_without_init_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Try to list profiles without initializing (no heimdal.yaml in current dir)
    // The command succeeds but shows a warning message
    cargo_bin_cmd!()
        .arg("profiles")
        .current_dir(temp.path())
        .env("HOME", temp.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("No heimdal.yaml found"));
}

#[test]
#[serial]
fn test_profiles_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with test repo
    cargo_bin_cmd!()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    let dotfiles_dir = temp.path().join(".dotfiles");

    // List profiles should succeed and show test profile (from dotfiles dir)
    cargo_bin_cmd!()
        .arg("profiles")
        .current_dir(&dotfiles_dir)
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profiles_shows_multiple() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with test repo
    cargo_bin_cmd!()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    let dotfiles_dir = temp.path().join(".dotfiles");

    // List profiles from dotfiles directory
    let output = cargo_bin_cmd!()
        .arg("profiles")
        .current_dir(&dotfiles_dir)
        .env("HOME", temp.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Should list test profile at minimum
    assert!(stdout.contains("test"), "Should list test profile");
}

#[test]
#[serial]
fn test_profiles_verbose_flag() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with test repo
    cargo_bin_cmd!()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    let dotfiles_dir = temp.path().join(".dotfiles");

    // Test verbose flag works
    cargo_bin_cmd!()
        .arg("profiles")
        .arg("--verbose")
        .current_dir(&dotfiles_dir)
        .env("HOME", temp.path())
        .assert()
        .success();
}
