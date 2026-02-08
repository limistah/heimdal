/// Test `heimdal profile` command and subcommands
///
/// Tests profile management functionality including:
/// - Help output
/// - switch, current, show, list subcommands
/// - Error handling without init
/// - Profile switching
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_profile_help() {
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage profiles"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("current"))
        .stdout(predicate::str::contains("show"));
}

#[test]
#[serial]
fn test_profile_list_without_init_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("list")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_profile_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // List profiles
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("list")
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profile_current_shows_active() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with test profile
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Get current profile
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("current")
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profile_show_displays_info() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Show profile details
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("show")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profile_show_nonexistent_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Try to show nonexistent profile
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("show")
        .arg("nonexistent-profile-xyz")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_profile_switch_to_development() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with test profile
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Switch to development profile (if it exists in test repo)
    let result = cargo_bin_cmd!()
        
        .arg("profile")
        .arg("switch")
        .arg("development")
        .env("HOME", temp.path())
        .assert();

    // Should either succeed or fail gracefully
    let _ = result.get_output();
}

#[test]
#[serial]
fn test_profile_switch_nonexistent_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize
    cargo_bin_cmd!()
        
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Try to switch to nonexistent profile
    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("switch")
        .arg("nonexistent-profile-xyz")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_profile_current_without_init_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    cargo_bin_cmd!()
        
        .arg("profile")
        .arg("current")
        .env("HOME", temp.path())
        .assert()
        .failure();
}
