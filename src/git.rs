#![allow(dead_code)]

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitRepo {
    pub path: PathBuf,
}

impl GitRepo {
    /// Clone a remote URL to dest directory.
    pub fn clone(url: &str, dest: &Path) -> Result<Self> {
        let status = Command::new("git")
            .args(["clone", url, &dest.to_string_lossy()])
            .status()
            .map_err(|e| {
                crate::error::HeimdallError::Git(format!("Cannot run git: {}", e))
            })?;

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

    // Stub methods for future phases
    pub fn status(&self) -> Result<Vec<GitFile>> {
        Err(anyhow::anyhow!("git status not yet implemented"))
    }

    pub fn diff(&self, _verbose: bool) -> Result<String> {
        Err(anyhow::anyhow!("git diff not yet implemented"))
    }

    pub fn commit(
        &self,
        _message: &str,
        _files: Option<&[String]>,
        _dry_run: bool,
    ) -> Result<()> {
        Err(anyhow::anyhow!("git commit not yet implemented"))
    }

    pub fn pull(&self, _dry_run: bool) -> Result<()> {
        Err(anyhow::anyhow!("git pull not yet implemented"))
    }

    pub fn push(&self, _dry_run: bool) -> Result<()> {
        Err(anyhow::anyhow!("git push not yet implemented"))
    }

    pub fn rollback(&self, _target: Option<&str>, _dry_run: bool) -> Result<()> {
        Err(anyhow::anyhow!("git rollback not yet implemented"))
    }

    pub fn current_commit(&self) -> Result<String> {
        Err(anyhow::anyhow!("git current_commit not yet implemented"))
    }
}

pub struct GitFile {
    pub status: GitStatus,
    pub path: String,
}

#[allow(dead_code)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}
