use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn write_yaml(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

// --- load_config tests ---

#[test]
fn test_minimal_config() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    assert_eq!(cfg.heimdal.version, "1");
    assert!(cfg.profiles.contains_key("default"));
}

#[test]
fn test_full_config_parses() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
  repo: "git@github.com:user/dotfiles.git"
profiles:
  default:
    packages:
      homebrew: [git, vim]
      apt: [git, vim]
    dotfiles:
      - .vimrc
      - source: .config/nvim
        target: ~/.config/nvim
    hooks:
      post_apply: ["echo done"]
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    assert_eq!(profile.packages.homebrew, vec!["git", "vim"]);
    assert_eq!(profile.dotfiles.len(), 2);
}

#[test]
fn test_simple_dotfile_string() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - .zshrc
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    match &profile.dotfiles[0] {
        heimdal::config::DotfileEntry::Simple(s) => assert_eq!(s, ".zshrc"),
        _ => panic!("expected Simple variant"),
    }
}

#[test]
fn test_full_dotfile_mapping() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - source: .config/nvim
        target: ~/.config/nvim
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    match &profile.dotfiles[0] {
        heimdal::config::DotfileEntry::Mapped(m) => {
            assert_eq!(m.source, ".config/nvim");
            assert_eq!(m.target, "~/.config/nvim");
        }
        _ => panic!("expected Mapped variant"),
    }
}

#[test]
fn test_dotfile_with_condition() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - source: .bashrc
        target: ~/.bashrc
        when:
          os: [linux]
          hostname: "work-*"
          profile: [default]
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    match &profile.dotfiles[0] {
        heimdal::config::DotfileEntry::Mapped(m) => {
            let cond = m.when.as_ref().unwrap();
            assert_eq!(cond.os, vec!["linux"]);
            assert_eq!(cond.hostname.as_deref(), Some("work-*"));
            assert_eq!(cond.profile, vec!["default"]);
        }
        _ => panic!("expected Mapped variant"),
    }
}

#[test]
fn test_hook_string_variant() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    hooks:
      post_apply: ["echo hello"]
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    assert_eq!(profile.hooks.post_apply.len(), 1);
    match &profile.hooks.post_apply[0] {
        heimdal::config::HookEntry::Simple(s) => assert_eq!(s, "echo hello"),
        _ => panic!("expected Simple hook"),
    }
}

#[test]
fn test_hook_object_variant() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    hooks:
      post_apply:
        - command: "echo done"
          os: [macos]
          fail_on_error: false
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let profile = cfg.profiles.get("default").unwrap();
    match &profile.hooks.post_apply[0] {
        heimdal::config::HookEntry::Full {
            command,
            os,
            fail_on_error,
            ..
        } => {
            assert_eq!(command, "echo done");
            assert_eq!(os, &vec!["macos"]);
            assert!(!fail_on_error);
        }
        _ => panic!("expected Full hook"),
    }
}

#[test]
fn test_profile_extends_resolves() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  base:
    dotfiles:
      - .vimrc
    packages:
      homebrew: [vim]
  child:
    extends: base
    dotfiles:
      - .zshrc
    packages:
      homebrew: [zsh]
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let resolved = heimdal::config::resolve_profile(&cfg, "child").unwrap();
    // child inherits .vimrc from base and adds .zshrc
    assert_eq!(resolved.dotfiles.len(), 2);
    // packages are merged
    assert!(resolved.packages.homebrew.contains(&"vim".to_string()));
    assert!(resolved.packages.homebrew.contains(&"zsh".to_string()));
}

#[test]
fn test_extends_cycle_detected() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  a:
    extends: b
  b:
    extends: a
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    assert!(heimdal::config::resolve_profile(&cfg, "a").is_err());
}

