//! Package groups commands

use crate::package::{GroupRegistry, PackageDatabase};
use crate::utils::{error, header, info, success, warning};
use anyhow::{Context, Result};

/// List all available package groups
pub fn list_groups(category: Option<String>) -> Result<()> {
    let registry = GroupRegistry::new();

    let groups = if let Some(cat) = category {
        header(&format!("Package Groups - Category: {}", cat));
        registry.list_by_category(&cat)
    } else {
        header("Available Package Groups");
        registry.list()
    };

    if groups.is_empty() {
        warning("No package groups found");
        return Ok(());
    }

    let mut current_category = String::new();

    for group in groups {
        // Print category header if it changed
        if group.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            println!("\n{}:", group.category.to_uppercase());
            current_category = group.category.clone();
        }

        println!("  {} ({})", group.name, group.id);
        println!("    {}", group.description);
        println!(
            "    {} required package{}, {} optional",
            group.packages.len(),
            if group.packages.len() == 1 { "" } else { "s" },
            group.optional_packages.len()
        );
    }

    println!();
    info("Use 'heimdal packages show-group <id>' to see package details");
    info("Use 'heimdal packages add-group <id>' to install a group");

    Ok(())
}

/// Show details of a specific package group
pub fn show_group(id: &str) -> Result<()> {
    let registry = GroupRegistry::new();

    let group = registry
        .get(id)
        .context(format!("Package group '{}' not found", id))?;

    header(&format!("Package Group: {}", group.name));
    println!("ID: {}", group.id);
    println!("Category: {}", group.category);
    println!("Description: {}", group.description);
    println!();

    // Show required packages
    println!("Required Packages ({}):", group.packages.len());
    for pkg in &group.packages {
        println!("  • {}", pkg);
    }

    // Show optional packages
    if !group.optional_packages.is_empty() {
        println!();
        println!("Optional Packages ({}):", group.optional_packages.len());
        for pkg in &group.optional_packages {
            println!("  ○ {}", pkg);
        }
    }

    println!();
    info(&format!(
        "Install with: heimdal packages add-group {}",
        group.id
    ));
    if !group.optional_packages.is_empty() {
        info(&format!(
            "Include optional: heimdal packages add-group {} --include-optional",
            group.id
        ));
    }

    Ok(())
}

