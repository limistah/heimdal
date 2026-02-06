use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::state::conflict::ConflictResolver;
use crate::state::versioned::HeimdallStateV2;

/// Check for file drift (modifications outside heimdal)
pub fn cmd_check_drift(all: bool) -> Result<()> {
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
