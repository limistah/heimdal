use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use predicates::str::contains;
use serial_test::serial;

mod common;

#[test]
fn test_apply_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--help"])
        .assert()
        .success()
        .stdout(contains("dry-run"))
        .stdout(contains("force"))
        .stdout(contains("backup"));
}

#[test]
#[serial]
fn test_apply_fails_without_init() {
    let home = assert_fs::TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(contains("init").or(contains("initialized")));
}

#[test]
#[serial]
fn test_apply_dry_run_creates_no_files() {
    let home = common::setup_home("default");
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dry-run"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(
            contains("Dry-run")
                .or(contains("dry-run"))
                .or(contains("preview")),
        );
    assert!(
        !home.path().join(".vimrc").exists(),
        ".vimrc must NOT exist after dry-run"
    );
}

#[test]
#[serial]
fn test_apply_creates_symlinks() {
    let home = common::setup_home("default");
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .success();
    let link = home.path().join(".vimrc");
    assert!(link.is_symlink(), ".vimrc must be a symlink");
    let target = std::fs::read_link(&link).unwrap();
    assert!(
        target.to_string_lossy().contains(".vimrc"),
        "symlink must point to source .vimrc"
    );
}

#[test]
#[serial]
fn test_apply_idempotent() {
    let home = common::setup_home("default");
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .success();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_apply_force_overwrites_conflict() {
    let home = common::setup_home("default");
    std::fs::write(home.path().join(".vimrc"), "existing content").unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--force"])
        .env("HOME", home.path())
        .assert()
        .success();
    assert!(
        home.path().join(".vimrc").is_symlink(),
        ".vimrc must be a symlink after --force"
    );
}

#[test]
#[serial]
fn test_apply_conflict_without_force_fails() {
    let home = common::setup_home("default");
    std::fs::write(home.path().join(".vimrc"), "existing content").unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_apply_backup_preserves_original() {
    let home = common::setup_home("default");
    std::fs::write(home.path().join(".vimrc"), "original content").unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--backup"])
        .env("HOME", home.path())
        .assert()
        .success();
    assert!(
        home.path().join(".vimrc").is_symlink(),
        ".vimrc must be a symlink after --backup"
    );
    let backup_dir = home
        .path()
        .join(".dotfiles")
        .join(".heimdal")
        .join("backups");
    assert!(backup_dir.exists(), "backup directory must exist");
}

#[test]
#[serial]
fn test_apply_dotfiles_only_flag() {
    let home = common::setup_home("default");
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["apply", "--dotfiles-only"])
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_apply_condition_os_filter() {
    // Dotfile with when.os = [windows] — must be skipped on linux/macos
    let home = assert_fs::TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();
    let heimdal_dir = home.child(".heimdal");
    heimdal_dir.create_dir_all().unwrap();

    heimdal_dir
        .child("state.json")
        .write_str(&format!(
            r#"{{
        "version": 1, "machine_id": "test-id", "hostname": "testhost",
        "username": "testuser", "os": "linux", "active_profile": "default",
        "dotfiles_path": "{}", "repo_url": "https://example.com",
        "last_apply": null, "last_sync": null, "heimdal_version": "3.0.0"
    }}"#,
            dotfiles.path().display()
        ))
        .unwrap();

    dotfiles
        .child("heimdal.yaml")
        .write_str(
            r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - source: .vimrc
        target: ~/.vimrc
        when:
          os: [windows]
"#,
        )
        .unwrap();
    dotfiles.child(".vimrc").write_str("\" vim config").unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("apply")
        .env("HOME", home.path())
        .assert()
        .success();
    assert!(
        !home.path().join(".vimrc").exists(),
        ".vimrc must NOT be linked (os filter)"
    );
}