/// Install all packages from a group
pub fn add_group(id: &str, include_optional: bool, dry_run: bool, no_install: bool) -> Result<()> {
    let registry = GroupRegistry::new();

    let group = registry
        .get(id)
        .context(format!("Package group '{}' not found", id))?;

    header(&format!("Installing Package Group: {}", group.name));
    println!("{}", group.description);
    println!();

    // Determine which packages to install
    let packages_to_install: Vec<&str> = if include_optional {
        group.all_packages()
    } else {
        group.packages.iter().map(|s| s.as_str()).collect()
    };

    info(&format!(
        "Installing {} package{}...",
        packages_to_install.len(),
        if packages_to_install.len() == 1 {
            ""
        } else {
            "s"
        }
    ));
    println!();

    if dry_run {
        warning("DRY RUN - No packages will be installed");
        println!();
    }

    // Get the package database for detailed info
    let db = PackageDatabase::new();

    #[allow(unused_mut)] // Will be used when installation is fully implemented
    let mut installed = 0;
    #[allow(unused_mut)] // Will be used when installation is fully implemented
    let mut already_installed = 0;
    let mut not_found = Vec::new();
    let mut failed = Vec::new();

    for pkg_name in &packages_to_install {
        // Find package in database for better info
        let pkg_info = db.get(pkg_name);

        let display_name = if let Some(info) = pkg_info {
            format!("{} ({})", info.name, info.description)
        } else {
            pkg_name.to_string()
        };

        if dry_run {
            info(&format!("Would install: {}", display_name));
            continue;
        }

        // Check if package exists in database
        if pkg_info.is_none() {
            warning(&format!("Package '{}' not found in database", pkg_name));
            not_found.push(pkg_name.to_string());

            if !no_install {
                warning(
                    "  Skipping installation because the package was not found in the database",
                );
            }
            continue;
        }

        if no_install {
            info(&format!("Skipped installation: {}", display_name));
            continue;
        }

        // NOTE: Group installation is not yet fully implemented
        // This is a simulation for demonstration purposes
        warning(&format!(
            "Note: Package installation from groups is not yet implemented ({})",
            pkg_name
        ));
        failed.push((
            pkg_name.to_string(),
            "group installation not implemented".to_string(),
        ));
        continue;

        // TODO: Implement actual package installation
        // This should integrate with the existing package manager logic
        // e.g., reuse the logic from commands/packages/add.rs
        /*
        // Install the package using the package manager
        info(&format!("Installing: {}", display_name));

        // Simulate installation check
        if is_package_installed_simulation(pkg_name) {
            info(&format!("  ✓ Already installed: {}", pkg_name));
            already_installed += 1;
        } else {
            // In real implementation: call package manager here
            match install_package_simulation(pkg_name) {
                Ok(_) => {
                    success(&format!("  ✓ Installed: {}", pkg_name));
                    installed += 1;
                }
                Err(e) => {
                    error(&format!("  ✗ Failed to install {}: {}", pkg_name, e));
                    failed.push((pkg_name.to_string(), e.to_string()));
                }
            }
        }
        */
    }

    // Print summary
    println!();
    header("Installation Summary");

    if dry_run {
        info(&format!(
            "Would install {} package{}",
            packages_to_install.len(),
            if packages_to_install.len() == 1 {
                ""
            } else {
                "s"
            }
        ));
    } else {
        if installed > 0 {
            success(&format!("✓ Installed: {} packages", installed));
        }
        if already_installed > 0 {
            info(&format!(
                "○ Already installed: {} packages",
                already_installed
            ));
        }
        if !not_found.is_empty() {
            warning(&format!("⚠ Not found: {} packages", not_found.len()));
            for pkg in &not_found {
                warning(&format!("  - {}", pkg));
            }
        }
        if !failed.is_empty() {
            error(&format!("✗ Failed: {} packages", failed.len()));
            for (pkg, err) in &failed {
                error(&format!("  - {}: {}", pkg, err));
            }
        }
    }

    Ok(())
}

/// Search for groups matching a query
pub fn search_groups(query: &str) -> Result<()> {
    let registry = GroupRegistry::new();
    let results = registry.search(query);

    header(&format!("Package Groups matching '{}'", query));

    if results.is_empty() {
        warning("No matching groups found");
        info("Use 'heimdal packages list-groups' to see all available groups");
        return Ok(());
    }

    println!(
        "Found {} matching group{}:\n",
        results.len(),
        if results.len() == 1 { "" } else { "s" }
    );

    let mut current_category = String::new();

    for group in results {
        if group.category != current_category {
            if !current_category.is_empty() {
                println!();
            }
            println!("{}:", group.category.to_uppercase());
            current_category = group.category.clone();
        }

        println!("  {} ({})", group.name, group.id);
        println!("    {}", group.description);
        println!(
            "    {} required, {} optional",
            group.packages.len(),
            group.optional_packages.len()
        );
    }

    println!();
    info("Use 'heimdal packages show-group <id>' for details");

    Ok(())
}

// Simulation functions for testing (will be replaced with real implementation)
fn is_package_installed_simulation(_pkg: &str) -> bool {
    // In real implementation, check if package is actually installed
    false
}

fn install_package_simulation(_pkg: &str) -> Result<()> {
    // In real implementation, call actual package manager
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_groups() {
        let result = list_groups(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_groups_by_category() {
        let result = list_groups(Some("development".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_group() {
        let result = show_group("web-dev");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_invalid_group() {
        let result = show_group("invalid-group-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_search_groups() {
        let result = search_groups("rust");
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_groups_no_results() {
        let result = search_groups("xyzabc123nonexistent");
        assert!(result.is_ok());
    }
}
