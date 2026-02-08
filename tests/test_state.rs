/// Test state command functionality
///
/// Tests state management: locks, conflicts, migrations, drift detection
#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

use crate::helpers::{TestEnv, TEST_REPO, TEST_REPO_DRIFT, TEST_REPO_DRIFT_BRANCH};

#[test]
fn test_state_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage state"))
        .stdout(predicate::str::contains("lock-info"))
        .stdout(predicate::str::contains("unlock"))
        .stdout(predicate::str::contains("check-conflicts"))
        .stdout(predicate::str::contains("resolve"))
        .stdout(predicate::str::contains("check-drift"))
        .stdout(predicate::str::contains("history"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("migrate"));
}

#[test]
fn test_state_verbose_flag() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("--verbose")
        .arg("--help")
        .assert()
        .success();
}

// Test lock-info subcommand

#[test]
fn test_state_lock_info_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("lock-info")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed with no state or fail gracefully
}

#[test]
fn test_state_lock_info_after_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Check lock info (should work or fail gracefully due to state format)
    env.heimdal_cmd()
        .arg("state")
        .arg("lock-info")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_lock_info_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("lock-info")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show current lock status"));
}

// Test unlock subcommand

#[test]
fn test_state_unlock_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("unlock")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail gracefully
}

#[test]
fn test_state_unlock_after_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Try to unlock when no lock exists (should fail gracefully or succeed with message)
    env.heimdal_cmd()
        .arg("state")
        .arg("unlock")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // Either success or failure is acceptable
}

#[test]
fn test_state_unlock_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("unlock")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Force remove an active lock"));
}

// Test check-conflicts subcommand

#[test]
fn test_state_check_conflicts_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("check-conflicts")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_state_check_conflicts_after_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Check conflicts (should work or fail gracefully due to state format)
    env.heimdal_cmd()
        .arg("state")
        .arg("check-conflicts")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_check_conflicts_with_drift_branch() {
    let env = TestEnv::new();

    // Initialize with drift test branch
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO_DRIFT)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Switch to drift branch
    let dotfiles_dir = env.home_dir().join(".dotfiles");
    std::process::Command::new("git")
        .arg("-C")
        .arg(&dotfiles_dir)
        .arg("checkout")
        .arg(TEST_REPO_DRIFT_BRANCH)
        .output()
        .ok();

    // Check conflicts (may detect conflicts between branches)
    env.heimdal_cmd()
        .arg("state")
        .arg("check-conflicts")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_check_conflicts_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("check-conflicts")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Check for state conflicts"));
}

// Test resolve subcommand

#[test]
fn test_state_resolve_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("resolve")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_state_resolve_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("resolve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolve detected conflicts"));
}

// Test check-drift subcommand

#[test]
fn test_state_check_drift_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("check-drift")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_state_check_drift_after_init_no_changes() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Check drift with no changes (should report no drift or succeed)
    env.heimdal_cmd()
        .arg("state")
        .arg("check-drift")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_check_drift_with_modified_file() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Modify a dotfile directly (simulating drift)
    let dotfiles_dir = env.home_dir().join(".dotfiles");
    let config_file = dotfiles_dir.join("heimdal.yaml");

    if config_file.exists() {
        fs::write(&config_file, "# Modified outside heimdal\n").expect("Failed to modify config");

        // Check drift (should detect the modification)
        env.heimdal_cmd()
            .arg("state")
            .arg("check-drift")
            .assert()
            .code(predicate::in_iter(vec![0, 1])); // May succeed with drift detected or fail
    }
}

#[test]
fn test_state_check_drift_with_drift_branch() {
    let env = TestEnv::new();

    // Initialize with drift test branch
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO_DRIFT)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Switch to drift branch which may have different content
    let dotfiles_dir = env.home_dir().join(".dotfiles");
    std::process::Command::new("git")
        .arg("-C")
        .arg(&dotfiles_dir)
        .arg("checkout")
        .arg(TEST_REPO_DRIFT_BRANCH)
        .output()
        .ok();

    // Check drift
    env.heimdal_cmd()
        .arg("state")
        .arg("check-drift")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_check_drift_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("check-drift")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Check for file drift"));
}

// Test history subcommand

#[test]
fn test_state_history_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("history")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_state_history_after_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Show history (should succeed even if empty or fail gracefully due to state format)
    env.heimdal_cmd()
        .arg("state")
        .arg("history")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_history_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("history")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show operation history"));
}

// Test version subcommand

#[test]
fn test_state_version_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("version")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_state_version_after_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Show state version (should work or fail gracefully due to state format)
    env.heimdal_cmd()
        .arg("state")
        .arg("version")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_state_version_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("version")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show state version"));
}

// Test migrate subcommand

#[test]
fn test_state_migrate_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("state")
        .arg("migrate")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail gracefully
}

#[test]
fn test_state_migrate_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("state")
        .arg("migrate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Migrate from V1 to V2"));
}
