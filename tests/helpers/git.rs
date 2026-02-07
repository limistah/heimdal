/// Git-related test helpers
use std::path::Path;
use std::process::Command;

/// Initialize a git repository in the given directory
pub fn init_repo(path: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git init: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Configure git user for testing
pub fn config_user(path: &Path, name: &str, email: &str) -> Result<(), String> {
    Command::new("git")
        .args(&["config", "user.name", name])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to set git user.name: {}", e))?;

    Command::new("git")
        .args(&["config", "user.email", email])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to set git user.email: {}", e))?;

    Ok(())
}

/// Create a commit in the repository
pub fn commit(path: &Path, message: &str) -> Result<(), String> {
    Command::new("git")
        .args(&["add", "."])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to git add: {}", e))?;

    let output = Command::new("git")
        .args(&["commit", "-m", message])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to git commit: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Check if repository is clean (no uncommitted changes)
pub fn is_clean(path: &Path) -> bool {
    if let Ok(output) = Command::new("git")
        .args(&["status", "--porcelain"])
        .current_dir(path)
        .output()
    {
        output.stdout.is_empty()
    } else {
        false
    }
}

/// Get the current branch name
pub fn current_branch(path: &Path) -> Option<String> {
    Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

/// Count commits in the repository
pub fn commit_count(path: &Path) -> usize {
    Command::new("git")
        .args(&["rev-list", "--count", "HEAD"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}
