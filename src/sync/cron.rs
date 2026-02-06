use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

/// Enable auto-sync by adding a cron job
pub fn enable_auto_sync(interval: &str) -> Result<()> {
    // Parse interval (e.g., "1h", "30m", "daily")
    let cron_schedule = parse_interval_to_cron(interval)?;

    // Get the path to the heimdal binary
    let heimdal_path =
        std::env::current_exe().with_context(|| "Failed to get current executable path")?;

    // Create cron entry
    let cron_entry = format!(
        "{} {} sync --quiet\n",
        cron_schedule,
        heimdal_path.display()
    );

    // Get existing crontab
    let output = Command::new("crontab").arg("-l").output();

    let existing_crontab = if let Ok(out) = output {
        if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Remove any existing heimdal sync entries
    let lines: Vec<String> = existing_crontab
        .lines()
        .filter(|line| !line.contains("heimdal") || !line.contains("sync"))
        .map(|s| s.to_string())
        .collect();

    // Add new entry
    let mut new_crontab = lines.join("\n");
    if !new_crontab.is_empty() && !new_crontab.ends_with('\n') {
        new_crontab.push('\n');
    }
    new_crontab.push_str(&cron_entry);

    // Write to temporary file
    let temp_file = "/tmp/heimdal_crontab";
    fs::write(temp_file, &new_crontab).with_context(|| "Failed to write temporary crontab file")?;

    // Install new crontab
    let status = Command::new("crontab")
        .arg(temp_file)
        .status()
        .with_context(|| "Failed to install crontab")?;

    // Clean up
    let _ = fs::remove_file(temp_file);

    if !status.success() {
        anyhow::bail!("Failed to install crontab");
    }

    Ok(())
}

/// Disable auto-sync by removing the cron job
pub fn disable_auto_sync() -> Result<()> {
    // Get existing crontab
    let output = Command::new("crontab")
        .arg("-l")
        .output()
        .with_context(|| "Failed to get existing crontab")?;

    if !output.status.success() {
        anyhow::bail!("No crontab found");
    }

    let existing_crontab = String::from_utf8_lossy(&output.stdout).to_string();

    // Remove heimdal sync entries
    let lines: Vec<String> = existing_crontab
        .lines()
        .filter(|line| !line.contains("heimdal") || !line.contains("sync"))
        .map(|s| s.to_string())
        .collect();

    if lines.len() == existing_crontab.lines().count() {
        anyhow::bail!("No heimdal auto-sync job found in crontab");
    }

    let new_crontab = lines.join("\n");

    if new_crontab.trim().is_empty() {
        // Remove crontab entirely if empty
        let status = Command::new("crontab")
            .arg("-r")
            .status()
            .with_context(|| "Failed to remove crontab")?;

        if !status.success() {
            anyhow::bail!("Failed to remove crontab");
        }
    } else {
        // Write to temporary file
        let temp_file = "/tmp/heimdal_crontab";
        fs::write(temp_file, &new_crontab)
            .with_context(|| "Failed to write temporary crontab file")?;

        // Install new crontab
        let status = Command::new("crontab")
            .arg(temp_file)
            .status()
            .with_context(|| "Failed to install crontab")?;

        // Clean up
        let _ = fs::remove_file(temp_file);

        if !status.success() {
            anyhow::bail!("Failed to install crontab");
        }
    }

    Ok(())
}

/// Check if auto-sync is enabled
pub fn is_auto_sync_enabled() -> Result<Option<String>> {
    let output = Command::new("crontab").arg("-l").output();

    if let Ok(out) = output {
        if out.status.success() {
            let crontab = String::from_utf8_lossy(&out.stdout).to_string();

            for line in crontab.lines() {
                if line.contains("heimdal") && line.contains("sync") {
                    return Ok(Some(line.to_string()));
                }
            }
        }
    }

    Ok(None)
}

/// Parse interval string to cron schedule
fn parse_interval_to_cron(interval: &str) -> Result<String> {
    let interval_lower = interval.to_lowercase();

    if interval_lower == "hourly" || interval_lower == "1h" {
        // Every hour
        Ok("0 * * * *".to_string())
    } else if interval_lower == "daily" || interval_lower == "1d" {
        // Every day at 9 AM
        Ok("0 9 * * *".to_string())
    } else if interval_lower == "weekly" || interval_lower == "1w" {
        // Every Monday at 9 AM
        Ok("0 9 * * 1".to_string())
    } else if interval_lower.ends_with('h') {
        // Parse hours (e.g., "2h", "4h")
        let hours = interval_lower
            .trim_end_matches('h')
            .parse::<u32>()
            .with_context(|| format!("Invalid hour format: {}", interval))?;

        if hours > 24 {
            anyhow::bail!("Hours must be between 1 and 24");
        }

        // Run every N hours
        Ok(format!("0 */{} * * *", hours))
    } else if interval_lower.ends_with('m') {
        // Parse minutes (e.g., "30m", "15m")
        let minutes = interval_lower
            .trim_end_matches('m')
            .parse::<u32>()
            .with_context(|| format!("Invalid minute format: {}", interval))?;

        if minutes > 59 {
            anyhow::bail!("Minutes must be between 1 and 59");
        }

        // Run every N minutes
        Ok(format!("*/{} * * * *", minutes))
    } else {
        anyhow::bail!(
            "Invalid interval format: {}. Use formats like: 1h, 30m, daily, weekly",
            interval
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval() {
        assert_eq!(parse_interval_to_cron("1h").unwrap(), "0 * * * *");
        assert_eq!(parse_interval_to_cron("hourly").unwrap(), "0 * * * *");
        assert_eq!(parse_interval_to_cron("daily").unwrap(), "0 9 * * *");
        assert_eq!(parse_interval_to_cron("weekly").unwrap(), "0 9 * * 1");
        assert_eq!(parse_interval_to_cron("2h").unwrap(), "0 */2 * * *");
        assert_eq!(parse_interval_to_cron("30m").unwrap(), "*/30 * * * *");
    }
}
