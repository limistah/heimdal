use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{DotfileCondition, DotfileEntry};
use crate::utils::{expand_path, info, step, warning};

pub struct ApplyContext {
    pub dotfiles_dir: PathBuf,
    pub home_dir: PathBuf,
    pub dry_run: bool,
    pub force: bool,
    pub backup: bool,
}

#[derive(Debug)]
pub enum LinkResult {
    #[allow(dead_code)]
    Created {
        src: PathBuf,
        dest: PathBuf,
    },
    AlreadyLinked {
        dest: PathBuf,
    },
    Skipped {
        dest: PathBuf,
        reason: String,
    },
    Backed {
        dest: PathBuf,
        backup: PathBuf,
    },
    Conflict {
        dest: PathBuf,
        reason: String,
    },
}

static STOW_SKIP: &[&str] = &[
    ".git",
    ".heimdal",
    "heimdal.yaml",
    ".stowrc",
    "README.md",
    "README",
    "LICENSE",
    "CHANGELOG.md",
    "Makefile",
];

pub fn apply_mappings(
    ctx: &ApplyContext,
    entries: &[DotfileEntry],
    active_profile: &str,
) -> Result<Vec<LinkResult>> {
    let os = crate::utils::os_name();
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut results = Vec::new();

    for entry in entries {
        let (src_rel, dest_str, condition) = match entry {
            DotfileEntry::Simple(s) => (s.as_str(), format!("~/{}", s), None),
            DotfileEntry::Mapped(m) => (m.source.as_str(), m.target.clone(), m.when.clone()),
        };

        if !should_link(&condition, active_profile, os, &hostname) {
            results.push(LinkResult::Skipped {
                dest: expand_path(&dest_str),
                reason: "condition not met".to_string(),
            });
            continue;
        }

        let src = ctx.dotfiles_dir.join(src_rel);

        // Guard against path traversal (e.g., source: "../../etc/passwd")
        if let (Ok(canonical_src), Ok(canonical_dir)) = (
            src.canonicalize()
                .or_else(|_| Ok::<_, std::io::Error>(src.clone())),
            ctx.dotfiles_dir.canonicalize(),
        ) {
            if !canonical_src.starts_with(&canonical_dir) {
                results.push(LinkResult::Skipped {
                    dest: expand_path(&dest_str),
                    reason: format!(
                        "source '{}' escapes dotfiles directory — skipped for safety",
                        src_rel
                    ),
                });
                continue;
            }
        }

        let dest = expand_path(&dest_str);
        results.push(link_one(&src, &dest, ctx)?);
    }
    Ok(results)
}

/// GNU Stow-style walk: symlink top-level entries from dotfiles_dir into home_dir.
///
/// Only top-level entries are processed (depth 1). Each entry becomes a single
/// symlink in home_dir at the same relative path. This means:
///   - `.vimrc` in dotfiles → `~/.vimrc` symlink
///   - `.config/` directory → `~/.config` symlink (the whole dir, not its contents)
///
/// If you need file-level control within subdirectories, use explicit `dotfiles:`
/// mappings in heimdal.yaml instead.
pub fn apply_stow_walk(ctx: &ApplyContext) -> Result<Vec<LinkResult>> {
    let mut results = Vec::new();
    for entry in WalkDir::new(&ctx.dotfiles_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if STOW_SKIP.contains(&name.as_str()) {
            continue;
        }
        let rel = entry.path().strip_prefix(&ctx.dotfiles_dir).unwrap();
        // home_dir is already a resolved absolute path from dirs::home_dir(),
        // so no shellexpand needed here unlike apply_mappings which takes strings from config.
        let dest = ctx.home_dir.join(rel);
        results.push(link_one(entry.path(), &dest, ctx)?);
    }
    Ok(results)
}

