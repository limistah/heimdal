use crate::history::HistoryEntry;
use std::io::Write;

/// Append a shell command to the local plaintext staging file.
///
/// This is called on every shell command via the shell hook (`precmd` / `PROMPT_COMMAND`).
/// All errors are swallowed so that a transient failure (full disk, missing state file,
/// etc.) never interrupts the user's shell session. The shell hook also runs this in the
/// background (`&>/dev/null &`), so exit codes are invisible — explicit silent-fail here
/// makes the contract clear regardless of the calling context.
pub fn run(cmd: &str, exit: i32, dir: &str, session: &str) -> anyhow::Result<()> {
    if let Err(e) = try_record(cmd, exit, dir, session) {
        // Errors are intentionally swallowed. The shell hook discards them too,
        // but being explicit here means the behaviour is the same when called
        // programmatically (e.g. tests, scripts).
        let _ = e; // suppress unused-variable warning in release builds
        #[cfg(debug_assertions)]
        eprintln!("[heimdal history record] error (ignored): {e}");
    }
    Ok(())
}

fn try_record(cmd: &str, exit: i32, dir: &str, session: &str) -> anyhow::Result<()> {
    if cmd.trim().is_empty() {
        return Ok(());
    }

    // Use the hostname from State so it matches the encrypted filename written by
    // `heimdal history sync` — both derive the hostname from the same single source.
    let state = crate::state::State::load()?;

    let entry = HistoryEntry {
        ts: chrono::Utc::now(),
        cmd: cmd.to_string(),
        dir: dir.to_string(),
        exit,
        host: state.hostname.clone(),
        session: if session.is_empty() {
            std::process::id().to_string()
        } else {
            session.to_string()
        },
    };

    let staging = crate::history::staging_path()?;
    if let Some(parent) = staging.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(&entry)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&staging)?;
    writeln!(file, "{}", json)?;
    Ok(())
}
