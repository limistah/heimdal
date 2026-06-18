use anyhow::Result;
use colored::Colorize;
use std::time::Instant;

use crate::cli::ApplyArgs;
use crate::config::CommandContext;
use crate::hooks::run_hooks;
use crate::packages::install_for_profile;
use crate::progress::ApplyProgress;
use crate::symlink::{
    apply_mappings, apply_stow_walk, compile_ignore_patterns, ApplyContext, LinkResult,
};
use crate::utils::{get_verbosity, home_dir, Verbosity};

pub fn run(args: ApplyArgs) -> Result<()> {
    let _lock = crate::lock::HeimdallLock::acquire()?;
    let ctx = CommandContext::load()?;

    // Build progress: 5 stages. Use noop in quiet mode.
    let progress = if get_verbosity() == Verbosity::Quiet {
        ApplyProgress::noop()
    } else {
        ApplyProgress::new(5)
    };

    if args.dry_run {
        crate::utils::info("Dry-run mode — no changes will be made");
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

    // ── [1/5] Pre-apply hooks ────────────────────────────────────────────────
    let t = Instant::now();
    let stage1 = progress.stage(1, "Pre-apply hooks");
    if !args.packages_only {
        progress.suspend(|| run_hooks(&ctx.profile.hooks.pre_apply, args.dry_run))?;
    }
    stage1.finish_success(t.elapsed());

    // ── [2/5] Installing packages ────────────────────────────────────────────
    let t = Instant::now();
    let stage2 = progress.stage(2, "Installing packages");
    if !args.dotfiles_only {
        let failures = install_for_profile(
            &ctx.profile,
            args.dry_run,
            &stage2,
            ctx.config.parallel_jobs,
        )?;
        if failures.is_empty() {
            stage2.finish_success(t.elapsed());
        } else {
            stage2.finish_warn(t.elapsed(), &failures);
        }
    } else {
        stage2.finish_success(t.elapsed());
    }

    // ── [3/5] Symlinks ───────────────────────────────────────────────────────
    let t = Instant::now();
    let stage3 = progress.stage(3, "Symlinks");
    if !args.packages_only {
        let mut linked: u64 = 0;
        let mut warnings: usize = 0;

        let mut on_result = |result: &LinkResult| {
            match result {
                LinkResult::Created { .. }
                | LinkResult::AlreadyLinked { .. }
                | LinkResult::Backed { .. } => {
                    linked += 1;
                }
                _ => {}
            }
            match result {
                LinkResult::Conflict { dest, reason } => {
                    stage3.println(&format!(
                        "         {} conflict  {} — {}",
                        "!".yellow(),
                        dest.display(),
                        reason
                    ));
                    warnings += 1;
                }
                LinkResult::Backed { dest, backup } => {
                    stage3.println(&format!(
                        "         {} backed    {} → {}",
                        "!".yellow(),
                        dest.display(),
                        backup.display()
                    ));
                }
                LinkResult::Skipped { dest, reason } => {
                    stage3.println(&format!(
                        "         · skipped   {} — {}",
                        dest.display(),
                        reason
                    ));
                }
                _ => {}
            }
            stage3.set_message(format!("{} linked", linked));
        };

        let results = if ctx.profile.dotfiles.is_empty() {
            apply_stow_walk(&apply_ctx, &mut on_result)?
        } else {
            apply_mappings(
                &apply_ctx,
                &ctx.profile.dotfiles,
                &ctx.state.active_profile,
                &mut on_result,
            )?
        };
        drop(on_result); // release borrows on linked/warnings before reading them

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

        stage3.finish_with_counts(t.elapsed(), linked, warnings);
    } else {
        stage3.finish_success(t.elapsed());
    }

    // ── [4/5] Templates ──────────────────────────────────────────────────────
    let t = Instant::now();
    let stage4 = progress.stage(4, "Templates");
    if !args.packages_only {
        for tmpl in &ctx.profile.templates {
            let src = ctx.state.dotfiles_path.join(&tmpl.src);
            let dest = crate::utils::expand_path(&tmpl.dest);
            let vars = crate::templates::build_vars(&tmpl.vars, "env");
            if let Err(e) = crate::templates::render_file(&src, &dest, &vars, args.dry_run) {
                crate::utils::warning(&format!("Template '{}' failed: {}", tmpl.src, e));
            }
        }
        #[cfg(target_os = "macos")]
        if let Some(ref defaults_config) = ctx.config.defaults {
            if defaults_config.enabled {
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
    stage4.finish_success(t.elapsed());

    // ── [5/5] Post-apply hooks ───────────────────────────────────────────────
    let t = Instant::now();
    let stage5 = progress.stage(5, "Post-apply hooks");
    if !args.packages_only {
        progress.suspend(|| run_hooks(&ctx.profile.hooks.post_apply, args.dry_run))?;
    }
    stage5.finish_success(t.elapsed());

    // Persist state timestamp
    if !args.dry_run {
        let mut s = ctx.state;
        s.last_apply = Some(chrono::Utc::now());
        s.save()?;
    }

    Ok(())
}
