// Integration tests for heimdal apply command
//
// Tests cover:
// - Package installation (with platform-specific package names)
// - Symlink creation
// - Dry-run mode
// - Force mode
// - Error handling

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_apply_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Apply configuration"))
        .stdout(predicates::str::contains("--dry-run"))
        .stdout(predicates::str::contains("--force"));
}

#[test]
#[serial]
fn test_apply_without_init() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Apply should fail when not initialized
    // The error message can be either "not initialized" or a file system error
    let result = Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", temp.path())
        .assert()
        .failure();

    // Accept either error message (implementation-dependent)
    let stderr = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        stderr.contains("not initialized") || stderr.contains("No such file or directory"),
        "Expected initialization error, got: {}",
        stderr
    );
}

#[test]
#[serial]
fn test_apply_dry_run_after_init() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run (won't actually install or create symlinks)
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Dry-run mode"))
        .stdout(predicates::str::contains("Installing Packages"));
}

#[test]
#[serial]
fn test_apply_dry_run_shows_packages() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run and check that it shows the packages
    let output = Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show packages from the test profile
    assert!(
        stdout.contains("git") || stdout.contains("vim") || stdout.contains("curl"),
        "Apply should show packages to be installed"
    );
}

#[test]
#[serial]
fn test_apply_dry_run_shows_symlinks() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run and check that it shows symlinks
    let output = Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show symlinks that would be created
    assert!(
        stdout.contains("Creating Symlinks") || stdout.contains("Linking"),
        "Apply should show symlinks to be created"
    );

    // Should mention specific files from the test profile
    assert!(
        stdout.contains(".bashrc") || stdout.contains(".vimrc"),
        "Apply should show specific dotfiles"
    );
}

#[test]
#[serial]
fn test_apply_dry_run_development_profile() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize with development profile
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "development"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run for development profile
    let output = Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Development profile should include packages from both base and dev sources
    // The test repo has: git, vim, curl (base) + additional packages in development profile
    assert!(
        stdout.contains("ripgrep") || stdout.contains("rg") || stdout.contains("git") || stdout.contains("vim"),
        "Development profile should include packages (git, vim, or development-specific packages). Got: {}",
        stdout
    );
}

#[test]
#[serial]
fn test_apply_dry_run_verbose() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run and verbose
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run", "--verbose"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success();
}

#[test]
#[serial]
fn test_apply_shows_would_run_commands() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run should show what commands would be run
    let output = Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show package manager commands that would be executed
    assert!(
        stdout.contains("Would run:") || stdout.contains("Would create symlink"),
        "Dry-run should show commands that would be executed"
    );
}

#[test]
#[serial]
fn test_apply_detects_package_manager() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run should detect and show package manager
    let output = Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .get_output()
        .clone();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention which package manager is being used, or report that none was found
    assert!(
        stdout.contains("package manager:")
            || stdout.contains("brew")
            || stdout.contains("apt")
            || stdout.contains("pacman")
            || stdout.contains("dnf")
            || stdout.contains("apk")
            || stdout.contains("No supported package manager found"),
        "Apply should show which package manager is detected or report none found. Got: {}",
        stdout
    );
}

#[test]
#[serial]
fn test_apply_shows_summary() {
    let temp = assert_fs::TempDir::new().unwrap();
    let dotfiles_dir = temp.child(".dotfiles");

    // Initialize heimdal
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["init", "--repo", TEST_REPO, "--profile", "test"])
        .env("HOME", temp.path())
        .assert()
        .success();

    // Apply with dry-run should show a summary
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", temp.path())
        .current_dir(&dotfiles_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Summary").or(predicates::str::contains("Installed:")));
}
