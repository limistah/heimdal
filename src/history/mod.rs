pub mod cache;
pub mod shell;
pub mod store;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub ts: DateTime<Utc>,
    pub cmd: String,
    pub dir: String,
    pub exit: i32,
    pub host: String,
    pub session: String,
}

/// Path to the local plaintext staging file.
/// Commands are appended here by the shell hook; flushed to the encrypted dotfiles
/// file on `heimdal history sync`.
pub fn staging_path() -> Result<PathBuf> {
    Ok(crate::utils::home_dir()?
        .join(".heimdal")
        .join("history_staging.jsonl"))
}

/// Path to the local merged plaintext cache (never committed to git).
pub fn cache_path() -> Result<PathBuf> {
    Ok(crate::utils::home_dir()?
        .join(".heimdal")
        .join("history.cache"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_serializes_to_json() {
        let entry = HistoryEntry {
            ts: chrono::DateTime::parse_from_rfc3339("2026-04-01T07:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            cmd: "cargo build".into(),
            dir: "/home/user/proj".into(),
            exit: 0,
            host: "mbp-work".into(),
            session: "abc123".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("cargo build"));
        assert!(json.contains("mbp-work"));
    }

    #[test]
    fn staging_path_is_under_home_heimdal() {
        let path = staging_path().unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".heimdal"));
        assert!(path_str.ends_with("history_staging.jsonl"));
    }
}
