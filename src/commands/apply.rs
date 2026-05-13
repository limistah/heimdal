use anyhow::Result;

use crate::cli::ApplyArgs;
use crate::config::CommandContext;
use crate::hooks::run_hooks;
use crate::packages::install_for_profile;
use crate::symlink::{apply_mappings, apply_stow_walk, print_results, ApplyContext, LinkResult};
use crate::utils::{home_dir, info, success};

pub fn run(args: ApplyArgs) -> Result<()> {
    // Acquire lock to prevent concurrent operations
    let _lock = crate::lock::HeimdallLock::acquire()?;

    let ctx = CommandContext::load()?;

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    let apply_ctx = ApplyContext {
        dotfiles_dir: ctx.state.dotfiles_path.clone(),
        home_dir: home_dir()?,
        dry_run: args.dry_run,
        force: args.force,
        backup: args.backup,
    };

    if !args.packages_only {
        run_hooks(&ctx.profile.hooks.pre_apply, args.dry_run)?;
    }

    if !args.dotfiles_only {
        install_for_profile(&ctx.profile, args.dry_run)?;
    }

    if !args.packages_only {
        let results = if ctx.profile.dotfiles.is_empty() {
            apply_stow_walk(&apply_ctx)?
        } else {
            apply_mappings(&apply_ctx, &ctx.profile.dotfiles, &ctx.state.active_profile)?
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
        for tmpl in &ctx.profile.templates {
            let src = ctx.state.dotfiles_path.join(&tmpl.src);
            let dest = crate::utils::expand_path(&tmpl.dest);
            let vars = crate::templates::build_vars(&tmpl.vars, "env");
            if let Err(e) = crate::templates::render_file(&src, &dest, &vars, args.dry_run) {
                crate::utils::warning(&format!("Template '{}' failed: {}", tmpl.src, e));
            }
        }
    }

    if !args.packages_only {
        run_hooks(&ctx.profile.hooks.post_apply, args.dry_run)?;
    }

    if !args.dry_run {
        let mut s = ctx.state;
        s.last_apply = Some(chrono::Utc::now());
        s.save()?;
    }

    success("Apply complete");
    Ok(())
}
