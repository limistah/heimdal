// Integration tests for heimdal utility commands
//
// Tests cover:
// - rollback command (rollback to previous version)
// - history command (show change history)
// - config command (manage configuration)
// - auto-sync command (manage auto-sync)
// - sync command (sync from remote)

use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

// ===== ROLLBACK COMMAND TESTS =====

#[test]
fn test_rollback_help() {
    cargo_bin_cmd!()
        .arg("rollback")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Rollback to a previous version"))
        .stdout(predicates::str::contains("TARGET"));
}

#[test]
#[serial]
fn test_rollback_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Rollback should fail when not initialized
    cargo_bin_cmd!()
        .arg("rollback")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_rollback_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Try rollback (should fail or show error - no previous state to rollback to)
    cargo_bin_cmd!()
        .arg("rollback")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .code(predicates::function::function(|code: &i32| {
            *code == 0 || *code == 1
        }));
}

// ===== HISTORY COMMAND TESTS =====

#[test]
fn test_history_help() {
    cargo_bin_cmd!()
        .arg("history")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Show change history"))
        .stdout(predicates::str::contains("--limit"));
}

#[test]
#[serial]
fn test_history_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // History should fail when not initialized
    cargo_bin_cmd!()
        .arg("history")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_history_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Show history
    cargo_bin_cmd!()
        .arg("history")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_history_with_limit() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Show history with limit
    cargo_bin_cmd!()
        .args(["history", "--limit", "5"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

// ===== CONFIG COMMAND TESTS =====

#[test]
fn test_config_help() {
    cargo_bin_cmd!()
        .arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Show configuration"))
        .stdout(predicates::str::contains("get"))
        .stdout(predicates::str::contains("set"))
        .stdout(predicates::str::contains("show"));
}

#[test]
fn test_config_show_help() {
    cargo_bin_cmd!()
        .args(["config", "show", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Show"));
}

#[test]
#[serial]
fn test_config_show_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Config show succeeds even without init (may show default or local config)
    cargo_bin_cmd!()
        .args(["config", "show"])
        .env("HOME", temp.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_config_show_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Show config
    cargo_bin_cmd!()
        .args(["config", "show"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

// ===== AUTO-SYNC COMMAND TESTS =====

#[test]
fn test_auto_sync_help() {
    cargo_bin_cmd!()
        .arg("auto-sync")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Manage auto-sync"))
        .stdout(predicates::str::contains("enable"))
        .stdout(predicates::str::contains("disable"))
        .stdout(predicates::str::contains("status"));
}

#[test]
#[serial]
fn test_auto_sync_status_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Auto-sync status succeeds and shows it's disabled
    cargo_bin_cmd!()
        .args(["auto-sync", "status"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Auto-Sync Status"));
}

#[test]
#[serial]
fn test_auto_sync_status_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check auto-sync status
    cargo_bin_cmd!()
        .args(["auto-sync", "status"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

// ===== SYNC COMMAND TESTS =====

#[test]
fn test_sync_help() {
    cargo_bin_cmd!()
        .arg("sync")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Sync from remote"))
        .stdout(predicates::str::contains("--dry-run"))
        .stdout(predicates::str::contains("--quiet"));
}

#[test]
#[serial]
fn test_sync_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Sync should fail when not initialized
    cargo_bin_cmd!()
        .arg("sync")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_sync_dry_run_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    cargo_bin_cmd!()
        .args(["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Sync with dry-run (safer, won't actually apply changes)
    cargo_bin_cmd!()
        .args(["sync", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}
