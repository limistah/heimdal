/// Test `heimdal validate` command
///
/// Tests configuration validation functionality including:
/// - Help output
/// - Valid configuration
/// - Invalid configuration
/// - Missing configuration
/// - Custom config path
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;

const TEST_REPO: &str = "https://github.com/limistah/heimdal-dotfiles-test.git";

#[test]
fn test_validate_help() {
    cargo_bin_cmd!()
        .arg("validate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"))
        .stdout(predicate::str::contains("heimdal.yaml"));
}

#[test]
fn test_validate_missing_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Try to validate in directory without heimdal.yaml
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn test_validate_valid_config_from_repo() {
    let temp = assert_fs::TempDir::new().unwrap();

    // First initialize with test repo to get a valid config
    cargo_bin_cmd!()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .env("HOME", temp.path())
        .assert()
        .success();

    let dotfiles_dir = temp.child(".dotfiles");

    // Validate should succeed for valid config
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(dotfiles_dir.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("valid")
                .or(predicate::str::contains("✓"))
                .or(predicate::str::contains("success"))
                .or(predicate::str::contains("Valid")),
        );
}

#[test]
fn test_validate_invalid_yaml() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create invalid YAML
    let config_file = temp.child("heimdal.yaml");
    config_file
        .write_str("this is not: valid: yaml: syntax")
        .unwrap();

    // Validate should fail
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn test_validate_minimal_valid_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create minimal valid config
    let config_file = temp.child("heimdal.yaml");
    config_file
        .write_str(
            r#"
heimdal:
  version: "1.0"

profiles:
  default:
    description: "Minimal test profile"
    dotfiles:
      files: []
"#,
        )
        .unwrap();

    // Should validate successfully
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_validate_with_custom_path() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_dir = temp.child("custom-location");
    config_dir.create_dir_all().unwrap();

    // Create valid config in custom location
    let config_file = config_dir.child("heimdal.yaml");
    config_file
        .write_str(
            r#"
heimdal:
  version: "1.0"

profiles:
  default:
    description: "Custom path test"
    dotfiles:
      files: []
"#,
        )
        .unwrap();

    // Validate with custom config path
    cargo_bin_cmd!()
        .arg("validate")
        .arg("--config")
        .arg(config_file.path())
        .assert()
        .success();
}

#[test]
fn test_validate_empty_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create empty config file
    let config_file = temp.child("heimdal.yaml");
    config_file.write_str("").unwrap();

    // Should fail validation
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn test_validate_config_missing_profiles() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create config without profiles section
    let config_file = temp.child("heimdal.yaml");
    config_file
        .write_str(
            r#"
# Missing profiles section
some_other_key: value
"#,
        )
        .unwrap();

    // Should fail validation
    cargo_bin_cmd!()
        .arg("validate")
        .current_dir(temp.path())
        .assert()
        .failure();
}
