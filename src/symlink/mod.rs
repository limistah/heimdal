pub mod conflict;
pub mod linker;
pub mod stow;

pub use conflict::ConflictStrategy;
pub use linker::{Linker, SymlinkResult};
pub use stow::StowConfig;

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::conditions::evaluate_condition;
use crate::config::ResolvedProfile;
use crate::utils::{header, info, warning};

/// Create symlinks for a resolved profile
pub fn create_symlinks(
    profile: &ResolvedProfile,
    dotfiles_dir: &Path,
    dry_run: bool,
    force: bool,
) -> Result<SymlinkReport> {
    header("Creating Symlinks");

    let mut report = SymlinkReport::new();

    // Determine strategy
    let strategy = if force {
        ConflictStrategy::Force
    } else if dry_run {
        ConflictStrategy::Backup // Default to backup in dry-run
    } else {
        ConflictStrategy::Prompt // Ask user interactively
    };

    // Check if we should use stowrc
    if profile.dotfiles.use_stowrc {
        let stowrc_path = dotfiles_dir.join(".stowrc");

        if stowrc_path.exists() {
            info(&format!("Reading .stowrc: {}", stowrc_path.display()));

            let stow_config = StowConfig::parse(&stowrc_path)?;

            // Determine target directory
            let target_dir = if let Some(target) = stow_config.target {
                target
            } else {
                PathBuf::from(shellexpand::tilde("~").as_ref())
            };

            // Determine backup directory
            let backup_dir = PathBuf::from(shellexpand::tilde("~/.heimdal/backups").as_ref());

            // Combine ignore patterns from stowrc and config
            let mut ignore_patterns = stow_config.ignore.clone();
            ignore_patterns.extend(profile.dotfiles.ignore.clone());

            info(&format!("Target directory: {}", target_dir.display()));
            info(&format!("Backup directory: {}", backup_dir.display()));
            info(&format!("Ignoring {} patterns", ignore_patterns.len()));

            // Create linker
            let linker = Linker::new(dotfiles_dir.to_path_buf(), target_dir, backup_dir)
                .with_conflict_strategy(strategy);

            // Link all files
            let results = linker.link_all(&ignore_patterns, dry_run)?;
            report.add_results(results);
        } else {
            warning("use_stowrc=true but .stowrc not found, using defaults");

            // Use defaults
            let target_dir = PathBuf::from(shellexpand::tilde("~").as_ref());
            let backup_dir = PathBuf::from(shellexpand::tilde("~/.heimdal/backups").as_ref());

            let linker = Linker::new(dotfiles_dir.to_path_buf(), target_dir, backup_dir)
                .with_conflict_strategy(strategy);

            let results = linker.link_all(&profile.dotfiles.ignore, dry_run)?;
            report.add_results(results);
        }
    } else {
        // Use explicit file list from config
        if profile.dotfiles.files.is_empty() {
            info("No explicit symlinks configured");
            return Ok(report);
        }

        info(&format!(
            "Creating {} explicit symlinks...",
            profile.dotfiles.files.len()
        ));

        let target_dir = PathBuf::from(shellexpand::tilde("~").as_ref());
        let backup_dir = PathBuf::from(shellexpand::tilde("~/.heimdal/backups").as_ref());

        let linker = Linker::new(dotfiles_dir.to_path_buf(), target_dir, backup_dir)
            .with_conflict_strategy(strategy);

        for mapping in &profile.dotfiles.files {
            // Check if condition is met
            if let Some(condition) = &mapping.when {
                if !evaluate_condition(condition, &profile.name)? {
                    let source_path = dotfiles_dir.join(&mapping.source);
                    let target_expanded = shellexpand::tilde(&mapping.target);
                    let target_path = PathBuf::from(target_expanded.as_ref());

                    info(&format!("Skipping {} (condition not met)", mapping.source));
                    report.results.push(SymlinkResult::skipped(
                        source_path,
                        target_path,
                        "Condition not met".to_string(),
                    ));
                    continue;
                }
            }

            let source = dotfiles_dir.join(&mapping.source);
            let target_expanded = shellexpand::tilde(&mapping.target);
            let target = PathBuf::from(target_expanded.as_ref());

            let result = linker.create_symlink(&source, &target, dry_run)?;
            report.results.push(result);
        }
    }

    Ok(report)
}

/// Report of symlink operations
#[derive(Debug)]
pub struct SymlinkReport {
    pub results: Vec<SymlinkResult>,
}

impl SymlinkReport {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_results(&mut self, results: Vec<SymlinkResult>) {
        self.results.extend(results);
    }

    pub fn print_summary(&self) {
        header("Symlink Summary");

        let created = self
            .results
            .iter()
            .filter(|r| r.success && !r.skipped)
            .count();
        let skipped = self.results.iter().filter(|r| r.skipped).count();
        let failed = self.results.iter().filter(|r| !r.success).count();

        if created > 0 {
            info(&format!("✓ Created: {} symlinks", created));
        }

        if skipped > 0 {
            info(&format!("○ Skipped: {} symlinks", skipped));
        }

        if failed > 0 {
            warning(&format!("✗ Failed: {} symlinks", failed));
            for result in self.results.iter().filter(|r| !r.success) {
                warning(&format!(
                    "  - {}: {}",
                    result.target.display(),
                    result.message.as_deref().unwrap_or("Unknown error")
                ));
            }
        }
    }
}
