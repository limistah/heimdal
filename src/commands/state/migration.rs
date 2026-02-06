use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;

use crate::state::versioned::HeimdallStateV2;
use crate::state::HeimdallState;

/// Show state version information
pub fn cmd_version() -> Result<()> {
    // Try to load V2 state first
    match HeimdallStateV2::load() {
        Ok(state) => {
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
        }
        Err(_) => {
            // Try V1 state
            match HeimdallState::load() {
                Ok(_state) => {
                    println!("{}", "State Version Information:".cyan().bold());
                    println!("  Schema version: {}", "V1 (legacy)".yellow());
                    println!(
                        "\n{}",
                        "Migration required to enable new features:".yellow()
                    );
                    println!("  - State locking and coordination");
                    println!("  - Conflict detection and resolution");
                    println!("  - File drift tracking");
                    println!("  - Operation history");
                    println!("\n{}", "Run 'heimdal state migrate' to upgrade".green());
                }
                Err(e) => {
                    return Err(anyhow!("Failed to load state file: {}", e));
                }
            }
        }
    }

    Ok(())
}

/// Migrate from V1 to V2 state format
pub fn cmd_migrate(no_backup: bool, force: bool) -> Result<()> {
    println!("{}", "State Migration: V1 → V2".cyan().bold());

    // Check if already V2
    if HeimdallStateV2::load().is_ok() && !force {
        println!("{}", "State is already V2".green());
        println!("{}", "Use --force to re-migrate".yellow());
        return Ok(());
    }

    // Load V1 state
    let v1_state = HeimdallState::load().map_err(|e| anyhow!("Failed to load V1 state: {}", e))?;

    println!("\n{}", "Current state (V1):".yellow());
    println!("  Active profile: {}", v1_state.active_profile);
    println!("  Dotfiles path: {}", v1_state.dotfiles_path.display());
    println!("  Repo URL: {}", v1_state.repo_url);

    // Create backup
    if !no_backup {
        let backup_path = create_backup()?;
        println!(
            "\n{}",
            format!("Backup created: {}", backup_path.display()).green()
        );
    }

    // Perform migration
    println!("\n{}", "Migrating to V2...".cyan());
    let v2_state = HeimdallStateV2::migrate_from_v1(v1_state)?;

    println!("\n{}", "New state (V2):".green());
    println!("  Schema version: V{}", v2_state.version);
    println!("  Machine ID: {}", v2_state.machine.id);
    println!("  State serial: {}", v2_state.lineage.serial);
    println!("  Lineage ID: {}", v2_state.lineage.id);

    println!("\n{}", "New features enabled:".green().bold());
    println!("  ✓ State locking and coordination");
    println!("  ✓ Conflict detection and resolution");
    println!("  ✓ File drift tracking");
    println!("  ✓ Operation history");
    println!("  ✓ Multi-machine support");

    println!("\n{}", "Migration completed successfully!".green().bold());

    Ok(())
}

/// Show operation history
pub fn cmd_history(limit: usize) -> Result<()> {
    let state = HeimdallStateV2::load().map_err(|e| anyhow!("Failed to load state: {}", e))?;

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

// Helper functions
fn create_backup() -> Result<std::path::PathBuf> {
    let state_path = HeimdallState::state_path()?;
    let backup_dir = HeimdallStateV2::backup_dir()?;

    fs::create_dir_all(&backup_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("state_v1_{}.json", timestamp));

    fs::copy(&state_path, &backup_path)?;

    Ok(backup_path)
}
