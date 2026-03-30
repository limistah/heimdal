use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;

// ── helper ────────────────────────────────────────────────────────────────────

fn stow_dotfiles() -> TempDir {
    let dir = TempDir::new().unwrap();
    // .stowrc signals Stow
    dir.child(".stowrc").write_str("--target=$HOME\n").unwrap();
    dir.child(".vimrc").write_str("\" vim").unwrap();
    dir.child(".zshrc").write_str("# zsh").unwrap();
    dir.child(".gitconfig").write_str("[core]").unwrap();
    dir
}

fn dotbot_dotfiles() -> TempDir {
    let dir = TempDir::new().unwrap();
    dir.child("install.conf.yaml")
        .write_str(
            r#"
- link:
    ~/.vimrc: .vimrc
    ~/.zshrc: .zshrc
"#,
        )
        .unwrap();
    dir.child(".vimrc").write_str("\" vim").unwrap();
    dir.child(".zshrc").write_str("# zsh").unwrap();
    dir
}

// ── detection tests ───────────────────────────────────────────────────────────

#[test]
fn test_import_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--preview"));
}

#[test]
fn test_import_detects_stow() {
    let dir = stow_dotfiles();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "stow",
            "--preview",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("stow").or(predicate::str::contains("Stow")));
}

#[test]
fn test_import_detects_dotbot() {
    let dir = dotbot_dotfiles();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "dotbot",
            "--preview",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(".vimrc").or(predicate::str::contains("vimrc")));
}

#[test]
fn test_import_auto_detects_stow_from_stowrc() {
    let dir = stow_dotfiles();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--preview",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("stow")
                .or(predicate::str::contains("Stow"))
                .or(predicate::str::contains(".vimrc")),
        );
}

#[test]
fn test_import_stow_produces_valid_yaml() {
    let dir = stow_dotfiles();
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().join("heimdal.yaml");

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "stow",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Output file must exist and contain valid YAML with profile + dotfiles
    assert!(output_path.exists(), "output heimdal.yaml not written");
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(
        content.contains("profiles"),
        "no profiles section:\n{}",
        content
    );
    assert!(
        content.contains("dotfiles") || content.contains("vimrc"),
        "no dotfiles in output:\n{}",
        content
    );
}

#[test]
fn test_import_dotbot_produces_valid_yaml() {
    let dir = dotbot_dotfiles();
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().join("heimdal.yaml");

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "dotbot",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("profiles"));
}

#[test]
fn test_import_preview_does_not_write_file() {
    let dir = stow_dotfiles();
    let output_dir = TempDir::new().unwrap();
    let output_path = output_dir.path().join("heimdal.yaml");

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "stow",
            "--output",
            output_path.to_str().unwrap(),
            "--preview",
        ])
        .assert()
        .success();

    // With --preview, the file must NOT be written
    assert!(!output_path.exists(), "preview should not write the file");
}

#[test]
fn test_import_unknown_tool_fails() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&[
            "import",
            "--path",
            dir.path().to_str().unwrap(),
            "--from",
            "totally_unknown_tool",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("totally_unknown_tool")
                .or(predicate::str::contains("unknown")),
        );
}
