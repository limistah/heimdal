use crate::cli::{ApplyArgs, SyncArgs};
use crate::config::{load_config, resolve_profile};
use crate::git::GitRepo;
use crate::hooks::run_hooks;
use crate::state::State;
use crate::utils::{info, success};
use anyhow::Result;

pub fn run(args: SyncArgs) -> Result<()> {
    let mut state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = load_config(&config_path)?;
    let profile = resolve_profile(&config, &state.active_profile)?;

    if args.dry_run {
        info("Dry-run mode — no changes will be made");
    }

    // pre_sync hooks
    run_hooks(&profile.hooks.pre_sync, args.dry_run)?;

    // pull
    let repo = GitRepo::open(&state.dotfiles_path);
    info("Pulling from remote...");
    repo.pull(args.dry_run)?;

    // Sync history if enabled in config (default: true for both flags)
    let history_enabled = config.history.as_ref().map(|h| h.enabled).unwrap_or(true);
    let history_sync = config.history.as_ref().map(|h| h.sync).unwrap_or(true);
    if history_enabled && history_sync {
        crate::commands::history::sync::run_sync(args.dry_run)?;
    }

    // apply
    crate::commands::apply::run(ApplyArgs {
        dry_run: args.dry_run,
        ..Default::default()
    })?;

    // post_sync hooks
    run_hooks(&profile.hooks.post_sync, args.dry_run)?;

    if !args.dry_run {
        state.last_sync = Some(chrono::Utc::now());
        state.save()?;
    }

    success("Sync complete");
    Ok(())
}
