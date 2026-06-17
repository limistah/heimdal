use anyhow::Result;

use crate::cli::ApplyArgs;
use crate::config::CommandContext;
use crate::hooks::run_hooks;
use crate::packages::install_for_profile;
use crate::symlink::{
    apply_mappings, apply_stow_walk, compile_ignore_patterns, print_results, ApplyContext,
    LinkResult,
};
use crate::utils::{home_dir, info, success, verbose};

pub fn run(args: ApplyArgs) -> Result<()> {
    // Acquire lock to prevent concurrent operations
    let _lock = crate::lock::HeimdallLock::acquire()?;

    let ctx = CommandContext::load()?;

    verbose(&format!("Profile: {}", ctx.state.active_profile));
    verbose(&format!(
        "Dotfiles dir: {}",
        ctx.state.dotfiles_path.display()
    ));

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    let ignore_patterns = compile_ignore_patterns(&ctx.profile.ignore);

    let apply_ctx = ApplyContext {
        dotfiles_dir: ctx.state.dotfiles_path.clone(),
        home_dir: home_dir()?,
        dry_run: args.dry_run,
        force: args.force,
        backup: args.backup,
        ignore_patterns,
    };

    if !args.packages_only {
        verbose("Running pre-apply hooks");
        run_hooks(&ctx.profile.hooks.pre_apply, args.dry_run)?;
    }

    if !args.dotfiles_only {
        verbose("Installing packages");
        install_for_profile(
            &ctx.profile,
            args.dry_run,
            &crate::progress::ApplyProgress::noop().stage(1, ""),
            ctx.config.parallel_jobs,
        )?;
    }

    if !args.packages_only {
        verbose("Creating symlinks");
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
            verbose(&format!(
                "Rendering template: {} → {}",
                src.display(),
                dest.display()
            ));
            let vars = crate::templates::build_vars(&tmpl.vars, "env");
            if let Err(e) = crate::templates::render_file(&src, &dest, &vars, args.dry_run) {
                crate::utils::warning(&format!("Template '{}' failed: {}", tmpl.src, e));
            }
        }
    }

    // Export macOS defaults (if configured)
    #[cfg(target_os = "macos")]
    if !args.packages_only {
        if let Some(ref defaults_config) = ctx.config.defaults {
            if defaults_config.enabled {
                verbose("Exporting macOS defaults");
                if let Err(e) = crate::defaults::export_all(
                    &ctx.state.dotfiles_path,
                    defaults_config,
                    args.dry_run,
                ) {
                    crate::utils::warning(&format!("Defaults export failed: {}", e));
                }
            }
        }
    }

    if !args.packages_only {
        verbose("Running post-apply hooks");
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
