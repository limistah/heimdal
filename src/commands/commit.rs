use crate::cli::CommitArgs;
use crate::git::GitRepo;
use crate::state::State;
use crate::utils::success;
use anyhow::Result;

pub fn run(args: CommitArgs) -> Result<()> {
    let state = State::load()?;
    let repo = GitRepo::open(&state.dotfiles_path);

    let message = args.message.as_deref().unwrap_or("Update dotfiles");
    let files = if args.files.is_empty() {
        None
    } else {
        Some(args.files.as_slice())
    };

    repo.commit(message, files, false)?;

    if args.push {
        repo.push(false)?;
        success("Committed and pushed");
    } else {
        success("Committed");
    }
    Ok(())
}
