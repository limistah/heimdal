use crate::history::HistoryEntry;
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::{
    io::{BufRead, Write},
    path::Path,
};

/// Encrypt a single `HistoryEntry` and append it as one base64url line to `path`.
/// Creates the file (and parent directories) if they don't exist.
pub fn append_encrypted(path: &Path, entry: &HistoryEntry, key: &[u8; 32]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec(entry)?;
    let blob = crate::crypto::encrypt(key, &json)?;
    let line = URL_SAFE_NO_PAD.encode(&blob);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Decrypt all entries in an encrypted JSONL file.
/// Returns an error if any line fails to decrypt (wrong key or corruption).
pub fn read_encrypted(path: &Path, key: &[u8; 32]) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let blob = URL_SAFE_NO_PAD
            .decode(line)
            .map_err(|e| anyhow::anyhow!("line {}: invalid base64: {e}", i + 1))?;
        let json = crate::crypto::decrypt(key, &blob)
            .map_err(|e| anyhow::anyhow!("line {}: {e}", i + 1))?;
        let entry: HistoryEntry = serde_json::from_slice(&json)
            .map_err(|e| anyhow::anyhow!("line {}: invalid JSON: {e}", i + 1))?;
        entries.push(entry);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryEntry;
    use tempfile::TempDir;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    fn test_entry(cmd: &str) -> HistoryEntry {
        HistoryEntry {
            ts: chrono::Utc::now(),
            cmd: cmd.into(),
            dir: "/tmp".into(),
            exit: 0,
            host: "test-host".into(),
            session: "sess1".into(),
        }
    }

    #[test]
    fn append_and_read_single_entry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl.enc");
        let key = test_key();

        append_encrypted(&path, &test_entry("ls -la"), &key).unwrap();

        let entries = read_encrypted(&path, &key).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cmd, "ls -la");
    }

    #[test]
    fn append_multiple_entries_preserves_order() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl.enc");
        let key = test_key();

        append_encrypted(&path, &test_entry("first"), &key).unwrap();
        append_encrypted(&path, &test_entry("second"), &key).unwrap();
        append_encrypted(&path, &test_entry("third"), &key).unwrap();

        let entries = read_encrypted(&path, &key).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].cmd, "first");
        assert_eq!(entries[2].cmd, "third");
    }

    #[test]
    fn wrong_key_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl.enc");
        let key = test_key();
        let wrong_key = [99u8; 32];

        append_encrypted(&path, &test_entry("secret"), &key).unwrap();
        assert!(read_encrypted(&path, &wrong_key).is_err());
    }
}
