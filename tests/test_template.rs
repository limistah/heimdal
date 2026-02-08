/// Test template command functionality
///
/// Tests template preview, list, and variables subcommands
#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

use crate::helpers::{TestEnv, TEST_REPO};

#[test]
fn test_template_help() {
    cargo_bin_cmd!()
        
        .arg("template")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage templates"))
        .stdout(predicate::str::contains("preview"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("variables"));
}

#[test]
fn test_template_verbose_flag() {
    cargo_bin_cmd!()
        
        .arg("template")
        .arg("--verbose")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_template_list_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("template")
        .arg("list")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_template_variables_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("template")
        .arg("variables")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_template_preview_without_init() {
    let env = TestEnv::new();

    env.heimdal_cmd()
        .arg("template")
        .arg("preview")
        .arg("nonexistent.tmpl")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("no heimdal"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_template_list_after_init() {
    let env = TestEnv::new();

    // Initialize with test profile
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // List templates (should succeed even if empty)
    env.heimdal_cmd()
        .arg("template")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_template_variables_after_init() {
    let env = TestEnv::new();

    // Initialize with test profile
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Show variables (should show default variables)
    env.heimdal_cmd()
        .arg("template")
        .arg("variables")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("home")
                .or(predicate::str::contains("user"))
                .or(predicate::str::contains("hostname"))
                .or(predicate::str::contains("No variables")),
        );
}

#[test]
fn test_template_preview_nonexistent_file() {
    let env = TestEnv::new();

    // Initialize with test profile
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Try to preview nonexistent template
    env.heimdal_cmd()
        .arg("template")
        .arg("preview")
        .arg("nonexistent.tmpl")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("error")),
        );
}

#[test]
fn test_template_preview_with_template_file() {
    let env = TestEnv::new();

    // Initialize with test profile
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Create a simple template file
    let dotfiles_dir = env.home_dir().join(".dotfiles");
    let template_file = dotfiles_dir.join("test.tmpl");
    fs::write(&template_file, "Hello {{ user }}!\nHome: {{ home }}\n")
        .expect("Failed to write template");

    // Preview the template
    let output = env
        .heimdal_cmd()
        .arg("template")
        .arg("preview")
        .arg("test.tmpl")
        .assert()
        .success();

    // Should contain rendered output
    output.stdout(
        predicate::str::contains("Hello")
            .and(predicate::str::contains("Home:"))
            .or(predicate::str::contains("user"))
            .or(predicate::str::contains("home")),
    );
}

#[test]
fn test_template_list_with_template_files() {
    let env = TestEnv::new();

    // Initialize with test profile
    env.heimdal_cmd()
        .arg("init")
        .arg("--repo")
        .arg(TEST_REPO)
        .arg("--profile")
        .arg("test")
        .assert()
        .success();

    // Create template files
    let dotfiles_dir = env.home_dir().join(".dotfiles");
    fs::write(dotfiles_dir.join("config.tmpl"), "{{ user }}").expect("Failed to write template");
    fs::write(dotfiles_dir.join("bashrc.tmpl"), "{{ home }}").expect("Failed to write template");

    // List templates
    let output = env
        .heimdal_cmd()
        .arg("template")
        .arg("list")
        .assert()
        .success();

    // Should list the template files
    output.stdout(
        predicate::str::contains("config.tmpl")
            .or(predicate::str::contains("bashrc.tmpl"))
            .or(predicate::str::contains(".tmpl"))
            .or(predicate::str::contains("No templates")),
    );
}

#[test]
fn test_template_preview_help() {
    cargo_bin_cmd!()
        
        .arg("template")
        .arg("preview")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Preview how a template will be rendered",
        ));
}

#[test]
fn test_template_list_help() {
    cargo_bin_cmd!()
        
        .arg("template")
        .arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("List all template files"));
}

#[test]
fn test_template_variables_help() {
    cargo_bin_cmd!()
        
        .arg("template")
        .arg("variables")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Show all available variables"));
}
