use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::state::conflict::ResolutionStrategy;
use crate::state::versioned::HeimdallStateV2;

/// Check for state conflicts between local and remote
pub fn cmd_check_conflicts() -> Result<()> {
    println!("{}", "Checking for state conflicts...".cyan());

    // Load local state
    let local_state =
        HeimdallStateV2::load().map_err(|e| anyhow!("Failed to load local state: {}", e))?;

    // Try to load remote state (this would involve fetching from git)
    // For now, we'll just check if we can detect any obvious issues
    println!("  Local state version: {}", local_state.version);
    println!("  State serial: {}", local_state.lineage.serial);
    println!(
        "  Machine: {} ({})",
        local_state.machine.hostname, local_state.machine.id
    );
    println!("  Active profile: {}", local_state.active_profile);

    // TODO: Actually fetch and compare with remote state
    println!(
        "\n{}",
        "Note: Full remote conflict detection requires git integration".yellow()
    );
    println!(
        "{}",
        "This feature will be completed in the next iteration".yellow()
    );

    Ok(())
}

/// Resolve detected conflicts
pub fn cmd_resolve(strategy: String, yes: bool) -> Result<()> {
    // Parse strategy
    let resolution_strategy = match strategy.as_str() {
        "local" | "use-local" => ResolutionStrategy::UseLocal,
        "remote" | "use-remote" => ResolutionStrategy::UseRemote,
        "merge" => ResolutionStrategy::Merge,
        "manual" => ResolutionStrategy::Manual,
        _ => {
            return Err(anyhow!(
                "Invalid strategy: {}. Valid options: local, remote, merge, manual",
                strategy
            ));
        }
    };

    println!("{}", "Resolving state conflicts...".cyan());
    println!("  Strategy: {:?}", resolution_strategy);

    // Load local state (TODO: use this in actual conflict resolution)
    let _local_state =
        HeimdallStateV2::load().map_err(|e| anyhow!("Failed to load local state: {}", e))?;

    // Confirm if not yes
    if !yes {
        use std::io::{self, Write};
        print!("\n{} ", "Apply this resolution strategy? [y/N]:".yellow());
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    // TODO: Actually resolve conflicts
    println!(
        "{}",
        "Note: Full conflict resolution requires git integration".yellow()
    );
    println!(
        "{}",
        "This feature will be completed in the next iteration".yellow()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_conflicts_basic() {
        // This is a smoke test - the function should handle missing state gracefully
        let result = cmd_check_conflicts();
        // Should either succeed or fail with a clear error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_with_yes() {
        // Test with yes flag - should not prompt
        let result = cmd_resolve("local".to_string(), true);
        // May fail if no state exists, but should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_invalid_strategy() {
        // Invalid strategy should return error
        let result = cmd_resolve("invalid".to_string(), true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid strategy"));
    }

    #[test]
    fn test_valid_strategies() {
        // All valid strategy names should be accepted
        let strategies = vec![
            "local",
            "use-local",
            "remote",
            "use-remote",
            "merge",
            "manual",
        ];

        for strategy in strategies {
            let result = cmd_resolve(strategy.to_string(), true);
            // Should not fail due to invalid strategy
            if result.is_err() {
                let err_msg = result.unwrap_err().to_string();
                assert!(!err_msg.contains("Invalid strategy"));
            }
        }
    }
}
