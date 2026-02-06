use anyhow::Result;
use colored::*;

use crate::package::database::{PackageCategory, PackageDatabase};
use crate::utils::header;

/// Run the packages search command
pub fn run_search(query: &str, category: Option<&str>) -> Result<()> {
    header(&format!("Searching for: {}", query));

    let db = PackageDatabase::new();

    // Parse category if provided
    let category_filter = if let Some(cat_str) = category {
        Some(parse_category(cat_str)?)
    } else {
        None
    };

    // Search packages
    let results = db.search(query);

    // Filter by category if specified
    let filtered_results: Vec<_> = if let Some(cat) = category_filter {
        results.into_iter().filter(|p| p.category == cat).collect()
    } else {
        results
    };

    if filtered_results.is_empty() {
        println!("{}", "No packages found.".yellow());
        return Ok(());
    }

    println!();
    println!(
        "Found {} package{}:",
        filtered_results.len(),
        if filtered_results.len() == 1 { "" } else { "s" }
    );
    println!();

    for pkg in filtered_results {
        println!("  {} {}", "→".cyan(), pkg.name.bold().green());
        println!("    {}", pkg.description.bright_black());
        println!("    Category: {}", format!("{:?}", pkg.category).yellow());
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
        println!();
    }

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
