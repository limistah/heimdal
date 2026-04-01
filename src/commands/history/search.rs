use crate::history::cache::read_cache;
use crate::utils::info;
use anyhow::Result;

pub fn run(query: Option<&str>, interactive: bool) -> Result<()> {
    let cache_path = crate::history::cache_path()?;
    let entries = read_cache(&cache_path)?;

    if entries.is_empty() {
        info("No history found. Run `heimdal sync` to pull history from other machines.");
        return Ok(());
    }

    if interactive || query.is_none() {
        run_interactive(&entries)
    } else {
        run_filter(&entries, query.unwrap())
    }
}

fn run_interactive(entries: &[crate::history::HistoryEntry]) -> Result<()> {
    let items: Vec<String> = entries
        .iter()
        .rev()
        .map(|e| format!("[{}] {}", e.host, e.cmd))
        .collect();

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("history")
        .items(&items)
        .default(0)
        .interact_opt()?;

    if let Some(idx) = selection {
        let cmd = &entries[entries.len() - 1 - idx].cmd;
        print!("{}", cmd);
    }
    Ok(())
}

fn run_filter(entries: &[crate::history::HistoryEntry], query: &str) -> Result<()> {
    let q = query.to_lowercase();
    for entry in entries.iter().rev() {
        if entry.cmd.to_lowercase().contains(&q) {
            println!("[{}] {}", entry.host, entry.cmd);
        }
    }
    Ok(())
}
