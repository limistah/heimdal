use anyhow::Result;
use colored::*;

use crate::package::database::PackageDatabase;
use crate::package::dependencies::DependencyAnalyzer;
use crate::utils::header;

/// Run the packages info command
pub fn run_info(package_name: &str) -> Result<()> {
    header(&format!("Package Information: {}", package_name));

    let db = PackageDatabase::new();

    // Get package info from database
    let package_info = db.get(package_name);

    if let Some(info) = package_info {
        println!();
        println!("{}", info.name.bold().green());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        println!("{}", info.description);
        println!();

        println!(
            "{}: {}",
            "Category".bright_black(),
            format!("{:?}", info.category).yellow()
        );

        if !info.tags.is_empty() {
            println!(
                "{}: {}",
                "Tags".bright_black(),
                info.tags.join(", ").bright_black()
            );
        }

        if !info.alternatives.is_empty() {
            println!();
            println!("{}", "Alternatives".bold());
            for alt in &info.alternatives {
                println!("  {} {}", "→".cyan(), alt);
            }
        }

        if !info.related.is_empty() {
            println!();
            println!("{}", "Related Packages".bold());
            for rel in &info.related {
                println!("  {} {}", "→".cyan(), rel);
            }
        }

        // Show dependencies
        let analyzer = DependencyAnalyzer::new();
        let graph = analyzer.graph();
        let deps = graph.get_dependencies(package_name);
        if !deps.is_empty() {
            println!();
            println!("{}", "Dependencies".bold());
            for dep in deps {
                let icon = if dep.required { "⚠" } else { "💡" };
                let label = if dep.required {
                    "(required)"
                } else {
                    "(optional)"
                };
                println!("  {} {} {}", icon, dep.package.cyan(), label.bright_black());
            }
        }

        println!();
    } else {
        println!();
        println!(
            "{}",
            format!("Package '{}' not found in database.", package_name).yellow()
        );
        println!();
        println!("{}", "Note:".bright_black());
        println!(
            "  {}",
            "This package may still be available from package managers.".bright_black()
        );
        println!(
            "  {}",
            "The database contains only popular packages.".bright_black()
        );
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_existing_package() {
        // Test with a package we know exists
        let db = PackageDatabase::new();
        let pkg = db.get("git");
        assert!(pkg.is_some());
    }

    #[test]
    fn test_info_nonexistent_package() {
        let db = PackageDatabase::new();
        let pkg = db.get("this-package-definitely-does-not-exist-12345");
        assert!(pkg.is_none());
    }
}
