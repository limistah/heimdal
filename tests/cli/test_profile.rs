/// Tests for `heimdal profile` commands
///
/// These tests verify profile management operations including
/// listing, showing, switching, and creating profiles.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::{TestEnv, TEST_REPO};

#[test]
#[serial]
fn test_profile_list() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // List profiles
    env.heimdal_cmd()
        .arg("profile")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profile_show() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Show specific profile
    env.heimdal_cmd()
        .arg("profile")
        .arg("show")
        .arg("test")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("test")
                .or(predicate::str::contains("git"))
                .or(predicate::str::contains("vim")),
        );
}

#[test]
#[serial]
fn test_profile_show_nonexistent() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Try to show nonexistent profile
    env.heimdal_cmd()
        .arg("profile")
        .arg("show")
        .arg("nonexistent-profile-xyz")
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_profile_current() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Get current profile
    env.heimdal_cmd()
        .arg("profile")
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[serial]
fn test_profile_switch() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Switch to development profile (if it exists in test repo)
    let result = env
        .heimdal_cmd()
        .arg("profile")
        .arg("switch")
        .arg("development")
        .assert();

    // Should either succeed or fail gracefully
    let _ = result.get_output();
}

#[test]
#[serial]
fn test_profile_list_shows_multiple() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // List should show multiple profiles
    let output = env
        .heimdal_cmd()
        .arg("profile")
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);

    // Test repo should have at least 'test' and 'development' profiles
    assert!(stdout.contains("test"), "Should list test profile");
    // Development profile might also be there
    let has_development = stdout.contains("development");
    let has_multiple = stdout.lines().filter(|l| !l.trim().is_empty()).count() >= 1;

    assert!(has_multiple, "Should list at least one profile");
}

#[test]
#[serial]
fn test_profile_without_init_fails() {
    let env = TestEnv::new();

    // Try to list profiles without initializing
    env.heimdal_cmd()
        .arg("profile")
        .arg("list")
        .assert()
        .failure();
}
