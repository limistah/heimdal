use crate::cli::{ApplyArgs, RollbackArgs};
use crate::git::GitRepo;
use crate::state::State;
use crate::utils::{info, success, warning};
use anyhow::Result;

pub fn run(args: RollbackArgs) -> Result<()> {
    let state = State::load()?;
    let repo = GitRepo::open(&state.dotfiles_path);

    let target = args.target.as_deref();
    let rev = target.unwrap_or("HEAD~1");

    if args.dry_run {
        info(&format!("[dry-run] Would roll back to {}", rev));
        return Ok(());
    }

    if !args.force
        && !crate::utils::confirm(&format!(
            "Roll back to {}? This discards uncommitted changes in {} (git reset --hard).",
            rev,
            state.dotfiles_path.display()
        ))
    {
        info("Cancelled.");
        return Ok(());
    }

    repo.rollback(target, false)?;
    info(&format!("Rolled back to {}", rev));

    // Re-apply after rollback. The git reset above already happened and is
    // not undone here — a failure just means the dotfiles repo and the
    // symlinked-in config are out of sync until the user resolves it.
    if let Err(e) = crate::commands::apply::run(ApplyArgs::default()) {
        warning(&format!("Rolled back to {} but re-apply failed: {e}", rev));
        info("Fix the conflict, then run `heimdal apply --force` to finish.");
        return Err(e);
    }

    success("Rollback complete");
    Ok(())
}
