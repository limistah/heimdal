use crate::cli::AutoSyncCmd;
use crate::utils::{info, success};
use anyhow::{bail, Result};
use std::path::PathBuf;

pub fn run(action: AutoSyncCmd) -> Result<()> {
    match action {
        AutoSyncCmd::Enable { interval } => enable(&interval),
        AutoSyncCmd::Disable => disable(),
        AutoSyncCmd::Status => status(),
    }
}

fn enable(interval: &str) -> Result<()> {
    let interval_secs = parse_interval(interval)?;

    #[cfg(target_os = "macos")]
    {
        enable_launchd(interval_secs)?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            enable_systemd(interval_secs)?;
        } else {
            enable_cron(interval_secs)?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        bail!("AutoSync is not supported on this platform. Supported platforms: macOS, Linux");
    }

    Ok(())
}

fn disable() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        disable_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            disable_systemd()?;
        } else {
            disable_cron()?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        bail!("AutoSync is not supported on this platform. Supported platforms: macOS, Linux");
    }

    Ok(())
}

fn status() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        status_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        if has_systemd() {
            status_systemd()?;
        } else {
            status_cron()?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        bail!("AutoSync is not supported on this platform. Supported platforms: macOS, Linux");
    }

    Ok(())
}

fn parse_interval(s: &str) -> Result<u64> {
    let s = s.trim().to_lowercase();
    if let Some(n) = s.strip_suffix('h') {
        Ok(n.parse::<u64>()? * 3600)
    } else if let Some(n) = s.strip_suffix('m') {
        Ok(n.parse::<u64>()? * 60)
    } else if let Some(n) = s.strip_suffix('s') {
        Ok(n.parse::<u64>()?)
    } else {
        // Assume minutes if no suffix
        Ok(s.parse::<u64>()? * 60)
    }
}

// ============================================================================
// macOS launchd implementation
// ============================================================================

#[cfg(target_os = "macos")]
const LAUNCHD_LABEL: &str = "com.heimdal.autosync";

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> PathBuf {
    dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", LAUNCHD_LABEL))
}

#[cfg(target_os = "macos")]
fn enable_launchd(interval_secs: u64) -> Result<()> {
    let plist_path = launchd_plist_path();
    let heimdal_path = std::env::current_exe()?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>sync</string>
    </array>
    <key>StartInterval</key>
    <integer>{}</integer>
    <key>RunAtLoad</key>
    <false/>
    <key>StandardOutPath</key>
    <string>/tmp/heimdal-autosync.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/heimdal-autosync.log</string>
</dict>
</plist>"#,
        LAUNCHD_LABEL,
        heimdal_path.display(),
        interval_secs
    );

    crate::utils::ensure_parent_exists(&plist_path)?;
    std::fs::write(&plist_path, plist)?;

    // Load the agent
    std::process::Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .status()?;

    success(&format!(
        "AutoSync enabled (every {} seconds)",
        interval_secs
    ));
    info(&format!("Plist: {}", plist_path.display()));
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_launchd() -> Result<()> {
    let plist_path = launchd_plist_path();

    if plist_path.exists() {
        // Try to unload, but don't fail if it's not loaded
        let _ = std::process::Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap()])
            .output();
        std::fs::remove_file(&plist_path)?;
        success("AutoSync disabled.");
    } else {
        info("AutoSync was not enabled.");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn status_launchd() -> Result<()> {
    let output = std::process::Command::new("launchctl")
        .args(["list", LAUNCHD_LABEL])
        .output()?;

    if output.status.success() {
        success("AutoSync is enabled (launchd).");
        let plist_path = launchd_plist_path();
        info(&format!("Plist: {}", plist_path.display()));
    } else {
        info("AutoSync is not enabled.");
    }
    Ok(())
}

// ============================================================================
// Linux systemd implementation (TODO - will implement in Task 17)
// ============================================================================

#[cfg(target_os = "linux")]
fn has_systemd() -> bool {
    std::path::Path::new("/run/systemd/system").exists()
}

#[cfg(target_os = "linux")]
fn enable_systemd(_interval_secs: u64) -> Result<()> {
    info("Linux systemd AutoSync implementation coming soon");
    info("For now, you can manually create a systemd user timer in ~/.config/systemd/user/");
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_systemd() -> Result<()> {
    info("Linux systemd AutoSync implementation coming soon");
    Ok(())
}

#[cfg(target_os = "linux")]
fn status_systemd() -> Result<()> {
    info("Linux systemd AutoSync implementation coming soon");
    info("Check with: systemctl --user list-timers");
    Ok(())
}

// ============================================================================
// Cron fallback implementation (TODO - will implement in Task 18)
// ============================================================================

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn enable_cron(_interval_secs: u64) -> Result<()> {
    info("Cron-based AutoSync implementation coming soon");
    info("For now, you can manually add a cron job:");
    info("  crontab -e");
    info("  */60 * * * * heimdal sync  # adjust interval as needed");
    Ok(())
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn disable_cron() -> Result<()> {
    info("Cron-based AutoSync implementation coming soon");
    info("Remove the heimdal sync entry from your crontab:");
    info("  crontab -e");
    Ok(())
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn status_cron() -> Result<()> {
    info("Cron-based AutoSync implementation coming soon");
    info("Check your crontab with: crontab -l");
    Ok(())
}
