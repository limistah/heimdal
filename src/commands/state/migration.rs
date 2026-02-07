use anyhow::Result;
use colored::Colorize;

use crate::state::versioned::HeimdallStateV2;

/// Show state version information
pub fn cmd_version() -> Result<()> {
    let state = HeimdallStateV2::load()?;

    println!("{}", "State Version Information:".cyan().bold());
    println!(
        "  Schema version: {}",
        format!("V{}", state.version).green()
    );
    println!("  Heimdal version: {}", state.heimdal_version);
    println!("  State serial: {}", state.lineage.serial);
    println!("  Lineage ID: {}", state.lineage.id);
    println!("  Machine ID: {}", state.machine.id);
    println!("  Machine hostname: {}", state.machine.hostname);
    println!("  OS: {} ({})", state.machine.os, state.machine.arch);

    if let Some(last_sync) = state.last_sync {
        println!("  Last sync: {}", last_sync.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    if let Some(last_apply) = state.last_apply {
        println!(
            "  Last apply: {}",
            last_apply.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    println!(
        "\n{}",
        format!("Tracked files: {}", state.checksums.len()).cyan()
    );
    println!(
        "{}",
        format!("History entries: {}", state.history.len()).cyan()
    );

    Ok(())
}

/// Migration command - now deprecated since V1 is removed
pub fn cmd_migrate(_no_backup: bool, _force: bool) -> Result<()> {
    println!("{}", "State migration is no longer supported.".yellow());
    println!(
        "{}",
        "All new installations use V2 state format automatically.".green()
    );
    Ok(())
}

/// Show operation history
pub fn cmd_history(limit: usize) -> Result<()> {
    let state = HeimdallStateV2::load()?;

    if state.history.is_empty() {
        println!("{}", "No operation history found".yellow());
        return Ok(());
    }

    println!("{}", "Operation History:".cyan().bold());
    println!();

    let entries = state.history.iter().rev().take(limit);

    for (i, entry) in entries.enumerate() {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let op_color = match entry.operation.as_str() {
            "apply" => "green",
            "sync" => "blue",
            "commit" => "yellow",
            "init" => "magenta",
            _ => "white",
        };

        println!(
            "{}. {} {}",
            i + 1,
            entry.operation.color(op_color).bold(),
            format!("({})", timestamp).dimmed()
        );

        if !entry.description.is_empty() {
            println!("   {}", entry.description.dimmed());
        }

        println!("   Serial: {}", entry.serial.to_string().dimmed());

        println!();
    }

    if state.history.len() > limit {
        println!(
            "{}",
            format!("Showing {} of {} entries", limit, state.history.len()).dimmed()
        );
    }

    Ok(())
}
