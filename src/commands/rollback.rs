use crate::cli::{ApplyArgs, RollbackArgs};
use crate::git::GitRepo;
use crate::state::State;
use crate::utils::{info, success};
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

    repo.rollback(target, false)?;
    info(&format!("Rolled back to {}", rev));

    // Re-apply after rollback
    crate::commands::apply::run(ApplyArgs::default())?;

    success("Rollback complete");
    Ok(())
}
