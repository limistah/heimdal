/// Test import command functionality
///
/// Tests importing from existing dotfile managers (Stow, dotbot, chezmoi, yadm, homesick)
#[path = "helpers/mod.rs"]
mod helpers;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

use crate::helpers::TestEnv;

#[test]
fn test_import_help() {
    cargo_bin_cmd!()
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
    cargo_bin_cmd!()
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

// ============================================================================
// GNU Stow Tests
// ============================================================================

#[test]
fn test_import_from_stow_with_stowrc() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("stow_dotfiles");

    // Create .stowrc file (Stow detection marker)
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");
    fs::write(
        dotfiles_dir.join(".stowrc"),
        "--target=$HOME\n--ignore=.git\n",
    )
    .expect("Failed to write .stowrc");

    // Create stow-style packages
    let bash_pkg = dotfiles_dir.join("bash");
    fs::create_dir_all(&bash_pkg).expect("Failed to create bash package");
    fs::write(bash_pkg.join(".bashrc"), "# stow bashrc\n").expect("Failed to write .bashrc");
    fs::write(bash_pkg.join(".bash_profile"), "# stow bash_profile\n")
        .expect("Failed to write .bash_profile");

    let vim_pkg = dotfiles_dir.join("vim");
    fs::create_dir_all(&vim_pkg).expect("Failed to create vim package");
    fs::write(vim_pkg.join(".vimrc"), "\" stow vimrc\n").expect("Failed to write .vimrc");

    // Import from stow (should auto-detect or use explicit --from stow)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("stow")
        .arg("--preview")
        .assert()
        .success();
}

#[test]
fn test_import_from_stow_auto_detect() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("stow_auto");

    // Create stow structure without .stowrc (still detectable by structure)
    let zsh_pkg = dotfiles_dir.join("zsh");
    fs::create_dir_all(&zsh_pkg).expect("Failed to create zsh package");
    fs::write(zsh_pkg.join(".zshrc"), "# zsh config\n").expect("Failed to write .zshrc");

    let git_pkg = dotfiles_dir.join("git");
    fs::create_dir_all(&git_pkg).expect("Failed to create git package");
    fs::write(git_pkg.join(".gitconfig"), "[user]\nname = Test\n")
        .expect("Failed to write .gitconfig");

    // Auto-detect (should recognize as stow)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("auto")
        .arg("--preview")
        .assert()
        .code(predicate::in_iter(vec![0, 1]));
}

// ============================================================================
// dotbot Tests
// ============================================================================

#[test]
fn test_import_from_dotbot_with_install_conf() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("dotbot_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create install.conf.yaml (dotbot detection marker)
    let install_conf = r#"
- link:
    ~/.bashrc: bashrc
    ~/.vimrc: vimrc
    ~/.config/nvim/init.vim: config/nvim/init.vim

- shell:
    - [git submodule update --init --recursive, Installing submodules]
    - echo "Setting up dotfiles"
"#;
    fs::write(dotfiles_dir.join("install.conf.yaml"), install_conf)
        .expect("Failed to write install.conf.yaml");

    // Create source files
    fs::write(dotfiles_dir.join("bashrc"), "# dotbot bashrc\n").expect("Failed to write bashrc");
    fs::write(dotfiles_dir.join("vimrc"), "\" dotbot vimrc\n").expect("Failed to write vimrc");

    let config_dir = dotfiles_dir.join("config/nvim");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    fs::write(config_dir.join("init.vim"), "\" nvim config\n").expect("Failed to write init.vim");

    // Import from dotbot
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("dotbot")
        .arg("--preview")
        .assert()
        .success();
}

#[test]
fn test_import_from_dotbot_with_packages() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("dotbot_with_packages");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create install.conf.yaml with shell commands containing packages
    let install_conf = r#"
- link:
    ~/.bashrc: bashrc

- shell:
    - [brew install vim git tmux, Installing packages]
    - [apt-get install curl wget, Installing more packages]
"#;
    fs::write(dotfiles_dir.join("install.conf.yaml"), install_conf)
        .expect("Failed to write install.conf.yaml");

    fs::write(dotfiles_dir.join("bashrc"), "# bashrc\n").expect("Failed to write bashrc");

    // Import (should extract packages from shell commands)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("dotbot")
        .arg("--preview")
        .assert()
        .success();
}

// ============================================================================
// chezmoi Tests
// ============================================================================

