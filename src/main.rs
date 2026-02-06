use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;

mod cli;
mod commands;
mod config;
mod git;
mod hooks;
mod import;
mod package;
mod profile;
mod state;
mod symlink;
mod sync;
mod templates;
mod utils;
mod wizard;

use cli::{
    AutoSyncAction, BranchAction, Cli, Commands, ConfigAction, PackagesAction, ProfileAction,
    RemoteAction,
};
use utils::{error, header, info, success};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logger
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Execute command
    match cli.command {
        Commands::Wizard => {
            wizard::run_wizard()?;
        }
        Commands::Import { path, from, output } => {
            cmd_import(path.as_deref(), &from, output.as_deref())?;
        }
        Commands::Init {
            profile,
            repo,
            path,
        } => {
            cmd_init(&profile, &repo, path.as_deref())?;
        }
        Commands::Apply { dry_run, force } => {
            cmd_apply(dry_run, force)?;
        }
        Commands::Sync { quiet, dry_run } => {
            cmd_sync(quiet, dry_run)?;
        }
        Commands::Status { verbose } => {
            commands::run_status(verbose)?;
        }
        Commands::Diff {
            verbose,
            interactive,
        } => {
            commands::run_diff(verbose, interactive)?;
        }
        Commands::Commit {
            message,
            auto,
            push,
            files,
        } => {
            cmd_commit(message.as_deref(), auto, push, files)?;
        }
        Commands::Push { remote, branch } => {
            cmd_push(remote.as_deref(), branch.as_deref())?;
        }
        Commands::Pull { rebase } => {
            cmd_pull(rebase)?;
        }
        Commands::Branch { action } => match action {
            cli::BranchAction::Current => {
                cmd_branch_current()?;
            }
            cli::BranchAction::List => {
                cmd_branch_list()?;
            }
            cli::BranchAction::Create { name } => {
                cmd_branch_create(&name)?;
            }
            cli::BranchAction::Switch { name } => {
                cmd_branch_switch(&name)?;
            }
            cli::BranchAction::Info => {
                cmd_branch_info()?;
            }
        },
        Commands::Remote { action } => match action {
            RemoteAction::List { verbose } => {
                cmd_remote_list(verbose)?;
            }
            RemoteAction::Add { name, url } => {
                cmd_remote_add(&name, &url)?;
            }
            RemoteAction::Remove { name } => {
                cmd_remote_remove(&name)?;
            }
            RemoteAction::SetUrl { name, url } => {
                cmd_remote_set_url(&name, &url)?;
            }
            RemoteAction::Show { name } => {
                cmd_remote_show(&name)?;
            }
            RemoteAction::Setup => {
                cmd_remote_setup()?;
            }
        },
        Commands::Profiles => {
            cmd_profiles()?;
        }
        Commands::Profile { action } => match action {
            ProfileAction::Switch { name, no_apply } => {
                cmd_profile_switch(&name, !no_apply)?;
            }
            ProfileAction::Current => {
                cmd_profile_current()?;
            }
            ProfileAction::Show { name, resolved } => {
                cmd_profile_show(name.as_deref(), resolved)?;
            }
            ProfileAction::List { verbose } => {
                cmd_profile_list(verbose)?;
            }
            ProfileAction::Diff { profile1, profile2 } => {
                cmd_profile_diff(profile1.as_deref(), &profile2)?;
            }
            ProfileAction::Templates => {
                cmd_profile_templates()?;
            }
            ProfileAction::Create { name, template } => {
                cmd_profile_create(&name, &template)?;
            }
            ProfileAction::Clone { source, target } => {
                cmd_profile_clone(&source, &target)?;
            }
        },
        Commands::Rollback { target } => {
            cmd_rollback(target.as_deref())?;
        }
        Commands::AutoSync { action } => match action {
            AutoSyncAction::Enable { interval } => {
                cmd_auto_sync_enable(&interval)?;
            }
            AutoSyncAction::Disable => {
                cmd_auto_sync_disable()?;
            }
            AutoSyncAction::Status => {
                cmd_auto_sync_status()?;
            }
        },
        Commands::Validate { config } => {
            cmd_validate(config.as_deref())?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Get { key } => {
                cmd_config_get(&key)?;
            }
            ConfigAction::Set { key, value } => {
                cmd_config_set(&key, &value)?;
            }
            ConfigAction::Show => {
                cmd_config_show()?;
            }
        },
        Commands::History { limit } => {
            cmd_history(limit)?;
        }
        Commands::Packages { action } => match action {
            PackagesAction::Add {
                name,
                manager,
                profile,
                no_install,
            } => {
                commands::packages::run_add(
                    &name,
                    manager.as_deref(),
                    profile.as_deref(),
                    no_install,
                )?;
            }
            PackagesAction::Remove {
                name,
                profile,
                force,
                no_uninstall,
            } => {
                commands::packages::run_remove(&name, profile.as_deref(), force, no_uninstall)?;
            }
            PackagesAction::Search { query, category } => {
                commands::packages::run_search(&query, category.as_deref())?;
            }
            PackagesAction::Info { name } => {
                commands::packages::run_info(&name)?;
            }
            PackagesAction::List { installed, profile } => {
                commands::packages::run_list(installed, profile.as_deref())?;
            }
        },
    }

    Ok(())
}

