use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use super::conflict::{
    create_backup, detect_conflict, resolve_conflict, ConflictResolution, ConflictStrategy,
};
use crate::utils::{info, step, success, symlink_error, SymlinkErrorType};

/// Result of a symlink operation
#[derive(Debug, Clone)]
pub struct SymlinkResult {
    #[allow(dead_code)]
    pub source: PathBuf,
    pub target: PathBuf,
    pub success: bool,
    pub message: Option<String>,
    pub skipped: bool,
}

impl SymlinkResult {
    pub fn success(source: PathBuf, target: PathBuf) -> Self {
        Self {
            source,
            target,
            success: true,
            message: None,
            skipped: false,
        }
    }

    pub fn skipped(source: PathBuf, target: PathBuf, reason: String) -> Self {
        Self {
            source,
            target,
            success: true,
            message: Some(reason),
            skipped: true,
        }
    }

    pub fn failed(source: PathBuf, target: PathBuf, error: String) -> Self {
        Self {
            source,
            target,
            success: false,
            message: Some(error),
            skipped: false,
        }
    }
}

/// Symlink manager for creating dotfile symlinks
///
/// The linker uses a recursive file-level symlinking strategy:
/// - Individual files are symlinked (not directories)
/// - Directory structures are recreated in the target
/// - This allows fine-grained control (e.g., track only specific files in ~/.config)
///
/// Example:
/// ```
/// Source:                          Target:
/// ~/.dotfiles/config/nvim/init.lua → ~/.config/nvim/init.lua (symlink)
/// ~/.dotfiles/config/tmux.conf     → ~/.config/tmux.conf     (symlink)
///
/// The ~/.config directory is NOT symlinked as a whole, allowing users
/// to track only specific config files without overriding everything.
/// ```
pub struct Linker {
    dotfiles_dir: PathBuf,
    target_dir: PathBuf,
    backup_dir: PathBuf,
    conflict_strategy: ConflictStrategy,
}

impl Linker {
    pub fn new(dotfiles_dir: PathBuf, target_dir: PathBuf, backup_dir: PathBuf) -> Self {
        // Canonicalize dotfiles_dir to make it absolute
        let dotfiles_dir = dotfiles_dir.canonicalize().unwrap_or(dotfiles_dir);

        Self {
            dotfiles_dir,
            target_dir,
            backup_dir,
            conflict_strategy: ConflictStrategy::Prompt,
        }
    }

