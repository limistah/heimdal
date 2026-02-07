use std::fs;
/// Test fixture management
use std::path::{Path, PathBuf};

/// Create a sample heimdal.yaml config for testing
pub fn create_basic_config(path: &Path) -> Result<(), std::io::Error> {
    let config = r#"heimdal:
  version: "1.0"
  stow_compat: true

ignore:
  - .git
  - .gitignore
  - heimdal.yaml
  - README.md

sources:
  packages:
    - git
    - vim
    - curl

profiles:
  test:
    sources:
      - packages
    dotfiles:
      use_stowrc: true
      files:
        - source: ".bashrc"
          target: "~/.bashrc"
        - source: ".vimrc"
          target: "~/.vimrc"
"#;

    fs::write(path.join("heimdal.yaml"), config)?;
    Ok(())
}

/// Create sample dotfiles for testing
pub fn create_sample_dotfiles(dotfiles_dir: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dotfiles_dir)?;

    // Create sample .bashrc
    fs::write(
        dotfiles_dir.join(".bashrc"),
        "# Test bashrc\nexport PATH=$PATH:/usr/local/bin\n",
    )?;

    // Create sample .vimrc
    fs::write(
        dotfiles_dir.join(".vimrc"),
        "\" Test vimrc\nset number\nset expandtab\n",
    )?;

    // Create sample .config/nvim directory
    let nvim_dir = dotfiles_dir.join(".config").join("nvim");
    fs::create_dir_all(&nvim_dir)?;
    fs::write(
        nvim_dir.join("init.vim"),
        "\" Test nvim config\nset number\n",
    )?;

    Ok(())
}

/// Create a minimal valid heimdal config
pub fn create_minimal_config(path: &Path) -> Result<(), std::io::Error> {
    let config = r#"heimdal:
  version: "1.0"

profiles:
  default:
    dotfiles:
      files: []
"#;

    fs::write(path.join("heimdal.yaml"), config)?;
    Ok(())
}

/// Create an invalid heimdal config for testing error handling
pub fn create_invalid_config(path: &Path) -> Result<(), std::io::Error> {
    let config = r#"heimdal:
version: "1.0"
  profiles:
wrong_indent: true
"#;

    fs::write(path.join("heimdal.yaml"), config)?;
    Ok(())
}

/// Get path to a fixture file in tests/fixtures/
pub fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}
