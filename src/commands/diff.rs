use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Select};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::state::HeimdallState;
use crate::utils::{header, info, success, warning};

/// Type of change detected
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// File has been modified
    Modified,
    /// File has been added (new file)
    Added,
    /// File has been deleted
    Deleted,
    /// File has been renamed
    Renamed,
    /// File type changed (file -> symlink, etc.)
    TypeChanged,
}

impl ChangeType {
    /// Get a colored icon for display
    fn icon(&self) -> ColoredString {
        match self {
            ChangeType::Modified => "M".yellow(),
            ChangeType::Added => "A".green(),
            ChangeType::Deleted => "D".red(),
            ChangeType::Renamed => "R".blue(),
            ChangeType::TypeChanged => "T".magenta(),
        }
    }

    /// Get a description of the change type
    fn description(&self) -> &str {
        match self {
            ChangeType::Modified => "modified",
            ChangeType::Added => "added",
            ChangeType::Deleted => "deleted",
            ChangeType::Renamed => "renamed",
            ChangeType::TypeChanged => "type changed",
        }
    }
}

/// Information about a changed file
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Path to the file relative to dotfiles directory
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Old path (for renames)
    pub old_path: Option<PathBuf>,
    /// Number of lines added (if applicable)
    pub lines_added: Option<usize>,
    /// Number of lines removed (if applicable)
    pub lines_removed: Option<usize>,
}

/// Summary of all changes
#[derive(Debug)]
pub struct DiffSummary {
    pub modified: Vec<FileChange>,
    pub added: Vec<FileChange>,
    pub deleted: Vec<FileChange>,
    pub renamed: Vec<FileChange>,
    pub untracked: Vec<PathBuf>,
}

impl DiffSummary {
    /// Create an empty diff summary
    fn new() -> Self {
        Self {
            modified: Vec::new(),
            added: Vec::new(),
            deleted: Vec::new(),
            renamed: Vec::new(),
            untracked: Vec::new(),
        }
    }

    /// Check if there are any changes
    fn has_changes(&self) -> bool {
        !self.modified.is_empty()
            || !self.added.is_empty()
            || !self.deleted.is_empty()
            || !self.renamed.is_empty()
            || !self.untracked.is_empty()
    }

    /// Get total number of changes
    fn total_changes(&self) -> usize {
        self.modified.len()
            + self.added.len()
            + self.deleted.len()
            + self.renamed.len()
            + self.untracked.len()
    }

    /// Display the diff summary
    fn display(&self, verbose: bool) {
        if !self.has_changes() {
            println!();
            println!("{}", "No changes detected.".green());
            println!();
            return;
        }

        println!();
        println!(
            "{} ({} total)",
            "Changes Detected".bold().yellow(),
            self.total_changes()
        );
        println!("{}", "─".repeat(60).bright_black());

        // Display modified files
        if !self.modified.is_empty() {
            println!();
            println!("{} ({} files)", "Modified".bold(), self.modified.len());
            for change in &self.modified {
                self.display_change(change, verbose);
            }
        }

        // Display added files
        if !self.added.is_empty() {
            println!();
            println!("{} ({} files)", "Added".bold().green(), self.added.len());
            for change in &self.added {
                self.display_change(change, verbose);
            }
        }

        // Display deleted files
        if !self.deleted.is_empty() {
            println!();
            println!("{} ({} files)", "Deleted".bold().red(), self.deleted.len());
            for change in &self.deleted {
                self.display_change(change, verbose);
            }
        }

        // Display renamed files
        if !self.renamed.is_empty() {
            println!();
            println!("{} ({} files)", "Renamed".bold().blue(), self.renamed.len());
            for change in &self.renamed {
                self.display_change(change, verbose);
            }
        }

        // Display untracked files
        if !self.untracked.is_empty() {
            println!();
            println!(
                "{} ({} files)",
                "Untracked".bold().bright_black(),
                self.untracked.len()
            );
            for path in &self.untracked {
                println!("  {} {}", "?".bright_black(), path.display());
            }
        }

        println!();
    }

    /// Display a single file change
    fn display_change(&self, change: &FileChange, verbose: bool) {
        let icon = change.change_type.icon();
        let path = change.path.display();

        if verbose {
            // Verbose mode: show line counts
            if let (Some(added), Some(removed)) = (change.lines_added, change.lines_removed) {
                println!(
                    "  {} {} (+{} -{})",
                    icon,
                    path,
                    added.to_string().green(),
                    removed.to_string().red()
                );
            } else {
                println!("  {} {}", icon, path);
            }

            // Show old path for renames
            if let Some(old_path) = &change.old_path {
                println!("      {} {}", "from".bright_black(), old_path.display());
            }
        } else {
            // Normal mode: just show file
            println!("  {} {}", icon, path);
        }
    }
}

