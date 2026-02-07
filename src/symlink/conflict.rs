use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::utils::{confirm, error, info, warning};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    Prompt, // Ask user what to do
    Backup, // Automatically backup existing file
    Force,  // Overwrite without asking
    #[allow(dead_code)]
    Skip, // Don't create symlink
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    Overwrite,
    Backup,
    #[allow(dead_code)]
    Skip,
}

/// Detect if a conflict exists at the target path
pub fn detect_conflict(target: &Path) -> bool {
    if !target.exists() {
        return false;
    }

    // If it's already a symlink, check if it points to the right place
    if target.is_symlink() {
        // We'll handle this in the linker by comparing symlink targets
        return false;
    }

    // File or directory exists and is not a symlink
    true
}

/// Resolve a conflict based on strategy
pub fn resolve_conflict(
    target: &Path,
    source: &Path,
    strategy: ConflictStrategy,
    dry_run: bool,
) -> Result<ConflictResolution> {
    match strategy {
        ConflictStrategy::Prompt => {
            if dry_run {
                info(&format!("Would prompt for conflict: {}", target.display()));
                return Ok(ConflictResolution::Backup);
            }

            warning(&format!("Conflict detected: {}", target.display()));
            info(&format!("  Existing: {}", target.display()));
            info(&format!("  New symlink to: {}", source.display()));

            println!("\nWhat would you like to do?");
            println!("  1. Backup existing file and create symlink");
            println!("  2. Overwrite (delete existing)");
            println!("  3. Skip this file");

            let choice = crate::utils::prompt("Enter choice [1/2/3]");

            match choice.as_str() {
                "1" => Ok(ConflictResolution::Backup),
                "2" => {
                    if confirm("Are you sure you want to overwrite?") {
                        Ok(ConflictResolution::Overwrite)
                    } else {
                        Ok(ConflictResolution::Skip)
                    }
                }
                "3" => Ok(ConflictResolution::Skip),
                _ => {
                    error("Invalid choice, skipping");
                    Ok(ConflictResolution::Skip)
                }
            }
        }
        ConflictStrategy::Backup => Ok(ConflictResolution::Backup),
        ConflictStrategy::Force => Ok(ConflictResolution::Overwrite),
        ConflictStrategy::Skip => Ok(ConflictResolution::Skip),
    }
}

/// Create a backup of a file/directory
pub fn create_backup(target: &Path, backup_dir: &Path, dry_run: bool) -> Result<PathBuf> {
    use std::fs;
    use std::time::SystemTime;

    // Create timestamp-based backup
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    let filename = target
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    let backup_path = backup_dir.join(format!("{}.{}", filename.to_string_lossy(), timestamp));

    if dry_run {
        info(&format!(
            "Would backup {} to {}",
            target.display(),
            backup_path.display()
        ));
        return Ok(backup_path);
    }

    // Create backup directory if needed
    fs::create_dir_all(backup_dir)?;

    // Copy the file/directory
    if target.is_dir() {
        copy_dir_all(target, &backup_path)?;
    } else {
        fs::copy(target, &backup_path)?;
    }

    info(&format!("Backed up to: {}", backup_path.display()));

    Ok(backup_path)
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    use std::fs;

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
