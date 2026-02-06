use super::{DotfileMapping, DotfileTool, ImportResult, Importer};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Importer for chezmoi dotfiles
///
/// Chezmoi uses a specific naming convention:
/// - Regular files: `dot_filename` → `.filename`
/// - Executable files: `executable_filename`
/// - Directories: `dot_directory/` → `.directory/`
/// - Templates: `filename.tmpl`
/// - Private files: `private_filename`
///
/// Reference: https://www.chezmoi.io/reference/source-state-attributes/
pub struct ChezmoiImporter;

impl ChezmoiImporter {
    /// Parse chezmoi filename to get the actual destination name
    fn parse_filename(name: &str) -> Option<String> {
        let mut result = name.to_string();

        // Remove prefixes in order
        let prefixes = [
            "encrypted_",
            "private_",
            "readonly_",
            "executable_",
            "symlink_",
            "dot_",
            "literal_",
        ];

        for prefix in &prefixes {
            if result.starts_with(prefix) {
                result = result[prefix.len()..].to_string();
            }
        }

        // Remove .tmpl suffix if present
        if result.ends_with(".tmpl") {
            result = result[..result.len() - 5].to_string();
        }

        // Replace dot_ in the middle (for nested dots)
        result = result.replace("dot_", ".");

        // Add leading dot if needed (unless it's a special file)
        if !result.starts_with('.') && !result.is_empty() {
            result = format!(".{}", result);
        }

        Some(result)
    }

    /// Scan chezmoi source directory for dotfiles
    fn scan_chezmoi_dir(source_dir: &Path) -> Result<Vec<DotfileMapping>> {
        let mut mappings = Vec::new();

        if !source_dir.exists() {
            return Ok(mappings);
        }

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            // Skip chezmoi metadata files
            if filename_str.starts_with(".chezmoi") {
                continue;
            }

            // Skip .git directory
            if filename_str == ".git" {
                continue;
            }

            // Parse the filename to get destination
            if let Some(dest_name) = Self::parse_filename(&filename_str) {
                let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                let destination = home_dir.join(&dest_name);

                mappings.push(DotfileMapping {
                    source: path.clone(),
                    destination,
                    category: Self::categorize_file(&dest_name),
                });

                // If it's a directory, recursively scan it
                if path.is_dir() {
                    // TODO: Handle nested directories properly
                    // For now, just add the directory itself
                }
            }
        }

        Ok(mappings)
    }

    /// Categorize a dotfile based on its name
    fn categorize_file(name: &str) -> Option<String> {
        if name.contains("bash") || name.contains("zsh") || name.contains("fish") {
            Some("shell".to_string())
        } else if name.contains("vim") || name.contains("nvim") {
            Some("editor".to_string())
        } else if name.contains("git") {
            Some("git".to_string())
        } else if name.contains("tmux") {
            Some("tmux".to_string())
        } else {
            None
        }
    }
}

impl Importer for ChezmoiImporter {
    fn detect(path: &Path) -> bool {
        // Chezmoi typically uses ~/.local/share/chezmoi as source directory
        // or looks for .chezmoi* files
        path.join(".chezmoiignore").exists()
            || path.join(".chezmoi.toml.tmpl").exists()
            || path.join(".chezmoi.yaml.tmpl").exists()
            || path.join(".chezmoi.json.tmpl").exists()
            || path.join(".chezmoiversion").exists()
    }

    fn import(path: &Path) -> Result<ImportResult> {
        info_fmt!("Importing from chezmoi directory: {}", path.display());

        let dotfiles = Self::scan_chezmoi_dir(path)?;

        info_fmt!("Found {} dotfiles in chezmoi format", dotfiles.len());

        Ok(ImportResult {
            tool: DotfileTool::Chezmoi,
            dotfiles,
            packages: Vec::new(), // Chezmoi doesn't manage packages
            stow_compat: false,   // Different structure from Stow
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filename() {
        assert_eq!(
            ChezmoiImporter::parse_filename("dot_bashrc"),
            Some(".bashrc".to_string())
        );

        assert_eq!(
            ChezmoiImporter::parse_filename("dot_config"),
            Some(".config".to_string())
        );

        assert_eq!(
            ChezmoiImporter::parse_filename("executable_dot_local"),
            Some(".local".to_string())
        );

        assert_eq!(
            ChezmoiImporter::parse_filename("private_dot_ssh"),
            Some(".ssh".to_string())
        );

        assert_eq!(
            ChezmoiImporter::parse_filename("dot_gitconfig.tmpl"),
            Some(".gitconfig".to_string())
        );
    }

    #[test]
    fn test_categorize_file() {
        assert_eq!(
            ChezmoiImporter::categorize_file(".bashrc"),
            Some("shell".to_string())
        );

        assert_eq!(
            ChezmoiImporter::categorize_file(".vimrc"),
            Some("editor".to_string())
        );

        assert_eq!(
            ChezmoiImporter::categorize_file(".gitconfig"),
            Some("git".to_string())
        );
    }
}
