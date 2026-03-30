use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;

fn setup_home_multi_profile() -> TempDir {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();

    dotfiles.child("heimdal.yaml").write_str(r#"heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
  work:
    extends: default
    dotfiles: []
  personal:
    dotfiles: []
"#).unwrap();

    // Write state
    let state_dir = home.child(".heimdal");
    state_dir.create_dir_all().unwrap();
    state_dir.child("state.json").write_str(&serde_json::json!({
        "version": 1,
        "machine_id": "test-id",
        "hostname": "testhost",
        "username": "testuser",
        "os": "linux",
        "active_profile": "default",
        "dotfiles_path": dotfiles.path(),
        "repo_url": "https://example.com/dotfiles.git",
        "last_apply": null,
        "last_sync": null,
        "heimdal_version": "3.0.0"
    }).to_string()).unwrap();

    home
}

// ── profile list ─────────────────────────────────────────────────────────────

#[test]
fn test_profile_list_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "list", "--help"])
        .assert().success();
}

#[test]
#[serial]
fn test_profile_list_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "list"])
        .env("HOME", home.path())
        .assert().failure();
}

#[test]
#[serial]
fn test_profile_list_shows_all_profiles() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "list"])
        .env("HOME", home.path())
        .assert().success()
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("work"))
        .stdout(predicate::str::contains("personal"));
}

#[test]
#[serial]
fn test_profile_list_marks_active_profile() {
    let home = setup_home_multi_profile();
    let output = Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "list"])
        .env("HOME", home.path())
        .assert().success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&output);
    // The active profile (default) should be marked distinctly — e.g. with * or "(active)" or colored
    // We check that the output contains some marker near "default"
    assert!(
        text.contains("* default") || text.contains("default *") || text.contains("default (active)") || text.contains("→ default"),
        "Active profile 'default' not marked in output:\n{}", text
    );
}

// ── profile current ───────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_profile_current_shows_active() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "current"])
        .env("HOME", home.path())
        .assert().success()
        .stdout(predicate::str::contains("default"));
}

// ── profile switch ────────────────────────────────────────────────────────────

#[test]
fn test_profile_switch_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "switch", "--help"])
        .assert().success();
}

#[test]
#[serial]
fn test_profile_switch_updates_state() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "switch", "work"])
        .env("HOME", home.path())
        .assert().success();

    // Verify state file updated
    let state_path = home.path().join(".heimdal/state.json");
    let state: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&state_path).unwrap()
    ).unwrap();
    assert_eq!(state["active_profile"], "work");
}

#[test]
#[serial]
fn test_profile_switch_nonexistent_fails() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "switch", "nonexistent"])
        .env("HOME", home.path())
        .assert().failure()
        .stderr(predicate::str::contains("nonexistent").or(predicate::str::contains("not found")));
}

#[test]
#[serial]
fn test_profile_switch_same_profile_is_ok() {
    let home = setup_home_multi_profile();
    // Switching to the already-active profile should not fail
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "switch", "default"])
        .env("HOME", home.path())
        .assert().success();
}

// ── profile show ──────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_profile_show_displays_dotfiles() {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();
    dotfiles.child("heimdal.yaml").write_str(r#"heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - .vimrc
      - .zshrc
"#).unwrap();
    let state_dir = home.child(".heimdal");
    state_dir.create_dir_all().unwrap();
    state_dir.child("state.json").write_str(&serde_json::json!({
        "version": 1, "machine_id": "x", "hostname": "h", "username": "u",
        "os": "linux", "active_profile": "default",
        "dotfiles_path": dotfiles.path(),
        "repo_url": "", "last_apply": null, "last_sync": null,
        "heimdal_version": "3.0.0"
    }).to_string()).unwrap();

    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "show"])
        .env("HOME", home.path())
        .assert().success()
        .stdout(predicate::str::contains(".vimrc"));
}

// ── profile create ────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_profile_create_adds_to_config() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "create", "newprofile"])
        .env("HOME", home.path())
        .assert().success();

    // Verify newprofile appears in heimdal.yaml
    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(content.contains("newprofile"), "newprofile not found in heimdal.yaml:\n{}", content);
}

#[test]
#[serial]
fn test_profile_create_with_extends() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "create", "child", "--extends", "default"])
        .env("HOME", home.path())
        .assert().success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(content.contains("child"), "child profile not found");
    assert!(content.contains("extends"), "extends not written");
}

#[test]
#[serial]
fn test_profile_create_duplicate_fails() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "create", "default"])
        .env("HOME", home.path())
        .assert().failure()
        .stderr(predicate::str::contains("already exists").or(predicate::str::contains("exists")));
}

// ── profile clone ─────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_profile_clone_creates_copy() {
    let home = setup_home_multi_profile();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["profile", "clone", "default", "myclone"])
        .env("HOME", home.path())
        .assert().success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(content.contains("myclone"), "myclone not found in heimdal.yaml");
}
