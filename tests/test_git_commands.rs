// Integration tests for heimdal git-related commands
//
// Tests cover:
// - diff command (show changes)
// - commit command (commit changes with auto-generated messages)

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

// ===== DIFF COMMAND TESTS =====

#[test]
fn test_diff_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Show local changes"))
        .stdout(predicates::str::contains("--verbose"))
        .stdout(predicates::str::contains("--interactive"));
}

#[test]
#[serial]
fn test_diff_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Diff should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

#[test]
#[serial]
fn test_diff_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Run diff (should show no changes initially)
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

// ===== COMMIT COMMAND TESTS =====

#[test]
fn test_commit_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("commit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Commit changes"))
        .stdout(predicates::str::contains("--message"))
        .stdout(predicates::str::contains("--push"));
}

#[test]
#[serial]
fn test_commit_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Commit should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["commit", "-m", "test"])
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}
