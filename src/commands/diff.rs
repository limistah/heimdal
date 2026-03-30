use crate::cli::DiffArgs;
use crate::git::GitRepo;
use crate::state::State;
use crate::utils::info;
use anyhow::Result;

pub fn run(args: DiffArgs) -> Result<()> {
    let state = State::load()?;
    let repo = GitRepo::open(&state.dotfiles_path);
    let diff = repo.diff(args.verbose)?;
    if diff.trim().is_empty() {
        info("No changes.");
    } else {
        print!("{}", diff);
    }
    Ok(())
}
