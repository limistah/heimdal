use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::serial;

fn setup_home_with_template() -> TempDir {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();

    dotfiles.child("heimdal.yaml").write_str(r#"heimdal:
  version: "1"
profiles:
  default:
    templates:
      - src: .gitconfig.tmpl
        dest: ~/.gitconfig
        vars:
          name: "Test User"
          email: "test@example.com"
    dotfiles: []
"#).unwrap();

    dotfiles.child(".gitconfig.tmpl").write_str(
        "[user]\n    name = {{ name }}\n    email = {{ email }}\n"
    ).unwrap();

    let state_dir = home.child(".heimdal");
    state_dir.create_dir_all().unwrap();
    state_dir.child("state.json").write_str(&serde_json::json!({
        "version": 1, "machine_id": "x", "hostname": "testhost", "username": "testuser",
        "os": "linux", "active_profile": "default",
        "dotfiles_path": dotfiles.path(),
        "repo_url": "", "last_apply": null, "last_sync": null,
        "heimdal_version": "3.0.0"
    }).to_string()).unwrap();

    home
}

#[test]
fn test_template_list_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["template", "list", "--help"])
        .assert().success();
}

#[test]
fn test_template_preview_help() {
    Command::cargo_bin("heimdal").unwrap()
        .args(&["template", "preview", "--help"])
        .assert().success();
}

#[test]
#[serial]
fn test_template_list_shows_templates() {
    let home = setup_home_with_template();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["template", "list"])
        .env("HOME", home.path())
        .assert().success()
        .stdout(predicate::str::contains(".gitconfig.tmpl"));
}

#[test]
#[serial]
fn test_template_preview_renders_vars() {
    let home = setup_home_with_template();
    Command::cargo_bin("heimdal").unwrap()
        .args(&["template", "preview", ".gitconfig.tmpl"])
        .env("HOME", home.path())
        .assert().success()
        .stdout(predicate::str::contains("Test User"))
        .stdout(predicate::str::contains("test@example.com"));
}

#[test]
fn test_template_render_substitutes_variables() {
    let content = "Hello, {{ name }}! You are on {{ os }}.";
    let mut vars = std::collections::HashMap::new();
    vars.insert("name".to_string(), "World".to_string());
    vars.insert("os".to_string(), "linux".to_string());
    let result = heimdal::templates::render_string(content, &vars);
    assert_eq!(result, "Hello, World! You are on linux.");
}

#[test]
fn test_template_undefined_var_preserved() {
    // Undefined variables are preserved as-is (not replaced with empty string)
    let content = "value = {{ undefined_var }}";
    let vars = std::collections::HashMap::new();
    let result = heimdal::templates::render_string(content, &vars);
    assert!(result.contains("undefined_var"), "Expected placeholder to be preserved, got: {}", result);
}

#[test]
fn test_template_system_vars_available() {
    let sys_vars = heimdal::templates::system_vars();
    assert!(sys_vars.contains_key("hostname"), "missing hostname");
    assert!(sys_vars.contains_key("username"), "missing username");
    assert!(sys_vars.contains_key("os"), "missing os");
    assert!(sys_vars.contains_key("home"), "missing home");
}