    pub fn with_conflict_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.conflict_strategy = strategy;
        self
    }

    /// Symlink all files in dotfiles directory (recursive file-level symlinking)
    ///
    /// Instead of symlinking entire directories (which would override everything),
    /// this recursively symlinks individual files within directories. This allows
    /// users to track only specific files (e.g., ~/.config/nvim/init.lua) without
    /// overriding their entire ~/.config directory.
    pub fn link_all(
        &self,
        ignore_patterns: &[String],
        dry_run: bool,
    ) -> Result<Vec<SymlinkResult>> {
        // Verify dotfiles directory exists
        if !self.dotfiles_dir.exists() {
            let error_msg = crate::utils::config_error(
                &self.dotfiles_dir.display().to_string(),
                crate::utils::ConfigErrorType::FileNotFound,
            );
            eprintln!("{}", error_msg);
            anyhow::bail!(
                "Dotfiles directory does not exist: {}",
                self.dotfiles_dir.display()
            );
        }

        // Start recursive symlinking from dotfiles root
        self.link_directory_recursive(
            &self.dotfiles_dir,
            &self.target_dir,
            ignore_patterns,
            dry_run,
        )
    }

    /// Recursively symlink files in a directory
    ///
    /// For each entry in the source directory:
    /// - If it's a file: create a symlink
    /// - If it's a directory: recurse into it (creating target directories as needed)
    ///
    /// This ensures we never symlink entire directories, only individual files.
    fn link_directory_recursive(
        &self,
        source_dir: &Path,
        target_dir: &Path,
        ignore_patterns: &[String],
        dry_run: bool,
    ) -> Result<Vec<SymlinkResult>> {
        let mut results = Vec::new();

        // Read directory entries
        let entries = fs::read_dir(source_dir)
            .with_context(|| format!("Failed to read directory: {}", source_dir.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| {
                format!("Failed to read directory entry in {}", source_dir.display())
            })?;

            let source_path = entry.path();

            // Check if should be ignored
            if should_ignore(&source_path, ignore_patterns) {
                continue;
            }

            // Get the file/directory name
            let name = match source_path.file_name() {
                Some(n) => n,
                None => continue,
            };

            let target_path = target_dir.join(name);

            if source_path.is_dir() {
                // It's a directory - recurse into it
                // Don't symlink the directory itself, recurse and symlink files inside
                let sub_results = self.link_directory_recursive(
                    &source_path,
                    &target_path,
                    ignore_patterns,
                    dry_run,
                )?;
                results.extend(sub_results);
            } else {
                // It's a file - create symlink
                let result = self.create_symlink(&source_path, &target_path, dry_run)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Create a single symlink
    pub fn create_symlink(
        &self,
        source: &Path,
        target: &Path,
        dry_run: bool,
    ) -> Result<SymlinkResult> {
        // Make source path absolute
        let source = if source.is_absolute() {
            source.to_path_buf()
        } else {
            self.dotfiles_dir.join(source)
        };

        if !source.exists() {
            return Ok(SymlinkResult::failed(
                source.clone(),
                target.to_path_buf(),
                "Source does not exist".to_string(),
            ));
        }

        // Check if target already exists and is correct symlink
        if target.is_symlink() {
            if let Ok(existing) = fs::read_link(target) {
                if existing == source {
                    return Ok(SymlinkResult::skipped(
                        source,
                        target.to_path_buf(),
                        "Already linked correctly".to_string(),
                    ));
                }
            }
        }

        // Check for conflicts
        if detect_conflict(target) {
            let resolution = resolve_conflict(target, &source, self.conflict_strategy, dry_run)?;

            match resolution {
                ConflictResolution::Skip => {
                    return Ok(SymlinkResult::skipped(
                        source,
                        target.to_path_buf(),
                        "User chose to skip".to_string(),
                    ));
                }
                ConflictResolution::Backup => {
                    // Create backup before proceeding
                    create_backup(target, &self.backup_dir, dry_run)?;

                    if !dry_run {
                        // Remove original
                        if target.is_dir() {
                            fs::remove_dir_all(target)?;
                        } else {
                            fs::remove_file(target)?;
                        }
                    }
                }
                ConflictResolution::Overwrite => {
                    if !dry_run {
                        // Remove original
                        if target.is_dir() {
                            fs::remove_dir_all(target)?;
                        } else {
                            fs::remove_file(target)?;
                        }
                    }
                }
            }
        }

        // Create parent directory if needed
        if let Some(parent) = target.parent() {
            if !dry_run && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Create symlink
        step(&format!(
            "Linking {} -> {}",
            target.display(),
            source.display()
        ));

        if dry_run {
            info(&format!(
                "Would create symlink: {} -> {}",
                target.display(),
                source.display()
            ));
            return Ok(SymlinkResult::success(source, target.to_path_buf()));
        }

        unix_fs::symlink(&source, target).map_err(|e| {
            let error_type = match e.kind() {
                std::io::ErrorKind::PermissionDenied => SymlinkErrorType::PermissionDenied,
                std::io::ErrorKind::NotFound => SymlinkErrorType::DirectoryNotFound,
                _ => SymlinkErrorType::FileExists,
            };

            let error_msg = symlink_error(
                &source.display().to_string(),
                &target.display().to_string(),
                error_type,
            );
            eprintln!("{}", error_msg);

            anyhow::anyhow!(
                "Failed to create symlink: {} -> {}",
                target.display(),
                source.display()
            )
        })?;

        success(&format!("Linked: {}", target.display()));

        Ok(SymlinkResult::success(source, target.to_path_buf()))
    }
}

/// Check if a path should be ignored based on patterns
fn should_ignore(path: &Path, patterns: &[String]) -> bool {
    if let Some(file_name) = path.file_name() {
        let name = file_name.to_string_lossy();

        for pattern in patterns {
            // Simple pattern matching (exact match or wildcard)
            if pattern.contains('*') {
                // Basic glob matching
                if glob_match(pattern, &name) {
                    return true;
                }
            } else if name == pattern.as_str() {
                return true;
            }
        }
    }

    false
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // Very basic implementation - just handle * wildcard
    if pattern == "*" {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }

    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }

    pattern == text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore() {
        let patterns = vec![
            ".git".to_string(),
            ".DS_Store".to_string(),
            "*.log".to_string(),
        ];

        assert!(should_ignore(Path::new(".git"), &patterns));
        assert!(should_ignore(Path::new(".DS_Store"), &patterns));
        assert!(should_ignore(Path::new("test.log"), &patterns));
        assert!(!should_ignore(Path::new("README.md"), &patterns));
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.log", "test.log"));
        assert!(glob_match("*.log", "error.log"));
        assert!(!glob_match("*.log", "test.txt"));

        assert!(glob_match("test*", "test.log"));
        assert!(glob_match("test*", "test"));
        assert!(!glob_match("test*", "other.log"));
    }
}
