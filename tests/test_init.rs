use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;
use std::process;

fn git_init_local_repo(dir: &std::path::Path) {
    // Create a local git repo with a valid heimdal.yaml
    let out = process::Command::new("git")
        .args(["init", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    for args in &[
        vec![
            "-C",
            dir.to_str().unwrap(),
            "config",
            "user.email",
            "test@test.com",
        ],
        vec!["-C", dir.to_str().unwrap(), "config", "user.name", "Test"],
    ] {
        let out = process::Command::new("git").args(args).output().unwrap();
        assert!(
            out.status.success(),
            "git config failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let yaml = r#"heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
  work:
    dotfiles: []
"#;
    std::fs::write(dir.join("heimdal.yaml"), yaml).unwrap();

    for args in &[
        vec!["-C", dir.to_str().unwrap(), "add", "."],
        vec!["-C", dir.to_str().unwrap(), "commit", "-m", "init"],
    ] {
        let out = process::Command::new("git").args(args).output().unwrap();
        assert!(
            out.status.success(),
            "git command {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn test_init_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--repo"))
        .stdout(predicate::str::contains("--profile"));
}

#[test]
#[serial]
fn test_init_requires_repo() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["init", "--profile", "default"])
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_requires_profile() {
    let home = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["init", "--repo", "https://example.com/dotfiles.git"])
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_no_clone_creates_state() {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();

    // Write a valid heimdal.yaml in the dotfiles dir
    dotfiles
        .child("heimdal.yaml")
        .write_str(
            r#"heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
"#,
        )
        .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "init",
            "--repo",
            "https://example.com/dotfiles.git",
            "--profile",
            "default",
            "--no-clone",
        ])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    // State file must exist
    let state_path = home.path().join(".heimdal/state.json");
    assert!(state_path.exists(), "state.json must be created");

    // State file must be valid JSON with correct fields
    let state_str = std::fs::read_to_string(&state_path).unwrap();
    let state: serde_json::Value = serde_json::from_str(&state_str).unwrap();
    assert_eq!(state["active_profile"], "default");
    assert_eq!(state["repo_url"], "https://example.com/dotfiles.git");
    assert_eq!(state["version"], 1);
}

#[test]
#[serial]
fn test_init_no_clone_unknown_profile_fails() {
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
    dotfiles: []
"#,
        )
        .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "init",
            "--repo",
            "https://example.com/dotfiles.git",
            "--profile",
            "nonexistent",
            "--no-clone",
        ])
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent").or(predicate::str::contains("not found")));
}

#[test]
#[serial]
fn test_init_no_clone_missing_config_fails() {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();
    // No heimdal.yaml written

    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "init",
            "--repo",
            "https://example.com/dotfiles.git",
            "--profile",
            "default",
            "--no-clone",
        ])
        .env("HOME", home.path())
        .assert()
        .failure();
}

#[test]
#[serial]
fn test_init_clone_from_local_repo() {
    let home = TempDir::new().unwrap();

    // Create a local bare-ish git repo to clone from
    let source_dir = TempDir::new().unwrap();
    git_init_local_repo(source_dir.path());

    let repo_url = format!("file://{}", source_dir.path().display());

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(["init", "--repo", &repo_url, "--profile", "default"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized").or(predicate::str::contains("Cloned")));

    let state_path = home.path().join(".heimdal/state.json");
    assert!(
        state_path.exists(),
        "state.json must be created after clone"
    );

    let state_str = std::fs::read_to_string(&state_path).unwrap();
    let state: serde_json::Value = serde_json::from_str(&state_str).unwrap();
    assert_eq!(state["active_profile"], "default");
}

#[test]
#[serial]
fn test_init_custom_path() {
    let home = TempDir::new().unwrap();
    let custom_dotfiles = home.child("my-dotfiles");
    custom_dotfiles.create_dir_all().unwrap();
    custom_dotfiles
        .child("heimdal.yaml")
        .write_str(
            r#"heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
"#,
        )
        .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "init",
            "--repo",
            "https://example.com/dotfiles.git",
            "--profile",
            "default",
            "--path",
            custom_dotfiles.path().to_str().unwrap(),
            "--no-clone",
        ])
        .env("HOME", home.path())
        .assert()
        .success();

    // State must always be at ~/.heimdal/state.json regardless of --path
    let state_path = home.path().join(".heimdal/state.json");
    assert!(state_path.exists(), "state.json must be at ~/.heimdal/state.json");
}

#[test]
#[serial]
fn test_init_shows_next_steps() {
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
    dotfiles: []
"#,
        )
        .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args([
            "init",
            "--repo",
            "https://example.com/dotfiles.git",
            "--profile",
            "default",
            "--no-clone",
        ])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("apply").or(predicate::str::contains("next")));
}
