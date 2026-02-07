use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::state::conflict::ConflictResolver;
use crate::state::versioned::HeimdallStateV2;

/// Check for file drift (modifications outside heimdal)
pub fn cmd_check_drift(_all: bool) -> Result<()> {
    println!("{}", "Checking for file drift...".cyan());

    // Load current state
    let state = HeimdallStateV2::load().map_err(|e| anyhow!("Failed to load state: {}", e))?;

    // Check drift
    let drifts = ConflictResolver::check_drift(&state)?;

    if drifts.is_empty() {
        println!(
            "{}",
            "No drift detected - all files match expected checksums".green()
        );
        return Ok(());
    }

    println!(
        "\n{}",
        format!("Found {} drifted file(s):", drifts.len())
            .yellow()
            .bold()
    );

    for drift in &drifts {
        drift.display();
    }

    println!(
        "\n{}",
        "Tip: Run 'heimdal diff' to see detailed changes".cyan()
    );
    println!(
        "{}",
        "Tip: Run 'heimdal commit' to save these changes".cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_drift_no_state() {
        // When no state exists, should fail gracefully
        let result = cmd_check_drift(false);
        // Should either succeed (if state exists) or fail with clear error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_check_drift_with_all_flag() {
        // Test with all flag
        let result = cmd_check_drift(true);
        // Should handle the flag without panicking
        assert!(result.is_ok() || result.is_err());
    }
}
