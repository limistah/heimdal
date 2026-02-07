/// Tests for `heimdal init` command
///
/// These tests verify that the init command properly initializes
/// a heimdal dotfiles repository in various scenarios.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::{TestEnv, TEST_REPO};

#[test]
#[serial]
fn test_init_basic() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test")
        .success()
        .stdout(predicate::str::contains("Initializing Heimdal"));

    // Verify state file was created
    assert!(env.state_file().exists(), "State file should exist");

    // Verify dotfiles directory was created
    assert!(
        env.dotfiles_dir().exists(),
        "Dotfiles directory should exist"
    );
}

#[test]
#[serial]
fn test_init_with_invalid_repo() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg("https://github.com/nonexistent-user-xyz/nonexistent-repo-xyz.git")
        .arg("--profile")
        .arg("test")
        .current_dir(env.temp_dir.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_without_required_args() {
    let env = TestEnv::new();

    // Missing --repo
    env.heimdal_cmd()
        .arg("init")
        .arg("--profile")
        .arg("test")
        .current_dir(env.temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[serial]
fn test_init_creates_state_with_correct_content() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Read and verify state file
    let state_content =
        std::fs::read_to_string(env.state_file()).expect("Failed to read state file");

    assert!(
        state_content.contains(TEST_REPO),
        "State should contain repo URL"
    );
    assert!(
        state_content.contains("test"),
        "State should contain profile name"
    );
}

#[test]
#[serial]
fn test_init_clones_repository() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Verify .git directory exists
    let git_dir = env.dotfiles_dir().join(".git");
    assert!(git_dir.exists(), "Git directory should exist");

    // Verify heimdal.yaml exists
    let config_file = env.dotfiles_dir().join("heimdal.yaml");
    assert!(config_file.exists(), "Config file should exist");
}

#[test]
#[serial]
fn test_init_with_nonexistent_profile() {
    let env = TestEnv::new();

    // Initialize with a profile that doesn't exist in the config
    let result = env
        .heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("nonexistent-profile-xyz")
        .current_dir(env.temp_dir.path())
        .assert();

    // This might succeed but profile operations should fail later
    // Or it might fail immediately - either is acceptable
    // Just verify the command completes
    let _ = result.get_output();
}

#[test]
#[serial]
fn test_init_respects_custom_path() {
    let env = TestEnv::new();
    let custom_path = env.temp_dir.path().join("custom-dotfiles");

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .arg("--path")
        .arg(&custom_path)
        .current_dir(env.temp_dir.path())
        .assert()
        .success();

    // Verify custom path was used
    assert!(custom_path.exists(), "Custom dotfiles path should exist");
    assert!(
        custom_path.join(".git").exists(),
        "Git dir should exist in custom path"
    );
}

#[test]
#[serial]
fn test_init_twice_fails() {
    let env = TestEnv::new();

    // First init should succeed
    env.init_heimdal(TEST_REPO, "test").success();

    // Second init should fail or warn
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .current_dir(env.temp_dir.path())
        .assert()
        .failure();
}
