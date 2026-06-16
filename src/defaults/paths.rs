//! Configuration helpers for defaults sync.

use crate::config::DefaultsConfig;
use std::path::{Path, PathBuf};

/// Get the defaults export directory path within the dotfiles repo.
pub fn get_defaults_dir(dotfiles_path: &Path, config: &DefaultsConfig) -> PathBuf {
    dotfiles_path.join(&config.path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_defaults_dir() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };
        let dotfiles = PathBuf::from("/home/user/.dotfiles");
        let result = get_defaults_dir(&dotfiles, &config);
        assert_eq!(result, PathBuf::from("/home/user/.dotfiles/macos-defaults"));
    }

    #[test]
    fn test_get_defaults_dir_custom_path() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "prefs".to_string(),
        };
        let dotfiles = PathBuf::from("/Users/alice/.dotfiles");
        let result = get_defaults_dir(&dotfiles, &config);
        assert_eq!(result, PathBuf::from("/Users/alice/.dotfiles/prefs"));
    }
}
