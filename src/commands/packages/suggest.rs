use anyhow::Result;
use colored::*;
use std::env;

use crate::package::SuggestionEngine;
use crate::utils::header;

/// Run the packages suggest command
pub fn run_suggest(directory: Option<&str>) -> Result<()> {
    header("Smart Package Suggestions");

    let engine = SuggestionEngine::new();

    // Use provided directory or current directory
    let target_dir = if let Some(dir) = directory {
        std::path::PathBuf::from(dir)
    } else {
        env::current_dir()?
    };

    println!();
    println!(
        "Analyzing directory: {}",
        target_dir.display().to_string().cyan()
    );
    println!();

    // Detect tools and suggest packages
    let suggestions = engine.suggest_packages(&target_dir)?;

    if suggestions.is_empty() {
        println!("{}", "No suggestions found for this directory.".yellow());
        println!();
        println!("💡 Tips:");
        println!("  • Make sure you're in a project directory");
        println!("  • Check for common files like package.json, Cargo.toml, etc.");
        println!(
            "  • Use {} to browse all packages",
            "heimdal packages search".cyan()
        );
        return Ok(());
    }

    println!(
        "Found {} suggestion{} based on detected project files:",
        suggestions.len(),
        if suggestions.len() == 1 { "" } else { "s" }
    );
    println!();

    // Group by relevance
    let high_relevance: Vec<_> = suggestions.iter().filter(|s| s.relevance >= 80).collect();
    let medium_relevance: Vec<_> = suggestions
        .iter()
        .filter(|s| s.relevance >= 60 && s.relevance < 80)
        .collect();
    let low_relevance: Vec<_> = suggestions.iter().filter(|s| s.relevance < 60).collect();

    // Display high relevance suggestions
    if !high_relevance.is_empty() {
        println!("{}", "🔥 Highly Recommended".bold().green());
        println!();
        for suggestion in high_relevance {
            print_suggestion(suggestion);
        }
    }

    // Display medium relevance suggestions
    if !medium_relevance.is_empty() {
        println!("{}", "💡 Recommended".bold().yellow());
        println!();
        for suggestion in medium_relevance {
            print_suggestion(suggestion);
        }
    }

    // Display low relevance suggestions
    if !low_relevance.is_empty() && low_relevance.len() <= 5 {
        println!("{}", "📌 Also Consider".bold().bright_black());
        println!();
        for suggestion in low_relevance {
            print_suggestion(suggestion);
        }
    }

    // Footer with action hints
    println!();
    println!("{}", "Quick Actions:".bold());
    println!(
        "  {} <name>         - Install a suggested package",
        "heimdal packages add".cyan()
    );
    println!(
        "  {} <name>         - Get detailed package info",
        "heimdal packages info".cyan()
    );
    println!(
        "  {} <query>       - Search for more packages",
        "heimdal packages search".cyan()
    );

    Ok(())
}

/// Print a single suggestion
fn print_suggestion(suggestion: &crate::package::PackageSuggestion) {
    let pkg = suggestion.package;

    // Relevance indicator
    let relevance_bar = match suggestion.relevance {
        90..=100 => "█████".green(),
        80..=89 => "████░".green(),
        70..=79 => "███░░".yellow(),
        60..=69 => "██░░░".yellow(),
        _ => "█░░░░".bright_black(),
    };

    println!(
        "  {} {} {}",
        relevance_bar,
        "→".cyan(),
        pkg.name.bold().green()
    );
    println!("    {}", pkg.description.bright_black());
    println!("    Reason: {}", suggestion.reason.italic());

    if !suggestion.detected_files.is_empty() && suggestion.detected_files.len() <= 3 {
        let files: Vec<_> = suggestion
            .detected_files
            .iter()
            .take(3)
            .map(|f| {
                std::path::Path::new(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(f)
            })
            .collect();
        println!("    Detected: {}", files.join(", ").bright_black());
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_suggest_for_nodejs_project() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let result = run_suggest(Some(temp.path().to_str().unwrap()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_suggest_for_empty_directory() {
        let temp = TempDir::new().unwrap();

        let result = run_suggest(Some(temp.path().to_str().unwrap()));
        assert!(result.is_ok());
    }
}