#[test]
fn test_extends_child_wins_on_conflict() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  base:
    hooks:
      post_apply: ["echo base"]
  child:
    extends: base
    hooks:
      post_apply: ["echo child"]
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let resolved = heimdal::config::resolve_profile(&cfg, "child").unwrap();
    // hooks are replaced by child, not merged
    assert_eq!(resolved.hooks.post_apply.len(), 1);
    match &resolved.hooks.post_apply[0] {
        heimdal::config::HookEntry::Simple(s) => assert_eq!(s, "echo child"),
        _ => panic!("expected Simple hook"),
    }
}

#[test]
fn test_validate_config_valid() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - .vimrc
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let errors = heimdal::config::validate_config(&cfg);
    assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
}

#[test]
fn test_validate_config_unknown_extends() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  child:
    extends: nonexistent
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let errors = heimdal::config::validate_config(&cfg);
    assert!(!errors.is_empty());
    assert!(errors[0].contains("nonexistent"));
}

#[test]
fn test_validate_config_circular_extends() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  a:
    extends: b
  b:
    extends: a
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let errors = heimdal::config::validate_config(&cfg);
    assert!(!errors.is_empty());
}

#[test]
fn test_validate_config_absolute_source_path() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - source: /etc/passwd
        target: ~/.passwd
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let errors = heimdal::config::validate_config(&cfg);
    assert!(!errors.is_empty());
    assert!(errors[0].contains("relative"));
}

#[test]
fn test_validate_config_dotdot_source_path() {
    let tmp = TempDir::new().unwrap();
    let path = write_yaml(
        &tmp,
        "heimdal.yaml",
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles:
      - source: ../../etc/passwd
        target: ~/.passwd
"#,
    );
    let cfg = heimdal::config::load_config(&path).unwrap();
    let errors = heimdal::config::validate_config(&cfg);
    assert!(!errors.is_empty());
}

#[test]
fn test_dotfile_entry_simple_source() {
    let entry = heimdal::config::DotfileEntry::Simple(".bashrc".to_string());
    assert_eq!(entry.source(), ".bashrc");
}

#[test]
fn test_dotfile_entry_simple_target() {
    let entry = heimdal::config::DotfileEntry::Simple(".bashrc".to_string());
    assert_eq!(entry.target(), "~/.bashrc");
}

#[test]
fn test_dotfile_entry_mapped_source() {
    let entry = heimdal::config::DotfileEntry::Mapped(heimdal::config::DotfileMapping {
        source: "config/nvim".to_string(),
        target: "~/.config/nvim".to_string(),
        when: None,
    });
    assert_eq!(entry.source(), "config/nvim");
}

#[test]
fn test_dotfile_entry_mapped_target() {
    let entry = heimdal::config::DotfileEntry::Mapped(heimdal::config::DotfileMapping {
        source: "config/nvim".to_string(),
        target: "~/.config/nvim".to_string(),
        when: None,
    });
    assert_eq!(entry.target(), "~/.config/nvim");
}

// --- CLI integration tests ---

#[test]
fn test_validate_valid_config_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let dotfiles = tmp.path().join(".dotfiles");
    std::fs::create_dir_all(&dotfiles).unwrap();
    let config_path = dotfiles.join("heimdal.yaml");
    fs::write(
        &config_path,
        r#"
heimdal:
  version: "1"
profiles:
  default:
    dotfiles: []
"#,
    )
    .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["validate", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_validate_invalid_config_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let dotfiles = tmp.path().join(".dotfiles");
    std::fs::create_dir_all(&dotfiles).unwrap();
    let config_path = dotfiles.join("heimdal.yaml");
    fs::write(
        &config_path,
        r#"
heimdal:
  version: "1"
profiles:
  child:
    extends: nonexistent
"#,
    )
    .unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["validate", "--config", config_path.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn test_validate_bad_yaml_exits_nonzero() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("heimdal.yaml");
    fs::write(&config_path, "this is: not: valid: yaml: :::").unwrap();

    Command::cargo_bin("heimdal")
        .unwrap()
        .args(&["validate", "--config", config_path.to_str().unwrap()])
        .assert()
        .failure();
}
