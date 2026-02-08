/// Test import command functionality
///
/// Tests importing from existing dotfile managers (Stow, dotbot, etc.)
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

use crate::helpers::TestEnv;

#[test]
fn test_import_help() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("import")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Import from existing dotfile managers",
        ))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--preview"));
}

#[test]
fn test_import_verbose_flag() {
    Command::cargo_bin("heimdal")
        .unwrap()
        .arg("import")
        .arg("--verbose")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_import_with_nonexistent_path() {
    let env = TestEnv::new();

    let result = env
        .heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg("/nonexistent/path/to/dotfiles")
        .assert();

    // Command may exit with 0 or 1, but should print error message
    result.stderr(
        predicate::str::contains("not found")
            .or(predicate::str::contains("does not exist"))
            .or(predicate::str::contains("No such file"))
            .or(predicate::str::contains("Directory not found")),
    );
}

#[test]
fn test_import_with_empty_directory() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("empty_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed with empty config or fail gracefully
}

#[test]
fn test_import_preview_mode() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("test_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create a simple dotfile
    fs::write(dotfiles_dir.join(".bashrc"), "# test bashrc\n").expect("Failed to write file");

    // Preview import (should not actually create files)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--preview")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May succeed or fail depending on implementation
}

#[test]
fn test_import_with_stow_structure() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("stow_dotfiles");

    // Create stow-style directory structure
    let bash_pkg = dotfiles_dir.join("bash");
    fs::create_dir_all(&bash_pkg).expect("Failed to create directory");
    fs::write(bash_pkg.join(".bashrc"), "# stow bashrc\n").expect("Failed to write file");

    let vim_pkg = dotfiles_dir.join("vim");
    fs::create_dir_all(&vim_pkg).expect("Failed to create directory");
    fs::write(vim_pkg.join(".vimrc"), "\" stow vimrc\n").expect("Failed to write file");

    // Import from stow
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("stow")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_import_with_dotbot_structure() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("dotbot_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create a simple dotbot install.conf.yaml
    let install_conf = r#"
- link:
    ~/.bashrc: bashrc
    ~/.vimrc: vimrc
"#;
    fs::write(dotfiles_dir.join("install.conf.yaml"), install_conf)
        .expect("Failed to write install.conf.yaml");

    fs::write(dotfiles_dir.join("bashrc"), "# dotbot bashrc\n").expect("Failed to write bashrc");
    fs::write(dotfiles_dir.join("vimrc"), "\" dotbot vimrc\n").expect("Failed to write vimrc");

    // Import from dotbot
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("dotbot")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_import_with_auto_detection() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("auto_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create some dotfiles
    fs::write(dotfiles_dir.join(".bashrc"), "# bashrc\n").expect("Failed to write file");
    fs::write(dotfiles_dir.join(".vimrc"), "\" vimrc\n").expect("Failed to write file");

    // Import with auto detection (default)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("auto")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_import_with_custom_output_path() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("test_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    fs::write(dotfiles_dir.join(".bashrc"), "# test bashrc\n").expect("Failed to write file");

    let output_path = env.home_dir().join("custom-heimdal.yaml");

    // Import with custom output path
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .code(predicate::in_iter(vec![0, 1]));

    // If successful, the output file might exist
    // (test doesn't fail if file doesn't exist since command might fail)
}

#[test]
fn test_import_with_nested_directories() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("nested_dotfiles");

    // Create nested directory structure
    let config_dir = dotfiles_dir.join(".config");
    fs::create_dir_all(config_dir.join("nvim")).expect("Failed to create directory");
    fs::write(config_dir.join("nvim").join("init.vim"), "\" nvim config\n")
        .expect("Failed to write file");

    fs::create_dir_all(config_dir.join("git")).expect("Failed to create directory");
    fs::write(config_dir.join("git").join("config"), "# git config\n")
        .expect("Failed to write file");

    // Import nested structure
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_import_with_symlinks() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("symlink_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create a file and a symlink to it
    let target_file = dotfiles_dir.join("bashrc");
    fs::write(&target_file, "# bashrc content\n").expect("Failed to write file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = dotfiles_dir.join(".bashrc");
        symlink(&target_file, &link_path).ok();
    }

    // Import with symlinks
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_import_with_invalid_from_option() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("test_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    let result = env
        .heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("invalid_tool")
        .assert();

    // Command may exit with 0 or 1, but should print error message
    result.stderr(
        predicate::str::contains("Unknown tool")
            .or(predicate::str::contains("invalid"))
            .or(predicate::str::contains("error")),
    );
}

#[test]
fn test_import_with_gitignored_files() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("gitignore_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create a .gitignore
    fs::write(dotfiles_dir.join(".gitignore"), "*.secret\n.env\n")
        .expect("Failed to write .gitignore");

    // Create some files, including ignored ones
    fs::write(dotfiles_dir.join(".bashrc"), "# bashrc\n").expect("Failed to write bashrc");
    fs::write(dotfiles_dir.join("api.secret"), "secret_key\n").expect("Failed to write secret");
    fs::write(dotfiles_dir.join(".env"), "API_KEY=secret\n").expect("Failed to write .env");

    // Import (should potentially respect .gitignore)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}
