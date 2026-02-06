mod generator;
mod package_detector;
mod prompts;
mod scanner;

pub use generator::*;
pub use package_detector::*;
pub use prompts::*;
pub use scanner::*;

use crate::import::{detect_tool, import_from_tool};
use anyhow::Result;
use console::{style, Term};
use dialoguer::{Confirm, Input, Select};

/// Main wizard entry point
pub fn run_wizard() -> Result<()> {
    let term = Term::stderr();
    term.clear_screen()?;

    print_welcome();

    // Step 1: What do you want to do?
    let action = Select::new()
        .with_prompt("What would you like to do?")
        .items(&[
            "Start fresh (create new dotfiles repo)",
            "Import existing dotfiles",
            "Clone existing Heimdal repo",
        ])
        .interact()?;

    match action {
        0 => wizard_start_fresh()?,
        1 => wizard_import()?,
        2 => wizard_clone()?,
        _ => unreachable!(),
    }

    Ok(())
}

fn print_welcome() {
    println!();
    println!("┌──────────────────────────────────────────────────┐");
    println!(
        "│  {}                              │",
        style("Welcome to Heimdal!").bold().cyan()
    );
    println!(
        "│  {}              │",
        style("Universal dotfile & package manager").dim()
    );
    println!("└──────────────────────────────────────────────────┘");
    println!();
}

