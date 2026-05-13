use crate::cli::{ApplyArgs, SyncArgs};
use crate::config::CommandContext;
use crate::git::GitRepo;
use crate::hooks::run_hooks;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(args: SyncArgs) -> Result<()> {
    let ctx = CommandContext::load()?;

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    // pre_sync hooks
    run_hooks(&ctx.profile.hooks.pre_sync, args.dry_run)?;

    // pull
    let repo = GitRepo::open(&ctx.state.dotfiles_path);
    info("Pulling from remote...");
    repo.pull(args.dry_run)?;

    // Sync history if enabled in config (default: true for both flags)
    let history_enabled = ctx
        .config
        .history
        .as_ref()
        .map(|h| h.enabled)
        .unwrap_or(true);
    let history_sync = ctx.config.history.as_ref().map(|h| h.sync).unwrap_or(true);
    if history_enabled && history_sync {
        crate::commands::history::sync::run_sync(args.dry_run)?;
    }

    // apply
    crate::commands::apply::run(ApplyArgs {
        dry_run: args.dry_run,
        ..Default::default()
    })?;

    // post_sync hooks
    run_hooks(&ctx.profile.hooks.post_sync, args.dry_run)?;

    if !args.dry_run {
        let mut s = ctx.state;
        s.last_sync = Some(chrono::Utc::now());
        s.save()?;
    }

    success("Sync complete");
    Ok(())
}
