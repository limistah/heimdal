/// Test secret command functionality
///
/// Tests secret management: add, get, remove, list
///
/// NOTE: Most tests are marked as #[ignore] because they require system keychain access
/// which may not be available in CI environments or requires user authentication.
/// Run with: cargo test -- --ignored
#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

use crate::helpers::{TestEnv, TEST_REPO};

#[test]
fn test_secret_help() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage secrets"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_secret_verbose_flag() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("--verbose")
        .arg("--help")
        .assert()
        .success();
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_list_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("secret")
        .arg("list")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_add_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("API_KEY")
        .arg("--value")
        .arg("secret_value")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_get_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("API_KEY")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_remove_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("secret")
        .arg("remove")
        .arg("--force")
        .arg("API_KEY")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_list_after_init() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // List secrets (should succeed even if empty)
    env.heimdal_cmd()
        .arg("secret")
        .arg("list")
        .assert()
        .success();
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_add_and_get() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Add a secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("TEST_API_KEY")
        .arg("--value")
        .arg("my_secret_value_123")
        .assert()
        .success();

    // Get the secret
    let output = env
        .heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("TEST_API_KEY")
        .assert()
        .success();

    // Should contain the secret value
    output.stdout(predicate::str::contains("my_secret_value_123"));
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_add_and_list() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Add multiple secrets
    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("API_KEY_1")
        .arg("--value")
        .arg("value1")
        .assert()
        .success();

    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("API_KEY_2")
        .arg("--value")
        .arg("value2")
        .assert()
        .success();

    // List secrets (should show names but not values)
    env.heimdal_cmd()
        .arg("secret")
        .arg("list")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("API_KEY_1")
                .or(predicate::str::contains("API_KEY_2"))
                .or(predicate::str::contains("2")) // could show count
                .or(predicate::str::contains("secret")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_remove() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Add a secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("TEMP_SECRET")
        .arg("--value")
        .arg("temp_value")
        .assert()
        .success();

    // Verify it exists
    env.heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("TEMP_SECRET")
        .assert()
        .success();

    // Remove the secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("remove")
        .arg("--force")
        .arg("TEMP_SECRET")
        .assert()
        .success();

    // Verify it's gone
    env.heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("TEMP_SECRET")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_get_nonexistent() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Try to get nonexistent secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("NONEXISTENT_SECRET")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_remove_nonexistent() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Try to remove nonexistent secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("remove")
        .arg("--force")
        .arg("NONEXISTENT_SECRET")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
#[ignore = "requires system keychain access"]
fn test_secret_update_existing() {
    let env = TestEnv::new();

    // Initialize
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Add a secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("UPDATE_TEST")
        .arg("--value")
        .arg("old_value")
        .assert()
        .success();

    // Update the secret
    env.heimdal_cmd()
        .arg("secret")
        .arg("add")
        .arg("UPDATE_TEST")
        .arg("--value")
        .arg("new_value")
        .assert()
        .success();

    // Get the secret and verify it has the new value
    let output = env
        .heimdal_cmd()
        .arg("secret")
        .arg("get")
        .arg("UPDATE_TEST")
        .assert()
        .success();

    output.stdout(predicate::str::contains("new_value"));
}

#[test]
fn test_secret_add_help() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Add or update a secret"));
}

#[test]
fn test_secret_get_help() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("get")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Get a secret value"));
}

#[test]
fn test_secret_remove_help() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("remove")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove a secret"));
}

#[test]
fn test_secret_list_help() {
    cargo_bin_cmd!()
        
        .arg("secret")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List all secret names"));
}