fn cmd_init(profile: &str, repo: &str, path: Option<&str>) -> Result<()> {
    header("Initializing Heimdal");

    // Determine dotfiles path (default: ~/.dotfiles)
    let dotfiles_path = if let Some(p) = path {
        PathBuf::from(shellexpand::tilde(p).as_ref())
    } else {
        PathBuf::from(shellexpand::tilde("~/.dotfiles").as_ref())
    };

    info(&format!("Profile: {}", profile));
    info(&format!("Repository: {}", repo));
    info(&format!("Dotfiles path: {}", dotfiles_path.display()));

    // Check if dotfiles directory already exists
    if dotfiles_path.exists() {
        anyhow::bail!(
            "Dotfiles directory already exists: {}\nIf you want to reinitialize, remove it first.",
            dotfiles_path.display()
        );
    }

    // Clone the repository
    info(&format!("Cloning repository: {}", repo));
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg("--recurse-submodules")
        .arg(repo)
        .arg(&dotfiles_path)
        .status()
        .with_context(|| "Failed to execute git clone. Is git installed?")?;

    if !status.success() {
        anyhow::bail!("Failed to clone repository. Check the URL and your network connection.");
    }

    success(&format!(
        "Repository cloned to: {}",
        dotfiles_path.display()
    ));

    // Verify heimdal.yaml exists
    let config_path = dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        anyhow::bail!(
            "Repository does not contain heimdal.yaml file. This doesn't appear to be a Heimdal-managed dotfiles repository."
        );
    }

    // Load and validate config
    info("Validating configuration...");
    let config = config::load_config(&config_path)?;
    config::validate_config(&config)?;

    // Verify profile exists
    if !config.profiles.contains_key(profile) {
        anyhow::bail!(
            "Profile '{}' not found in configuration. Available profiles: {}",
            profile,
            config
                .profiles
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    success("Configuration validated successfully!");

    // Save state
    info("Saving state...");
    let state =
        state::HeimdallState::new(profile.to_string(), dotfiles_path.clone(), repo.to_string());
    state.save()?;

    success(&format!(
        "State saved to: {}",
        state::HeimdallState::state_path()?.display()
    ));

    // Print next steps
    header("Initialization Complete!");
    info("Next steps:");
    info("  1. Review your configuration: cat ~/.dotfiles/heimdal.yaml");
    info("  2. Apply the configuration: heimdal apply");
    info("  3. Check status: heimdal status");

    Ok(())
}

fn cmd_apply(dry_run: bool, force: bool) -> Result<()> {
    header("Applying Configuration");
    if dry_run {
        info("Dry-run mode: no changes will be made");
    }

    // Try to load state first
    let state_result = state::HeimdallState::load();

    let (config_path, profile_name) = if let Ok(state) = state_result {
        // Use state file for profile and path
        let config_path = state.dotfiles_path.join("heimdal.yaml");
        if !config_path.exists() {
            anyhow::bail!(
                "Configuration file not found at: {}\nYour dotfiles directory may have been moved or deleted.",
                config_path.display()
            );
        }
        (config_path, state.active_profile)
    } else {
        // Fall back to searching for heimdal.yaml (for testing/development)
        let config_paths = vec!["heimdal.yaml", "~/.dotfiles/heimdal.yaml"];

        let mut config_path = None;
        for path_str in config_paths {
            let expanded = shellexpand::tilde(path_str);
            let path = std::path::Path::new(expanded.as_ref());
            if path.exists() {
                config_path = Some(path.to_path_buf());
                break;
            }
        }

        let config_path = config_path
            .ok_or_else(|| anyhow::anyhow!("No heimdal.yaml found. Run 'heimdal init' first."))?;

        // Use first profile as fallback
        let temp_config = config::load_config(&config_path)?;
        let profile_name = temp_config.profiles.keys().next().unwrap().clone();

        (config_path, profile_name)
    };

    // Canonicalize config path
    let config_path = config_path.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize config path: {}",
            config_path.display()
        )
    })?;

    info(&format!("Loading config: {}", config_path.display()));
    info(&format!("Using profile: {}", profile_name));

    // Determine dotfiles directory (parent of heimdal.yaml)
    let dotfiles_dir = config_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid config path"))?
        .to_path_buf();

    // Load and validate config
    let config = config::load_config(&config_path)?;
    config::validate_config(&config)?;

    // Resolve profile
    let resolved = config::resolve_profile(&config, &profile_name)?;

    // Execute global pre_apply hooks
    if !config.hooks.pre_apply.is_empty() {
        info("Running global pre_apply hooks...");
        hooks::execute_hooks(
            &config.hooks.pre_apply,
            dry_run,
            hooks::HookContext::PreApply,
        )?;
    }

    // Execute profile pre_apply hooks
    if let Some(profile) = config.profiles.get(&profile_name) {
        if !profile.hooks.pre_apply.is_empty() {
            info("Running profile pre_apply hooks...");
            hooks::execute_hooks(
                &profile.hooks.pre_apply,
                dry_run,
                hooks::HookContext::PreApply,
            )?;
        }
    }

    // Install packages
    let pkg_report = package::install_packages(&resolved, &config.mappings, dry_run)?;
    pkg_report.print_summary();

    // Create symlinks
    let sym_report = symlink::create_symlinks(&resolved, &dotfiles_dir, dry_run, force)?;
    sym_report.print_summary();

    // Execute profile post_apply hooks
    if let Some(profile) = config.profiles.get(&profile_name) {
        if !profile.hooks.post_apply.is_empty() {
            info("Running profile post_apply hooks...");
            hooks::execute_hooks(
                &profile.hooks.post_apply,
                dry_run,
                hooks::HookContext::PostApply,
            )?;
        }
    }

    // Execute global post_apply hooks
    if !config.hooks.post_apply.is_empty() {
        info("Running global post_apply hooks...");
        hooks::execute_hooks(
            &config.hooks.post_apply,
            dry_run,
            hooks::HookContext::PostApply,
        )?;
    }

    // Update state with apply time (if not dry-run)
    if !dry_run {
        if let Ok(mut state) = state::HeimdallState::load() {
            state.update_apply_time();
            state.save()?;
        }
    }

    success("Configuration applied successfully!");

    Ok(())
}

