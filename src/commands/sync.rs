use crate::cli::{ApplyArgs, SyncArgs};
use crate::config::CommandContext;
use crate::git::GitRepo;
use crate::hooks::run_hooks;
use crate::progress::ApplyProgress;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(args: SyncArgs) -> Result<()> {
    // Acquire lock to prevent concurrent operations
    let _lock = crate::lock::HeimdallLock::acquire()?;

    let ctx = CommandContext::load()?;

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    // pre_sync hooks
    // NOTE: sync uses a noop progress display — hook stdout/stderr is piped and
    // discarded on success; on failure the error propagates but no output is printed
    // above a progress bar. Consider adding a dedicated sync progress in a future task.
    {
        let p = ApplyProgress::noop();
        let stage = p.stage(1, "Pre-sync hooks");
        let mut vp = stage.hook_viewport();
        run_hooks(&ctx.profile.hooks.pre_sync, args.dry_run, &stage, &mut vp)?;
    }

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
    {
        let p = ApplyProgress::noop();
        let stage = p.stage(1, "Post-sync hooks");
        let mut vp = stage.hook_viewport();
        run_hooks(&ctx.profile.hooks.post_sync, args.dry_run, &stage, &mut vp)?;
    }

    if !args.dry_run {
        let mut s = ctx.state;
        s.last_sync = Some(chrono::Utc::now());
        s.save()?;
    }

    success("Sync complete");
    Ok(())
}
