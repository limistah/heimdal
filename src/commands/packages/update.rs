use anyhow::Result;
use colored::Colorize;

use crate::package::database::DatabaseLoader;
use crate::utils::{error, header, info, success};

/// Run the packages update command
pub fn run_update(force: bool) -> Result<()> {
    header("Package Database Update");

    let loader = DatabaseLoader::new()?;

    // Show current cache info if it exists
    if let Ok(Some(metadata)) = loader.get_cache_info() {
        info("Current database:");
        println!("  Version: {}", metadata.version);
        println!("  Last updated: {}", metadata.last_updated);
        println!("  Packages: {}", metadata.package_count);
        println!("  Size: {} KB", metadata.size_bytes / 1024);
        println!();
    }

    // Download the database
    if force {
        info("Force updating package database...");
    } else {
        info("Updating package database...");
    }

    match loader.download() {
        Ok(_) => {
            success("Package database updated successfully!");

            // Show new cache info
            if let Ok(Some(metadata)) = loader.get_cache_info() {
                println!();
                info("New database:");
                println!("  Version: {}", metadata.version);
                println!("  Last updated: {}", metadata.last_updated);
                println!("  Packages: {}", metadata.package_count);
                println!("  Size: {} KB", metadata.size_bytes / 1024);
            }

            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to update package database: {}", e));
            Err(e)
        }
    }
}

/// Show package database cache information
pub fn run_cache_info() -> Result<()> {
    header("Package Database Cache Info");

    let loader = DatabaseLoader::new()?;

    match loader.get_cache_info() {
        Ok(Some(metadata)) => {
            println!("{}: {}", "Version".bold(), metadata.version);
            println!("{}: {}", "Last Updated".bold(), metadata.last_updated);
            println!("{}: {}", "Packages".bold(), metadata.package_count);
            println!(
                "{}: {} KB ({} bytes)",
                "Size".bold(),
                metadata.size_bytes / 1024,
                metadata.size_bytes
            );

            // Calculate age
            if let Ok(parsed_time) = chrono::DateTime::parse_from_rfc3339(&metadata.last_updated) {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(parsed_time);
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;

                if days > 0 {
                    println!("{}: {} days, {} hours ago", "Age".bold(), days, hours);
                } else {
                    println!("{}: {} hours ago", "Age".bold(), hours);
                }

                // Show update recommendation
                if days >= 7 {
                    println!();
                    info("Database is more than 7 days old. Consider updating:");
                    println!("  {}", "heimdal packages update".cyan());
                }
            }

            Ok(())
        }
        Ok(None) => {
            info("No cached database found.");
            println!();
            info("Download the package database with:");
            println!("  {}", "heimdal packages update".cyan());
            Ok(())
        }
        Err(e) => {
            error(&format!("Failed to read cache info: {}", e));
            Err(e)
        }
    }
}

/// Clear the package database cache
pub fn run_cache_clear() -> Result<()> {
    header("Clear Package Database Cache");

    let loader = DatabaseLoader::new()?;

    loader.clear_cache()?;
    success("Package database cache cleared!");

    println!();
    info("Download a fresh database with:");
    println!("  {}", "heimdal packages update".cyan());

    Ok(())
}
