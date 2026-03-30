use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;
use std::path::Path;
use std::process;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn git(dir: &Path, args: &[&str]) {
    let out = process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git not found");
    assert!(
        out.status.success(),
        "git {:?} failed:\n{}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Create a local git repo with heimdal.yaml and one dotfile, commit it.
/// Returns the temp dir (keep alive for test duration).
fn setup_dotfiles_repo() -> TempDir {
    let repo = TempDir::new().unwrap();
    git(repo.path(), &["init"]);
    git(repo.path(), &["config", "user.email", "test@test.com"]);
    git(repo.path(), &["config", "user.name", "Test"]);

    repo.child("heimdal.yaml")
        .write_str(
            "heimdal:\n  version: \"1\"\nprofiles:\n  default:\n    dotfiles:\n      - .vimrc\n",
        )
        .unwrap();
    repo.child(".vimrc").write_str("\" vim config").unwrap();
    git(repo.path(), &["add", "."]);
    git(repo.path(), &["commit", "-m", "init"]);
    repo
}

/// Initialise heimdal in a fresh home with a local dotfiles repo already cloned.
/// Returns (home TempDir, dotfiles path).
fn setup_initialized_home() -> (TempDir, std::path::PathBuf) {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();

    // Populate dotfiles (simulate clone)
    let src_repo = setup_dotfiles_repo();
    // Copy files from src_repo into dotfiles dir
    for entry in std::fs::read_dir(src_repo.path()).unwrap() {
        let entry = entry.unwrap();
        let dest = dotfiles.path().join(entry.file_name());
        if entry.file_type().unwrap().is_file() {
            std::fs::copy(entry.path(), &dest).unwrap();
        }
    }
    // Init git in the dotfiles dir
    git(dotfiles.path(), &["init"]);
    git(dotfiles.path(), &["config", "user.email", "test@test.com"]);
    git(dotfiles.path(), &["config", "user.name", "Test"]);
    git(dotfiles.path(), &["add", "."]);
    git(dotfiles.path(), &["commit", "-m", "init"]);

    // Write state file
    let state_dir = home.path().join(".heimdal");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::write(
        state_dir.join("state.json"),
        serde_json::json!({
            "version": 1,
            "machine_id": "test-id",
            "hostname": "testhost",
            "username": "testuser",
            "os": "linux",
            "active_profile": "default",
            "dotfiles_path": dotfiles.path(),
            "repo_url": "file:///fake",
            "last_apply": null,
            "last_sync": null,
            "heimdal_version": "3.0.0"
        })
        .to_string(),
    )
    .unwrap();

    let dotfiles_path = dotfiles.path().to_owned();
    (home, dotfiles_path)
}

// ── status tests ─────────────────────────────────────────────────────────────

#[test]
fn test_status_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["status", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_status_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("status")
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("init").or(predicate::str::contains("initialized")));
}

#[test]
#[serial]
fn test_status_shows_profile_and_path() {
    let (home, _) = setup_initialized_home();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("status")
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("default").and(predicate::str::contains(".dotfiles")));
}

// ── diff tests ────────────────────────────────────────────────────────────────

#[test]
fn test_diff_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["diff", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_diff_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_diff_clean_repo_exits_zero() {
    let (home, _) = setup_initialized_home();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_diff_shows_changes() {
    let (home, dotfiles) = setup_initialized_home();
    std::fs::write(dotfiles.join(".vimrc"), "\" modified").unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("diff")
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(".vimrc").or(predicate::str::contains("modified")));
}

// ── commit tests ──────────────────────────────────────────────────────────────

#[test]
fn test_commit_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["commit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--message").or(predicate::str::contains("-m")));
}

#[test]
#[serial]
fn test_commit_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["commit", "-m", "test"])
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_commit_nothing_to_commit_exits_zero() {
    let (home, _) = setup_initialized_home();
    // Nothing changed — commit should succeed (possibly with "nothing to commit" msg)
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["commit", "-m", "test commit"])
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_commit_modified_file() {
    let (home, dotfiles) = setup_initialized_home();
    std::fs::write(dotfiles.join(".vimrc"), "\" updated config").unwrap();
    git(dotfiles.as_path(), &["add", ".vimrc"]);

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["commit", "-m", "update vimrc"])
        .env("HOME", home.path())
        .assert()
        .success();

    // Verify commit exists
    let log = process::Command::new("git")
        .args(&["log", "--oneline", "-1"])
        .current_dir(&dotfiles)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&log.stdout).contains("update vimrc"));
}

// ── rollback tests ────────────────────────────────────────────────────────────

#[test]
fn test_rollback_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["rollback", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_rollback_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("rollback")
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_rollback_dry_run() {
    let (home, dotfiles) = setup_initialized_home();
    // Add a second commit to have something to roll back to
    std::fs::write(dotfiles.join(".vimrc"), "\" v2").unwrap();
    git(dotfiles.as_path(), &["add", "."]);
    git(dotfiles.as_path(), &["commit", "-m", "v2"]);

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["rollback", "--dry-run"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("dry")
                .or(predicate::str::contains("HEAD~1"))
                .or(predicate::str::contains("Would")),
        );
}
