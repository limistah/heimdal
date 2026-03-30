use crate::utils::{confirm, info, prompt_string, success, warning};
use anyhow::Result;

pub fn run() -> Result<()> {
    println!("\nWelcome to Heimdal — Universal Dotfile Manager\n");

    let choices = vec![
        "Fresh setup (new dotfiles)",
        "Import from another tool",
        "Clone existing dotfiles",
    ];
    let selection = dialoguer::Select::new()
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()
        .unwrap_or(0);

    match selection {
        0 => fresh_setup(),
        1 => import_setup(),
        2 => clone_setup(),
        _ => fresh_setup(),
    }
}

fn fresh_setup() -> Result<()> {
    info("Fresh setup — I'll help you create your first heimdal.yaml");

    let profile = prompt_string("Profile name", "default");
    let repo = prompt_string("Git repository URL (or press Enter to skip)", "");

    // Detect OS and suggest packages
    let os = crate::utils::os_name();
    let suggested = match os {
        "macos" => vec!["git", "vim", "ripgrep", "fd"],
        "linux" => vec!["git", "vim", "ripgrep", "fd-find"],
        _ => vec!["git", "vim"],
    };

    info(&format!("Detected OS: {}", os));
    info(&format!("Suggested packages: {}", suggested.join(", ")));

    let dotfiles_path = crate::utils::home_dir()?.join(".dotfiles");

    if !dotfiles_path.exists() {
        std::fs::create_dir_all(&dotfiles_path)?;
        info(&format!("Created directory: {}", dotfiles_path.display()));
    }

    let config_path = dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        crate::config::create_minimal_config(&config_path, &profile)?;
        info("Created minimal heimdal.yaml — edit it to add your dotfiles");
    }

    // Write state
    let repo_url = if repo.is_empty() {
        "file://local".to_string()
    } else {
        repo
    };
    crate::state::State::create(profile.clone(), dotfiles_path.clone(), repo_url)?;

    success(&format!("Initialized with profile '{}'", profile));
    info("Next steps:");
    info("  1. Edit ~/.dotfiles/heimdal.yaml to list your dotfiles");
    info("  2. Run 'heimdal apply' to create symlinks");
    info("  3. Run 'heimdal validate' to check your config");
    Ok(())
}

fn import_setup() -> Result<()> {
    info("Import — I'll detect your existing dotfile manager");

    let path_str = prompt_string("Path to your existing dotfiles", "~/.dotfiles");
    let path = crate::utils::expand_path(&path_str);

    if !path.exists() {
        warning(&format!("'{}' does not exist.", path.display()));
        return Ok(());
    }

    let tool = crate::import::detect_tool(&path);
    let tool_name = tool.as_ref().map(|t| t.as_str()).unwrap_or("stow");
    info(&format!("Detected: {} format", tool_name));

    let result = crate::import::import_from(&path, tool)?;
    info(&format!("Found {} dotfile mappings", result.dotfiles.len()));

    let profile = prompt_string("Profile name for imported config", "default");

    let yaml = crate::import::generate_heimdal_yaml(&result, &profile)?;
    let output_path = path.join("heimdal.yaml");

    if output_path.exists()
        && !confirm(&format!(
            "'{}' already exists. Overwrite?",
            output_path.display()
        ))
    {
        info("Cancelled.");
        return Ok(());
    }

    std::fs::write(&output_path, &yaml)?;
    success(&format!("Written: {}", output_path.display()));
    info("Next: run 'heimdal init --repo <url> --profile <name> --no-clone'");
    Ok(())
}

fn clone_setup() -> Result<()> {
    info("Clone — I'll set up your dotfiles from an existing repository");

    let repo = prompt_string("Git repository URL", "");
    if repo.is_empty() {
        warning("No repository URL provided.");
        return Ok(());
    }

    let profile = prompt_string("Profile name", "default");
    let path_str = prompt_string("Local path", "~/.dotfiles");
    let path = crate::utils::expand_path(&path_str);

    if path.exists() {
        info(&format!(
            "'{}' already exists, skipping clone.",
            path.display()
        ));
    } else {
        info("Cloning repository...");
        crate::git::GitRepo::clone(&repo, &path)?;
    }

    let config_path = path.join("heimdal.yaml");
    if !config_path.exists() {
        crate::config::create_minimal_config(&config_path, &profile)?;
    }

    let config = crate::config::load_config(&config_path)?;
    if !config.profiles.contains_key(&profile) {
        warning(&format!(
            "Profile '{}' not found in config. Available: {}",
            profile,
            config
                .profiles
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    crate::state::State::create(profile.clone(), path.clone(), repo)?;
    success(&format!("Initialized with profile '{}'", profile));
    info("Run 'heimdal apply' to create symlinks and install packages.");
    Ok(())
}
