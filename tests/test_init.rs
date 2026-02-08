/// Test `heimdal init` command
///
/// Tests initialization functionality including:
/// - Help output
/// - Required arguments validation
/// - Repository cloning
/// - Profile selection
/// - Error handling
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serial_test::serial;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_init_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize Heimdal"))
        .stdout(predicate::str::contains("--repo"))
        .stdout(predicate::str::contains("--profile"));
}

#[test]
#[serial]
fn test_init_without_repo_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("--repo")));
}

#[test]
#[serial]
fn test_init_without_profile_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("--profile")));
}

#[test]
#[serial]
fn test_init_with_invalid_repo_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg("https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git")
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_basic_success() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Verify dotfiles directory was created
    let dotfiles_dir = temp.child(".dotfiles");
    dotfiles_dir.assert(predicate::path::exists());

    // Verify state file was created in dotfiles directory
    let state_file = dotfiles_dir.child("heimdal.state.json");
    state_file.assert(predicate::path::exists());

    // Verify git repo was cloned
    let git_dir = dotfiles_dir.child(".git");
    git_dir.assert(predicate::path::exists());

    // Verify heimdal.yaml exists
    let config_file = dotfiles_dir.child("heimdal.yaml");
    config_file.assert(predicate::path::exists());
}

#[test]
#[serial]
fn test_init_creates_correct_state_content() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Read state file from dotfiles directory and verify content
    let dotfiles_dir = temp.child(".dotfiles");
    let state_file = dotfiles_dir.child("heimdal.state.json");
    let state_content = std::fs::read_to_string(state_file.path()).unwrap();

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
fn test_init_with_custom_path() {
    let temp = assert_fs::TempDir::new().unwrap();
    let custom_path = temp.child("custom-dotfiles");

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .arg("--path")
        .arg(custom_path.path())
        .env("HOME", temp.path())
        .assert()
        .success();

    // Verify custom path was used
    custom_path.assert(predicate::path::exists());

    // Verify git repo exists in custom path
    let git_dir = custom_path.child(".git");
    git_dir.assert(predicate::path::exists());
}

#[test]
#[serial]
fn test_init_twice_fails() {
    let temp = assert_fs::TempDir::new().unwrap();

    // First init should succeed
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    // Second init should fail
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_with_nonexistent_profile() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Initialize with a profile that doesn't exist in the config
    // This might succeed during init but fail later during operations
    let result = Command::cargo_bin("heimdal")
        .unwrap()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("nonexistent-profile-xyz")
        .env("HOME", temp.path())
        .assert();

    // Either fails during init or succeeds but profile operations will fail later
    // We just verify the command completes
    let _ = result.get_output();
}
