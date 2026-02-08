// Integration tests for heimdal git-related commands
//
// Tests cover:
// - diff command (show changes)
// - commit command (commit changes)
// - push/pull commands (sync with remote)
// - branch command (manage branches)
// - remote command (manage remotes)

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
        .stdout(predicates::str::contains("Commit changes"));
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

// ===== PUSH COMMAND TESTS =====

#[test]
fn test_push_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("push")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Push"));
}

#[test]
#[serial]
fn test_push_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Push should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("push")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

// ===== PULL COMMAND TESTS =====

#[test]
fn test_pull_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("pull")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Pull"));
}

#[test]
#[serial]
fn test_pull_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Pull should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("pull")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not initialized"));
}

// ===== BRANCH COMMAND TESTS =====

#[test]
fn test_branch_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("branch")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("branch"));
}

#[test]
#[serial]
fn test_branch_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Branch should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("branch")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_branch_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // List branches (use 'list' subcommand)
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["branch", "list"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

// ===== REMOTE COMMAND TESTS =====

#[test]
fn test_remote_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("remote")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("remote"));
}

#[test]
#[serial]
fn test_remote_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Remote should fail when not initialized
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("remote")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_remote_list_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // List remotes (use 'list' subcommand)
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["remote", "list"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}
