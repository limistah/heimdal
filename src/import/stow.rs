use super::{DotfileMapping, DotfileTool, ImportResult, Importer};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Importer for GNU Stow structured dotfiles
pub struct StowImporter;

impl Importer for StowImporter {
    fn detect(path: &Path) -> bool {
        if !path.exists() || !path.is_dir() {
            return false;
        }

        // Check for .stowrc file
        if path.join(".stowrc").exists() {
            return true;
        }

        // Check for Stow-like structure: subdirectories containing dotfiles
        // Example: vim/.vimrc, zsh/.zshrc, etc.
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // Check if this directory contains dotfiles
                    if Self::dir_contains_dotfiles(&entry_path) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn import(path: &Path) -> Result<ImportResult> {
        let packages = Self::list_stow_packages(path)?;
        let mut dotfiles = Vec::new();

        for package in &packages {
            let package_path = path.join(package);
            let mappings = Self::scan_package(&package_path, package)?;
            dotfiles.extend(mappings);
        }

        Ok(ImportResult {
            tool: DotfileTool::Stow,
            dotfiles,
            packages: Vec::new(), // Stow doesn't manage packages
            stow_compat: true,    // Enable Stow compatibility mode
        })
    }
}

impl StowImporter {
    /// Check if a directory contains files that look like dotfiles
    fn dir_contains_dotfiles(path: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Check for dotfiles or common config files
                if name_str.starts_with('.')
                    || name_str == "config"
                    || name_str.ends_with(".conf")
                    || name_str.ends_with("rc")
                {
                    return true;
                }
            }
        }
        false
    }

    /// List all Stow packages (subdirectories)
    fn list_stow_packages(path: &Path) -> Result<Vec<String>> {
        let mut packages = Vec::new();

        for entry in fs::read_dir(path).context("Failed to read dotfiles directory")? {
            let entry = entry?;
            let entry_path = entry.path();

            // Skip hidden directories and non-directories
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if entry_path.is_dir() && !name_str.starts_with('.') {
                // Check if this looks like a Stow package
                if Self::dir_contains_dotfiles(&entry_path) {
                    packages.push(name_str.to_string());
                }
            }
        }

        Ok(packages)
    }

    /// Scan a Stow package directory for dotfiles
    fn scan_package(package_path: &Path, package_name: &str) -> Result<Vec<DotfileMapping>> {
        let mut mappings = Vec::new();
        Self::scan_directory_recursive(package_path, package_path, &mut mappings, package_name)?;
        Ok(mappings)
    }

    /// Recursively scan a directory for files
    fn scan_directory_recursive(
        base_path: &Path,
        current_path: &Path,
        mappings: &mut Vec<DotfileMapping>,
        category: &str,
    ) -> Result<()> {
        for entry in fs::read_dir(current_path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Recursively scan subdirectories
                Self::scan_directory_recursive(base_path, &entry_path, mappings, category)?;
            } else if entry_path.is_file() {
                // Calculate relative path from package base
                let relative = entry_path
                    .strip_prefix(base_path)
                    .context("Failed to strip prefix")?;

                // In Stow, files are symlinked to home directory
                let destination = dirs::home_dir()
                    .context("Failed to get home directory")?
                    .join(relative);

                mappings.push(DotfileMapping {
                    source: entry_path.clone(),
                    destination,
                    category: Some(category.to_string()),
                });
            }
        }

        Ok(())
    }

    /// Parse .stowrc file if it exists
    #[allow(dead_code)]
    fn parse_stowrc(path: &Path) -> Result<StowConfig> {
        let stowrc_path = path.join(".stowrc");
        if !stowrc_path.exists() {
            return Ok(StowConfig::default());
        }

        let content = fs::read_to_string(&stowrc_path).context("Failed to read .stowrc")?;

        let mut config = StowConfig::default();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse options
            if line.starts_with("--target=") {
                config.target = line
                    .strip_prefix("--target=")
                    .unwrap_or_default()
                    .to_string();
            } else if line.starts_with("--dir=") {
                config.dir = Some(line.strip_prefix("--dir=").unwrap_or_default().to_string());
            }
        }

        Ok(config)
    }
}

/// Configuration from .stowrc
#[derive(Debug, Default)]
#[allow(dead_code)]
struct StowConfig {
    target: String,
    dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_stow_structure() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create a Stow-like structure
        let vim_dir = base.join("vim");
        fs::create_dir(&vim_dir).unwrap();
        fs::write(vim_dir.join(".vimrc"), "set number").unwrap();

        assert!(StowImporter::detect(base));
    }

    #[test]
    fn test_detect_stowrc() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create .stowrc file
        fs::write(base.join(".stowrc"), "--target=$HOME").unwrap();

        assert!(StowImporter::detect(base));
    }

    #[test]
    fn test_list_packages() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create packages
        let vim_dir = base.join("vim");
        fs::create_dir(&vim_dir).unwrap();
        fs::write(vim_dir.join(".vimrc"), "").unwrap();

        let zsh_dir = base.join("zsh");
        fs::create_dir(&zsh_dir).unwrap();
        fs::write(zsh_dir.join(".zshrc"), "").unwrap();

        let packages = StowImporter::list_stow_packages(base).unwrap();
        assert_eq!(packages.len(), 2);
        assert!(packages.contains(&"vim".to_string()));
        assert!(packages.contains(&"zsh".to_string()));
    }
}