fn cmd_sync(quiet: bool, dry_run: bool) -> Result<()> {
    if !quiet {
        header("Syncing Configuration");
    }

    // Load state
    let mut state = state::HeimdallState::load()?;

    if !quiet {
        info(&format!("Dotfiles path: {}", state.dotfiles_path.display()));
        info(&format!("Active profile: {}", state.active_profile));
    }

    // Load config for hooks
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = config::load_config(&config_path)?;

    // Execute global pre_sync hooks
    if !config.hooks.pre_sync.is_empty() {
        if !quiet {
            info("Running global pre_sync hooks...");
        }
        hooks::execute_hooks(&config.hooks.pre_sync, dry_run, hooks::HookContext::PreSync)?;
    }

    // Execute profile pre_sync hooks
    if let Some(profile) = config.profiles.get(&state.active_profile) {
        if !profile.hooks.pre_sync.is_empty() {
            if !quiet {
                info("Running profile pre_sync hooks...");
            }
            hooks::execute_hooks(
                &profile.hooks.pre_sync,
                dry_run,
                hooks::HookContext::PreSync,
            )?;
        }
    }

    // Initialize git repo
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Pull from git using new sync module
    if !quiet {
        info("Pulling latest changes from git...");
    }

    if !dry_run {
        let options = git::SyncOptions {
            pull: true,
            push: false,
            rebase: false,
            auto_stash: true,
            dry_run: false,
        };

        match repo.sync(&options)? {
            git::SyncResult::Success => {
                if !quiet {
                    success("Git pull completed successfully!");
                }

                // Update sync time
                state.update_sync_time();
                state.save()?;
            }
            git::SyncResult::Conflicts(files) => {
                error("Merge conflicts detected in:");
                for file in &files {
                    println!("  - {}", file);
                }
                println!();
                info("Please resolve conflicts manually and run 'heimdal sync' again");
                anyhow::bail!("Merge conflicts detected");
            }
            git::SyncResult::UpToDate | git::SyncResult::NothingToSync => {
                if !quiet {
                    info("Already up to date");
                }
            }
        }
    } else {
        if !quiet {
            info("Dry-run: Would pull from git");
        }
    }

    // Apply configuration
    if !quiet {
        info("Applying configuration...");
    }

    cmd_apply(dry_run, false)?;

    // Execute profile post_sync hooks
    if let Some(profile) = config.profiles.get(&state.active_profile) {
        if !profile.hooks.post_sync.is_empty() {
            if !quiet {
                info("Running profile post_sync hooks...");
            }
            hooks::execute_hooks(
                &profile.hooks.post_sync,
                dry_run,
                hooks::HookContext::PostSync,
            )?;
        }
    }

    // Execute global post_sync hooks
    if !config.hooks.post_sync.is_empty() {
        if !quiet {
            info("Running global post_sync hooks...");
        }
        hooks::execute_hooks(
            &config.hooks.post_sync,
            dry_run,
            hooks::HookContext::PostSync,
        )?;
    }

    if !quiet {
        success("Sync completed successfully!");
    }

    Ok(())
}

