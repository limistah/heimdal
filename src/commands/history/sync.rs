use crate::history::{cache, staging_path, store, HistoryEntry};
use crate::state::State;
use crate::utils::{info, success};
use anyhow::Result;
use std::io::BufRead;

pub fn run() -> Result<()> {
    run_sync(false)
}

/// Called by `heimdal sync` after git pull. Pass `dry_run = true` to skip writes.
pub fn run_sync(dry_run: bool) -> Result<()> {
    let state = State::load()?;

    let bifrost = match crate::key::load() {
        Ok(k) => k,
        Err(_) => {
            info("Bifrost key not configured — skipping history sync. Run: heimdal key gen");
            return Ok(());
        }
    };

    let history_key = crate::crypto::kdf::history_key(&bifrost);

    flush_staging(
        &state.dotfiles_path,
        &state.hostname,
        &state.machine_id,
        &history_key,
        dry_run,
    )?;

    if !dry_run {
        cache::rebuild(&state.dotfiles_path, &history_key)?;
    }

    if !dry_run {
        success("History synced and cache rebuilt.");
    }
    Ok(())
}

fn flush_staging(
    dotfiles_path: &std::path::Path,
    hostname: &str,
    machine_id: &str,
    key: &[u8; 32],
    dry_run: bool,
) -> Result<()> {
    let staging = staging_path()?;
    if !staging.exists() {
        return Ok(());
    }

    let enc_path = dotfiles_path
        .join("history")
        .join(format!("{}-{}.jsonl.enc", hostname, machine_id));

    // Atomic drain: rename staging before reading so shell hook entries written
    // concurrently land in a fresh staging file and are not silently dropped.
    let tmp_staging = staging.with_extension(format!("flushing.{}", std::process::id()));
    if dry_run {
        // In dry-run mode read without draining
        let file = std::fs::File::open(&staging)?;
        let reader = std::io::BufReader::new(file);
        let count = reader
            .lines()
            .filter(|l| l.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false))
            .count();
        if count > 0 {
            info(&format!("[dry-run] Would flush {} history entries.", count));
        }
        return Ok(());
    }

    // Intentional: if two `heimdal sync` processes run concurrently (e.g. background
    // auto-sync races a manual sync), the second rename will fail with ENOENT because
    // the first already moved the file. That error propagates and the second process
    // exits cleanly — there is nothing left to flush. This is the correct outcome.
    std::fs::rename(&staging, &tmp_staging)?;

    let file = std::fs::File::open(&tmp_staging)?;
    let reader = std::io::BufReader::new(file);
    let mut flushed = 0usize;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: HistoryEntry = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        store::append_encrypted(&enc_path, &entry, key)?;
        flushed += 1;
    }

    let _ = std::fs::remove_file(&tmp_staging);

    ensure_gitignore(dotfiles_path)?;

    if flushed > 0 {
        info(&format!("Flushed {} history entries.", flushed));
    }
    Ok(())
}

fn ensure_gitignore(dotfiles_path: &std::path::Path) -> Result<()> {
    let gitignore = dotfiles_path.join(".gitignore");
    let entry = "history.cache\n";
    let content = std::fs::read_to_string(&gitignore).unwrap_or_default();
    if !content.contains("history.cache") {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gitignore)?;
        use std::io::Write;
        write!(f, "{}", entry)?;
    }
    Ok(())
}