pub fn link_one(src: &Path, dest: &Path, ctx: &ApplyContext) -> Result<LinkResult> {
    if !src.exists() {
        return Ok(LinkResult::Skipped {
            dest: dest.to_owned(),
            reason: format!("source not found: {}", src.display()),
        });
    }

    // Already correctly linked?
    if dest.is_symlink() {
        if let Ok(target) = std::fs::read_link(dest) {
            if target == src {
                return Ok(LinkResult::AlreadyLinked {
                    dest: dest.to_owned(),
                });
            }
        }
    }

    // Conflict: dest exists (as a real file/dir or wrong symlink)
    if dest.exists() || dest.is_symlink() {
        if ctx.force {
            if !ctx.dry_run {
                if dest.is_dir() && !dest.is_symlink() {
                    std::fs::remove_dir_all(dest)?;
                } else {
                    std::fs::remove_file(dest)?;
                }
            }
            // fall through to create symlink
        } else if ctx.backup {
            let backup_dir = ctx.dotfiles_dir.join(".heimdal").join("backups");
            let ts = Utc::now().format("%Y%m%dT%H%M%SZ");
            let base_name = dest
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("backup");
            let backup_name = format!("{}.{}", base_name, ts);
            let backup = backup_dir.join(&backup_name);

            if ctx.dry_run {
                // In dry-run, show what would happen but don't actually do it
                return Ok(LinkResult::Skipped {
                    dest: dest.to_owned(),
                    reason: format!("[preview] would back up to {}", backup.display()),
                });
            }

            crate::utils::ensure_parent_exists(&backup)?;
            std::fs::rename(dest, &backup)?;
            crate::utils::ensure_parent_exists(dest)?;
            create_symlink(src, dest)?;
            return Ok(LinkResult::Backed {
                dest: dest.to_owned(),
                backup,
            });
        } else {
            return Ok(LinkResult::Conflict {
                dest: dest.to_owned(),
                reason: "file exists. Use --force to overwrite or --backup to save original"
                    .to_string(),
            });
        }
    }

    if !ctx.dry_run {
        crate::utils::ensure_parent_exists(dest)?;
        create_symlink(src, dest)?;
    }

    Ok(LinkResult::Created {
        src: src.to_owned(),
        dest: dest.to_owned(),
    })
}

#[cfg(unix)]
fn create_symlink(src: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(src, dest).map_err(|e| {
        crate::error::HeimdallError::Symlink {
            path: src.display().to_string(),
            reason: e.to_string(),
        }
        .into()
    })
}

#[cfg(windows)]
fn create_symlink(src: &Path, dest: &Path) -> Result<()> {
    if src.is_dir() {
        std::os::windows::fs::symlink_dir(src, dest)
    } else {
        std::os::windows::fs::symlink_file(src, dest)
    }
    .map_err(|e| {
        crate::error::HeimdallError::Symlink {
            path: src.display().to_string(),
            reason: e.to_string(),
        }
        .into()
    })
}

pub fn should_link(
    condition: &Option<DotfileCondition>,
    active_profile: &str,
    os: &str,
    hostname: &str,
) -> bool {
    let Some(cond) = condition else { return true };
    if !cond.os.is_empty() && !cond.os.iter().any(|o| o == os) {
        return false;
    }
    if !cond.profile.is_empty() && !cond.profile.iter().any(|p| p == active_profile) {
        return false;
    }
    if let Some(pattern) = &cond.hostname {
        match glob::Pattern::new(pattern) {
            Ok(pat) => {
                if !pat.matches(hostname) {
                    return false;
                }
            }
            Err(_) => {
                crate::utils::warning(&format!(
                    "Invalid hostname glob pattern '{}' — skipping dotfile for safety",
                    pattern
                ));
                return false;
            }
        }
    }
    true
}

