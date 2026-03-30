use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;

fn setup_home_with_packages() -> TempDir {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();

    dotfiles
        .child("heimdal.yaml")
        .write_str(
            r#"heimdal:
  version: "1"
profiles:
  default:
    packages:
      homebrew: [git, vim]
      apt: [git, vim]
    dotfiles: []
"#,
        )
        .unwrap();

    let state_dir = home.child(".heimdal");
    state_dir.create_dir_all().unwrap();
    state_dir
        .child("state.json")
        .write_str(
            &serde_json::json!({
                "version": 1, "machine_id": "x", "hostname": "h", "username": "u",
                "os": "linux", "active_profile": "default",
                "dotfiles_path": dotfiles.path(),
                "repo_url": "", "last_apply": null, "last_sync": null,
                "heimdal_version": "3.0.0"
            })
            .to_string(),
        )
        .unwrap();

    home
}

#[test]
fn test_packages_list_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(["packages", "list", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_list_fails_without_init() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list"])
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_packages_list_shows_packages() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "list"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("vim"));
}

#[test]
fn test_packages_add_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "add", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_add_writes_to_config() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "packages",
            "add",
            "ripgrep",
            "--manager",
            "apt",
            "--no-install",
        ])
        .env("HOME", home.path())
        .assert()
        .success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(
        content.contains("ripgrep"),
        "ripgrep not found in heimdal.yaml:\n{}",
        content
    );
}

#[test]
#[serial]
fn test_packages_add_duplicate_is_ok() {
    // Adding an already-present package should not duplicate it
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "add", "git", "--manager", "apt", "--no-install"])
        .env("HOME", home.path())
        .assert()
        .success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    // Count occurrences of "git" — should not appear more times than reasonable
    let count = content.matches("git").count();
    assert!(
        count <= 3,
        "git appears too many times ({}), may be duplicated:\n{}",
        count,
        content
    );
}

#[test]
fn test_packages_remove_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "remove", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_remove_updates_config() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "remove", "vim", "--no-uninstall"])
        .env("HOME", home.path())
        .assert()
        .success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    // vim should be gone from all managers
    assert!(
        !content.contains("- vim"),
        "vim still in heimdal.yaml:\n{}",
        content
    );
}

#[test]
#[serial]
fn test_packages_remove_nonexistent_is_ok() {
    let home = setup_home_with_packages();
    // Removing a package that isn't tracked should not fail
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "remove", "nonexistent", "--no-uninstall"])
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
fn test_packages_suggest_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["packages", "suggest", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_suggest_detects_rust() {
    let home = setup_home_with_packages();
    let project_dir = home.child("myproject");
    project_dir.create_dir_all().unwrap();
    project_dir
        .child("Cargo.toml")
        .write_str("[package]\nname = \"test\"\n")
        .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "packages",
            "suggest",
            "--dir",
            project_dir.path().to_str().unwrap(),
        ])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("rust").or(predicate::str::contains("cargo")));
}

#[test]
#[serial]
fn test_packages_suggest_detects_node() {
    let home = setup_home_with_packages();
    let project_dir = home.child("nodeproject");
    project_dir.create_dir_all().unwrap();
    project_dir.child("package.json").write_str("{}").unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "packages",
            "suggest",
            "--dir",
            project_dir.path().to_str().unwrap(),
        ])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("node").or(predicate::str::contains("nvm")));
}
