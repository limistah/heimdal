pub mod commit;
pub mod messages;
pub mod sync;
pub mod tracking;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// Re-export sync types
pub use sync::{SyncOptions, SyncResult};

/// Represents a Git repository
pub struct GitRepo {
    path: PathBuf,
}

impl GitRepo {
    /// Create a new GitRepo instance
    pub fn new(path: &Path) -> Result<Self> {
        if !path.join(".git").exists() {
            anyhow::bail!("Not a git repository: {}", path.display());
        }
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Check if the repo has uncommitted changes
    pub fn has_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .context("Failed to run git status")?;

        Ok(!output.stdout.is_empty())
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("branch")
            .arg("--show-current")
            .output()
            .context("Failed to get current branch")?;

        let branch = String::from_utf8(output.stdout)?.trim().to_string();

        Ok(branch)
    }

    /// Check if we're ahead of remote
    pub fn is_ahead_of_remote(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("rev-list")
            .arg("@{u}..")
            .output();

        match output {
            Ok(out) => Ok(!out.stdout.is_empty()),
            Err(_) => Ok(false), // No upstream configured
        }
    }

    /// Check if we're behind remote
    pub fn is_behind_remote(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("rev-list")
            .arg("..@{u}")
            .output();

        match output {
            Ok(out) => Ok(!out.stdout.is_empty()),
            Err(_) => Ok(false), // No upstream configured
        }
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("branch")
            .arg("--list")
            .arg("--format=%(refname:short)")
            .output()
            .context("Failed to list branches")?;

        if !output.status.success() {
            anyhow::bail!("Failed to list branches");
        }

        let branches = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(branches)
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("branch")
            .arg(name)
            .output()
            .context("Failed to create branch")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create branch: {}", err);
        }

        Ok(())
    }

    /// Switch to a branch
    pub fn switch_branch(&self, name: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("checkout")
            .arg(name)
            .output()
            .context("Failed to switch branch")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to switch branch: {}", err);
        }

        Ok(())
    }

    /// List all remotes
    pub fn list_remotes(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("remote")
            .output()
            .context("Failed to list remotes")?;

        if !output.status.success() {
            anyhow::bail!("Failed to list remotes");
        }

        let remotes = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(remotes)
    }

    /// Get remote URL
    pub fn get_remote_url(&self, remote: &str) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("remote")
            .arg("get-url")
            .arg(remote)
            .output()
            .context("Failed to get remote URL")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get remote URL: {}", err);
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Add a remote
    pub fn add_remote(&self, name: &str, url: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("remote")
            .arg("add")
            .arg(name)
            .arg(url)
            .output()
            .context("Failed to add remote")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to add remote: {}", err);
        }

        Ok(())
    }

    /// Remove a remote
    pub fn remove_remote(&self, name: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("remote")
            .arg("remove")
            .arg(name)
            .output()
            .context("Failed to remove remote")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove remote: {}", err);
        }

        Ok(())
    }

    /// Set remote URL
    pub fn set_remote_url(&self, name: &str, url: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("remote")
            .arg("set-url")
            .arg(name)
            .arg(url)
            .output()
            .context("Failed to set remote URL")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to set remote URL: {}", err);
        }

        Ok(())
    }

    /// Check if a remote exists
    pub fn has_remote(&self, name: &str) -> Result<bool> {
        let remotes = self.list_remotes()?;
        Ok(remotes.contains(&name.to_string()))
    }

    /// Push to remote with optional remote and branch specification
    pub fn push_to(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.path).arg("push");

        if let Some(r) = remote {
            cmd.arg(r);
            if let Some(b) = branch {
                cmd.arg(b);
            }
        }

        let output = cmd.output().context("Failed to push to remote")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Push failed: {}", stderr);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_repo_detection() {
        let temp = TempDir::new().unwrap();
        let result = GitRepo::new(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_git_repo_initialization() {
        let temp = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(temp.path())
            .output()
            .unwrap();

        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(temp.path())
            .output()
            .unwrap();

        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(temp.path())
            .output()
            .unwrap();

        let repo = GitRepo::new(temp.path()).unwrap();
        assert!(!repo.has_changes().unwrap());
    }
}
