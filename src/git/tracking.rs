use anyhow::{Context, Result};
use std::process::Command;

use super::GitRepo;

/// Type of file change
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Modified,
    Added,
    Deleted,
    Renamed,
    TypeChanged,
    Untracked,
}

impl ChangeType {
    /// Get color for display
    pub fn color(&self) -> colored::Color {
        use colored::Color;
        match self {
            ChangeType::Modified => Color::Yellow,
            ChangeType::Added => Color::Green,
            ChangeType::Deleted => Color::Red,
            ChangeType::Renamed => Color::Cyan,
            ChangeType::TypeChanged => Color::Magenta,
            ChangeType::Untracked => Color::BrightBlack,
        }
    }

    /// Get symbol for display
    pub fn symbol(&self) -> &str {
        match self {
            ChangeType::Modified => "M",
            ChangeType::Added => "A",
            ChangeType::Deleted => "D",
            ChangeType::Renamed => "R",
            ChangeType::TypeChanged => "T",
            ChangeType::Untracked => "?",
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            ChangeType::Modified => "Modified",
            ChangeType::Added => "Added",
            ChangeType::Deleted => "Deleted",
            ChangeType::Renamed => "Renamed",
            ChangeType::TypeChanged => "Type changed",
            ChangeType::Untracked => "Untracked",
        }
    }
}

/// Represents a file change in the repository
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: Option<usize>,
    pub lines_removed: Option<usize>,
    pub old_path: Option<String>, // For renames
}

impl FileChange {
    /// Format change for display
    pub fn format(&self) -> String {
        use colored::Colorize;

        let symbol = self.change_type.symbol().color(self.change_type.color());
        let path = &self.path;

        if let Some(old_path) = &self.old_path {
            format!("{} {} → {}", symbol, old_path, path)
        } else {
            format!("{} {}", symbol, path)
        }
    }

    /// Get line statistics string
    pub fn line_stats(&self) -> Option<String> {
        match (self.lines_added, self.lines_removed) {
            (Some(added), Some(removed)) => {
                use colored::Colorize;
                Some(format!(
                    "+{} -{}",
                    added.to_string().green(),
                    removed.to_string().red()
                ))
            }
            _ => None,
        }
    }
}

impl GitRepo {
    /// Get all file changes in the repository
    pub fn get_changes(&self) -> Result<Vec<FileChange>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("status")
            .arg("--porcelain")
            .arg("-u")
            .output()
            .context("Failed to run git status")?;

        let status_output = String::from_utf8(output.stdout)?;
        let mut changes = Vec::new();

        for line in status_output.lines() {
            if line.is_empty() {
                continue;
            }

            let change = parse_status_line(line)?;
            changes.push(change);
        }

        Ok(changes)
    }

    /// Get detailed changes with line counts
    pub fn get_changes_with_stats(&self) -> Result<Vec<FileChange>> {
        let mut changes = self.get_changes()?;

        // Get numstat for modified files
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("diff")
            .arg("--numstat")
            .arg("HEAD")
            .output()
            .context("Failed to run git diff --numstat")?;

        let numstat_output = String::from_utf8(output.stdout)?;
        let stats = parse_numstat(&numstat_output);

        // Match stats with changes
        for change in &mut changes {
            if let Some((added, removed)) = stats.get(&change.path) {
                change.lines_added = Some(*added);
                change.lines_removed = Some(*removed);
            }
        }

        Ok(changes)
    }

    /// Get list of tracked files that are modified
    pub fn get_modified_files(&self) -> Result<Vec<String>> {
        let changes = self.get_changes()?;
        Ok(changes
            .into_iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .map(|c| c.path)
            .collect())
    }

    /// Get list of untracked files
    pub fn get_untracked_files(&self) -> Result<Vec<String>> {
        let changes = self.get_changes()?;
        Ok(changes
            .into_iter()
            .filter(|c| c.change_type == ChangeType::Untracked)
            .map(|c| c.path)
            .collect())
    }
}

/// Parse a single line from git status --porcelain
fn parse_status_line(line: &str) -> Result<FileChange> {
    if line.len() < 3 {
        anyhow::bail!("Invalid status line: {}", line);
    }

    let index_status = line.chars().next().unwrap_or(' ');
    let worktree_status = line.chars().nth(1).unwrap_or(' ');
    let path = line[3..].trim();

    // Determine change type based on status codes
    let change_type = match (index_status, worktree_status) {
        ('M', _) | (_, 'M') => ChangeType::Modified,
        ('A', _) | (_, 'A') => ChangeType::Added,
        ('D', _) | (_, 'D') => ChangeType::Deleted,
        ('R', _) | (_, 'R') => ChangeType::Renamed,
        ('T', _) | (_, 'T') => ChangeType::TypeChanged,
        ('?', '?') => ChangeType::Untracked,
        _ => ChangeType::Modified, // Default
    };

    // Handle renames (path format: "old -> new")
    let (path, old_path) = if path.contains(" -> ") {
        let parts: Vec<&str> = path.split(" -> ").collect();
        if parts.len() == 2 {
            (parts[1].to_string(), Some(parts[0].to_string()))
        } else {
            (path.to_string(), None)
        }
    } else {
        (path.to_string(), None)
    };

    Ok(FileChange {
        path,
        change_type,
        lines_added: None,
        lines_removed: None,
        old_path,
    })
}

/// Parse git diff --numstat output
fn parse_numstat(output: &str) -> std::collections::HashMap<String, (usize, usize)> {
    let mut stats = std::collections::HashMap::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(added), Ok(removed)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>())
            {
                let file = parts[2].to_string();
                stats.insert(file, (added, removed));
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_line_modified() {
        let line = " M README.md";
        let change = parse_status_line(line).unwrap();
        assert_eq!(change.change_type, ChangeType::Modified);
        assert_eq!(change.path, "README.md");
    }

    #[test]
    fn test_parse_status_line_added() {
        let line = "A  new_file.txt";
        let change = parse_status_line(line).unwrap();
        assert_eq!(change.change_type, ChangeType::Added);
        assert_eq!(change.path, "new_file.txt");
    }

    #[test]
    fn test_parse_status_line_untracked() {
        let line = "?? untracked.txt";
        let change = parse_status_line(line).unwrap();
        assert_eq!(change.change_type, ChangeType::Untracked);
        assert_eq!(change.path, "untracked.txt");
    }

    #[test]
    fn test_change_type_symbols() {
        assert_eq!(ChangeType::Modified.symbol(), "M");
        assert_eq!(ChangeType::Added.symbol(), "A");
        assert_eq!(ChangeType::Deleted.symbol(), "D");
    }
}
