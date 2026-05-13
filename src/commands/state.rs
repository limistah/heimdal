use crate::cli::StateCmd;
use crate::config::CommandContext;
use crate::state::State;
use crate::utils::info;
use anyhow::Result;

pub fn run(action: StateCmd) -> Result<()> {
    match action {
        StateCmd::LockInfo => lock_info(),
        StateCmd::Unlock { force } => unlock(force),
        StateCmd::CheckDrift => check_drift(),
        StateCmd::CheckConflicts => check_conflicts(),
        StateCmd::History { limit } => history(limit),
    }
}

fn lock_info() -> Result<()> {
    match crate::lock::HeimdallLock::info()? {
        Some(info) => {
            let running = crate::lock::HeimdallLock::is_process_running(info.pid);
            crate::utils::info(&format!(
                "Lock held by PID {} on {}",
                info.pid, info.hostname
            ));
            crate::utils::info(&format!(
                "Started: {}",
                info.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            crate::utils::info(&format!(
                "Status: {}",
                if running { "active" } else { "stale" }
            ));
        }
        None => {
            crate::utils::info("No active lock.");
        }
    }
    Ok(())
}

fn unlock(force: bool) -> Result<()> {
    if !force {
        crate::utils::info("Use --force to remove a lock file.");
        return Ok(());
    }
    crate::lock::HeimdallLock::force_unlock()?;
    crate::utils::success("Lock removed.");
    Ok(())
}

fn check_drift() -> Result<()> {
    let ctx = CommandContext::load()?;

    let mut drift_count = 0;
    for entry in &ctx.profile.dotfiles {
        let src_rel = entry.source();
        let target_str = entry.target();
        let src = ctx.state.dotfiles_path.join(src_rel);
        let dest = crate::utils::expand_path(&target_str);

        if !dest.is_symlink() {
            crate::utils::warning(&format!(
                "Not linked: {} (run 'heimdal apply')",
                dest.display()
            ));
            drift_count += 1;
        } else if let Ok(link_target) = std::fs::read_link(&dest) {
            if link_target != src {
                crate::utils::warning(&format!(
                    "Drift: {} → {} (expected → {})",
                    dest.display(),
                    link_target.display(),
                    src.display()
                ));
                drift_count += 1;
            }
        }
    }

    if drift_count == 0 {
        crate::utils::success("No drift detected — all symlinks are correct.");
    } else {
        anyhow::bail!(
            "{} drifted symlink(s) found. Run 'heimdal apply' to fix.",
            drift_count
        );
    }
    Ok(())
}

fn check_conflicts() -> Result<()> {
    let ctx = CommandContext::load()?;

    let mut conflict_count = 0;
    for entry in &ctx.profile.dotfiles {
        let target_str = entry.target();
        let dest = crate::utils::expand_path(&target_str);
        if dest.exists() && !dest.is_symlink() {
            crate::utils::warning(&format!(
                "Conflict: '{}' exists and is not a symlink. Use 'heimdal apply --force' or '--backup'.",
                dest.display()
            ));
            conflict_count += 1;
        }
    }

    if conflict_count == 0 {
        crate::utils::success("No conflicts detected.");
    } else {
        anyhow::bail!("{} conflict(s) found.", conflict_count);
    }
    Ok(())
}

fn history(limit: usize) -> Result<()> {
    let state = State::load()?;
    info(&format!("Machine:       {}", state.hostname));
    info(&format!("Profile:       {}", state.active_profile));
    if let Some(t) = state.last_apply {
        info(&format!(
            "Last apply:    {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    }
    if let Some(t) = state.last_sync {
        info(&format!(
            "Last sync:     {}",
            t.format("%Y-%m-%d %H:%M UTC")
        ));
    }
    info(&format!(
        "(Full history not available — showing last {} events from state)",
        limit
    ));
    Ok(())
}