fn cmd_profiles() -> Result<()> {
    header("Available Profiles");

    // Try to find heimdal.yaml in current directory or ~/.dotfiles
    let config_paths = vec!["heimdal.yaml", "~/.dotfiles/heimdal.yaml"];

    for path_str in config_paths {
        let expanded = shellexpand::tilde(path_str);
        let path = std::path::Path::new(expanded.as_ref());

        if path.exists() {
            match config::load_config(path) {
                Ok(config) => {
                    info(&format!("Found config: {}", path.display()));
                    println!();
                    for (name, profile) in &config.profiles {
                        if let Some(extends) = &profile.extends {
                            println!("  {} (extends: {})", name, extends);
                        } else {
                            println!("  {}", name);
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    error(&format!("Failed to load config: {}", e));
                    return Err(e);
                }
            }
        }
    }

    error("No heimdal.yaml found in current directory or ~/.dotfiles");
    info("Run 'heimdal init' to set up Heimdal on this machine");

    Ok(())
}

fn cmd_rollback(target: Option<&str>) -> Result<()> {
    header("Rolling Back Configuration");

    // Load state
    let state = state::HeimdallState::load()?;

    info(&format!("Dotfiles path: {}", state.dotfiles_path.display()));

    // Determine target commit
    let target_commit = target.unwrap_or("HEAD^");
    info(&format!("Rolling back to: {}", target_commit));

    // Show what will be rolled back
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&state.dotfiles_path)
        .arg("log")
        .arg("--oneline")
        .arg("-n")
        .arg("1")
        .arg(target_commit)
        .output()
        .with_context(|| "Failed to get commit information")?;

    if !output.status.success() {
        anyhow::bail!("Invalid commit target: {}", target_commit);
    }

    let commit_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
    info(&format!("Target commit: {}", commit_info));

    // Ask for confirmation
    print!(
        "\nAre you sure you want to rollback? This will reset your dotfiles repository. [y/N]: "
    );
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        info("Rollback cancelled");
        return Ok(());
    }

    // Perform rollback
    info("Performing git reset...");
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(&state.dotfiles_path)
        .arg("reset")
        .arg("--hard")
        .arg(target_commit)
        .status()
        .with_context(|| "Failed to execute git reset")?;

    if !status.success() {
        anyhow::bail!("Git reset failed");
    }

    success("Rollback completed successfully!");

    // Re-apply configuration
    info("Re-applying configuration...");
    cmd_apply(false, false)?;

    success("Configuration rolled back and applied successfully!");

    Ok(())
}

fn cmd_auto_sync_enable(interval: &str) -> Result<()> {
    header("Enabling Auto-Sync");
    info(&format!("Interval: {}", interval));

    // Verify Heimdal is initialized
    let _state = state::HeimdallState::load()?;

    // Enable auto-sync
    sync::enable_auto_sync(interval).with_context(|| "Failed to enable auto-sync")?;

    success("Auto-sync enabled successfully!");
    info("Your dotfiles will now sync automatically in the background");
    info(&format!("To check status, run: heimdal auto-sync status"));

    Ok(())
}

fn cmd_auto_sync_disable() -> Result<()> {
    header("Disabling Auto-Sync");

    // Disable auto-sync
    sync::disable_auto_sync().with_context(|| "Failed to disable auto-sync")?;

    success("Auto-sync disabled successfully!");

    Ok(())
}

fn cmd_auto_sync_status() -> Result<()> {
    header("Auto-Sync Status");

    match sync::is_auto_sync_enabled()? {
        Some(cron_entry) => {
            success("Auto-sync is ENABLED");
            info(&format!("Cron entry: {}", cron_entry));
        }
        None => {
            info("Auto-sync is DISABLED");
            info("To enable auto-sync, run: heimdal auto-sync enable <interval>");
            info("Example: heimdal auto-sync enable 1h");
        }
    }

    Ok(())
}

