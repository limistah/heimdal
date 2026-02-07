use anyhow::{Context, Result};
use std::process::Command;

use super::GitRepo;
use crate::utils::{error, info, step, success, warning};

/// Sync operation result
#[derive(Debug, Clone, PartialEq)]
pub enum SyncResult {
    Success,
    Conflicts(Vec<String>),
    NothingToSync,
    UpToDate,
}

/// Sync options
pub struct SyncOptions {
    pub pull: bool,
    pub push: bool,
    pub rebase: bool,
    pub auto_stash: bool,
    pub dry_run: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            pull: true,
            push: false,
            rebase: false,
            auto_stash: false,
            dry_run: false,
        }
    }
}

impl GitRepo {
    /// Sync repository with remote (pull and optionally push)
    pub fn sync(&self, options: &SyncOptions) -> Result<SyncResult> {
        if options.dry_run {
            step("Dry-run: Checking sync status...");

            if options.pull {
                if self.is_behind_remote()? {
                    info("Would pull changes from remote");
                } else {
                    info("Already up to date with remote");
                }
            }

            if options.push && self.is_ahead_of_remote()? {
                info("Would push local commits to remote");
            }

            return Ok(SyncResult::NothingToSync);
        }

        // Check if we have local changes
        if self.has_changes()? {
            if options.auto_stash {
                step("Stashing local changes...");
                self.stash()?;
            } else {
                warning("You have uncommitted changes. Commit or stash them first.");
                return Err(anyhow::anyhow!(
                    "Uncommitted changes. Use --auto-stash or commit first."
                ));
            }
        }

        let mut result = SyncResult::UpToDate;

        // Pull from remote
        if options.pull {
            if self.is_behind_remote()? {
                step("Pulling changes from remote...");
                self.pull(options.rebase)?;

                // Check for conflicts
                if self.has_merge_conflicts()? {
                    let conflicts = self.get_conflicted_files()?;
                    error(&format!(
                        "Merge conflicts detected in {} files",
                        conflicts.len()
                    ));
                    for file in &conflicts {
                        error(&format!("  - {}", file));
                    }
                    return Ok(SyncResult::Conflicts(conflicts));
                }

                success("Pulled changes successfully");
                result = SyncResult::Success;
            } else {
                info("Already up to date with remote");
            }
        }

        // Pop stash if we stashed
        if options.auto_stash && self.has_stash()? {
            step("Restoring stashed changes...");
            self.stash_pop()?;
        }

        // Push to remote
        if options.push && self.is_ahead_of_remote()? {
            step("Pushing changes to remote...");
            self.push()?;
            success("Pushed changes to remote");
            result = SyncResult::Success;
        }

        Ok(result)
    }

    /// Check if there are merge conflicts
    pub fn has_merge_conflicts(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("ls-files")
            .arg("--unmerged")
            .output()
            .context("Failed to check for conflicts")?;

        Ok(!output.stdout.is_empty())
    }

    /// Get list of conflicted files
    pub fn get_conflicted_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("diff")
            .arg("--name-only")
            .arg("--diff-filter=U")
            .output()
            .context("Failed to get conflicted files")?;

