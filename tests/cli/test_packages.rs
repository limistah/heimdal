/// Tests for package management commands
///
/// These tests verify package listing and management functionality.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::{platform, TestEnv, TEST_REPO};

#[test]
#[serial]
fn test_packages_list() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // List packages
    env.heimdal_cmd()
        .arg("packages")
        .arg("list")
        .arg("--profile")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("git").or(predicate::str::contains("vim")));
}

#[test]
#[serial]
fn test_packages_list_without_profile_flag() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // List packages for current profile (should use test from state)
    env.heimdal_cmd()
        .arg("packages")
        .arg("list")
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_info() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Get info about a specific package
    env.heimdal_cmd()
        .arg("packages")
        .arg("info")
        .arg("git")
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_detect_manager() {
    // Skip if no package manager available
    if platform::package_manager().is_none() {
        return;
    }

    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Package manager should be detected
    // This is implicit in package operations working
    env.heimdal_cmd()
        .arg("packages")
        .arg("list")
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_list_nonexistent_profile() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Try to list packages for nonexistent profile
    env.heimdal_cmd()
        .arg("packages")
        .arg("list")
        .arg("--profile")
        .arg("nonexistent-xyz")
        .assert()
        .failure();
}