fn cmd_validate(config_path: Option<&str>) -> Result<()> {
    header("Validating Configuration");

    let path = config_path.unwrap_or("heimdal.yaml");
    let expanded = shellexpand::tilde(path);
    let config_file = std::path::Path::new(expanded.as_ref());

    if !config_file.exists() {
        error(&format!("Config file not found: {}", config_file.display()));
        anyhow::bail!("Config file not found");
    }

    info(&format!("Loading config: {}", config_file.display()));

    match config::load_config(config_file) {
        Ok(config) => {
            success("YAML syntax is valid");

            info("Validating configuration...");
            match config::validate_config(&config) {
                Ok(_) => {
                    success("Configuration is valid");

                    // Show summary
                    println!();
                    info(&format!("Version: {}", config.heimdal.version));
                    info(&format!("Repo: {}", config.heimdal.repo));
                    info(&format!("Profiles: {}", config.profiles.len()));

                    for (name, profile) in &config.profiles {
                        if let Some(extends) = &profile.extends {
                            println!("  - {} (extends: {})", name, extends);
                        } else {
                            println!("  - {}", name);
                        }
                    }
                }
                Err(e) => {
                    error(&format!("Validation failed: {}", e));
                    return Err(e);
                }
            }
        }
        Err(e) => {
            error(&format!("Failed to parse YAML: {}", e));
            return Err(e);
        }
    }

    Ok(())
}

fn cmd_commit(message: Option<&str>, auto: bool, push: bool, files: Vec<String>) -> Result<()> {
    header("Commit Changes");

    // Load state to get dotfiles path
    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if there are changes
    if !repo.has_changes()? {
        info("No changes to commit");
        return Ok(());
    }

    if auto {
        // Auto-generate commit message
        info("Auto-generating commit message...");
        repo.commit_auto(push, false)?;
    } else {
        // Use provided message or prompt for one
        let commit_message = if let Some(msg) = message {
            msg.to_string()
        } else {
            // Prompt for message
            use dialoguer::Input;
            Input::<String>::new()
                .with_prompt("Commit message")
                .interact_text()?
        };

        let options = git::commit::CommitOptions {
            message: commit_message,
            files: if files.is_empty() { None } else { Some(files) },
            push,
            dry_run: false,
        };

        repo.commit(&options)?;
    }

    Ok(())
}

fn cmd_push(_remote: Option<&str>, _branch: Option<&str>) -> Result<()> {
    header("Push to Remote");

    // Load state to get dotfiles path
    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if there are local commits to push
    if !repo.is_ahead_of_remote()? {
        info("Nothing to push - repository is up to date");
        return Ok(());
    }

    info("Pushing changes to remote...");
    repo.push()?;
    success("Pushed successfully");

    Ok(())
}

fn cmd_pull(rebase: bool) -> Result<()> {
    header("Pull from Remote");

    // Load state to get dotfiles path
    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    info("Pulling changes from remote...");
    repo.pull(rebase)?;
    success("Pulled successfully");

    // Check if we need to reapply
    if repo.has_changes()? {
        info("Changes detected. Run 'heimdal apply' to update your system");
    }

    Ok(())
}

fn cmd_branch_current() -> Result<()> {
    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    let branch = repo.current_branch()?;
    println!("{}", branch);

    Ok(())
}

