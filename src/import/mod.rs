pub mod chezmoi;
pub mod dotbot;
pub mod homesick;
pub mod stow;
pub mod yadm;

use anyhow::Result;
use dialoguer::Select;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a tool that manages dotfiles
#[derive(Debug, Clone, PartialEq)]
pub enum DotfileTool {
    Stow,
    Dotbot,
    Chezmoi,
    Yadm,
    Homesick,
    Manual,
}

impl DotfileTool {
    pub fn name(&self) -> &str {
        match self {
            DotfileTool::Stow => "GNU Stow",
            DotfileTool::Dotbot => "dotbot",
            DotfileTool::Chezmoi => "chezmoi",
            DotfileTool::Yadm => "yadm",
            DotfileTool::Homesick => "homesick",
            DotfileTool::Manual => "Manual/Custom",
        }
    }
}

/// Result of importing from another tool
#[derive(Debug)]
pub struct ImportResult {
    pub tool: DotfileTool,
    pub dotfiles: Vec<DotfileMapping>,
    pub packages: Vec<String>,
    pub stow_compat: bool,
}

/// Maps a source file to its destination
#[derive(Debug, Clone)]
pub struct DotfileMapping {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub category: Option<String>,
}

/// Strategies for resolving conflicts during import
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// Skip conflicting files
    Skip,
    /// Overwrite existing files without backup
    Overwrite,
    /// Backup existing files, then overwrite
    Backup,
    /// Ask user for each conflicting file
    Ask,
}

/// Options for importing dotfiles
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub conflict_resolution: ConflictResolution,
    pub dry_run: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            conflict_resolution: ConflictResolution::Ask,
            dry_run: false,
        }
    }
}

/// Trait for importing from different dotfile tools
pub trait Importer {
    fn detect(path: &Path) -> bool;
    fn import(path: &Path) -> Result<ImportResult>;
}

/// Detect which dotfile tool is being used in a directory
pub fn detect_tool(path: &Path) -> Option<DotfileTool> {
    if !path.exists() || !path.is_dir() {
        return None;
    }

    // Check for Stow structure (subdirectories with dotfiles)
    if stow::StowImporter::detect(path) {
        return Some(DotfileTool::Stow);
    }

    // Check for dotbot config
    if dotbot::DotbotImporter::detect(path) {
        return Some(DotfileTool::Dotbot);
    }

    // Check for chezmoi
    if chezmoi::ChezmoiImporter::detect(path) {
        return Some(DotfileTool::Chezmoi);
    }

    // Check for yadm
    if yadm::YadmImporter::detect(path) {
        return Some(DotfileTool::Yadm);
    }

    // Check for homesick
    if homesick::HomesickImporter::detect(path) {
        return Some(DotfileTool::Homesick);
    }

    // Default to manual
    Some(DotfileTool::Manual)
}

/// Import from the detected tool
pub fn import_from_tool(path: &Path, tool: &DotfileTool) -> Result<ImportResult> {
    match tool {
        DotfileTool::Stow => stow::StowImporter::import(path),
        DotfileTool::Dotbot => dotbot::DotbotImporter::import(path),
        DotfileTool::Chezmoi => chezmoi::ChezmoiImporter::import(path),
        DotfileTool::Yadm => yadm::YadmImporter::import(path),
        DotfileTool::Homesick => homesick::HomesickImporter::import(path),
        DotfileTool::Manual => anyhow::bail!("Manual import not yet implemented"),
    }
}

/// Generate a unique backup path for a file
/// Handles files with no extension and ensures uniqueness
fn generate_backup_path(path: &Path) -> PathBuf {
    // Get the file name
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

    // Create initial backup path by appending .backup to the full name
    let initial_backup = path.with_file_name(format!("{}.backup", file_name));

    // If it doesn't exist, use it
    if !initial_backup.exists() {
        return initial_backup;
    }

    // Otherwise, find a unique name with a counter
    let mut counter = 1;
    loop {
        let backup_with_counter = path.with_file_name(format!("{}.backup.{}", file_name, counter));
        if !backup_with_counter.exists() {
            return backup_with_counter;
        }
        counter += 1;
    }
}

/// Detect conflicting files (files that already exist at destination)
pub fn detect_conflicts(result: &ImportResult) -> Vec<DotfileMapping> {
    result
        .dotfiles
        .iter()
        .filter(|mapping| {
            let dest = &mapping.destination;
            dest.exists()
                && (dest.is_file()
                    || fs::symlink_metadata(dest)
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false))
        })
        .cloned()
        .collect()
}

