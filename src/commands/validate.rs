use crate::cli::ValidateArgs;
use crate::config::{load_config, validate_config};
use crate::utils::{success, warning};
use anyhow::Result;

pub fn run(args: ValidateArgs) -> Result<()> {
    // Determine config path
    let config_path = match &args.config {
        Some(p) => crate::utils::expand_path(p),
        None => {
            // Try loading state to get dotfiles path, fall back to ~/.dotfiles
            let dotfiles = crate::state::State::load()
                .map(|s| s.dotfiles_path)
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(".dotfiles")
                });
            dotfiles.join("heimdal.yaml")
        }
    };

    let config = load_config(&config_path)?;
    let errors = validate_config(&config);

    if errors.is_empty() {
        let profile_count = config.profiles.len();
        success(&format!(
            "Config is valid ({} profile{})",
            profile_count,
            if profile_count == 1 { "" } else { "s" }
        ));
        // List profile names
        let mut names: Vec<_> = config.profiles.keys().collect();
        names.sort();
        for name in names {
            crate::utils::info(&format!("  \u{2022} {}", name));
        }
    } else {
        for err in &errors {
            warning(err);
        }
        anyhow::bail!("{} validation error(s) found", errors.len());
    }

    Ok(())
}