fn cmd_branch_list() -> Result<()> {
    header("Git Branches");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    let current = repo.current_branch()?;
    let branches = repo.list_branches()?;

    for branch in branches {
        if branch == current {
            println!("* {}", branch.green());
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}

fn cmd_branch_create(name: &str) -> Result<()> {
    header("Create Branch");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    info(&format!("Creating branch '{}'...", name));
    repo.create_branch(name)?;

    info(&format!("Switching to branch '{}'...", name));
    repo.switch_branch(name)?;

    success(&format!("Created and switched to branch '{}'", name));

    Ok(())
}

fn cmd_branch_switch(name: &str) -> Result<()> {
    header("Switch Branch");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    info(&format!("Switching to branch '{}'...", name));
    repo.switch_branch(name)?;
    success(&format!("Switched to branch '{}'", name));

    Ok(())
}

fn cmd_branch_info() -> Result<()> {
    header("Branch Information");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    let tracking = repo.get_tracking_info()?;
    println!("{}", tracking.format());

    Ok(())
}

fn cmd_remote_list(verbose: bool) -> Result<()> {
    if verbose {
        header("Git Remotes (with URLs)");
    } else {
        header("Git Remotes");
    }

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    let remotes = repo.list_remotes()?;

    if remotes.is_empty() {
        info("No remotes configured");
        println!();
        info("Add a remote with: heimdal remote add <name> <url>");
        return Ok(());
    }

    for remote in remotes {
        if verbose {
            match repo.get_remote_url(&remote) {
                Ok(url) => println!("{}\t{}", remote, url),
                Err(_) => println!("{}\t<error getting URL>", remote),
            }
        } else {
            println!("{}", remote);
        }
    }

    Ok(())
}

fn cmd_remote_add(name: &str, url: &str) -> Result<()> {
    header("Add Remote");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if remote already exists
    if repo.has_remote(name)? {
        error(&format!("Remote '{}' already exists", name));
        info(&format!(
            "Use 'heimdal remote set-url {} <url>' to change the URL",
            name
        ));
        anyhow::bail!("Remote already exists");
    }

    info(&format!("Adding remote '{}' -> {}", name, url));
    repo.add_remote(name, url)?;
    success(&format!("Added remote '{}'", name));

    Ok(())
}

fn cmd_remote_remove(name: &str) -> Result<()> {
    header("Remove Remote");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if remote exists
    if !repo.has_remote(name)? {
        error(&format!("Remote '{}' does not exist", name));
        anyhow::bail!("Remote not found");
    }

    info(&format!("Removing remote '{}'...", name));
    repo.remove_remote(name)?;
    success(&format!("Removed remote '{}'", name));

    Ok(())
}

fn cmd_remote_set_url(name: &str, url: &str) -> Result<()> {
    header("Set Remote URL");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if remote exists
    if !repo.has_remote(name)? {
        error(&format!("Remote '{}' does not exist", name));
        info(&format!(
            "Use 'heimdal remote add {} <url>' to add it",
            name
        ));
        anyhow::bail!("Remote not found");
    }

    info(&format!("Setting URL for remote '{}' to: {}", name, url));
    repo.set_remote_url(name, url)?;
    success(&format!("Updated remote '{}'", name));

    Ok(())
}

fn cmd_remote_show(name: &str) -> Result<()> {
    header(&format!("Remote: {}", name));

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check if remote exists
    if !repo.has_remote(name)? {
        error(&format!("Remote '{}' does not exist", name));
        anyhow::bail!("Remote not found");
    }

    let url = repo.get_remote_url(name)?;
    println!("URL: {}", url);

    Ok(())
}

fn cmd_remote_setup() -> Result<()> {
    use dialoguer::{Confirm, Input};

    header("Interactive Remote Setup");

    let state = state::HeimdallState::load()?;
    let repo = git::GitRepo::new(&state.dotfiles_path)?;

    // Check current remotes
    let remotes = repo.list_remotes()?;

    if !remotes.is_empty() {
        info("Current remotes:");
        for remote in &remotes {
            if let Ok(url) = repo.get_remote_url(remote) {
                println!("  {} -> {}", remote, url);
            }
        }
        println!();

        let proceed = Confirm::new()
            .with_prompt("Do you want to add another remote?")
            .default(false)
            .interact()?;

        if !proceed {
            info("Setup cancelled");
            return Ok(());
        }
    }

    // Get remote name
    let name: String = Input::new()
        .with_prompt("Remote name (e.g., origin, upstream)")
        .default("origin".to_string())
        .interact_text()?;

    // Check if it exists
    if repo.has_remote(&name)? {
        let replace = Confirm::new()
            .with_prompt(format!("Remote '{}' already exists. Replace it?", name))
            .default(false)
            .interact()?;

        if !replace {
            info("Setup cancelled");
            return Ok(());
        }

        info(&format!("Removing existing remote '{}'...", name));
        repo.remove_remote(&name)?;
    }

    // Get remote URL
    let url: String = Input::new()
        .with_prompt("Remote URL (SSH or HTTPS)")
        .interact_text()?;

    // Add the remote
    info(&format!("Adding remote '{}' -> {}", name, url));
    repo.add_remote(&name, &url)?;
    success(&format!("Added remote '{}'", name));

    // Ask if they want to push
    let should_push = Confirm::new()
        .with_prompt("Do you want to push your current branch to this remote?")
        .default(true)
        .interact()?;

    if should_push {
        let branch = repo.current_branch()?;
        info(&format!(
            "Pushing branch '{}' to '{}/{}'...",
            branch, name, branch
        ));

        match repo.push_to(Some(&name), Some(&branch)) {
            Ok(_) => {
                success("Push successful!");
            }
            Err(e) => {
                error(&format!("Push failed: {}", e));
                info("You can push manually later with: heimdal push");
            }
        }
    }

    Ok(())
}

fn cmd_config_get(key: &str) -> Result<()> {
    error("Not yet implemented - coming in Phase 4");
    Ok(())
}

fn cmd_config_set(key: &str, value: &str) -> Result<()> {
    error("Not yet implemented - coming in Phase 4");
    Ok(())
}

fn cmd_config_show() -> Result<()> {
    error("Not yet implemented - coming in Phase 4");
    Ok(())
}

fn cmd_history(limit: usize) -> Result<()> {
    header("Change History");

    // Load state
    let state = state::HeimdallState::load()?;

    info(&format!(
        "Showing last {} commits from {}",
        limit,
        state.dotfiles_path.display()
    ));
    println!();

    // Get git log
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&state.dotfiles_path)
        .arg("log")
        .arg("--oneline")
        .arg("--decorate")
        .arg("--color=always")
        .arg("-n")
        .arg(limit.to_string())
        .output()
        .with_context(|| "Failed to execute git log")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get git history");
    }

    let log = String::from_utf8_lossy(&output.stdout);
    print!("{}", log);

    println!();
    info("To rollback to a specific commit, run: heimdal rollback <commit-hash>");
    info("To rollback to the previous commit, run: heimdal rollback");

    Ok(())
}

