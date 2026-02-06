use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Scanner for detecting existing dotfiles
pub struct DotfileScanner {
    base_path: PathBuf,
}

/// A scanned dotfile with metadata
#[derive(Debug, Clone)]
pub struct ScannedDotfile {
    pub path: PathBuf,
    pub relative_path: String,
    pub category: DotfileCategory,
    pub size: u64,
}

/// Category of dotfile
#[derive(Debug, Clone, PartialEq)]
pub enum DotfileCategory {
    Shell,  // .bashrc, .zshrc, etc.
    Editor, // .vimrc, .config/nvim, etc.
    Git,    // .gitconfig, .gitignore_global
    Ssh,    // .ssh/config
    Tmux,   // .tmux.conf
    Config, // .config/* directories
    Other,
}

impl DotfileCategory {
    pub fn as_str(&self) -> &str {
        match self {
            DotfileCategory::Shell => "Shell",
            DotfileCategory::Editor => "Editor",
            DotfileCategory::Git => "Git",
            DotfileCategory::Ssh => "SSH",
            DotfileCategory::Tmux => "Tmux",
            DotfileCategory::Config => "Config",
            DotfileCategory::Other => "Other",
        }
    }
}

impl DotfileScanner {
    /// Create a new scanner for the given directory
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }

    /// Scan the home directory for common dotfiles
    pub fn scan_home() -> Result<Vec<ScannedDotfile>> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let scanner = Self::new(home);
        scanner.scan()
    }

    /// Scan for dotfiles in the configured directory
    pub fn scan(&self) -> Result<Vec<ScannedDotfile>> {
        let mut dotfiles = Vec::new();

        // Common dotfiles in home directory
        let common_files = vec![
            ".bashrc",
            ".bash_profile",
            ".bash_logout",
            ".zshrc",
            ".zprofile",
            ".zshenv",
            ".profile",
            ".vimrc",
            ".gitconfig",
            ".gitignore_global",
            ".gitignore",
            ".tmux.conf",
            ".inputrc",
            ".curlrc",
            ".wgetrc",
        ];

        for file in common_files {
            let file_path = self.base_path.join(file);
            if file_path.exists() {
                if let Ok(metadata) = fs::metadata(&file_path) {
                    dotfiles.push(ScannedDotfile {
                        path: file_path.clone(),
                        relative_path: file.to_string(),
                        category: Self::categorize(file),
                        size: metadata.len(),
                    });
                }
            }
        }

        // Scan .config directory
        let config_dir = self.base_path.join(".config");
        if config_dir.exists() {
            dotfiles.extend(self.scan_config_dir(&config_dir)?);
        }

        // Scan .ssh directory (but only config, not keys!)
        let ssh_dir = self.base_path.join(".ssh");
        if ssh_dir.exists() {
            let ssh_config = ssh_dir.join("config");
            if ssh_config.exists() {
                if let Ok(metadata) = fs::metadata(&ssh_config) {
                    dotfiles.push(ScannedDotfile {
                        path: ssh_config.clone(),
                        relative_path: ".ssh/config".to_string(),
                        category: DotfileCategory::Ssh,
                        size: metadata.len(),
                    });
                }
            }
        }

        // Scan .vim directory
        let vim_dir = self.base_path.join(".vim");
        if vim_dir.exists() && vim_dir.is_dir() {
            if let Ok(metadata) = fs::metadata(&vim_dir) {
                dotfiles.push(ScannedDotfile {
                    path: vim_dir.clone(),
                    relative_path: ".vim".to_string(),
                    category: DotfileCategory::Editor,
                    size: metadata.len(),
                });
            }
        }

        Ok(dotfiles)
    }

    /// Scan .config directory for application configs
    fn scan_config_dir(&self, config_dir: &Path) -> Result<Vec<ScannedDotfile>> {
        let mut dotfiles = Vec::new();

        // Common applications in .config
        let common_apps = vec![
            "nvim",
            "vim",
            "alacritty",
            "kitty",
            "tmux",
            "fish",
            "starship",
            "bat",
            "htop",
            "git",
            "zsh",
        ];

        for app in common_apps {
            let app_path = config_dir.join(app);
            if app_path.exists() {
                if let Ok(metadata) = fs::metadata(&app_path) {
                    let relative_path = format!(".config/{}", app);
                    dotfiles.push(ScannedDotfile {
                        path: app_path.clone(),
                        relative_path,
                        category: Self::categorize_config_app(app),
                        size: metadata.len(),
                    });
                }
            }
        }

        Ok(dotfiles)
    }

    /// Categorize a dotfile based on its name
    fn categorize(filename: &str) -> DotfileCategory {
        let lower = filename.to_lowercase();

        if lower.contains("bash") || lower.contains("zsh") || lower.contains("profile") {
            DotfileCategory::Shell
        } else if lower.contains("vim") || lower.contains("nvim") {
            DotfileCategory::Editor
        } else if lower.contains("git") {
            DotfileCategory::Git
        } else if lower.contains("ssh") {
            DotfileCategory::Ssh
        } else if lower.contains("tmux") {
            DotfileCategory::Tmux
        } else {
            DotfileCategory::Other
        }
    }

    /// Categorize a config app directory
    fn categorize_config_app(app: &str) -> DotfileCategory {
        match app {
            "nvim" | "vim" | "alacritty" | "kitty" => DotfileCategory::Editor,
            "fish" | "zsh" | "starship" => DotfileCategory::Shell,
            "tmux" => DotfileCategory::Tmux,
            "git" => DotfileCategory::Git,
            _ => DotfileCategory::Config,
        }
    }

    /// Group dotfiles by category
    pub fn group_by_category(
        dotfiles: &[ScannedDotfile],
    ) -> Vec<(DotfileCategory, Vec<&ScannedDotfile>)> {
        let mut grouped: std::collections::HashMap<String, Vec<&ScannedDotfile>> =
            std::collections::HashMap::new();

        for dotfile in dotfiles {
            grouped
                .entry(dotfile.category.as_str().to_string())
                .or_default()
                .push(dotfile);
        }

        let mut result: Vec<(DotfileCategory, Vec<&ScannedDotfile>)> = Vec::new();

        // Order by category importance
        let categories = vec![
            DotfileCategory::Shell,
            DotfileCategory::Editor,
            DotfileCategory::Git,
            DotfileCategory::Tmux,
            DotfileCategory::Ssh,
            DotfileCategory::Config,
            DotfileCategory::Other,
        ];

        for category in categories {
            if let Some(files) = grouped.get(category.as_str()) {
                result.push((category.clone(), files.clone()));
            }
        }

        result
    }

    /// Format file size for display
    pub fn format_size(size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize() {
        assert_eq!(
            DotfileScanner::categorize(".bashrc"),
            DotfileCategory::Shell
        );
        assert_eq!(
            DotfileScanner::categorize(".vimrc"),
            DotfileCategory::Editor
        );
        assert_eq!(
            DotfileScanner::categorize(".gitconfig"),
            DotfileCategory::Git
        );
        assert_eq!(
            DotfileScanner::categorize(".tmux.conf"),
            DotfileCategory::Tmux
        );
    }

    #[test]
    fn test_categorize_config_app() {
        assert_eq!(
            DotfileScanner::categorize_config_app("nvim"),
            DotfileCategory::Editor
        );
        assert_eq!(
            DotfileScanner::categorize_config_app("fish"),
            DotfileCategory::Shell
        );
        assert_eq!(
            DotfileScanner::categorize_config_app("git"),
            DotfileCategory::Git
        );
    }

    #[test]
    fn test_format_size() {
        assert_eq!(DotfileScanner::format_size(512), "512 B");
        assert_eq!(DotfileScanner::format_size(2048), "2.0 KB");
        assert_eq!(DotfileScanner::format_size(2 * 1024 * 1024), "2.0 MB");
    }
}
