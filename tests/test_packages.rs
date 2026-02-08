// Integration tests for heimdal packages command
//
// Tests cover:
// - Help output for main command and subcommands
// - Package database operations (update, cache)
// - Package search and info
// - Package list operations
// - Package group operations

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_packages_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("packages")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Manage packages"))
        .stdout(predicates::str::contains("list"))
        .stdout(predicates::str::contains("search"))
        .stdout(predicates::str::contains("update"));
}

#[test]
fn test_packages_list_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("List all packages"));
}

#[test]
#[serial]
fn test_packages_list_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // List fails when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list"])
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_packages_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // List packages from the test profile
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Packages"));
}

#[test]
fn test_packages_update_database() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update package database (downloads package info)
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("database"));
}

#[test]
fn test_packages_search() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Search for a common package
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "search", "git"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("git"));
}

#[test]
fn test_packages_search_no_results() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Search for non-existent package
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "search", "thisdoesnotexist999999"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Found 0").or(predicates::str::contains("No packages")));
}

#[test]
fn test_packages_info() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Get info for a known package
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "info", "git"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("git"));
}

#[test]
fn test_packages_cache_info() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first to create cache
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Check cache info
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "cache-info"])
        .env("HOME", temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("cache").or(predicates::str::contains("Cache")));
}

#[test]
fn test_packages_cache_clear() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first to create cache
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Clear cache
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "cache-clear"])
        .env("HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_packages_list_groups() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // List package groups
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list-groups"])
        .env("HOME", temp.path())
        .assert()
        .success();
}

#[test]
fn test_packages_search_groups() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Update database first
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "update"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Search groups
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "search-groups", "dev"])
        .env("HOME", temp.path())
        .assert()
        .success();
}