fn cmd_import(path: Option<&str>, from: &str, output: Option<&str>) -> Result<()> {
    use console::style;
    use import::{detect_tool, import_from_tool, DotfileTool};
    use wizard::{
        ConfigGenerator, DetectedPackage, DotfileCategory, PackageCategory, PackageManager,
        ScannedDotfile,
    };

    header("Import Dotfiles");

    // Determine path
    let dotfiles_path = if let Some(p) = path {
        shellexpand::tilde(p).to_string()
    } else {
        shellexpand::tilde("~/dotfiles").to_string()
    };

    let path_buf = std::path::PathBuf::from(&dotfiles_path);

    if !path_buf.exists() {
        error(&format!("Directory not found: {}", dotfiles_path));
        return Ok(());
    }

    info(&format!("Importing from: {}", dotfiles_path));
    println!();

    // Determine tool
    let tool = if from == "auto" {
        detect_tool(&path_buf).unwrap_or_else(|| {
            info("No specific tool detected, using manual scanning");
            DotfileTool::Manual
        })
    } else {
        match from {
            "stow" => DotfileTool::Stow,
            "dotbot" => DotfileTool::Dotbot,
            _ => {
                error(&format!(
                    "Unknown tool: {}. Use: auto, stow, or dotbot",
                    from
                ));
                return Ok(());
            }
        }
    };

    println!(
        "{} Detected: {}",
        style("✓").green(),
        style(tool.name()).bold()
    );
    println!();

    // Import
    println!("{} Importing...", style("→").cyan());
    let import_result = import_from_tool(&path_buf, &tool)
        .with_context(|| format!("Failed to import from {}", tool.name()))?;

    println!(
        "{} Found {} files",
        style("✓").green(),
        import_result.dotfiles.len()
    );

    // Show sample files
    if !import_result.dotfiles.is_empty() {
        println!("\n{}:", style("Sample dotfiles").bold());
        for (i, mapping) in import_result.dotfiles.iter().take(5).enumerate() {
            let rel_source = mapping
                .source
                .strip_prefix(&path_buf)
                .unwrap_or(&mapping.source);
            println!("  {}. {}", i + 1, style(rel_source.display()).cyan());
        }
        if import_result.dotfiles.len() > 5 {
            println!(
                "  {} ... and {} more",
                style("").dim(),
                import_result.dotfiles.len() - 5
            );
        }
    }

    // Show packages
    if !import_result.packages.is_empty() {
        println!("\n{}:", style("Extracted packages").bold());
        for (i, pkg) in import_result.packages.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, style(pkg).cyan());
        }
        if import_result.packages.len() > 5 {
            println!(
                "  {} ... and {} more",
                style("").dim(),
                import_result.packages.len() - 5
            );
        }
    }

    // Generate configuration
    println!("\n{} Generating heimdal.yaml...", style("→").cyan());

    let mut generator = ConfigGenerator::new("personal");

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
                .strip_prefix(&path_buf)
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

    // Add packages
    if !import_result.packages.is_empty() {
        let detected_packages: Vec<DetectedPackage> = import_result
            .packages
            .into_iter()
            .map(|name| DetectedPackage {
                name,
                manager: PackageManager::Homebrew,
                category: PackageCategory::Other,
            })
            .collect();
        generator.add_packages(detected_packages);
    }

    // Save
    let output_path = if let Some(o) = output {
        std::path::PathBuf::from(shellexpand::tilde(o).to_string())
    } else {
        path_buf.join("heimdal.yaml")
    };

    generator
        .save(&output_path)
        .with_context(|| format!("Failed to save configuration to {}", output_path.display()))?;

    println!(
        "\n{} Saved to {}",
        style("✓").green().bold(),
        output_path.display()
    );

    println!("\n{}", style("Next steps:").bold());
    println!("  1. Review the generated heimdal.yaml");
    println!(
        "  2. Run: {} to preview changes",
        style("heimdal apply --dry-run").cyan()
    );
    println!(
        "  3. Run: {} to apply configuration",
        style("heimdal apply").cyan()
    );

    Ok(())
}

