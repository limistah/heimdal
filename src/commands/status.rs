use crate::cli::StatusArgs;
use crate::git::{GitRepo, GitStatus};
use crate::state::State;
use crate::utils::{info, success, warning};
use anyhow::Result;

pub fn run(_args: StatusArgs) -> Result<()> {
    let state = State::load()?;

    // Machine + profile info
    info(&format!("Profile:       {}", state.active_profile));
    info(&format!("Dotfiles:      {}", state.dotfiles_path.display()));
    info(&format!("Hostname:      {}", state.hostname));
    info(&format!("OS:            {}", state.os));

    if let Some(t) = state.last_apply {
        info(&format!(
            "Last apply:    {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        info("Last apply:    never");
    }
    if let Some(t) = state.last_sync {
        info(&format!(
            "Last sync:     {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        info("Last sync:     never");
    }

    // Git status
    let repo = GitRepo::open(&state.dotfiles_path);
    match repo.status() {
        Ok(files) if files.is_empty() => success("Working tree clean"),
        Ok(files) => {
            warning(&format!("{} uncommitted change(s):", files.len()));
            for f in &files {
                let marker = match f.status {
                    GitStatus::Modified => "M",
                    GitStatus::Added => "A",
                    GitStatus::Deleted => "D",
                    GitStatus::Untracked => "?",
                };
                info(&format!("  {} {}", marker, f.path));
            }
        }
        Err(e) => warning(&format!("Could not read git status: {}", e)),
    }

    Ok(())
}
