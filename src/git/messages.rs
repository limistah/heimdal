use super::tracking::{ChangeType, FileChange};
use anyhow::Result;

/// Generate a commit message based on file changes
pub fn generate_commit_message(changes: &[FileChange]) -> Result<String> {
    if changes.is_empty() {
        return Ok("Update configuration".to_string());
    }

    // Categorize changes
    let modified: Vec<_> = changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Modified)
        .collect();
    let added: Vec<_> = changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Added)
        .collect();
    let deleted: Vec<_> = changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Deleted)
        .collect();
    let renamed: Vec<_> = changes
        .iter()
        .filter(|c| c.change_type == ChangeType::Renamed)
        .collect();

    // Generate message based on changes
    let message = if changes.len() == 1 {
        // Single file change - be specific
        let change = &changes[0];
        let file_name = get_file_name(&change.path);

        match change.change_type {
            ChangeType::Modified => format!("Update {}", file_name),
            ChangeType::Added => format!("Add {}", file_name),
            ChangeType::Deleted => format!("Remove {}", file_name),
            ChangeType::Renamed => format!("Rename {}", file_name),
            _ => format!("Update {}", file_name),
        }
    } else if changes.len() <= 3 {
        // Few files - list them
        let files: Vec<String> = changes.iter().map(|c| get_file_name(&c.path)).collect();
        format!("Update {}", files.join(", "))
    } else {
        // Many files - summarize by type
        let mut parts = Vec::new();

        if !modified.is_empty() {
            parts.push(format!(
                "update {} file{}",
                modified.len(),
                plural(modified.len())
            ));
        }
        if !added.is_empty() {
            parts.push(format!("add {} file{}", added.len(), plural(added.len())));
        }
        if !deleted.is_empty() {
            parts.push(format!(
                "remove {} file{}",
                deleted.len(),
                plural(deleted.len())
            ));
        }
        if !renamed.is_empty() {
            parts.push(format!(
                "rename {} file{}",
                renamed.len(),
                plural(renamed.len())
            ));
        }

        if parts.is_empty() {
            "Update configuration".to_string()
        } else {
            capitalize(&parts.join(", "))
        }
    };

    Ok(message)
}

/// Generate a detailed commit message with body
#[allow(dead_code)]
pub fn generate_detailed_message(changes: &[FileChange]) -> Result<String> {
    let subject = generate_commit_message(changes)?;

    if changes.len() <= 1 {
        return Ok(subject);
    }

    let mut body = Vec::new();
    body.push(subject);
    body.push(String::new()); // Blank line

    // Add file details
    for change in changes {
        let line = match &change.old_path {
            Some(old) => format!(
                "- {} {} → {}",
                change.change_type.symbol(),
                old,
                change.path
            ),
            None => format!("- {} {}", change.change_type.symbol(), change.path),
        };
        body.push(line);
    }

    Ok(body.join("\n"))
}

/// Get just the filename from a path
fn get_file_name(path: &str) -> String {
    path.split('/').next_back().unwrap_or(path).to_string()
}

/// Get plural suffix
fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_change(path: &str, change_type: ChangeType) -> FileChange {
        FileChange {
            path: path.to_string(),
            change_type,
            lines_added: None,
            lines_removed: None,
            old_path: None,
        }
    }

    #[test]
    fn test_single_modified_file() {
        let changes = vec![mock_change(".zshrc", ChangeType::Modified)];
        let message = generate_commit_message(&changes).unwrap();
        assert_eq!(message, "Update .zshrc");
    }

    #[test]
    fn test_single_added_file() {
        let changes = vec![mock_change("new_file.txt", ChangeType::Added)];
        let message = generate_commit_message(&changes).unwrap();
        assert_eq!(message, "Add new_file.txt");
    }

    #[test]
    fn test_multiple_files() {
        let changes = vec![
            mock_change(".zshrc", ChangeType::Modified),
            mock_change(".vimrc", ChangeType::Modified),
        ];
        let message = generate_commit_message(&changes).unwrap();
        assert_eq!(message, "Update .zshrc, .vimrc");
    }

    #[test]
    fn test_many_files() {
        let changes = vec![
            mock_change("file1.txt", ChangeType::Modified),
            mock_change("file2.txt", ChangeType::Modified),
            mock_change("file3.txt", ChangeType::Added),
            mock_change("file4.txt", ChangeType::Added),
            mock_change("file5.txt", ChangeType::Deleted),
        ];
        let message = generate_commit_message(&changes).unwrap();
        // Message is capitalized and parts are comma-separated
        assert!(message.to_lowercase().contains("update 2 files"));
        assert!(message.to_lowercase().contains("add 2 files"));
        assert!(message.to_lowercase().contains("remove 1 file"));
    }

    #[test]
    fn test_get_file_name() {
        assert_eq!(get_file_name(".zshrc"), ".zshrc");
        assert_eq!(get_file_name("src/main.rs"), "main.rs");
        assert_eq!(get_file_name("a/b/c/file.txt"), "file.txt");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("world"), "World");
        assert_eq!(capitalize(""), "");
    }
}