// ========== Profile Management ==========

fn cmd_profile_switch(profile_name: &str, auto_apply: bool) -> Result<()> {
    header("Switch Profile");
    let (should_apply, _) = profile::switch_profile(profile_name, auto_apply)?;

    if should_apply {
        println!("\n{} Applying new profile configuration...", "→".blue());
        cmd_apply(false, false)?;
    } else {
        println!(
            "\n{} Run {} to apply the new profile configuration",
            "ℹ".blue(),
            "heimdal apply".cyan()
        );
    }

    Ok(())
}

fn cmd_profile_current() -> Result<()> {
    let profile_name = profile::get_current_profile()?;
    println!("{}", profile_name.cyan());
    Ok(())
}

fn cmd_profile_show(profile_name: Option<&str>, show_resolved: bool) -> Result<()> {
    profile::show_profile_info(profile_name, show_resolved)?;
    Ok(())
}

fn cmd_profile_list(verbose: bool) -> Result<()> {
    profile::list_profiles(verbose)?;
    Ok(())
}

fn cmd_profile_diff(profile1: Option<&str>, profile2: &str) -> Result<()> {
    profile::diff_profiles(profile1, profile2)?;
    Ok(())
}

fn cmd_profile_templates() -> Result<()> {
    println!("{}", "Available Profile Templates:".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());

    for template_name in profile::ProfileTemplates::list() {
        let desc = profile::ProfileTemplates::description(template_name).unwrap_or("");
        println!(
            "  {} {}",
            template_name.cyan(),
            format!("- {}", desc).dimmed()
        );
    }

    println!(
        "\n{} Use {} to create a profile from a template",
        "ℹ".blue(),
        "heimdal profile create <name> --template <template>".cyan()
    );

    Ok(())
}

fn cmd_profile_create(profile_name: &str, template_name: &str) -> Result<()> {
    header("Create Profile from Template");

    let state = state::HeimdallState::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");

    // Load existing config
    let mut config = config::load_config(&state.dotfiles_path)?;

    // Check if profile already exists
    if config.profiles.contains_key(profile_name) {
        error(&format!("Profile '{}' already exists", profile_name));
        anyhow::bail!("Profile already exists");
    }

    // Create from template
    let new_profile = profile::create_from_template(profile_name, template_name)?;

    // Add to config
    config
        .profiles
        .insert(profile_name.to_string(), new_profile);

    // Save config
    let config_str = serde_yaml::to_string(&config)?;
    std::fs::write(&config_path, config_str)?;

    success(&format!(
        "Created profile '{}' from template '{}'",
        profile_name, template_name
    ));

    println!(
        "\n{} Edit {} to customize the profile",
        "ℹ".blue(),
        config_path.display().to_string().cyan()
    );
    println!(
        "{} Run {} to switch to this profile",
        "ℹ".blue(),
        format!("heimdal profile switch {}", profile_name).cyan()
    );

    Ok(())
}

fn cmd_profile_clone(source_name: &str, target_name: &str) -> Result<()> {
    header("Clone Profile");

    let state = state::HeimdallState::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");

    // Load existing config
    let mut config = config::load_config(&state.dotfiles_path)?;

    // Check if source profile exists
    let source_profile = config
        .profiles
        .get(source_name)
        .ok_or_else(|| anyhow::anyhow!("Source profile '{}' not found", source_name))?;

    // Check if target profile already exists
    if config.profiles.contains_key(target_name) {
        error(&format!("Profile '{}' already exists", target_name));
        anyhow::bail!("Profile already exists");
    }

    // Clone the profile
    let cloned_profile = profile::clone_profile(source_profile);

    // Add to config
    config
        .profiles
        .insert(target_name.to_string(), cloned_profile);

    // Save config
    let config_str = serde_yaml::to_string(&config)?;
    std::fs::write(&config_path, config_str)?;

    success(&format!(
        "Cloned profile '{}' to '{}'",
        source_name, target_name
    ));

    println!(
        "\n{} Edit {} to customize the cloned profile",
        "ℹ".blue(),
        config_path.display().to_string().cyan()
    );
    println!(
        "{} Run {} to switch to this profile",
        "ℹ".blue(),
        format!("heimdal profile switch {}", target_name).cyan()
    );

    Ok(())
}
