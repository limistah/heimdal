use crate::cli::AutoSyncCmd;
use crate::utils::{info, success};
use anyhow::Result;
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
// Linux systemd implementation
// ============================================================================

#[cfg(target_os = "linux")]
fn has_systemd() -> bool {
    std::path::Path::new("/run/systemd/system").exists()
}

#[cfg(target_os = "linux")]
fn systemd_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".config/systemd/user")
}

#[cfg(target_os = "linux")]
fn enable_systemd(interval_secs: u64) -> Result<()> {
    let dir = systemd_dir();
    crate::utils::ensure_parent_exists(&dir.join("dummy"))?;

    let heimdal_path = std::env::current_exe()?;

    // Service file
    let service = format!(
        r#"[Unit]
Description=Heimdal dotfile sync

[Service]
Type=oneshot
ExecStart={} sync
"#,
        heimdal_path.display()
    );

    // Timer file
    let timer = format!(
        r#"[Unit]
Description=Heimdal autosync timer

[Timer]
OnBootSec=60
OnUnitActiveSec={}s
Unit=heimdal-autosync.service

[Install]
WantedBy=timers.target
"#,
        interval_secs
    );

    std::fs::write(dir.join("heimdal-autosync.service"), service)?;
    std::fs::write(dir.join("heimdal-autosync.timer"), timer)?;

    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;
    std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", "heimdal-autosync.timer"])
        .status()?;

    success(&format!(
        "AutoSync enabled (every {} seconds)",
        interval_secs
    ));
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_systemd() -> Result<()> {
    std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .status()?;

    let dir = systemd_dir();
    let _ = std::fs::remove_file(dir.join("heimdal-autosync.service"));
    let _ = std::fs::remove_file(dir.join("heimdal-autosync.timer"));

    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    success("AutoSync disabled.");
    Ok(())
}

#[cfg(target_os = "linux")]
fn status_systemd() -> Result<()> {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-active", "heimdal-autosync.timer"])
        .output()?;

    if output.status.success() {
        success("AutoSync is enabled (systemd timer).");
        // Show next run time
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "list-timers", "heimdal-autosync.timer"])
            .status();
    } else {
        info("AutoSync is not enabled.");
    }
    Ok(())
}

// ============================================================================
// Cron fallback implementation (Task 18)
// ============================================================================

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn enable_cron(interval_secs: u64) -> Result<()> {
    let heimdal_path = std::env::current_exe()?;
    let minutes = (interval_secs / 60).max(1);

    // Get existing crontab
    let output = std::process::Command::new("crontab").arg("-l").output();

    let mut crontab = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    };

    // Remove any existing heimdal entries
    crontab = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    // Add new entry
    let entry = format!("*/{} * * * * {} sync", minutes, heimdal_path.display());
    if !crontab.is_empty() && !crontab.ends_with('\n') {
        crontab.push('\n');
    }
    crontab.push_str(&entry);
    crontab.push('\n');

    // Write new crontab
    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(crontab.as_bytes())?;
    }
    child.wait()?;

    success(&format!(
        "AutoSync enabled (cron, every {} minutes)",
        minutes
    ));
    Ok(())
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn disable_cron() -> Result<()> {
    let output = std::process::Command::new("crontab").arg("-l").output()?;

    if !output.status.success() {
        info("No crontab found.");
        return Ok(());
    }

    let crontab: String = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(crontab.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    child.wait()?;

    success("AutoSync disabled.");
    Ok(())
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "macos", target_os = "linux"))
))]
fn status_cron() -> Result<()> {
    let output = std::process::Command::new("crontab").arg("-l").output();

    match output {
        Ok(o) if o.status.success() => {
            let crontab = String::from_utf8_lossy(&o.stdout);
            if crontab.contains("heimdal sync") {
                success("AutoSync is enabled (cron).");
                for line in crontab.lines() {
                    if line.contains("heimdal sync") {
                        info(&format!("  {}", line));
                    }
                }
            } else {
                info("AutoSync is not enabled.");
            }
        }
        _ => {
            info("AutoSync is not enabled (no crontab).");
        }
    }
    Ok(())
}
