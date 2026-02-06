use super::{DotfileMapping, DotfileTool, ImportResult, Importer};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Importer for yadm (Yet Another Dotfiles Manager)
///
/// Yadm is a git-based dotfile manager that uses a bare git repository
/// in $HOME/.yadm/repo.git and tracks files directly in the home directory.
///
/// Reference: https://yadm.io/
pub struct YadmImporter;

impl YadmImporter {
    /// Get list of files tracked by yadm
    fn get_yadm_files() -> Result<Vec<PathBuf>> {
        let output = Command::new("yadm").args(["list", "-a"]).output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to run 'yadm list': {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        let files: Vec<PathBuf> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| home_dir.join(line.trim()))
            .filter(|path| path.exists() && path.is_file())
            .collect();

        Ok(files)
    }

    /// Check if yadm command is available
    fn is_yadm_installed() -> bool {
        Command::new("yadm")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Categorize a dotfile based on its name
    fn categorize_file(name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();

        if name_lower.contains("bash") || name_lower.contains("zsh") || name_lower.contains("fish")
        {
            Some("shell".to_string())
        } else if name_lower.contains("vim")
            || name_lower.contains("nvim")
            || name_lower.contains("emacs")
        {
            Some("editor".to_string())
        } else if name_lower.contains("git") {
            Some("git".to_string())
        } else if name_lower.contains("tmux") || name_lower.contains("screen") {
            Some("tmux".to_string())
        } else if name_lower.contains("ssh") {
            Some("ssh".to_string())
        } else {
            None
        }
    }
}

impl Importer for YadmImporter {
    fn detect(path: &Path) -> bool {
        // Check for .yadm directory (contains the bare repo)
        let yadm_dir = path.join(".yadm");
        if yadm_dir.exists() && yadm_dir.is_dir() {
            return true;
        }

        // Also check if we're in home directory and yadm is configured
        if let Some(home) = dirs::home_dir() {
            if path == home {
                let yadm_repo = home.join(".yadm").join("repo.git");
                return yadm_repo.exists();
            }
        }

        false
    }

    fn import(path: &Path) -> Result<ImportResult> {
        info_fmt!("Importing from yadm repository: {}", path.display());

        // Check if yadm is installed
        if !Self::is_yadm_installed() {
            anyhow::bail!("yadm command not found. Please install yadm first.");
        }

        // Get list of tracked files
        let tracked_files = Self::get_yadm_files()?;

        info_fmt!("Found {} files tracked by yadm", tracked_files.len());

        // Convert to DotfileMapping
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let dotfiles: Vec<DotfileMapping> = tracked_files
            .iter()
            .map(|file| {
                let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

                DotfileMapping {
                    source: file.clone(),
                    destination: file.clone(), // Yadm tracks files in place
                    category: Self::categorize_file(filename),
                }
            })
            .collect();

        Ok(ImportResult {
            tool: DotfileTool::Yadm,
            dotfiles,
            packages: Vec::new(), // Yadm doesn't manage packages
            stow_compat: false,   // Yadm uses in-place tracking, not stow-style
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_file() {
        assert_eq!(
            YadmImporter::categorize_file(".bashrc"),
            Some("shell".to_string())
        );

        assert_eq!(
            YadmImporter::categorize_file(".vimrc"),
            Some("editor".to_string())
        );

        assert_eq!(
            YadmImporter::categorize_file(".gitconfig"),
            Some("git".to_string())
        );

        assert_eq!(
            YadmImporter::categorize_file(".tmux.conf"),
            Some("tmux".to_string())
        );

        assert_eq!(
            YadmImporter::categorize_file(".ssh/config"),
            Some("ssh".to_string())
        );
    }

    #[test]
    fn test_detect() {
        // This test requires actual .yadm directory, so we'll skip in CI
        // Just test that the function doesn't panic
        let temp = std::env::temp_dir();
        assert!(!YadmImporter::detect(&temp));
    }
}