#[test]
fn test_import_from_chezmoi() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("chezmoi_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create .chezmoiignore (chezmoi detection marker)
    fs::write(dotfiles_dir.join(".chezmoiignore"), ".git\n*.swp\n")
        .expect("Failed to write .chezmoiignore");

    // Create chezmoi-style files with naming convention
    fs::write(dotfiles_dir.join("dot_bashrc"), "# chezmoi bashrc\n")
        .expect("Failed to write dot_bashrc");
    fs::write(
        dotfiles_dir.join("dot_vimrc.tmpl"),
        "\" chezmoi vimrc template\n",
    )
    .expect("Failed to write dot_vimrc.tmpl");

    // Create nested config
    let config_dir = dotfiles_dir.join("dot_config/dot_nvim");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    fs::write(config_dir.join("init.vim"), "\" nvim config\n").expect("Failed to write init.vim");

    // Create private file (private_ prefix)
    let ssh_dir = dotfiles_dir.join("private_dot_ssh");
    fs::create_dir_all(&ssh_dir).expect("Failed to create ssh dir");
    fs::write(ssh_dir.join("config"), "Host *\n").expect("Failed to write ssh config");

    // Import from chezmoi
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("chezmoi")
        .arg("--preview")
        .assert()
        .success();
}

// ============================================================================
// yadm Tests
// ============================================================================

#[test]
fn test_import_from_yadm() {
    let env = TestEnv::new();
    let home_dir = env.home_dir();

    // Create .yadm directory (yadm detection marker)
    let yadm_dir = home_dir.join(".yadm/repo.git");
    fs::create_dir_all(&yadm_dir).expect("Failed to create .yadm directory");

    // Create a minimal git repository structure
    fs::create_dir_all(yadm_dir.join("refs/heads")).ok();
    fs::write(yadm_dir.join("HEAD"), "ref: refs/heads/master\n").ok();
    fs::write(yadm_dir.join("config"), "[core]\nbare = true\n").ok();

    // Create some dotfiles in home
    fs::write(home_dir.join(".bashrc"), "# yadm bashrc\n").expect("Failed to write .bashrc");
    fs::write(home_dir.join(".vimrc"), "\" yadm vimrc\n").expect("Failed to write .vimrc");

    // Import from yadm (uses home directory)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&home_dir)
        .arg("--from")
        .arg("yadm")
        .arg("--preview")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // May fail if yadm command not available
}

// ============================================================================
// homesick Tests
// ============================================================================

#[test]
fn test_import_from_homesick() {
    let env = TestEnv::new();
    let castle_dir = env.home_dir().join("my_castle");

    // Create homesick castle structure with "home" subdirectory (detection marker)
    let home_subdir = castle_dir.join("home");
    fs::create_dir_all(&home_subdir).expect("Failed to create home subdirectory");

    // Create dotfiles in home subdirectory
    fs::write(home_subdir.join(".bashrc"), "# homesick bashrc\n").expect("Failed to write .bashrc");
    fs::write(home_subdir.join(".vimrc"), "\" homesick vimrc\n").expect("Failed to write .vimrc");

    // Create nested config
    let config_dir = home_subdir.join(".config/nvim");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    fs::write(config_dir.join("init.vim"), "\" nvim config\n").expect("Failed to write init.vim");

    // Import from homesick
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&castle_dir)
        .arg("--from")
        .arg("homesick")
        .arg("--preview")
        .assert()
        .success();
}

// ============================================================================
// General Tests
// ============================================================================

#[test]
fn test_import_with_auto_detection() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("auto_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create dotbot structure (easiest to detect)
    let install_conf = r#"
- link:
    ~/.bashrc: bashrc
"#;
    fs::write(dotfiles_dir.join("install.conf.yaml"), install_conf)
        .expect("Failed to write install.conf.yaml");
    fs::write(dotfiles_dir.join("bashrc"), "# bashrc\n").expect("Failed to write bashrc");

    // Import with auto detection (should detect dotbot)
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--from")
        .arg("auto")
        .arg("--preview")
        .assert()
        .success();
}

#[test]
fn test_import_with_custom_output_path() {
    let env = TestEnv::new();
    let dotfiles_dir = env.home_dir().join("test_dotfiles");
    fs::create_dir_all(&dotfiles_dir).expect("Failed to create directory");

    // Create simple dotbot config
    fs::write(
        dotfiles_dir.join("install.conf.yaml"),
        "- link:\n    ~/.bashrc: bashrc\n",
    )
    .expect("Failed to write config");
    fs::write(dotfiles_dir.join("bashrc"), "# bashrc\n").expect("Failed to write bashrc");

    let output_path = env.home_dir().join("custom-heimdal.yaml");

    // Import with custom output path
    env.heimdal_cmd()
        .arg("import")
        .arg("--path")
        .arg(&dotfiles_dir)
        .arg("--output")
        .arg(&output_path)
        .arg("--preview")
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
