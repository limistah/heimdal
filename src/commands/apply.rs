use anyhow::Result;

use crate::cli::ApplyArgs;
use crate::config::{load_config, resolve_profile};
use crate::hooks::run_hooks;
use crate::packages::install_for_profile;
use crate::state::State;
use crate::symlink::{apply_mappings, apply_stow_walk, print_results, ApplyContext, LinkResult};
use crate::utils::{home_dir, info, success};

pub fn run(args: ApplyArgs) -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;
    let profile = resolve_profile(&config, &state.active_profile)?;

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    let ctx = ApplyContext {
        dotfiles_dir: state.dotfiles_path.clone(),
        home_dir: home_dir()?,
        dry_run: args.dry_run,
        force: args.force,
        backup: args.backup,
    };

    if !args.packages_only {
        run_hooks(&profile.hooks.pre_apply, args.dry_run)?;
    }

    if !args.dotfiles_only {
        install_for_profile(&profile, args.dry_run)?;
    }

    if !args.packages_only {
        let results = if profile.dotfiles.is_empty() {
            apply_stow_walk(&ctx)?
        } else {
            apply_mappings(&ctx, &profile.dotfiles, &state.active_profile)?
        };

        print_results(&results, args.dry_run);

        let conflicts: Vec<_> = results
            .iter()
            .filter(|r| matches!(r, LinkResult::Conflict { .. }))
            .collect();
        if !conflicts.is_empty() {
            anyhow::bail!(
                "{} conflict(s) found. Use --force to overwrite or --backup to save originals.",
                conflicts.len()
            );
        }
    }

    // Render templates
    if !args.packages_only {
        for tmpl in &profile.templates {
            let src = state.dotfiles_path.join(&tmpl.src);
            let dest = crate::utils::expand_path(&tmpl.dest);
            let vars = crate::templates::build_vars(&tmpl.vars, "env");
            if let Err(e) = crate::templates::render_file(&src, &dest, &vars, args.dry_run) {
                crate::utils::warning(&format!("Template '{}' failed: {}", tmpl.src, e));
            }
        }
    }

    if !args.packages_only {
        run_hooks(&profile.hooks.post_apply, args.dry_run)?;
    }

    if !args.dry_run {
        let mut s = state;
        s.last_apply = Some(chrono::Utc::now());
        s.save()?;
    }

    success("Apply complete");
    Ok(())
}
