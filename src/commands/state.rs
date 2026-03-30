use crate::cli::StateCmd;
use crate::state::State;
use crate::utils::info;
use anyhow::Result;

pub fn run(action: StateCmd) -> Result<()> {
    match action {
        StateCmd::LockInfo => lock_info(),
        StateCmd::Unlock { force: _ } => unlock(),
        StateCmd::CheckDrift => check_drift(),
        StateCmd::CheckConflicts => check_conflicts(),
        StateCmd::History { limit } => history(limit),
    }
}

fn lock_info() -> Result<()> {
    info("No lock mechanism — Heimdal v3 uses atomic writes instead of file locks.");
    Ok(())
}

fn unlock() -> Result<()> {
    info("No lock to remove — Heimdal v3 uses atomic writes instead of file locks.");
    Ok(())
}

fn check_drift() -> Result<()> {
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = crate::config::load_config(&config_path)?;
    let profile = crate::config::resolve_profile(&config, &state.active_profile)?;

    let mut drift_count = 0;
    for entry in &profile.dotfiles {
        let (src_rel, target_str) = match entry {
            crate::config::DotfileEntry::Simple(s) => (s.as_str(), format!("~/{}", s)),
            crate::config::DotfileEntry::Mapped(m) => (m.source.as_str(), m.target.clone()),
        };
        let src = state.dotfiles_path.join(src_rel);
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
    let state = State::load()?;
    let config_path = state.dotfiles_path.join("heimdal.yaml");
    let config = crate::config::load_config(&config_path)?;
    let profile = crate::config::resolve_profile(&config, &state.active_profile)?;

    let mut conflict_count = 0;
    for entry in &profile.dotfiles {
        let target_str = match entry {
            crate::config::DotfileEntry::Simple(s) => format!("~/{}", s),
            crate::config::DotfileEntry::Mapped(m) => m.target.clone(),
        };
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
