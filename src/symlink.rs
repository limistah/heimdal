use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::config::{DotfileCondition, DotfileEntry};
use crate::utils::{expand_path, info, step, verbose, warning};

pub struct ApplyContext {
    pub dotfiles_dir: PathBuf,
    pub home_dir: PathBuf,
    pub dry_run: bool,
    pub force: bool,
    pub backup: bool,
    pub ignore_patterns: Vec<glob::Pattern>,
}

#[derive(Debug)]
pub enum LinkResult {
    Created { src: PathBuf, dest: PathBuf },
    AlreadyLinked { dest: PathBuf },
    Skipped { dest: PathBuf, reason: String },
    Backed { dest: PathBuf, backup: PathBuf },
    Conflict { dest: PathBuf, reason: String },
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

/// Compile a list of ignore patterns, warning on invalid patterns
pub fn compile_ignore_patterns(patterns: &[String]) -> Vec<glob::Pattern> {
    patterns
        .iter()
        .filter_map(|p| match glob::Pattern::new(p) {
            Ok(pattern) => Some(pattern),
            Err(e) => {
                warning(&format!("Invalid ignore pattern '{}': {} — skipping", p, e));
                None
            }
        })
        .collect()
}

/// Check if a relative path matches any ignore pattern (case-insensitive)
fn matches_ignore(rel: &Path, patterns: &[glob::Pattern]) -> bool {
    let path_str = rel.to_string_lossy().replace('\\', "/");
    let options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    patterns.iter().any(|p| p.matches_with(&path_str, options))
}

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

        if crate::utils::get_verbosity() == crate::utils::Verbosity::Verbose {
            verbose(&format!("Processing entry: {} → {}", src_rel, dest_str));
        }

