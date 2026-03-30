use assert_fs::prelude::*;
use assert_fs::TempDir;

/// Create a temp home directory with a minimal dotfiles setup.
pub fn setup_home(profile: &str) -> TempDir {
    let home = TempDir::new().unwrap();
    let dotfiles = home.child(".dotfiles");
    dotfiles.create_dir_all().unwrap();
    let heimdal_dir = dotfiles.child(".heimdal");
    heimdal_dir.create_dir_all().unwrap();

    let state_json = format!(
        r#"{{
        "version": 1,
        "machine_id": "test-machine-id",
        "hostname": "testhost",
        "username": "testuser",
        "os": "linux",
        "active_profile": "{}",
        "dotfiles_path": "{}",
        "repo_url": "https://example.com/dotfiles.git",
        "last_apply": null,
        "last_sync": null,
        "heimdal_version": "3.0.0"
    }}"#,
        profile,
        dotfiles.path().display()
    );
    heimdal_dir.child("state.json").write_str(&state_json).unwrap();

    dotfiles.child("heimdal.yaml").write_str(&format!(
        "heimdal:\n  version: \"1\"\nprofiles:\n  {}:\n    dotfiles:\n      - .vimrc\n",
        profile
    )).unwrap();

    dotfiles.child(".vimrc").write_str("\" test vim config").unwrap();
    home
}