/// Resolve conflicts based on the chosen strategy
pub fn resolve_conflicts(
    conflicts: Vec<DotfileMapping>,
    strategy: &ConflictResolution,
) -> Result<Vec<DotfileMapping>> {
    match strategy {
        ConflictResolution::Skip => {
            // Skip all conflicting files
            Ok(Vec::new())
        }
        ConflictResolution::Overwrite => {
            // Proceed with all conflicts (will overwrite)
            Ok(conflicts)
        }
        ConflictResolution::Backup => {
            // Backup each file before proceeding
            for conflict in &conflicts {
                let backup_path = generate_backup_path(&conflict.destination);

                fs::copy(&conflict.destination, &backup_path)?;
                println!(
                    "  Backed up {} to {}",
                    conflict.destination.display(),
                    backup_path.display()
                );
            }
            Ok(conflicts)
        }
        ConflictResolution::Ask => {
            // Ask user for each conflicting file
            let mut resolved = Vec::new();

            for conflict in conflicts {
                let choices = vec![
                    "Skip this file",
                    "Overwrite (no backup)",
                    "Backup and overwrite",
                ];

                let selection = Select::new()
                    .with_prompt(format!(
                        "File exists: {} - What would you like to do?",
                        conflict.destination.display()
                    ))
                    .items(&choices)
                    .default(0)
                    .interact()?;

                match selection {
                    0 => {
                        // Skip
                        println!("  Skipping {}", conflict.destination.display());
                    }
                    1 => {
                        // Overwrite
                        resolved.push(conflict);
                    }
                    2 => {
                        // Backup and overwrite
                        let backup_path = generate_backup_path(&conflict.destination);

                        fs::copy(&conflict.destination, &backup_path)?;
                        println!("  Backed up to {}", backup_path.display());
                        resolved.push(conflict);
                    }
                    _ => unreachable!(),
                }
            }

            Ok(resolved)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_import_result(temp_dir: &TempDir, create_files: bool) -> ImportResult {
        let source1 = temp_dir.path().join("source1.txt");
        let source2 = temp_dir.path().join("source2.txt");
        let dest1 = temp_dir.path().join("dest1.txt");
        let dest2 = temp_dir.path().join("dest2.txt");

        // Create source files
        File::create(&source1).unwrap();
        File::create(&source2).unwrap();

        // Optionally create destination files (to simulate conflicts)
        if create_files {
            File::create(&dest1)
                .unwrap()
                .write_all(b"existing content")
                .unwrap();
            File::create(&dest2)
                .unwrap()
                .write_all(b"existing content")
                .unwrap();
        }

        ImportResult {
            tool: DotfileTool::Manual,
            dotfiles: vec![
                DotfileMapping {
                    source: source1,
                    destination: dest1,
                    category: Some("shell".to_string()),
                },
                DotfileMapping {
                    source: source2,
                    destination: dest2,
                    category: Some("editor".to_string()),
                },
            ],
            packages: vec![],
            stow_compat: false,
        }
    }

    #[test]
    fn test_detect_conflicts_none() {
        let temp_dir = TempDir::new().unwrap();
        let import_result = create_test_import_result(&temp_dir, false);

        let conflicts = detect_conflicts(&import_result);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_some() {
        let temp_dir = TempDir::new().unwrap();
        let import_result = create_test_import_result(&temp_dir, true);

        let conflicts = detect_conflicts(&import_result);
        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_resolve_conflicts_skip() {
        let temp_dir = TempDir::new().unwrap();
        let import_result = create_test_import_result(&temp_dir, true);
        let conflicts = detect_conflicts(&import_result);

        let resolved = resolve_conflicts(conflicts, &ConflictResolution::Skip).unwrap();
        assert_eq!(resolved.len(), 0);
    }

    #[test]
    fn test_resolve_conflicts_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let import_result = create_test_import_result(&temp_dir, true);
        let conflicts = detect_conflicts(&import_result);

        let resolved =
            resolve_conflicts(conflicts.clone(), &ConflictResolution::Overwrite).unwrap();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].destination, conflicts[0].destination);
        assert_eq!(resolved[1].destination, conflicts[1].destination);
    }

    #[test]
    fn test_resolve_conflicts_backup() {
        let temp_dir = TempDir::new().unwrap();
        let import_result = create_test_import_result(&temp_dir, true);
        let conflicts = detect_conflicts(&import_result);

        let resolved = resolve_conflicts(conflicts.clone(), &ConflictResolution::Backup).unwrap();
        assert_eq!(resolved.len(), 2);

        // Verify backup files were created
        let backup1 = conflicts[0].destination.with_extension("txt.backup");
        let backup2 = conflicts[1].destination.with_extension("txt.backup");
        assert!(backup1.exists(), "Backup file should exist: {:?}", backup1);
        assert!(backup2.exists(), "Backup file should exist: {:?}", backup2);
    }
}
