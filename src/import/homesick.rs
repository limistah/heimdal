use super::{DotfileMapping, DotfileTool, ImportResult, Importer};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Importer for homesick/homeshick dotfiles
///
/// Homesick uses a "castle" structure where dotfiles are organized in:
/// - $HOME/.homesick/repos/<castle-name>/home/
///
/// Files in the home/ directory are symlinked to $HOME
///
/// Reference: https://github.com/technicalpickles/homesick
pub struct HomesickImporter;

impl HomesickImporter {
    /// Scan a homesick castle for dotfiles
    fn scan_castle(castle_path: &Path) -> Result<Vec<DotfileMapping>> {
        let mut mappings = Vec::new();
        let home_subdir = castle_path.join("home");

        if !home_subdir.exists() {
            return Ok(mappings);
        }

        // Recursively scan the home directory
        Self::scan_directory(&home_subdir, &home_subdir, &mut mappings)?;

        Ok(mappings)
    }

    /// Recursively scan a directory for files
    fn scan_directory(
        base_dir: &Path,
        current_dir: &Path,
        mappings: &mut Vec<DotfileMapping>,
    ) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip .git directories
            if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }

            if path.is_dir() {
                // Recursively scan subdirectories
                Self::scan_directory(base_dir, &path, mappings)?;
            } else if path.is_file() {
                // Calculate relative path from base_dir
                let relative = path.strip_prefix(base_dir).unwrap_or(path.as_path());

                // Destination is in user's home directory
                let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                let destination = home_dir.join(relative);

                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let category = Self::categorize_file(filename);

                mappings.push(DotfileMapping {
                    source: path.clone(),
                    destination,
                    category,
                });
            }
        }

        Ok(())
    }

    /// Categorize a dotfile based on its name
    fn categorize_file(name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();

        if name_lower.contains("bash") || name_lower.contains("zsh") || name_lower.contains("fish")
        {
            Some("shell".to_string())
        } else if name_lower.contains("vim") || name_lower.contains("nvim") {
            Some("editor".to_string())
        } else if name_lower.contains("git") {
            Some("git".to_string())
        } else if name_lower.contains("tmux") {
            Some("tmux".to_string())
        } else {
            None
        }
    }
}

impl Importer for HomesickImporter {
    fn detect(path: &Path) -> bool {
        // Check if this looks like a homesick castle
        // A castle has a "home" subdirectory
        let home_subdir = path.join("home");

        if home_subdir.exists() && home_subdir.is_dir() {
            // Additional check: see if parent is .homesick/repos
            if let Some(parent) = path.parent() {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.file_name().and_then(|n| n.to_str()) == Some(".homesick") {
                        return true;
                    }
                }
            }

            // Or if the directory structure looks right
            return true;
        }

        false
    }

    fn import(path: &Path) -> Result<ImportResult> {
        info_fmt!("Importing from homesick castle: {}", path.display());

        let dotfiles = Self::scan_castle(path)?;

        info_fmt!("Found {} dotfiles in homesick castle", dotfiles.len());

        Ok(ImportResult {
            tool: DotfileTool::Homesick,
            dotfiles,
            packages: Vec::new(), // Homesick doesn't manage packages
            stow_compat: true,    // Similar to Stow's structure
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_file() {
        assert_eq!(
            HomesickImporter::categorize_file(".bashrc"),
            Some("shell".to_string())
        );

        assert_eq!(
            HomesickImporter::categorize_file(".vimrc"),
            Some("editor".to_string())
        );

        assert_eq!(
            HomesickImporter::categorize_file(".gitconfig"),
            Some("git".to_string())
        );
    }

    #[test]
    fn test_detect() {
        // Test that detect doesn't panic on non-existent paths
        let temp = std::env::temp_dir();
        assert!(!HomesickImporter::detect(&temp));
    }
}