/// Run the diff command
pub fn run_diff(verbose: bool, interactive: bool) -> Result<()> {
    header("Dotfile Changes");

    // Load state
    let state = HeimdallState::load()?;

    info(&format!("Repository: {}", state.dotfiles_path.display()));
    info(&format!("Profile: {}", state.active_profile));

    // Check if it's a git repository
    if !state.dotfiles_path.join(".git").exists() {
        anyhow::bail!(
            "Not a git repository: {}\nThe diff command requires git tracking.",
            state.dotfiles_path.display()
        );
    }

    // Get diff summary
    let summary = get_diff_summary(&state.dotfiles_path)?;

    // Display summary
    summary.display(verbose);

    if !summary.has_changes() {
        return Ok(());
    }

    // Interactive mode: offer actions
    if interactive {
        println!();
        offer_actions(&state.dotfiles_path, &summary)?;
    } else {
        // Non-interactive: just show tip
        println!();
        info("Tip: Use 'heimdal diff --interactive' to commit or discard changes");
    }

    Ok(())
}

/// Get diff summary from git
fn get_diff_summary(dotfiles_path: &Path) -> Result<DiffSummary> {
    let mut summary = DiffSummary::new();

    // Get git status --porcelain for machine-readable output
    let output = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("Failed to execute git status")?;

    if !output.status.success() {
        anyhow::bail!("Git status command failed");
    }

    let status_output = String::from_utf8_lossy(&output.stdout);

    // Parse git status output
    for line in status_output.lines() {
        if line.len() < 4 {
            continue;
        }

        let index_status = line.chars().nth(0).unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');
        let file_path = line[3..].trim();

        // Skip empty lines
        if file_path.is_empty() {
            continue;
        }

        let path = PathBuf::from(file_path);

        // Determine change type based on status codes
        match (index_status, worktree_status) {
            ('M', _) | (_, 'M') => {
                // Modified
                let (added, removed) = get_line_changes(dotfiles_path, &path)?;
                summary.modified.push(FileChange {
                    path,
                    change_type: ChangeType::Modified,
                    old_path: None,
                    lines_added: Some(added),
                    lines_removed: Some(removed),
                });
            }
            ('A', _) | (_, 'A') => {
                // Added
                summary.added.push(FileChange {
                    path,
                    change_type: ChangeType::Added,
                    old_path: None,
                    lines_added: None,
                    lines_removed: None,
                });
            }
            ('D', _) | (_, 'D') => {
                // Deleted
                summary.deleted.push(FileChange {
                    path,
                    change_type: ChangeType::Deleted,
                    old_path: None,
                    lines_added: None,
                    lines_removed: None,
                });
            }
            ('R', _) => {
                // Renamed
                let parts: Vec<&str> = file_path.split(" -> ").collect();
                if parts.len() == 2 {
                    summary.renamed.push(FileChange {
                        path: PathBuf::from(parts[1]),
                        change_type: ChangeType::Renamed,
                        old_path: Some(PathBuf::from(parts[0])),
                        lines_added: None,
                        lines_removed: None,
                    });
                }
            }
            ('?', '?') => {
                // Untracked
                summary.untracked.push(path);
            }
            _ => {
                // Other status codes we don't handle yet
            }
        }
    }

    Ok(summary)
}

/// Get line changes (additions/deletions) for a file
fn get_line_changes(dotfiles_path: &Path, file_path: &Path) -> Result<(usize, usize)> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("diff")
        .arg("--numstat")
        .arg("--")
        .arg(file_path)
        .output()
        .context("Failed to execute git diff")?;

    if !output.status.success() {
        return Ok((0, 0));
    }

    let diff_output = String::from_utf8_lossy(&output.stdout);
    let line = diff_output.lines().next().unwrap_or("");

    // Parse numstat output: "added\tremoved\tfilename"
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() >= 2 {
        let added = parts[0].parse::<usize>().unwrap_or(0);
        let removed = parts[1].parse::<usize>().unwrap_or(0);
        Ok((added, removed))
    } else {
        Ok((0, 0))
    }
}

