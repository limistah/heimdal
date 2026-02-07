/// Tests for `heimdal validate` command
///
/// These tests verify configuration validation functionality.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::{fixtures, TestEnv, TEST_REPO};

#[test]
#[serial]
fn test_validate_valid_config() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Validate should succeed for valid config
    env.heimdal_cmd()
        .arg("validate")
        .current_dir(env.dotfiles_dir())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_validate_invalid_yaml() {
    let env = TestEnv::new();

    // Create a directory with invalid YAML
    let config_dir = env.temp_dir.path().join("invalid-config");
    std::fs::create_dir_all(&config_dir).unwrap();

    fixtures::create_invalid_config(&config_dir).unwrap();

    // Validate should fail
    env.heimdal_cmd()
        .arg("validate")
        .current_dir(&config_dir)
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_validate_missing_config() {
    let env = TestEnv::new();

    // Try to validate in directory without heimdal.yaml
    env.heimdal_cmd()
        .arg("validate")
        .current_dir(env.temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
#[serial]
fn test_validate_from_dotfiles_dir() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Running validate from dotfiles directory should work
    env.heimdal_cmd()
        .arg("validate")
        .current_dir(env.dotfiles_dir())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("valid")
                .or(predicate::str::contains("✓"))
                .or(predicate::str::contains("success")),
        );
}

#[test]
#[serial]
fn test_validate_minimal_config() {
    let env = TestEnv::new();

    // Create minimal valid config
    let config_dir = env.temp_dir.path().join("minimal-config");
    std::fs::create_dir_all(&config_dir).unwrap();

    fixtures::create_minimal_config(&config_dir).unwrap();

    // Should validate successfully
    env.heimdal_cmd()
        .arg("validate")
        .current_dir(&config_dir)
        .assert()
        .success();
}
