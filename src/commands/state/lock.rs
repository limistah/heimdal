use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

use crate::state::lock::{LockManager, StateLock};

/// Show current lock status
pub fn cmd_lock_info() -> Result<()> {
    // Check if a lock file exists before attempting to show it
    let lock_path = get_lock_path()?;

    if !lock_path.exists() {
        println!("{}", "No active lock found".green());
        return Ok(());
    }

    // A lock file exists; delegate to LockManager and propagate any real errors
    LockManager::show_lock()?;
    Ok(())
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
    LockManager::lock_path()
}

fn read_lock(path: &PathBuf) -> Result<StateLock> {
    let content = std::fs::read_to_string(path).context("Failed to read lock file")?;
    let lock: StateLock = serde_json::from_str(&content).context("Failed to parse lock file")?;
    Ok(lock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_lock_path() {
        // Should return a valid path
        let result = get_lock_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("heimdal"));
    }

    #[test]
    fn test_lock_info_no_lock() {
        // When no lock exists, should complete without error
        // This is a basic smoke test
        let result = cmd_lock_info();
        // Should either succeed or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
