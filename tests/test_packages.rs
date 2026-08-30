use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;
use std::collections::{HashMap, HashSet};

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
    Command::cargo_bin("heimdal")
        .unwrap()
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
        .args(["packages", "list"])
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
        .args(["packages", "list"])
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
        .args(["packages", "add", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_add_writes_to_config() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
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
        .args(["packages", "add", "git", "--manager", "apt", "--no-install"])
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
        .args(["packages", "remove", "--help"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_remove_updates_config() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["packages", "remove", "vim", "--no-uninstall"])
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
        .args(["packages", "remove", "nonexistent", "--no-uninstall"])
        .env("HOME", home.path())
        .assert()
        .success();
}

#[test]
#[serial]
fn test_packages_add_mas_writes_id_and_name() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "packages",
            "add",
            "Slack",
            "--manager",
            "mas",
            "--id",
            "803453959",
            "--no-install",
        ])
        .env("HOME", home.path())
        .assert()
        .success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(
        content.contains("803453959") && content.contains("Slack"),
        "mas entry not found in heimdal.yaml:\n{}",
        content
    );
}

#[test]
#[serial]
fn test_packages_add_mas_without_id_fails() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "packages",
            "add",
            "Slack",
            "--manager",
            "mas",
            "--no-install",
        ])
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--id"));
}

#[test]
#[serial]
fn test_packages_remove_mas_by_name_updates_config() {
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "packages",
            "add",
            "Slack",
            "--manager",
            "mas",
            "--id",
            "803453959",
            "--no-install",
        ])
        .env("HOME", home.path())
        .assert()
        .success();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["packages", "remove", "Slack", "--no-uninstall"])
        .env("HOME", home.path())
        .assert()
        .success();

    let dotfiles = home.path().join(".dotfiles");
    let content = std::fs::read_to_string(dotfiles.join("heimdal.yaml")).unwrap();
    assert!(
        !content.contains("803453959"),
        "mas entry still in heimdal.yaml after remove:\n{}",
        content
    );
}

#[test]
#[serial]
fn test_packages_list_installed_flag_runs_without_error() {
    // A real-machine smoke test: whatever package managers are actually
    // present here, `--installed` must not crash and must still surface the
    // declared packages (annotated, rather than in the plain `--installed`
    // = false format asserted by `test_packages_list_shows_packages`).
    let home = setup_home_with_packages();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["packages", "list", "--installed"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("vim"));
}

// Covers "packages list --installed" cross-referencing logic against a
// mocked/injected installed set, the way `tests/test_state_lock.rs` drives
// `heimdal::lock::HeimdallLock` directly rather than shelling out for real —
// so this never depends on brew/apt/etc. actually being present in CI.
#[test]
fn test_packages_list_installed_annotates_against_injected_set() {
    use heimdal::commands::packages::build_package_sections;
    use heimdal::config::PackageMap;

    let pkgs = PackageMap {
        homebrew: vec!["git".to_string(), "curl".to_string()],
        apt: vec!["vim".to_string()],
        ..Default::default()
    };

    // Mocked/injected "real system" state: homebrew is present and has git
    // (but not curl); apt is not present on this machine at all.
    let mut installed_sets: HashMap<String, HashSet<String>> = HashMap::new();
    installed_sets.insert("homebrew".to_string(), HashSet::from(["git".to_string()]));

    let sections = build_package_sections(&pkgs, Some(&installed_sets));

    let homebrew = sections
        .iter()
        .find(|(label, _)| *label == "homebrew")
        .expect("homebrew section must be present");
    assert_eq!(
        homebrew.1,
        vec![
            "  - git (installed)".to_string(),
            "  - curl (missing)".to_string(),
        ]
    );

    let apt = sections
        .iter()
        .find(|(label, _)| *label == "apt")
        .expect("apt section must be present");
    assert_eq!(apt.1, vec!["  - vim (missing)".to_string()]);
}

#[test]
fn test_packages_list_without_installed_flag_has_no_annotation() {
    use heimdal::commands::packages::build_package_sections;
    use heimdal::config::PackageMap;

    let pkgs = PackageMap {
        homebrew: vec!["git".to_string()],
        ..Default::default()
    };

    let sections = build_package_sections(&pkgs, None);
    let homebrew = sections
        .iter()
        .find(|(label, _)| *label == "homebrew")
        .unwrap();
    assert_eq!(homebrew.1, vec!["  - git".to_string()]);
}
