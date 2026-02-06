use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

use crate::state::lock::{LockManager, StateLock};
use crate::state::versioned::HeimdallStateV2;

/// Show current lock status
pub fn cmd_lock_info() -> Result<()> {
    match LockManager::show_lock() {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.to_string().contains("No active lock") {
                println!("{}", "No active lock found".green());
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Force remove an active lock
pub fn cmd_unlock(force: bool) -> Result<()> {
    // Check if lock exists
    let lock_path = get_lock_path()?;

    if !lock_path.exists() {
        println!("{}", "No active lock found".green());
        return Ok(());
    }

    // Read lock info
    let lock = read_lock(&lock_path)?;

    // Show lock info
    println!("{}", "Active lock found:".yellow());
    println!("  Operation: {}", lock.operation.cyan());
    println!(
        "  Machine: {} ({})",
        lock.machine.hostname.cyan(),
        lock.machine.id
    );
    println!("  Process ID: {}", lock.machine.pid);
    println!(
        "  Created: {}",
        lock.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    if let Some(reason) = &lock.reason {
        println!("  Reason: {}", reason);
    }

    // Check if it's stale
    if lock.is_stale() {
        println!(
            "\n{}",
            "Lock appears to be stale (process not running)".yellow()
        );
    } else {
        println!(
            "\n{}",
            "Warning: Lock is still active (process running)"
                .red()
                .bold()
        );
    }

    // Confirm if not forced
    if !force {
        use std::io::{self, Write};
        print!("\n{} ", "Force remove this lock? [y/N]:".yellow());
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    // Remove lock
    LockManager::force_unlock()?;
    println!("{}", "Lock removed successfully".green());

    Ok(())
}

// Helper functions
fn get_lock_path() -> Result<PathBuf> {
    let state_dir = HeimdallStateV2::state_dir()?;
    Ok(state_dir.join(".heimdal.lock"))
}

fn read_lock(path: &PathBuf) -> Result<StateLock> {
    let content = std::fs::read_to_string(path).context("Failed to read lock file")?;
    let lock: StateLock = serde_json::from_str(&content).context("Failed to parse lock file")?;
    Ok(lock)
}
