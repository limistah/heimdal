use crate::history::HistoryEntry;
use anyhow::Result;
use std::{
    collections::HashSet,
    io::{BufRead, Write},
    path::Path,
};

/// Sort entries by timestamp (oldest first) and remove exact duplicates
/// (same ts + cmd + host).
pub fn merge_and_sort(mut entries: Vec<HistoryEntry>) -> Vec<HistoryEntry> {
    entries.sort_by_key(|e| e.ts);
    let mut seen = HashSet::new();
    entries
        .into_iter()
        .filter(|e| seen.insert((e.ts, e.cmd.clone(), e.host.clone())))
        .collect()
}

/// Write entries to a plaintext JSONL cache file (one JSON object per line).
pub fn write_cache(path: &Path, entries: &[HistoryEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    for entry in entries {
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

/// Read entries from a plaintext JSONL cache file.
pub fn read_cache(path: &Path) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(e) = serde_json::from_str::<HistoryEntry>(line) {
            entries.push(e);
        }
    }
    Ok(entries)
}

/// Decrypt all per-machine encrypted files in `dotfiles_path/history/`,
/// merge and sort, write to the local cache file.
pub fn rebuild(dotfiles_path: &Path, key: &[u8; 32]) -> Result<()> {
    let history_dir = dotfiles_path.join("history");
    let cache_path = crate::history::cache_path()?;

    if !history_dir.exists() {
        return Ok(());
    }

    let mut all_entries = Vec::new();
    for entry in std::fs::read_dir(&history_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "enc").unwrap_or(false) {
            match crate::history::store::read_encrypted(&path, key) {
                Ok(entries) => all_entries.extend(entries),
                Err(e) => {
                    crate::utils::warning(&format!(
                        "Skipping {}: {e}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
            }
        }
    }

    let merged = merge_and_sort(all_entries);
    write_cache(&cache_path, &merged)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryEntry;
    use tempfile::TempDir;

    fn entry(cmd: &str, host: &str, secs_ago: i64) -> HistoryEntry {
        HistoryEntry {
            ts: chrono::Utc::now() - chrono::Duration::seconds(secs_ago),
            cmd: cmd.into(),
            dir: "/tmp".into(),
            exit: 0,
            host: host.into(),
            session: "s".into(),
        }
    }

    #[test]
    fn merge_sorts_by_timestamp() {
        let entries = vec![
            entry("b", "host1", 10),
            entry("c", "host2", 5),
            entry("a", "host1", 20),
        ];
        let merged = merge_and_sort(entries);
        assert_eq!(merged[0].cmd, "a");
        assert_eq!(merged[1].cmd, "b");
        assert_eq!(merged[2].cmd, "c");
    }

    #[test]
    fn dedup_removes_exact_duplicates() {
        let ts = chrono::Utc::now();
        let e = HistoryEntry {
            ts,
            cmd: "git push".into(),
            dir: "/p".into(),
            exit: 0,
            host: "h".into(),
            session: "s".into(),
        };
        let merged = merge_and_sort(vec![e.clone(), e]);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn write_and_read_cache_roundtrip() {
        let dir = TempDir::new().unwrap();
        let cache = dir.path().join("history.cache");
        let entries = vec![entry("ls", "host1", 5), entry("pwd", "host2", 2)];
        write_cache(&cache, &entries).unwrap();
        let read = read_cache(&cache).unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].cmd, "ls");
    }
}