        // Check ignore patterns first
        let src_rel_path = Path::new(src_rel);
        if matches_ignore(src_rel_path, &ctx.ignore_patterns) {
            results.push(LinkResult::Skipped {
                dest: expand_path(&dest_str),
                reason: "matches ignore pattern".to_string(),
            });
            continue;
        }

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

/// GNU Stow-style walk with tree-folding: symlink entries from dotfiles_dir into home_dir.
///
/// This implements tree-folding behavior similar to GNU Stow:
///   - If `~/.config` doesn't exist, create `~/.config` → `dotfiles/.config` (whole-dir symlink)
///   - If `~/.config` already exists as a real directory, recurse into it and symlink each
///     subdirectory individually (e.g., `~/.config/nvim` → `dotfiles/.config/nvim`)
///   - This preserves untracked entries in `~/.config` (e.g., Raycast, VSCode, etc.)
///
/// Ignore patterns from the config are respected at all depths. Hardcoded STOW_SKIP patterns
/// (.git, heimdal.yaml, etc.) only apply at the top level (depth 0).
pub fn apply_stow_walk(ctx: &ApplyContext) -> Result<Vec<LinkResult>> {
    stow_walk_dir(&ctx.dotfiles_dir, &ctx.home_dir, 0, ctx)
}

/// Recursive helper for tree-folding stow walk
fn stow_walk_dir(
    src_dir: &Path,
    dest_dir: &Path,
    depth: usize,
    ctx: &ApplyContext,
) -> Result<Vec<LinkResult>> {
    let mut results = Vec::new();

    let entries = match std::fs::read_dir(src_dir) {
        Ok(e) => e,
        Err(err) => {
            warning(&format!(
                "Cannot read directory {}: {} — skipping",
                src_dir.display(),
                err
            ));
            return Ok(results);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                warning(&format!(
                    "Error reading entry in {}: {}",
                    src_dir.display(),
                    err
                ));
                continue;
            }
        };

        let src = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // At depth 0, check hardcoded STOW_SKIP list (+ always skip .heimdal)
        if depth == 0 && (name_str == ".heimdal" || STOW_SKIP.contains(&name_str.as_ref())) {
            verbose(&format!("Stow skip: {}", name_str));
            continue;
        }

        // Compute relative path from dotfiles_dir for ignore matching
        let rel = src.strip_prefix(&ctx.dotfiles_dir).unwrap_or(src.as_path());

        // Check user ignore patterns
        if matches_ignore(rel, &ctx.ignore_patterns) {
            results.push(LinkResult::Skipped {
                dest: dest_dir.join(&name),
                reason: "matches ignore pattern".to_string(),
            });
            continue;
        }

        let dest = dest_dir.join(&name);

        // Tree-folding decision logic:
        // 1. Get metadata about src and dest
        let src_is_dir = src.is_dir();
        let dest_metadata = std::fs::symlink_metadata(&dest);

        match dest_metadata {
            Ok(meta) if meta.is_symlink() => {
                // dest is a symlink
                if let Ok(target) = std::fs::read_link(&dest) {
                    if target == src {
                        // Already correctly linked
                        results.push(LinkResult::AlreadyLinked { dest: dest.clone() });
                    } else {
                        // Symlink points elsewhere — treat as conflict
                        results.push(link_one(&src, &dest, ctx)?);
                    }
                } else {
                    // Can't read symlink target — treat as conflict
                    results.push(link_one(&src, &dest, ctx)?);
                }
            }
            Ok(meta) if meta.is_dir() => {
                // dest exists as a real directory
                if src_is_dir {
                    // Both src and dest are real directories → recurse (tree-fold)
                    results.extend(stow_walk_dir(&src, &dest, depth + 1, ctx)?);
                } else {
                    // src is a file, dest is a dir → conflict
                    results.push(link_one(&src, &dest, ctx)?);
                }
            }
            Ok(_) => {
                // dest exists as a regular file → conflict
                results.push(link_one(&src, &dest, ctx)?);
            }
            Err(_) => {
                // dest doesn't exist → create symlink (works for both files and dirs)
                results.push(link_one(&src, &dest, ctx)?);
            }
        }
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

            // Compute relative path from home_dir to preserve directory structure in backups
            let rel_path = dest.strip_prefix(&ctx.home_dir).unwrap_or_else(|_| {
                // Fallback: just use the file name if dest isn't under home_dir
                Path::new(dest.file_name().unwrap_or_default())
            });

            // Mirror the relative path structure in backups, appending timestamp
            let backup_file_name = format!(
                "{}.{}",
                rel_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("backup"),
                ts
            );

            let backup = if let Some(parent) = rel_path.parent() {
                // Nested path: preserve parent structure
                backup_dir.join(parent).join(&backup_file_name)
            } else {
                // Top-level file
                backup_dir.join(&backup_file_name)
            };

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
            LinkResult::Created { src, dest } => {
                step(&format!("{}Linked: {}", prefix, dest.display()));
                verbose(&format!("{}  source: {}", prefix, src.display()));
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
            ignore_patterns: vec![],
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

    #[test]
    fn compile_ignore_patterns_valid() {
        let patterns = vec!["*.md".to_string(), ".DS_Store".to_string()];
        let compiled = compile_ignore_patterns(&patterns);
        assert_eq!(compiled.len(), 2);
    }

    #[test]
    fn compile_ignore_patterns_invalid_warns() {
        // Invalid glob pattern should be skipped with warning
        let patterns = vec!["[invalid".to_string(), "*.md".to_string()];
        let compiled = compile_ignore_patterns(&patterns);
        assert_eq!(compiled.len(), 1); // Only *.md compiled successfully
    }

    #[test]
    fn matches_ignore_case_insensitive() {
        let patterns = compile_ignore_patterns(&["*.md".to_string()]);
        assert!(matches_ignore(Path::new("README.MD"), &patterns));
        assert!(matches_ignore(Path::new("README.md"), &patterns));
        assert!(matches_ignore(Path::new("notes.Md"), &patterns));
        assert!(!matches_ignore(Path::new("README.txt"), &patterns));
    }

    #[test]
    fn matches_ignore_exact_name() {
        let patterns = compile_ignore_patterns(&[".DS_Store".to_string()]);
        assert!(matches_ignore(Path::new(".DS_Store"), &patterns));
        assert!(!matches_ignore(Path::new("other"), &patterns));
    }

    #[test]
    fn stow_walk_recurses_into_existing_dest_dir() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .config/nvim/ and .config/fish/
        let dotfiles = tmp.path().join("dotfiles");
        let config_dir = dotfiles.join(".config");
        std::fs::create_dir_all(config_dir.join("nvim")).unwrap();
        std::fs::create_dir_all(config_dir.join("fish")).unwrap();
        std::fs::write(config_dir.join("nvim").join("init.lua"), "-- nvim config").unwrap();
        std::fs::write(config_dir.join("fish").join("config.fish"), "# fish config").unwrap();

        // Setup: home already has .config/ as a real directory
        let home = tmp.path().join("home");
        std::fs::create_dir_all(home.join(".config")).unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns: vec![],
        };

        let results = apply_stow_walk(&ctx).unwrap();

        // Verify: .config itself should NOT be a symlink (it's a real dir)
        assert!(home.join(".config").is_dir());
        assert!(!home.join(".config").is_symlink());

        // Verify: .config/nvim and .config/fish should be symlinks
        assert!(home.join(".config/nvim").is_symlink());
        assert!(home.join(".config/fish").is_symlink());

        // Verify: we got Created results for both
        let created: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, LinkResult::Created { .. }))
            .collect();
        assert_eq!(created.len(), 2);
    }

    #[test]
    fn stow_walk_preserves_untracked_entries() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .config/nvim/
        let dotfiles = tmp.path().join("dotfiles");
        let config_dir = dotfiles.join(".config");
        std::fs::create_dir_all(config_dir.join("nvim")).unwrap();
        std::fs::write(config_dir.join("nvim").join("init.lua"), "-- nvim").unwrap();

        // Setup: home has .config/ with raycast (not in dotfiles)
        let home = tmp.path().join("home");
        std::fs::create_dir_all(home.join(".config").join("raycast")).unwrap();
        std::fs::write(home.join(".config/raycast/settings.json"), "{}").unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns: vec![],
        };

        apply_stow_walk(&ctx).unwrap();

        // Verify: raycast dir still exists (untracked)
        assert!(home.join(".config/raycast").is_dir());
        assert!(home.join(".config/raycast/settings.json").exists());

        // Verify: nvim is symlinked
        assert!(home.join(".config/nvim").is_symlink());
    }

    #[test]
    fn stow_walk_honors_ignore_patterns() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has notes.md, .DS_Store, notes.txt
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(&dotfiles).unwrap();
        std::fs::write(dotfiles.join("notes.md"), "notes").unwrap();
        std::fs::write(dotfiles.join(".DS_Store"), "ds").unwrap();
        std::fs::write(dotfiles.join("notes.txt"), "notes").unwrap();

        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();

        let ignore_patterns =
            compile_ignore_patterns(&["*.md".to_string(), ".DS_Store".to_string()]);

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns,
        };

        let results = apply_stow_walk(&ctx).unwrap();

        // Verify: notes.md and .DS_Store should be skipped
        let skipped: Vec<_> = results
            .iter()
            .filter(
                |r| matches!(r, LinkResult::Skipped { reason, .. } if reason.contains("ignore")),
            )
            .collect();
        assert_eq!(skipped.len(), 2);

        // Verify: notes.txt should be created
        assert!(home.join("notes.txt").is_symlink());
        assert!(!home.join("notes.md").exists());
        assert!(!home.join(".DS_Store").exists());
    }

    #[test]
    fn stow_walk_skips_heimdal_internal_files() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .git/, .heimdal/, heimdal.yaml, and .vimrc
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(dotfiles.join(".git")).unwrap();
        std::fs::create_dir_all(dotfiles.join(".heimdal")).unwrap();
        std::fs::write(dotfiles.join("heimdal.yaml"), "config").unwrap();
        std::fs::write(dotfiles.join(".vimrc"), "vim").unwrap();

        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns: vec![],
        };

        apply_stow_walk(&ctx).unwrap();

        // Verify: internal files not linked
        assert!(!home.join(".git").exists());
        assert!(!home.join(".heimdal").exists());
        assert!(!home.join("heimdal.yaml").exists());

        // Verify: .vimrc IS linked
        assert!(home.join(".vimrc").is_symlink());
    }

    #[test]
    fn stow_walk_stow_skip_only_at_depth_zero() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .config/app/LICENSE (nested, should be linked)
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(dotfiles.join(".config/app")).unwrap();
        std::fs::write(dotfiles.join(".config/app/LICENSE"), "MIT").unwrap();

        let home = tmp.path().join("home");
        std::fs::create_dir_all(home.join(".config")).unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns: vec![],
        };

        apply_stow_walk(&ctx).unwrap();

        // Verify: nested LICENSE should be symlinked (STOW_SKIP only at depth 0)
        assert!(home.join(".config/app").is_symlink());
    }

    #[test]
    fn stow_walk_conflict_on_real_file_at_dest() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .vimrc
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(&dotfiles).unwrap();
        std::fs::write(dotfiles.join(".vimrc"), "new").unwrap();

        // Setup: home has existing .vimrc as a real file
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join(".vimrc"), "existing").unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns: vec![],
        };

        let results = apply_stow_walk(&ctx).unwrap();

        // Verify: should report conflict
        let conflicts: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, LinkResult::Conflict { .. }))
            .collect();
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn stow_walk_backup_mirrors_rel_path() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has .config/nvim/init.lua
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(dotfiles.join(".config/nvim")).unwrap();
        std::fs::write(dotfiles.join(".config/nvim/init.lua"), "new").unwrap();

        // Setup: home has existing .config/nvim/init.lua
        let home = tmp.path().join("home");
        std::fs::create_dir_all(home.join(".config/nvim")).unwrap();
        std::fs::write(home.join(".config/nvim/init.lua"), "old").unwrap();

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: true,
            ignore_patterns: vec![],
        };

        let results = apply_stow_walk(&ctx).unwrap();

        // Verify: backup should mirror the relative path structure
        let backed: Vec<_> = results
            .iter()
            .filter_map(|r| match r {
                LinkResult::Backed { backup, .. } => Some(backup),
                _ => None,
            })
            .collect();

        assert_eq!(backed.len(), 1);
        // Backup path should include .config/nvim/ structure
        let backup_path_str = backed[0].to_string_lossy();
        assert!(backup_path_str.contains(".config"));
        assert!(backup_path_str.contains("nvim"));
    }

    #[test]
    fn apply_mappings_honors_ignore() {
        let tmp = TempDir::new().unwrap();

        // Setup: dotfiles has README.md
        let dotfiles = tmp.path().join("dotfiles");
        std::fs::create_dir_all(&dotfiles).unwrap();
        std::fs::write(dotfiles.join("README.md"), "readme").unwrap();

        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();

        let ignore_patterns = compile_ignore_patterns(&["*.md".to_string()]);

        let ctx = ApplyContext {
            dotfiles_dir: dotfiles.clone(),
            home_dir: home.clone(),
            dry_run: false,
            force: false,
            backup: false,
            ignore_patterns,
        };

        let entries = vec![DotfileEntry::Simple("README.md".to_string())];
        let results = apply_mappings(&ctx, &entries, "default").unwrap();

        // Verify: mapping should be skipped due to ignore pattern
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            LinkResult::Skipped { ref reason, .. } if reason.contains("ignore")
        ));
    }
}