/// Offer interactive actions for handling changes
fn offer_actions(dotfiles_path: &Path, summary: &DiffSummary) -> Result<()> {
    let actions = vec![
        "View detailed diff",
        "Commit all changes",
        "Commit specific files",
        "Discard all changes",
        "Discard specific files",
        "Exit (do nothing)",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(&actions)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // View detailed diff
            view_detailed_diff(dotfiles_path)?;
            // Offer actions again
            offer_actions(dotfiles_path, summary)?;
        }
        1 => {
            // Commit all changes
            commit_changes(dotfiles_path, None)?;
        }
        2 => {
            // Commit specific files
            let files = select_files_to_commit(summary)?;
            if !files.is_empty() {
                commit_changes(dotfiles_path, Some(files))?;
            }
        }
        3 => {
            // Discard all changes
            let confirm = Confirm::new()
                .with_prompt("Are you sure you want to discard ALL changes? This cannot be undone.")
                .default(false)
                .interact()?;

            if confirm {
                discard_changes(dotfiles_path, None)?;
            }
        }
        4 => {
            // Discard specific files
            let files = select_files_to_discard(summary)?;
            if !files.is_empty() {
                discard_changes(dotfiles_path, Some(files))?;
            }
        }
        5 => {
            // Exit
            info("Exiting without changes.");
        }
        _ => {}
    }

    Ok(())
}

/// View detailed git diff
fn view_detailed_diff(dotfiles_path: &Path) -> Result<()> {
    println!();
    info("Showing detailed diff...");
    println!();

    let status = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("diff")
        .arg("--color=always")
        .status()
        .context("Failed to execute git diff")?;

    if !status.success() {
        warning("Failed to show diff");
    }

    println!();
    Ok(())
}

/// Select files to commit from the summary
fn select_files_to_commit(summary: &DiffSummary) -> Result<Vec<PathBuf>> {
    use dialoguer::MultiSelect;

    let mut items = Vec::new();
    let mut paths = Vec::new();

    // Collect all changed files
    for change in &summary.modified {
        items.push(format!("{} {}", "M".yellow(), change.path.display()));
        paths.push(change.path.clone());
    }
    for change in &summary.added {
        items.push(format!("{} {}", "A".green(), change.path.display()));
        paths.push(change.path.clone());
    }
    for change in &summary.deleted {
        items.push(format!("{} {}", "D".red(), change.path.display()));
        paths.push(change.path.clone());
    }
    for change in &summary.renamed {
        items.push(format!("{} {}", "R".blue(), change.path.display()));
        paths.push(change.path.clone());
    }
    for path in &summary.untracked {
        items.push(format!("{} {}", "?".bright_black(), path.display()));
        paths.push(path.clone());
    }

    if items.is_empty() {
        return Ok(Vec::new());
    }

    let selections = MultiSelect::new()
        .with_prompt("Select files to commit (space to select, enter to confirm)")
        .items(&items)
        .interact()?;

    let selected_files: Vec<PathBuf> = selections.iter().map(|&i| paths[i].clone()).collect();

    Ok(selected_files)
}

/// Select files to discard from the summary
fn select_files_to_discard(summary: &DiffSummary) -> Result<Vec<PathBuf>> {
    use dialoguer::MultiSelect;

    let mut items = Vec::new();
    let mut paths = Vec::new();

    // Only include modified and deleted files (not added/untracked)
    for change in &summary.modified {
        items.push(format!("{} {}", "M".yellow(), change.path.display()));
        paths.push(change.path.clone());
    }
    for change in &summary.deleted {
        items.push(format!("{} {}", "D".red(), change.path.display()));
        paths.push(change.path.clone());
    }

    if items.is_empty() {
        warning("No files to discard (only modified/deleted files can be discarded)");
        return Ok(Vec::new());
    }

    let selections = MultiSelect::new()
        .with_prompt("Select files to discard (space to select, enter to confirm)")
        .items(&items)
        .interact()?;

    let selected_files: Vec<PathBuf> = selections.iter().map(|&i| paths[i].clone()).collect();

    Ok(selected_files)
}