pub fn print_results(results: &[LinkResult], dry_run: bool) {
    let prefix = if dry_run { "[preview] " } else { "" };
    for r in results {
        match r {
            LinkResult::Created { dest, .. } => {
                step(&format!("{}Linked: {}", prefix, dest.display()))
            }
            LinkResult::AlreadyLinked { dest } => {
                info(&format!("Already linked: {}", dest.display()))
            }
            LinkResult::Skipped { dest, reason } => {
                info(&format!("Skipped {}: {}", dest.display(), reason))
            }
            LinkResult::Backed { dest, backup } => step(&format!(
                "{}Backed {} \u{2192} {}",
                prefix,
                dest.display(),
                backup.display()
            )),
            LinkResult::Conflict { dest, reason } => {
                warning(&format!("Conflict at {}: {}", dest.display(), reason))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn ctx(tmp: &TempDir, dry_run: bool, force: bool, backup: bool) -> ApplyContext {
        ApplyContext {
            dotfiles_dir: tmp.path().to_owned(),
            home_dir: tmp.path().to_owned(),
            dry_run,
            force,
            backup,
        }
    }

    #[test]
    fn should_link_no_condition() {
        assert!(should_link(&None, "default", "linux", "host"));
    }

    #[test]
    fn should_link_os_match() {
        let c = Some(DotfileCondition {
            os: vec!["linux".into()],
            ..Default::default()
        });
        assert!(should_link(&c, "default", "linux", "host"));
        assert!(!should_link(&c, "default", "macos", "host"));
    }

    #[test]
    fn should_link_os_empty_allows_all() {
        let c = Some(DotfileCondition {
            os: vec![],
            ..Default::default()
        });
        assert!(should_link(&c, "default", "linux", "host"));
        assert!(should_link(&c, "default", "macos", "host"));
    }

    #[test]
    fn should_link_profile_filter() {
        let c = Some(DotfileCondition {
            profile: vec!["work".into()],
            ..Default::default()
        });
        assert!(should_link(&c, "work", "linux", "host"));
        assert!(!should_link(&c, "personal", "linux", "host"));
    }

    #[test]
    fn should_link_hostname_glob() {
        let c = Some(DotfileCondition {
            hostname: Some("work-*".into()),
            ..Default::default()
        });
        assert!(should_link(&c, "default", "linux", "work-laptop"));
        assert!(!should_link(&c, "default", "linux", "personal-mac"));
    }

    #[test]
    fn link_one_creates_symlink() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("subdir").join("linked");
        let r = link_one(&src, &dest, &ctx(&tmp, false, false, false)).unwrap();
        assert!(matches!(r, LinkResult::Created { .. }));
        assert!(dest.is_symlink());
    }

    #[test]
    fn link_one_dry_run_no_create() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("linked");
        link_one(&src, &dest, &ctx(&tmp, true, false, false)).unwrap();
        assert!(!dest.exists());
    }

    #[test]
    fn link_one_idempotent() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("linked");
        link_one(&src, &dest, &ctx(&tmp, false, false, false)).unwrap();
        let r = link_one(&src, &dest, &ctx(&tmp, false, false, false)).unwrap();
        assert!(matches!(r, LinkResult::AlreadyLinked { .. }));
    }

    #[test]
    fn link_one_conflict_without_force() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("linked");
        std::fs::write(&dest, "existing").unwrap();
        let r = link_one(&src, &dest, &ctx(&tmp, false, false, false)).unwrap();
        assert!(matches!(r, LinkResult::Conflict { .. }));
        assert!(!dest.is_symlink());
    }

    #[test]
    fn link_one_force_overwrites() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("linked");
        std::fs::write(&dest, "existing").unwrap();
        let r = link_one(&src, &dest, &ctx(&tmp, false, true, false)).unwrap();
        assert!(matches!(r, LinkResult::Created { .. }));
        assert!(dest.is_symlink());
    }

    #[test]
    fn link_one_backup_saves_original() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".heimdal").join("backups")).unwrap();
        let src = tmp.path().join("dotfile");
        std::fs::write(&src, "data").unwrap();
        let dest = tmp.path().join("linked");
        std::fs::write(&dest, "original").unwrap();
        let r = link_one(&src, &dest, &ctx(&tmp, false, false, true)).unwrap();
        assert!(matches!(r, LinkResult::Backed { .. }));
        assert!(dest.is_symlink());
    }

    #[test]
    fn link_one_missing_source_returns_skipped() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("nonexistent");
        let dest = tmp.path().join("linked");
        let r = link_one(&src, &dest, &ctx(&tmp, false, false, false)).unwrap();
        assert!(matches!(r, LinkResult::Skipped { .. }));
    }
}
