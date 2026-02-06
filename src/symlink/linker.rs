use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::conflict::{
    create_backup, detect_conflict, resolve_conflict, ConflictResolution, ConflictStrategy,
};
use super::stow::StowConfig;
use crate::utils::{info, step, success, warning};

/// Result of a symlink operation
#[derive(Debug, Clone)]
pub struct SymlinkResult {
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

    /// Symlink all files in dotfiles directory (stow-style)
    pub fn link_all(
        &self,
        ignore_patterns: &[String],
        dry_run: bool,
    ) -> Result<Vec<SymlinkResult>> {
        let mut results = Vec::new();

        // Verify dotfiles directory exists
        if !self.dotfiles_dir.exists() {
            anyhow::bail!(
                "Dotfiles directory does not exist: {}",
                self.dotfiles_dir.display()
            );
        }

        // Walk through dotfiles directory
        for entry in WalkDir::new(&self.dotfiles_dir).min_depth(1).max_depth(1) {
            let entry = entry.with_context(|| {
                format!(
                    "Failed to read directory entry in {}",
                    self.dotfiles_dir.display()
                )
            })?;
            let path = entry.path();

            // Check if should be ignored
            if should_ignore(path, ignore_patterns) {
                continue;
            }

            // Get relative path from dotfiles_dir
            let rel_path = path.strip_prefix(&self.dotfiles_dir)?;
            let target = self.target_dir.join(rel_path);

            // Create symlink
            let result = self.create_symlink(path, &target, dry_run)?;
            results.push(result);
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

        unix_fs::symlink(&source, target).with_context(|| {
            format!(
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
