use anyhow::Result;
use colored::*;

use crate::package::database::{PackageCategory, PackageDatabase};
use crate::utils::header;

/// Run the packages search command
pub fn run_search(query: &str, category: Option<&str>, tag: Option<&str>) -> Result<()> {
    header(&format!("Searching for: {}", query));

    let db = PackageDatabase::new();

    // Parse category if provided
    let category_filter = if let Some(cat_str) = category {
        Some(parse_category(cat_str)?)
    } else {
        None
    };

    // Search packages with fuzzy matching
    let mut results = db.search_fuzzy(query);

    // Get all installed packages once for efficient lookups
    let installed_packages = PackageDatabase::get_installed_packages();

    // Update installation status for each result
    for result in &mut results {
        result.installed = installed_packages.contains(&result.package.name);
    }

    // Filter by category if specified
    let filtered_results: Vec<_> = if let Some(cat) = category_filter {
        results
            .into_iter()
            .filter(|r| r.package.category == cat)
            .collect()
    } else {
        results
    };

    // Filter by tag if specified
    let final_results: Vec<_> = if let Some(tag_filter) = tag {
        let tag_lower = tag_filter.to_lowercase();
        filtered_results
            .into_iter()
            .filter(|r| {
                r.package
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&tag_lower))
            })
            .collect()
    } else {
        filtered_results
    };

    if final_results.is_empty() {
        println!("{}", "No packages found.".yellow());
        println!();
        println!("💡 Tips:");
        println!("  • Try a broader search term");
        println!("  • Check spelling (fuzzy matching is enabled)");
        println!("  • Remove category or tag filters");
        return Ok(());
    }

    println!();
    println!(
        "Found {} package{}:",
        final_results.len(),
        if final_results.len() == 1 { "" } else { "s" }
    );
    println!();

    for result in final_results {
        let pkg = result.package;

        // Installation status indicator
        let status_icon = if result.installed {
            "✓".green().bold()
        } else {
            "○".bright_black()
        };

        // Relevance indicator based on match type
        let relevance = match result.match_kind {
            crate::package::database::MatchKind::Exact
            | crate::package::database::MatchKind::NameContains => {
                "".to_string() // Exact or name matches don't need indicator
            }
            crate::package::database::MatchKind::DescriptionContains => {
                format!(" {}", "★".yellow())
            }
            crate::package::database::MatchKind::TagContains => {
                format!(" {}", "☆".bright_black())
            }
            crate::package::database::MatchKind::Fuzzy => {
                format!(" {}", "·".bright_black())
            }
        };

        println!(
            "  {} {} {}{}",
            status_icon,
            "→".cyan(),
            pkg.name.bold().green(),
            relevance
        );

        println!("    {}", pkg.description.bright_black());
        println!(
            "    Category: {}  Popularity: {}",
            pkg.category.as_str().yellow(),
            format!("{}★", pkg.popularity).bright_black()
        );

        if !pkg.tags.is_empty() {
            println!(
                "    Tags: {}",
                pkg.tags
                    .iter()
                    .map(|t| t.bright_black().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        // Show alternatives if available
        if !pkg.alternatives.is_empty() && pkg.alternatives.len() <= 3 {
            println!(
                "    Alternatives: {}",
                pkg.alternatives
                    .iter()
                    .map(|a| a.cyan().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        println!();
    }

    // Legend
    println!("{}", "Legend:".bright_black());
    println!(
        "  {} = installed, {} = not installed",
        "✓".green().bold(),
        "○".bright_black()
    );
    println!(
        "  {} = highly relevant, {} = relevant, {} = fuzzy match",
        "★".yellow(),
        "☆".bright_black(),
        "·".bright_black()
    );
    println!();

    Ok(())
}

/// Parse category string into PackageCategory enum
fn parse_category(s: &str) -> Result<PackageCategory> {
    match s.to_lowercase().as_str() {
        "essential" => Ok(PackageCategory::Essential),
        "development" | "dev" => Ok(PackageCategory::Development),
        "terminal" | "term" => Ok(PackageCategory::Terminal),
        "editor" => Ok(PackageCategory::Editor),
        "language" | "programming" => Ok(PackageCategory::Language),
        "container" | "containers" => Ok(PackageCategory::Container),
        "infrastructure" | "devops" => Ok(PackageCategory::Infrastructure),
        "database" | "db" => Ok(PackageCategory::Database),
        "network" | "networking" => Ok(PackageCategory::Network),
        "application" | "app" => Ok(PackageCategory::Application),
        "datascience" | "data-science" | "ml" => Ok(PackageCategory::DataScience),
        _ => anyhow::bail!("Unknown category: {}", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category() {
        assert!(matches!(
            parse_category("development").unwrap(),
            PackageCategory::Development
        ));
        assert!(matches!(
            parse_category("editor").unwrap(),
            PackageCategory::Editor
        ));
        assert!(parse_category("invalid").is_err());
    }

    #[test]
    fn test_search_basic() {
        let db = PackageDatabase::new();
        let results = db.search("git");
        assert!(!results.is_empty());
    }
}