/// Commit changes to git
fn commit_changes(dotfiles_path: &Path, files: Option<Vec<PathBuf>>) -> Result<()> {
    use dialoguer::Input;

    // Get commit message
    let message: String = Input::new().with_prompt("Commit message").interact_text()?;

    if message.trim().is_empty() {
        warning("Commit message cannot be empty. Cancelled.");
        return Ok(());
    }

    // Add files
    if let Some(file_list) = files {
        for file in file_list {
            let status = Command::new("git")
                .arg("-C")
                .arg(dotfiles_path)
                .arg("add")
                .arg(&file)
                .status()
                .context("Failed to execute git add")?;

            if !status.success() {
                warning(&format!("Failed to add file: {}", file.display()));
            }
        }
    } else {
        // Add all changes
        let status = Command::new("git")
            .arg("-C")
            .arg(dotfiles_path)
            .arg("add")
            .arg(".")
            .status()
            .context("Failed to execute git add")?;

        if !status.success() {
            anyhow::bail!("Failed to add files");
        }
    }

    // Commit
    let status = Command::new("git")
        .arg("-C")
        .arg(dotfiles_path)
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .status()
        .context("Failed to execute git commit")?;

    if status.success() {
        success("Changes committed successfully!");

        // Ask if user wants to push
        let push = Confirm::new()
            .with_prompt("Push to remote?")
            .default(false)
            .interact()?;

        if push {
            info("Pushing to remote...");
            let push_status = Command::new("git")
                .arg("-C")
                .arg(dotfiles_path)
                .arg("push")
                .status()
                .context("Failed to execute git push")?;

            if push_status.success() {
                success("Pushed to remote successfully!");
            } else {
                warning("Failed to push. You can push manually later.");
            }
        }
    } else {
        warning("Commit failed. Check git status.");
    }

    Ok(())
}

/// Discard changes using git restore
fn discard_changes(dotfiles_path: &Path, files: Option<Vec<PathBuf>>) -> Result<()> {
    if let Some(file_list) = files {
        for file in file_list {
            let status = Command::new("git")
                .arg("-C")
                .arg(dotfiles_path)
                .arg("restore")
                .arg(&file)
                .status()
                .context("Failed to execute git restore")?;

            if !status.success() {
                warning(&format!(
                    "Failed to discard changes for: {}",
                    file.display()
                ));
            }
        }
        success("Selected changes discarded.");
    } else {
        // Discard all changes
        let status = Command::new("git")
            .arg("-C")
            .arg(dotfiles_path)
            .arg("restore")
            .arg(".")
            .status()
            .context("Failed to execute git restore")?;

        if status.success() {
            success("All changes discarded.");
        } else {
            warning("Failed to discard changes.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_icon() {
        assert_eq!(
            ChangeType::Modified.icon().to_string(),
            "M".yellow().to_string()
        );
        assert_eq!(
            ChangeType::Added.icon().to_string(),
            "A".green().to_string()
        );
        assert_eq!(
            ChangeType::Deleted.icon().to_string(),
            "D".red().to_string()
        );
    }

    #[test]
    fn test_change_type_description() {
        assert_eq!(ChangeType::Modified.description(), "modified");
        assert_eq!(ChangeType::Added.description(), "added");
        assert_eq!(ChangeType::Deleted.description(), "deleted");
        assert_eq!(ChangeType::Renamed.description(), "renamed");
    }

    #[test]
    fn test_file_change_creation() {
        let change = FileChange {
            path: PathBuf::from("test.txt"),
            change_type: ChangeType::Modified,
            old_path: None,
            lines_added: Some(10),
            lines_removed: Some(5),
        };

        assert_eq!(change.path, PathBuf::from("test.txt"));
        assert_eq!(change.change_type, ChangeType::Modified);
        assert_eq!(change.lines_added, Some(10));
        assert_eq!(change.lines_removed, Some(5));
    }

    #[test]
    fn test_diff_summary_empty() {
        let summary = DiffSummary::new();
        assert!(!summary.has_changes());
        assert_eq!(summary.total_changes(), 0);
    }

    #[test]
    fn test_diff_summary_with_changes() {
        let mut summary = DiffSummary::new();
        summary.modified.push(FileChange {
            path: PathBuf::from("file1.txt"),
            change_type: ChangeType::Modified,
            old_path: None,
            lines_added: Some(5),
            lines_removed: Some(2),
        });
        summary.added.push(FileChange {
            path: PathBuf::from("file2.txt"),
            change_type: ChangeType::Added,
            old_path: None,
            lines_added: None,
            lines_removed: None,
        });

        assert!(summary.has_changes());
        assert_eq!(summary.total_changes(), 2);
    }

    #[test]
    fn test_diff_summary_untracked() {
        let mut summary = DiffSummary::new();
        summary.untracked.push(PathBuf::from("new_file.txt"));

        assert!(summary.has_changes());
        assert_eq!(summary.total_changes(), 1);
    }
}