fn wizard_start_fresh() -> Result<()> {
    println!("\n{} Starting fresh setup...\n", style("✓").green());

    // Ask for dotfiles location
    let dotfiles_path: String = Input::new()
        .with_prompt("Where should we create your dotfiles?")
        .default("~/dotfiles".to_string())
        .interact_text()?;

    let expanded_path = shellexpand::tilde(&dotfiles_path);
    println!(
        "\n{} Will create dotfiles at: {}",
        style("→").cyan(),
        expanded_path
    );

    // Ask about Git repository
    let setup_git = Confirm::new()
        .with_prompt("Initialize as Git repository?")
        .default(true)
        .interact()?;

    let mut repo_url = None;
    if setup_git {
        let url: String = Input::new()
            .with_prompt("Git repository URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()?;

        if !url.is_empty() {
            println!("{} Will set up remote: {}", style("→").cyan(), url);
            repo_url = Some(url);
        }
    }

    // Ask about scanning existing dotfiles
    let scan_existing = Confirm::new()
        .with_prompt("Scan your home directory for existing dotfiles?")
        .default(true)
        .interact()?;

    let mut scanned_dotfiles = Vec::new();
    if scan_existing {
        println!("\n{} Scanning for dotfiles...", style("→").cyan());

        match DotfileScanner::scan_home() {
            Ok(dotfiles) => {
                if dotfiles.is_empty() {
                    println!("{} No dotfiles found", style("ℹ").blue());
                } else {
                    println!("{} Found {} dotfiles\n", style("✓").green(), dotfiles.len());

                    // Group by category
                    let grouped = DotfileScanner::group_by_category(&dotfiles);

                    for (category, files) in grouped {
                        println!("  {} ({}):", style(category.as_str()).bold(), files.len());
                        for file in files.iter().take(5) {
                            println!(
                                "    • {} ({})",
                                file.relative_path,
                                DotfileScanner::format_size(file.size)
                            );
                        }
                        if files.len() > 5 {
                            println!("    ... and {} more", files.len() - 5);
                        }
                        println!();
                    }

                    scanned_dotfiles = dotfiles;
                }
            }
            Err(e) => {
                println!("{} Failed to scan: {}", style("⚠").yellow(), e);
            }
        }
    }

    // Ask about package detection
    let detect_packages = Confirm::new()
        .with_prompt("Detect installed packages on your system?")
        .default(true)
        .interact()?;

    let mut detected_packages = Vec::new();
    if detect_packages {
        println!("\n{} Detecting packages...", style("→").cyan());

        match PackageDetector::detect_all() {
            Ok(packages) => {
                let filtered = PackageDetector::filter_common(packages);

                if filtered.is_empty() {
                    println!("{} No packages found", style("ℹ").blue());
                } else {
                    println!("{} Found {} packages\n", style("✓").green(), filtered.len());

                    // Group by category
                    let grouped = PackageDetector::group_by_category(&filtered);

                    for (category, pkgs) in grouped {
                        println!("  {} ({}):", style(category.as_str()).bold(), pkgs.len());
                        for pkg in pkgs.iter().take(8) {
                            println!("    • {} (via {})", pkg.name, pkg.manager.as_str());
                        }
                        if pkgs.len() > 8 {
                            println!("    ... and {} more", pkgs.len() - 8);
                        }
                        println!();
                    }

                    detected_packages = filtered;
                }
            }
            Err(e) => {
                println!("{} Failed to detect packages: {}", style("⚠").yellow(), e);
            }
        }
    }

    // Generate configuration
    if !scanned_dotfiles.is_empty() || !detected_packages.is_empty() {
        let generate = Confirm::new()
            .with_prompt("Generate heimdal.yaml configuration?")
            .default(true)
            .interact()?;

        if generate {
            println!("\n{} Generating configuration...", style("→").cyan());

            // Ask for profile name
            let profile_name: String = Input::new()
                .with_prompt("Profile name")
                .default("personal".to_string())
                .interact_text()?;

            // Create generator
            let mut generator = ConfigGenerator::new(&profile_name);

            if let Some(url) = repo_url {
                generator = generator.with_repo_url(url);
            }

            generator.add_dotfiles(scanned_dotfiles);
            generator.add_packages(detected_packages);

            // Show preview
            println!("\n{}", style("Configuration preview:").bold());
            println!("{}", style("─".repeat(50)).dim());
            if let Ok(preview) = generator.preview(20) {
                println!("{}", preview);
                println!("{}", style("... (truncated)").dim());
            }
            println!("{}", style("─".repeat(50)).dim());

            // Ask to save
            let save = Confirm::new()
                .with_prompt("Save this configuration?")
                .default(true)
                .interact()?;

            if save {
                let config_path = std::path::Path::new(expanded_path.as_ref()).join("heimdal.yaml");
                match generator.save(&config_path) {
                    Ok(_) => {
                        println!(
                            "\n{} Saved to {}",
                            style("✓").green().bold(),
                            config_path.display()
                        );
                    }
                    Err(e) => {
                        println!("\n{} Failed to save: {}", style("✗").red(), e);
                        println!("  You can manually create the file later.");
                    }
                }
            }
        }
    }

    println!("\n{} Setup complete!", style("✓").green().bold());
    println!("\n{}", style("Next steps:").bold());
    println!("  1. Add your dotfiles to {}", expanded_path);
    println!(
        "  2. Run: {} to apply your configuration",
        style("heimdal apply").cyan()
    );
    println!("  3. Run: {} to see status", style("heimdal status").cyan());

    Ok(())
}

fn wizard_import() -> Result<()> {
    println!("\n{} Importing existing dotfiles...\n", style("→").cyan());

    // Ask where dotfiles are
    let locations = vec![
        "~/dotfiles",
        "~/.dotfiles",
        "~/dotfiles-local",
        "Custom path...",
    ];
    let location_idx = Select::new()
        .with_prompt("Where are your dotfiles?")
        .items(&locations)
        .interact()?;

    let dotfiles_path = if location_idx == locations.len() - 1 {
        // Custom path
        Input::<String>::new()
            .with_prompt("Enter the path to your dotfiles")
            .interact_text()?
    } else {
        locations[location_idx].to_string()
    };

    let expanded_path = shellexpand::tilde(&dotfiles_path);
    println!(
        "\n{} Will import from: {}",
        style("→").cyan(),
        expanded_path
    );

    // Check if directory exists
    let path = std::path::Path::new(expanded_path.as_ref());
    if !path.exists() {
        println!(
            "\n{} Directory does not exist: {}",
            style("✗").red(),
            expanded_path
        );
        println!("  Create it first, then run the wizard again.");
        return Ok(());
    }

    // Detect what kind of setup they have
    println!("\n{} Analyzing directory structure...", style("→").cyan());

    let detected_tool = detect_tool(path);

    if let Some(tool) = detected_tool {
        println!(
            "{} Detected: {} setup",
            style("✓").green(),
            style(tool.name()).bold()
        );

        // Ask if they want to import automatically
        let auto_import = Confirm::new()
            .with_prompt(format!("Convert {} configuration to Heimdal?", tool.name()))
            .default(true)
            .interact()?;

        if auto_import {
            println!("\n{} Importing from {}...", style("→").cyan(), tool.name());

            match import_from_tool(path, &tool) {
                Ok(import_result) => {
                    println!(
                        "{} Found {} files",
                        style("✓").green(),
                        import_result.dotfiles.len()
                    );

                    // Show some of the found files
                    if !import_result.dotfiles.is_empty() {
                        println!("\n{}:", style("Dotfiles to track").bold());
                        for (i, mapping) in import_result.dotfiles.iter().take(10).enumerate() {
                            let rel_source =
                                mapping.source.strip_prefix(path).unwrap_or(&mapping.source);
                            let rel_dest = mapping
                                .destination
                                .strip_prefix(dirs::home_dir().unwrap_or_default())
                                .unwrap_or(&mapping.destination);
                            println!(
                                "  {}. {} → ~/{}",
                                i + 1,
                                style(rel_source.display()).cyan(),
                                rel_dest.display()
                            );
                        }
                        if import_result.dotfiles.len() > 10 {
                            println!(
                                "  {} ... and {} more",
                                style("").dim(),
                                import_result.dotfiles.len() - 10
                            );
                        }
                    }

                    // Generate configuration
                    let generate = Confirm::new()
                        .with_prompt("Generate heimdal.yaml configuration?")
                        .default(true)
                        .interact()?;

                    if generate {
                        println!("\n{} Generating configuration...", style("→").cyan());

                        // Ask for profile name
                        let profile_name: String = Input::new()
                            .with_prompt("Profile name")
                            .default("personal".to_string())
                            .interact_text()?;

                        // Create generator
                        let mut generator = ConfigGenerator::new(&profile_name);

                        // Set Stow compatibility if needed
                        if import_result.stow_compat {
                            generator = generator.with_stow_compat(true);
                        }

                        // Add imported dotfiles
                        let dotfile_paths: Vec<ScannedDotfile> = import_result
                            .dotfiles
                            .iter()
                            .map(|mapping| {
                                let category = match mapping.category.as_deref() {
                                    Some("shell") => DotfileCategory::Shell,
                                    Some("editor") => DotfileCategory::Editor,
                                    Some("git") => DotfileCategory::Git,
                                    Some("ssh") => DotfileCategory::Ssh,
                                    Some("tmux") => DotfileCategory::Tmux,
                                    _ => DotfileCategory::Other,
                                };
                                let relative_path = mapping
                                    .source
                                    .strip_prefix(path)
                                    .unwrap_or(&mapping.source)
                                    .to_string_lossy()
                                    .to_string();
                                let size = mapping.source.metadata().map(|m| m.len()).unwrap_or(0);
                                ScannedDotfile {
                                    path: mapping.source.clone(),
                                    relative_path,
                                    category,
                                    size,
                                }
                            })
                            .collect();

                        generator.add_dotfiles(dotfile_paths);

                        // Add packages if any
                        if !import_result.packages.is_empty() {
                            let detected_packages: Vec<DetectedPackage> = import_result
                                .packages
                                .into_iter()
                                .map(|name| DetectedPackage {
                                    name,
                                    manager: PackageManager::Homebrew, // Default, can be improved
                                    category: PackageCategory::Other,
                                })
                                .collect();
                            generator.add_packages(detected_packages);
                        }

                        // Show preview
                        println!("\n{}", style("Configuration preview:").bold());
                        println!("{}", style("─".repeat(50)).dim());
                        if let Ok(preview) = generator.preview(20) {
                            println!("{}", preview);
                            println!("{}", style("... (truncated)").dim());
                        }
                        println!("{}", style("─".repeat(50)).dim());

                        // Ask to save
                        let save = Confirm::new()
                            .with_prompt("Save this configuration?")
                            .default(true)
                            .interact()?;

                        if save {
                            let config_path = path.join("heimdal.yaml");
                            match generator.save(&config_path) {
                                Ok(_) => {
                                    println!(
                                        "\n{} Saved to {}",
                                        style("✓").green().bold(),
                                        config_path.display()
                                    );
                                }
                                Err(e) => {
                                    println!("\n{} Failed to save: {}", style("✗").red(), e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("\n{} Failed to import: {}", style("✗").red(), e);
                    println!("  You can try manually creating the configuration.");
                }
            }
        }
    } else {
        println!(
            "{} No specific tool detected (manual setup)",
            style("ℹ").blue()
        );

        // Fall back to scanning
        let scan = Confirm::new()
            .with_prompt("Scan directory for dotfiles?")
            .default(true)
            .interact()?;

        if scan {
            println!("\n{} Scanning for dotfiles...", style("→").cyan());
            // Use existing scanner logic
            let scanner = DotfileScanner::new(path);
            match scanner.scan() {
                Ok(found) => {
                    if !found.is_empty() {
                        println!("{} Found {} files", style("✓").green(), found.len());
                        // Could generate config here too
                    } else {
                        println!("{} No dotfiles found", style("ℹ").blue());
                    }
                }
                Err(e) => {
                    println!("{} Failed to scan: {}", style("⚠").yellow(), e);
                }
            }
        }
    }

    println!("\n{} Import complete!", style("✓").green().bold());
    println!("\n{}", style("Next steps:").bold());
    println!("  1. Review the generated heimdal.yaml");
    println!(
        "  2. Run: {} to preview changes",
        style("heimdal apply --dry-run").cyan()
    );
    println!(
        "  3. Run: {} to apply your configuration",
        style("heimdal apply").cyan()
    );

    Ok(())
}

fn wizard_clone() -> Result<()> {
    println!(
        "\n{} Cloning existing Heimdal repository...\n",
        style("→").cyan()
    );

    // Ask for repository URL
    let repo_url: String = Input::new()
        .with_prompt("Git repository URL")
        .interact_text()?;

    if repo_url.is_empty() {
        println!("\n{} Repository URL cannot be empty", style("✗").red());
        return Ok(());
    }

    // Ask where to clone
    let dotfiles_path: String = Input::new()
        .with_prompt("Where should we clone your dotfiles?")
        .default("~/.dotfiles".to_string())
        .interact_text()?;

    let expanded_path = shellexpand::tilde(&dotfiles_path);
    println!("\n{} Will clone to: {}", style("→").cyan(), expanded_path);

    // Ask about profile
    let profile: String = Input::new()
        .with_prompt("Profile name (e.g., 'work-laptop', 'personal')")
        .default("base".to_string())
        .interact_text()?;

    println!("\n{} Cloning repository...", style("→").cyan());
    println!("{} Clone and init not yet implemented", style("ℹ").blue());
    println!(
        "\n  Manually run: {} --repo {} --profile {}",
        style("heimdal init").cyan(),
        repo_url,
        profile
    );

    Ok(())
}
