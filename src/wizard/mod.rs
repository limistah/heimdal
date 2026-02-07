mod generator;
mod package_detector;
mod prompts;
mod scanner;

pub use generator::*;
pub use package_detector::*;
pub use scanner::*;

use crate::import::{
    detect_conflicts, detect_tool, import_from_tool, resolve_conflicts, ConflictResolution,
};
use anyhow::Result;
use console::{style, Term};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};

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

/// Helper function to detect the package manager based on the current OS
fn detect_package_manager() -> PackageManager {
    use crate::utils::{detect_os, LinuxDistro, OperatingSystem};

    match detect_os() {
        OperatingSystem::MacOS => PackageManager::Homebrew,
        OperatingSystem::Linux(distro) => match distro {
            LinuxDistro::Debian | LinuxDistro::Ubuntu => PackageManager::Apt,
            LinuxDistro::Fedora | LinuxDistro::Rhel | LinuxDistro::CentOS => PackageManager::Dnf,
            LinuxDistro::Arch | LinuxDistro::Manjaro => PackageManager::Pacman,
            _ => PackageManager::Homebrew, // fallback
        },
        _ => PackageManager::Homebrew, // fallback
    }
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
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        spinner.set_message("Scanning for dotfiles...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        match DotfileScanner::scan_home() {
            Ok(dotfiles) => {
                if dotfiles.is_empty() {
                    spinner
                        .finish_with_message(format!("{} No dotfiles found!", style("⚠").yellow()));

                    println!("\n{}", style("This could mean:").bold());
                    println!("  • You don't have any dotfiles yet (that's ok!)");
                    println!("  • Your dotfiles are in a non-standard location");
                    println!("  • Heimdal couldn't detect your setup");

                    println!("\n{}", style("What would you like to do?").bold());
                    let choice = Select::new()
                        .with_prompt("Choose an option")
                        .items(&[
                            "Continue with empty configuration (add files later)",
                            "Exit wizard and set up manually",
                        ])
                        .default(0)
                        .interact()?;

                    match choice {
                        0 => {
                            println!("\n{} Creating minimal configuration...", style("→").cyan());
                            // Continue with empty scanned_dotfiles
                        }
                        1 => {
                            println!(
                                "\n{} Exiting wizard. Run 'heimdal wizard' when ready!",
                                style("→").cyan()
                            );
                            return Ok(());
                        }
                        _ => unreachable!(),
                    }
                } else {
                    spinner.finish_with_message(format!(
                        "{} Found {} dotfiles",
                        style("✓").green(),
                        dotfiles.len()
                    ));
                    println!();

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

                    // Ask user to select which files to track
                    let file_items: Vec<String> = dotfiles
                        .iter()
                        .map(|f| {
                            format!(
                                "{} ({}, {})",
                                f.relative_path,
                                f.category.as_str(),
                                DotfileScanner::format_size(f.size)
                            )
                        })
                        .collect();

                    let defaults = vec![true; file_items.len()]; // Select all by default

                    let selections = MultiSelect::new()
                        .with_prompt("Select files to track (Space to toggle, Enter to confirm)")
                        .items(&file_items)
                        .defaults(&defaults)
                        .interact()?;

                    scanned_dotfiles = selections.iter().map(|&i| dotfiles[i].clone()).collect();

                    println!(
                        "\n{} Selected {} files to track",
                        style("✓").green(),
                        scanned_dotfiles.len()
                    );
                }
            }
            Err(e) => {
                spinner.finish_with_message(format!(
                    "{} Failed to scan: {}",
                    style("⚠").yellow(),
                    e
                ));
            }
        }
    }

    // Ask how to handle packages
    let package_choice = Select::new()
        .with_prompt("How would you like to set up packages?")
        .items(&[
            "Use a package profile (recommended for new setups)",
            "Detect installed packages on my system",
            "Skip packages for now",
        ])
        .default(0)
        .interact()?;

    let mut detected_packages = Vec::new();

    match package_choice {
        // Use profile
        0 => {
            use crate::package::profiles::{PackageProfile, ProfileSelector};
            use crate::package::PackageDatabase;

            let selector = ProfileSelector::new();
            let options = selector.options();

            // Build display items with name and description
            let items: Vec<String> = options
                .iter()
                .map(|(name, desc)| format!("{} - {}", name, desc))
                .collect();

            println!("\n{} Select a package profile:", style("→").cyan());
            let selected_idx = Select::new()
                .with_prompt("Choose a profile")
                .items(&items)
                .default(1) // Default to Developer
                .interact()?;

            if let Some((name, _)) = options.get(selected_idx) {
                if let Some(profile_type) = selector.get_by_name(name) {
                    let profile = PackageProfile::from_type(profile_type.clone());
                    let packages = profile.resolve_packages();

                    // Create package database for descriptions
                    let db = PackageDatabase::new();

                    println!(
                        "\n{} Profile '{}' includes {} packages:",
                        style("✓").green(),
                        profile_type.display_name(),
                        packages.len()
                    );

                    // Show first 10 packages with descriptions
                    for pkg in packages.iter().take(10) {
                        if let Some(info) = db.get(pkg) {
                            println!("    • {} - {}", style(pkg).cyan(), info.description);
                        } else {
                            println!("    • {}", pkg);
                        }
                    }
                    if packages.len() > 10 {
                        println!("    ... and {} more", packages.len() - 10);
                    }

                    // Convert to DetectedPackage format
                    let manager = detect_package_manager();

                    detected_packages = packages
                        .into_iter()
                        .map(|name| crate::wizard::DetectedPackage {
                            name,
                            manager: manager.clone(),
                            category: crate::wizard::PackageCategory::Development, // Default category
                        })
                        .collect();
                }
            }
        }
        // Detect packages
        1 => {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
            spinner.set_message("Detecting packages...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));

            match PackageDetector::detect_all() {
                Ok(packages) => {
                    let filtered = PackageDetector::filter_common(packages);

                    if filtered.is_empty() {
                        spinner.finish_with_message(format!(
                            "{} No packages found",
                            style("⚠").yellow()
                        ));

                        println!("\n{}", style("Note:").bold());
                        println!("  • This could mean you don't have packages installed yet");
                        println!("  • Or they're installed in a non-standard location");
                        println!("  • You can add packages later using 'heimdal pkg add'");
                    } else {
                        spinner.finish_with_message(format!(
                            "{} Found {} packages",
                            style("✓").green(),
                            filtered.len()
                        ));
                        println!();

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

                        // Ask user to select which packages to track
                        let pkg_items: Vec<String> = filtered
                            .iter()
                            .map(|p| {
                                format!(
                                    "{} (via {}, {})",
                                    p.name,
                                    p.manager.as_str(),
                                    p.category.as_str()
                                )
                            })
                            .collect();

                        let defaults = vec![true; pkg_items.len()]; // Select all by default

                        let selections = MultiSelect::new()
                            .with_prompt(
                                "Select packages to track (Space to toggle, Enter to confirm)",
                            )
                            .items(&pkg_items)
                            .defaults(&defaults)
                            .interact()?;

                        detected_packages =
                            selections.iter().map(|&i| filtered[i].clone()).collect();

                        println!(
                            "\n{} Selected {} packages to track",
                            style("✓").green(),
                            detected_packages.len()
                        );
                    }
                }
                Err(e) => {
                    spinner.finish_with_message(format!(
                        "{} Failed to detect packages: {}",
                        style("⚠").yellow(),
                        e
                    ));
                }
            }
        }
        // Skip packages
        2 => {
            println!("\n{} Skipping package detection", style("→").cyan());
        }
        _ => unreachable!(),
    }

    // Analyze dependencies
    if !detected_packages.is_empty() {
        use crate::package::DependencyAnalyzer;

        println!("\n{} Analyzing package dependencies...", style("→").cyan());

        let analyzer = DependencyAnalyzer::new();
        let package_names: Vec<String> = detected_packages.iter().map(|p| p.name.clone()).collect();

        let analysis = analyzer.analyze(&package_names);

        // Show required missing dependencies
        if analysis.has_required_missing() {
            println!("\n{} Required dependencies:", style("⚠").yellow().bold());
            for missing in &analysis.required_missing {
                println!("  {}", missing.format_message());
            }

            // Ask if user wants to add them
            if Confirm::new()
                .with_prompt("Add required dependencies?")
                .default(true)
                .interact()?
            {
                let manager = detect_package_manager();

                for missing in &analysis.required_missing {
                    detected_packages.push(crate::wizard::DetectedPackage {
                        name: missing.dependency.package.clone(),
                        manager: manager.clone(),
                        category: crate::wizard::PackageCategory::Essential,
                    });
                }

                println!(
                    "{} Added {} required dependencies",
                    style("✓").green(),
                    analysis.required_missing.len()
                );
            }
        }

        // Show optional suggestions
        if analysis.has_optional_missing() || analysis.has_suggestions() {
            let mut all_suggestions = Vec::new();

            // Add optional missing as suggestions
            for missing in &analysis.optional_missing {
                all_suggestions.push((
                    missing.dependency.package.clone(),
                    format!(
                        "Works with {} - {}",
                        missing.for_package, missing.dependency.reason
                    ),
                ));
            }

            // Deduplicate
            all_suggestions.sort_by(|a, b| a.0.cmp(&b.0));
            all_suggestions.dedup_by(|a, b| a.0 == b.0);

            if !all_suggestions.is_empty() {
                println!("\n{} Package suggestions:", style("💡").blue().bold());

                // Show first 5 suggestions
                for (pkg, reason) in all_suggestions.iter().take(5) {
                    println!("  • {} - {}", style(pkg).cyan(), reason);
                }

                if all_suggestions.len() > 5 {
                    println!("  ... and {} more", all_suggestions.len() - 5);
                }

                if Confirm::new()
                    .with_prompt("Add suggested packages?")
                    .default(false)
                    .interact()?
                {
                    let manager = detect_package_manager();

                    for (pkg, _) in &all_suggestions {
                        detected_packages.push(crate::wizard::DetectedPackage {
                            name: pkg.clone(),
                            manager: manager.clone(),
                            category: crate::wizard::PackageCategory::Development,
                        });
                    }

                    println!(
                        "{} Added {} suggested packages",
                        style("✓").green(),
                        all_suggestions.len()
                    );
                }
            }
        }

        // Show summary if all good
        if !analysis.has_required_missing()
            && !analysis.has_optional_missing()
            && !analysis.has_suggestions()
        {
            println!(
                "\n{} All dependencies satisfied!",
                style("✓").green().bold()
            );
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
                Ok(mut import_result) => {
                    println!(
                        "{} Found {} files",
                        style("✓").green(),
                        import_result.dotfiles.len()
                    );

                    // Check for conflicts
                    let conflicts = detect_conflicts(&import_result);

                    if !conflicts.is_empty() {
                        println!(
                            "\n{} Found {} conflicting files (already exist):",
                            style("⚠").yellow().bold(),
                            conflicts.len()
                        );

                        // Show first few conflicts
                        for conflict in conflicts.iter().take(5) {
                            let rel_dest = conflict
                                .destination
                                .strip_prefix(dirs::home_dir().unwrap_or_default())
                                .unwrap_or(&conflict.destination);
                            println!("  • ~/{}", rel_dest.display());
                        }

                        if conflicts.len() > 5 {
                            println!("  ... and {} more", conflicts.len() - 5);
                        }

                        // Ask user how to handle conflicts
                        let choices = vec![
                            "Skip conflicting files",
                            "Backup and overwrite",
                            "Overwrite (no backup)",
                            "Ask for each file",
                        ];

                        let selection = Select::new()
                            .with_prompt("How would you like to handle conflicts?")
                            .items(&choices)
                            .default(0)
                            .interact()?;

                        let strategy = match selection {
                            0 => ConflictResolution::Skip,
                            1 => ConflictResolution::Backup,
                            2 => ConflictResolution::Overwrite,
                            3 => ConflictResolution::Ask,
                            _ => unreachable!(),
                        };

                        // Resolve conflicts
                        match resolve_conflicts(conflicts.clone(), &strategy) {
                            Ok(resolved) => {
                                // Remove all conflicts from import_result
                                import_result.dotfiles.retain(|mapping| {
                                    !conflicts
                                        .iter()
                                        .any(|c| c.destination == mapping.destination)
                                });

                                // Add back the resolved ones
                                import_result.dotfiles.extend(resolved.clone());

                                if strategy == ConflictResolution::Skip {
                                    println!(
                                        "\n{} Skipped {} conflicting files",
                                        style("→").cyan(),
                                        conflicts.len()
                                    );
                                } else {
                                    println!(
                                        "\n{} Resolved {} conflicts",
                                        style("✓").green(),
                                        resolved.len()
                                    );
                                }
                            }
                            Err(e) => {
                                println!(
                                    "\n{} Failed to resolve conflicts: {}",
                                    style("✗").red(),
                                    e
                                );
                            }
                        }
                    }

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
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
            spinner.set_message("Scanning for dotfiles...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));

            // Use existing scanner logic
            let scanner = DotfileScanner::new(path);
            match scanner.scan() {
                Ok(found) => {
                    if !found.is_empty() {
                        spinner.finish_with_message(format!(
                            "{} Found {} files",
                            style("✓").green(),
                            found.len()
                        ));
                        // Could generate config here too
                    } else {
                        spinner.finish_with_message(format!(
                            "{} No dotfiles found",
                            style("ℹ").blue()
                        ));
                    }
                }
                Err(e) => {
                    spinner.finish_with_message(format!(
                        "{} Failed to scan: {}",
                        style("⚠").yellow(),
                        e
                    ));
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
