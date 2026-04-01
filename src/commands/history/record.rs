use crate::history::HistoryEntry;
use anyhow::Result;
use std::io::Write;

pub fn run(cmd: &str, exit: i32, dir: &str, session: &str) -> Result<()> {
    if cmd.trim().is_empty() {
        return Ok(());
    }

    let entry = HistoryEntry {
        ts: chrono::Utc::now(),
        cmd: cmd.to_string(),
        dir: dir.to_string(),
        exit,
        host: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
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
