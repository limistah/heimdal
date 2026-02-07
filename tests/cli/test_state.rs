/// Tests for state management
///
/// These tests verify that heimdal properly manages its state file.
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

use crate::helpers::{TestEnv, TEST_REPO};

#[test]
#[serial]
fn test_state_file_created_on_init() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // State file should exist
    assert!(env.state_file().exists(), "State file should be created");
}

#[test]
#[serial]
fn test_state_file_is_valid_json() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Read and parse state file
    let state_content =
        std::fs::read_to_string(env.state_file()).expect("Failed to read state file");

    let _: serde_json::Value =
        serde_json::from_str(&state_content).expect("State file should be valid JSON");
}

#[test]
#[serial]
fn test_state_show_command() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Show state
    env.heimdal_cmd()
        .arg("state")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains(TEST_REPO).or(predicate::str::contains("test")));
}

#[test]
#[serial]
fn test_state_persists_across_commands() {
    let env = TestEnv::new();

    env.init_heimdal(TEST_REPO, "test").success();

    // Get initial state
    let initial_state =
        std::fs::read_to_string(env.state_file()).expect("Failed to read state file");

    // Run another command
    let _ = env
        .heimdal_cmd()
        .arg("profile")
        .arg("list")
        .assert()
        .success();

    // State should still exist and contain repo info
    let current_state =
        std::fs::read_to_string(env.state_file()).expect("State file should still exist");

    assert!(
        current_state.contains(TEST_REPO),
        "State should persist repo URL"
    );
}

#[test]
#[serial]
fn test_state_without_init() {
    let env = TestEnv::new();

    // Try to show state without init
    env.heimdal_cmd()
        .arg("state")
        .arg("show")
        .assert()
        .failure();
}
