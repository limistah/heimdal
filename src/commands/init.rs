use crate::cli::InitArgs;
use crate::config::{create_minimal_config, load_config};
use crate::error::HeimdallError;
use crate::git::GitRepo;
use crate::state::State;
use crate::utils::{expand_path, home_dir, info, step, success};
use anyhow::Result;

pub fn run(args: InitArgs) -> Result<()> {
    // 1. Determine dotfiles path
    let dotfiles_path = match &args.path {
        Some(p) => expand_path(p),
        None => home_dir()?.join(".dotfiles"),
    };

    // 2. Clone or reuse existing directory
    if args.no_clone {
        if !dotfiles_path.exists() {
            anyhow::bail!(
                "Directory '{}' does not exist. Either create it first or omit --no-clone to clone from the repository.",
                dotfiles_path.display()
            );
        }
        info(&format!(
            "Using existing directory: {}",
            dotfiles_path.display()
        ));
    } else if dotfiles_path.exists() {
        info(&format!(
            "Directory '{}' already exists, skipping clone.",
            dotfiles_path.display()
        ));
    } else {
        step(&format!("Cloning {} ...", args.repo));
        GitRepo::clone(&args.repo, &dotfiles_path)?;
        step("Clone complete");
    }

    // 3. Check for heimdal.yaml; create minimal one only when not using --no-clone
    let config_path = dotfiles_path.join("heimdal.yaml");
    if !config_path.exists() {
        if args.no_clone {
            anyhow::bail!(
                "No heimdal.yaml found in '{}'. \
                 When using --no-clone the directory must already contain a valid heimdal.yaml.",
                dotfiles_path.display()
            );
        }
        step("No heimdal.yaml found, creating minimal config...");
        create_minimal_config(&config_path, &args.profile)?;
    }

    // 4. Load and validate config
    let config = load_config(&config_path)?;

    // 5. Verify the requested profile exists
    if !config.profiles.contains_key(&args.profile) {
        let mut available: Vec<_> = config.profiles.keys().cloned().collect();
        available.sort();
        eprintln!(
            "Available profiles: {}",
            if available.is_empty() {
                "(none)".to_string()
            } else {
                available.join(", ")
            }
        );
        return Err(HeimdallError::ProfileNotFound {
            name: args.profile,
        }
        .into());
    }

    // 6. Write state file
    State::create(args.profile.clone(), dotfiles_path.clone(), args.repo.clone())?;

    // 7. Print success + next steps
    success(&format!(
        "Initialized heimdal with profile '{}' in {}",
        args.profile,
        dotfiles_path.display()
    ));
    info("Next steps:");
    info("  heimdal apply         — create symlinks and install packages");
    info("  heimdal validate      — check your heimdal.yaml");
    info("  heimdal status        — show current state");

    Ok(())
}