        let files = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(files)
    }

    /// Stash local changes
    pub fn stash(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("stash")
            .arg("push")
            .arg("-m")
            .arg("heimdal auto-stash")
            .output()
            .context("Failed to stash changes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Stash failed: {}", stderr);
        }

        Ok(())
    }

    /// Check if there's a stash
    pub fn has_stash(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("stash")
            .arg("list")
            .output()
            .context("Failed to list stash")?;

        Ok(!output.stdout.is_empty())
    }

    /// Pop the stash
    pub fn stash_pop(&self) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("stash")
            .arg("pop")
            .output()
            .context("Failed to pop stash")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Stash pop failed: {}", stderr);
        }

        Ok(())
    }

    /// Get current branch tracking information
    pub fn get_tracking_info(&self) -> Result<TrackingInfo> {
        let branch = self.current_branch()?;
        let ahead = self.commits_ahead()?;
        let behind = self.commits_behind()?;
        let upstream = self.get_upstream_branch()?;

        Ok(TrackingInfo {
            branch,
            upstream,
            ahead,
            behind,
        })
    }

    /// Get upstream branch name
    fn get_upstream_branch(&self) -> Result<Option<String>> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("@{u}")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                Ok(Some(String::from_utf8(out.stdout)?.trim().to_string()))
            }
            _ => Ok(None),
        }
    }

    /// Get number of commits ahead of remote
    fn commits_ahead(&self) -> Result<usize> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("rev-list")
            .arg("--count")
            .arg("@{u}..")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let count = String::from_utf8(out.stdout)?.trim().parse()?;
                Ok(count)
            }
            _ => Ok(0),
        }
    }

    /// Get number of commits behind remote
    fn commits_behind(&self) -> Result<usize> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("rev-list")
            .arg("--count")
            .arg("..@{u}")
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let count = String::from_utf8(out.stdout)?.trim().parse()?;
                Ok(count)
            }
            _ => Ok(0),
        }
    }

    /// Fetch from remote
    #[allow(dead_code)]
    pub fn fetch(&self) -> Result<()> {
        step("Fetching from remote...");

        let output = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("fetch")
            .output()
            .context("Failed to fetch from remote")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Fetch failed: {}", stderr);
        }

        Ok(())
    }
}

/// Branch tracking information
#[derive(Debug, Clone)]
pub struct TrackingInfo {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

impl TrackingInfo {
    /// Check if in sync with upstream
    pub fn is_in_sync(&self) -> bool {
        self.ahead == 0 && self.behind == 0
    }

    /// Format tracking info for display
    pub fn format(&self) -> String {
        use colored::Colorize;

        let mut parts = vec![format!("{}", self.branch.cyan())];

        if let Some(upstream) = &self.upstream {
            let status = if self.is_in_sync() {
                "up to date".green()
            } else if self.ahead > 0 && self.behind > 0 {
                format!("ahead {}, behind {}", self.ahead, self.behind).yellow()
            } else if self.ahead > 0 {
                format!("ahead {}", self.ahead).yellow()
            } else {
                format!("behind {}", self.behind).yellow()
            };

            parts.push(format!("tracking {}", upstream.bright_black()));
            parts.push(format!("({})", status));
        } else {
            parts.push("(no upstream)".bright_black().to_string());
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, GitRepo) {
        let temp = TempDir::new().unwrap();

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
    fn test_has_merge_conflicts_false() {
        let (_temp, repo) = setup_test_repo();
        assert!(!repo.has_merge_conflicts().unwrap());
    }

    #[test]
    fn test_tracking_info_no_upstream() {
        let (_temp, repo) = setup_test_repo();
        let info = repo.get_tracking_info();
        // No upstream configured, should have error or None
        assert!(info.is_err() || info.unwrap().upstream.is_none());
    }

    #[test]
    fn test_stash_operations() {
        let (temp, repo) = setup_test_repo();

        // Create and commit a file first
        fs::write(temp.path().join("file1.txt"), "initial").unwrap();
        repo.add_file("file1.txt").unwrap();
        repo.commit_staged("Initial commit").unwrap();

        // Make a change
        fs::write(temp.path().join("file1.txt"), "modified").unwrap();

        // Stash
        repo.stash().unwrap();
        assert!(repo.has_stash().unwrap());

        // File should be back to committed state
        let content = fs::read_to_string(temp.path().join("file1.txt")).unwrap();
        assert_eq!(content, "initial");

        // Pop stash
        repo.stash_pop().unwrap();

        // File should be modified again
        let content = fs::read_to_string(temp.path().join("file1.txt")).unwrap();
        assert_eq!(content, "modified");
    }
}
