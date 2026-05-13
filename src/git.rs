use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitRepo {
    pub path: PathBuf,
}

pub struct GitFile {
    pub status: GitStatus,
    pub path: String,
}

#[derive(Debug, PartialEq)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}

impl GitRepo {
    /// Clone a remote URL to dest directory.
    pub fn clone(url: &str, dest: &Path) -> Result<Self> {
        let dest_str = dest.to_str().ok_or_else(|| {
            crate::error::HeimdallError::Git("Dotfiles path contains invalid UTF-8".to_string())
        })?;
        let status = Command::new("git")
            .args(["clone", "--", url, dest_str])
            .status()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;

        if !status.success() {
            return Err(crate::error::HeimdallError::Git(format!(
                "git clone failed for '{}'. Check the URL and your network connection.",
                url
            ))
            .into());
        }

        Ok(Self {
            path: dest.to_owned(),
        })
    }

    /// Use an existing local directory as a GitRepo (for --no-clone).
    pub fn open(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }

    /// Parse `git status --porcelain` into structured output
    pub fn status(&self) -> Result<Vec<GitFile>> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;

        if !output.status.success() {
            return Err(crate::error::HeimdallError::Git(format!(
                "git status failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .into());
        }

        let mut files = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.len() < 3 {
                continue;
            }
            let xy = &line[..2];
            let path = line[3..].to_string();
            let status = match xy.trim() {
                "M" | " M" | "MM" => GitStatus::Modified,
                "A" | " A" => GitStatus::Added,
                "D" | " D" => GitStatus::Deleted,
                "??" => GitStatus::Untracked,
                _ => GitStatus::Modified, // treat unknown as modified
            };
            files.push(GitFile { status, path });
        }
        Ok(files)
    }

    /// Run `git diff HEAD` and return output as a string
    pub fn diff(&self, _verbose: bool) -> Result<String> {
        let output = Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(&self.path)
            .output()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;

        if !output.status.success() {
            return Err(crate::error::HeimdallError::Git(format!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .into());
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Stage the given files (or all via `git add -A`) then commit
    pub fn commit(&self, message: &str, files: Option<&[String]>, dry_run: bool) -> Result<()> {
        // First check if there's anything to commit
        let status_out = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        let has_changes = !String::from_utf8_lossy(&status_out.stdout)
            .trim()
            .is_empty();

        if dry_run {
            if has_changes {
                crate::utils::info(&format!("[dry-run] Would commit: {}", message));
            } else {
                crate::utils::info("[dry-run] Nothing to commit");
            }
            return Ok(());
        }

        if !has_changes {
            crate::utils::info("Nothing to commit, working tree clean");
            return Ok(());
        }

        // Stage files
        if let Some(file_list) = files {
            if !file_list.is_empty() {
                let mut cmd = Command::new("git");
                cmd.arg("add").args(file_list).current_dir(&self.path);
                let status = cmd.status().map_err(|e| {
                    crate::error::HeimdallError::Git(format!("Cannot run git: {}", e))
                })?;
                if !status.success() {
                    return Err(
                        crate::error::HeimdallError::Git("git add failed".to_string()).into(),
                    );
                }
            }
        } else {
            // Stage all tracked + new files
            let status = Command::new("git")
                .args(["add", "-A"])
                .current_dir(&self.path)
                .status()
                .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
            if !status.success() {
                return Err(
                    crate::error::HeimdallError::Git("git add -A failed".to_string()).into(),
                );
            }
        }

        // Re-check after staging
        let status_out = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        if String::from_utf8_lossy(&status_out.stdout)
            .trim()
            .is_empty()
        {
            crate::utils::info("Nothing to commit after staging");
            return Ok(());
        }

        // Commit
        let status = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.path)
            .status()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        if !status.success() {
            return Err(crate::error::HeimdallError::Git("git commit failed".to_string()).into());
        }
        Ok(())
    }

    pub fn pull(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            crate::utils::info("[dry-run] Would run: git pull");
            return Ok(());
        }
        let status = Command::new("git")
            .args(["pull"])
            .current_dir(&self.path)
            .status()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        if !status.success() {
            return Err(crate::error::HeimdallError::Git(
                "git pull failed. Check your network connection and remote configuration."
                    .to_string(),
            )
            .into());
        }
        Ok(())
    }

    pub fn push(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            crate::utils::info("[dry-run] Would run: git push");
            return Ok(());
        }
        let status = Command::new("git")
            .args(["push"])
            .current_dir(&self.path)
            .status()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        if !status.success() {
            return Err(crate::error::HeimdallError::Git(
                "git push failed. Check your remote configuration.".to_string(),
            )
            .into());
        }
        Ok(())
    }

    pub fn rollback(&self, target: Option<&str>, dry_run: bool) -> Result<()> {
        let rev = target.unwrap_or("HEAD~1");
        if dry_run {
            crate::utils::info(&format!("[dry-run] Would run: git reset --hard {}", rev));
            return Ok(());
        }
        let status = Command::new("git")
            .args(["reset", "--hard", rev])
            .current_dir(&self.path)
            .status()
            .map_err(|e| crate::error::HeimdallError::Git(format!("Cannot run git: {}", e)))?;
        if !status.success() {
            return Err(crate::error::HeimdallError::Git(format!(
                "git reset --hard {} failed",
                rev
            ))
            .into());
        }
        Ok(())
    }
}
