use anyhow::{Context, Result};
use std::process::Command;

use super::GitRepo;
use crate::utils::{info, step, success};

/// Commit options
pub struct CommitOptions {
    pub message: String,
    pub files: Option<Vec<String>>,
    pub push: bool,
    pub dry_run: bool,
}

impl GitRepo {
    /// Commit changes to the repository
    pub fn commit(&self, options: &CommitOptions) -> Result<()> {
        if options.dry_run {
            step("Dry-run: Would commit changes");
            info(&format!("Message: {}", options.message));
            if let Some(files) = &options.files {
                info(&format!("Files: {}", files.join(", ")));
            } else {
                info("Files: all staged files");
            }
            return Ok(());
        }

        // Add files to staging
        if let Some(files) = &options.files {
            step("Staging specific files...");
            for file in files {
                self.add_file(file)?;
            }
        } else {
            step("Staging all changes...");
            self.add_all()?;
        }

        // Check if there's anything to commit
        if !self.has_staged_changes()? {
            info("No changes to commit");
            return Ok(());
        }

        // Commit
        step("Creating commit...");
        self.commit_staged(&options.message)?;
        success(&format!("Committed: {}", options.message));

        // Push if requested
        if options.push {
            step("Pushing to remote...");
            self.push()?;
            success("Pushed to remote");
        }

        Ok(())
    }

    /// Add a specific file to staging
    pub fn add_file(&self, file: &str) -> Result<()> {
        Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("add")
            .arg(file)
            .output()
            .context(format!("Failed to add file: {}", file))?;
        Ok(())
    }

    /// Add all changes to staging
    pub fn add_all(&self) -> Result<()> {
        Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("add")
            .arg("-A")
            .output()
            .context("Failed to add all files")?;
        Ok(())
    }

    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("diff")
            .arg("--cached")
            .arg("--quiet")
            .status()
            .context("Failed to check staged changes")?;

        // Exit code 1 means there are differences (staged changes)
        Ok(!output.success())
    }

    /// Commit staged changes
    fn commit_staged(&self, message: &str) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()
            .context("Failed to create commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Commit failed: {}", stderr);
        }

        Ok(())
    }

    /// Push to remote
    pub fn push(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("push")
            .output()
            .context("Failed to push to remote")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Push failed: {}", stderr);
        }

        Ok(())
    }

    /// Pull from remote
    pub fn pull(&self, rebase: bool) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.path).arg("pull");

        if rebase {
            cmd.arg("--rebase");
        }

        let output = cmd.output().context("Failed to pull from remote")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Pull failed: {}", stderr);
        }

        Ok(())
    }

    /// Commit with auto-generated message
    pub fn commit_auto(&self, push: bool, dry_run: bool) -> Result<()> {
        use super::messages;

        let changes = self.get_changes_with_stats()?;
        if changes.is_empty() {
            info("No changes to commit");
            return Ok(());
        }

        let message = messages::generate_commit_message(&changes)?;

        let options = CommitOptions {
            message,
            files: None,
            push,
            dry_run,
        };

        self.commit(&options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, GitRepo) {
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
        (temp, repo)
    }

    #[test]
    fn test_add_file() {
        let (temp, repo) = setup_test_repo();

        // Create a file
        fs::write(temp.path().join("test.txt"), "hello").unwrap();

        // Add file
        repo.add_file("test.txt").unwrap();

        // Check if staged
        assert!(repo.has_staged_changes().unwrap());
    }

    #[test]
    fn test_commit() {
        let (temp, repo) = setup_test_repo();

        // Create and add a file
        fs::write(temp.path().join("test.txt"), "hello").unwrap();
        repo.add_file("test.txt").unwrap();

        // Commit
        let options = CommitOptions {
            message: "Test commit".to_string(),
            files: None,
            push: false,
            dry_run: false,
        };

        repo.commit(&options).unwrap();

        // Verify commit exists
        let output = Command::new("git")
            .arg("-C")
            .arg(repo.path())
            .arg("log")
            .arg("--oneline")
            .output()
            .unwrap();

        let log = String::from_utf8(output.stdout).unwrap();
        assert!(log.contains("Test commit"));
    }
}
