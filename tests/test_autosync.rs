use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;

/// Skip test if systemd is available (cron is only used as fallback on Linux)
#[cfg(target_os = "linux")]
macro_rules! skip_if_systemd {
    () => {
        if std::path::Path::new("/run/systemd/system").exists() {
            eprintln!("Skipping: systemd is available, cron fallback not used");
            return;
        }

        // Also skip if crontab command doesn't exist
        let has_crontab = std::process::Command::new("which")
            .arg("crontab")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_crontab {
            eprintln!("Skipping: crontab command not found");
            return;
        }
    };
}

/// Skip test if systemd is NOT available (systemd tests require functional systemd)
#[cfg(target_os = "linux")]
macro_rules! skip_if_no_systemd {
    () => {
        // Check if systemctl --user is functional
        let has_systemd = std::process::Command::new("systemctl")
            .args(["--user", "list-timers"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_systemd {
            eprintln!("Skipping: systemd user session not available");
            return;
        }
    };
}

/// Test that autosync shows basic help text
#[test]
fn test_autosync_help() {
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Enable background sync"))
        .stdout(predicate::str::contains("Disable background sync"))
        .stdout(predicate::str::contains("Show sync status"));
}

/// Test that status command provides meaningful output (not just stub message)
#[test]
#[cfg(target_os = "macos")]
fn test_autosync_status_macos_not_stub() {
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // The stub says "check your cron jobs" which is wrong for macOS
    // The real implementation should mention launchd or at least not mention cron
    assert!(
        combined.contains("not enabled") || combined.contains("disabled"),
        "Status should show not enabled/disabled, got: {}",
        combined
    );
}

// ============================================================================
// Cron fallback integration tests (Task 18)
// ============================================================================

/// Test that enable adds crontab entry with correct interval
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_enable_cron_adds_entry() {
    skip_if_systemd!();

    // Clean up any existing heimdal cron entries first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Enable autosync with 60m interval (should be */60 in cron)
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("60m");

    cmd.assert().success();

    // Check crontab contains entry
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();

    let crontab = String::from_utf8_lossy(&output.stdout);
    assert!(
        crontab.contains("heimdal sync"),
        "Crontab should contain heimdal sync entry, got: {}",
        crontab
    );
    assert!(
        crontab.contains("*/60 * * * *"),
        "Crontab should contain */60 interval, got: {}",
        crontab
    );

    // Clean up
    let filtered: String = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        let _ = stdin.write_all(filtered.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    let _ = child.wait();
}

/// Test that crontab entry contains correct command path and arguments
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_cron_entry_contains_correct_command() {
    skip_if_systemd!();

    // Clean up first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("30m");

    cmd.assert().success();

    // Check crontab entry format
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();

    let crontab = String::from_utf8_lossy(&output.stdout);
    assert!(
        crontab.contains("sync"),
        "Crontab entry should contain 'sync' argument, got: {}",
        crontab
    );
    assert!(
        crontab.contains("*/30 * * * *"),
        "Crontab should contain */30 interval for 30m, got: {}",
        crontab
    );

    // Clean up
    let filtered: String = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        let _ = stdin.write_all(filtered.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    let _ = child.wait();
}

/// Test that status shows "enabled (via cron)" after enable
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_status_shows_enabled_cron() {
    skip_if_systemd!();

    // Clean up first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("30m");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("enabled") && combined.contains("cron"),
        "Status should show enabled (cron), got: {}",
        combined
    );

    // Clean up
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();
    let crontab = String::from_utf8_lossy(&output.stdout);
    let filtered: String = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        let _ = stdin.write_all(filtered.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    let _ = child.wait();
}

/// Test that disable removes crontab entry
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_disable_cron_removes_entry() {
    skip_if_systemd!();

    // Clean up first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Enable first
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("30m");
    cmd.assert().success();

    // Verify entry exists
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();
    let crontab = String::from_utf8_lossy(&output.stdout);
    assert!(
        crontab.contains("heimdal sync"),
        "Crontab should contain entry after enable"
    );

    // Disable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Verify entry is removed
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();
    let crontab = String::from_utf8_lossy(&output.stdout);
    assert!(
        !crontab.contains("heimdal sync"),
        "Crontab should not contain entry after disable, got: {}",
        crontab
    );
}

/// Test that status shows "disabled" after disable
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_status_shows_disabled_after_disable_cron() {
    skip_if_systemd!();

    // Clean up first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Enable then disable
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("30m");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not enabled") || combined.contains("disabled"),
        "Status should show not enabled/disabled, got: {}",
        combined
    );
}

/// Test that multiple enable/disable cycles work correctly
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_cron_multiple_cycles() {
    skip_if_systemd!();

    // Clean up first
    let _ = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .and_then(|output| {
            let crontab = String::from_utf8_lossy(&output.stdout);
            let filtered: String = crontab
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
                stdin.write_all(filtered.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            child.wait()
        });

    // Cycle 1
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("30m");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Cycle 2
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("60m");
    cmd.assert().success();

    // Verify only one entry exists
    let output = std::process::Command::new("crontab")
        .arg("-l")
        .output()
        .unwrap();
    let crontab = String::from_utf8_lossy(&output.stdout);
    let count = crontab
        .lines()
        .filter(|l| l.contains("heimdal sync"))
        .count();
    assert_eq!(
        count, 1,
        "Should only have one heimdal entry, got: {}",
        crontab
    );

    // Clean up
    let filtered: String = crontab
        .lines()
        .filter(|l| !l.contains("heimdal sync"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        let _ = stdin.write_all(filtered.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    let _ = child.wait();
}

/// Test that enable on macOS doesn't reference cron in stub way
#[test]
#[cfg(target_os = "macos")]
fn test_autosync_enable_macos_not_cron_stub() {
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("1h");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // The stub suggests adding a cron job which is inappropriate for macOS
    // Real implementation should use launchd
    assert!(
        !combined.contains("crontab -e")
            || combined.contains("launchd")
            || combined.contains("LaunchAgent"),
        "macOS should use launchd, not suggest crontab directly, got: {}",
        combined
    );
}

/// Test that disable on macOS doesn't reference cron in stub way
#[test]
#[cfg(target_os = "macos")]
fn test_autosync_disable_macos_not_cron_stub() {
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // The stub suggests removing cron job which is inappropriate for macOS
    // Real implementation should use launchd
    assert!(
        !combined.contains("remove the cron job")
            || combined.contains("launchd")
            || combined.contains("LaunchAgent"),
        "macOS should use launchd, not mention cron job removal, got: {}",
        combined
    );
}

// ============================================================================
// macOS launchd integration tests (Task 16)
// ============================================================================

/// Test that enable creates plist file at correct location
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_enable_creates_plist() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable autosync with 10m interval
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Verify plist file exists
    assert!(
        plist_path.exists(),
        "Plist file should be created at {}",
        plist_path.display()
    );

    // Clean up
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);
}

/// Test that plist contains correct interval (600 seconds for 10m)
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_plist_contains_correct_interval() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable autosync with 10m interval
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Read plist and verify interval
    let content = std::fs::read_to_string(&plist_path).unwrap();
    assert!(
        content.contains("<key>StartInterval</key>"),
        "Plist should contain StartInterval key"
    );
    assert!(
        content.contains("<integer>600</integer>"),
        "Plist should contain 600 seconds (10 minutes)"
    );

    // Clean up
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);
}

/// Test that plist contains correct program path and arguments
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_plist_contains_correct_program() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Read plist and verify program arguments
    let content = std::fs::read_to_string(&plist_path).unwrap();
    assert!(
        content.contains("<key>ProgramArguments</key>"),
        "Plist should contain ProgramArguments"
    );
    assert!(
        content.contains("<string>sync</string>"),
        "Plist should contain 'sync' argument"
    );

    // Clean up
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);
}

/// Test that status shows "enabled" after enable
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_status_shows_enabled_after_enable() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("enabled") || combined.contains("active"),
        "Status should show enabled/active, got: {}",
        combined
    );

    // Clean up
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);
}

/// Test that disable removes plist file
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_disable_removes_plist() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable autosync first
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    // Verify plist exists
    assert!(plist_path.exists(), "Plist should exist after enable");

    // Disable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Verify plist is removed
    assert!(
        !plist_path.exists(),
        "Plist should be removed after disable"
    );
}

/// Test that status shows "disabled" after disable
#[test]
#[serial]
#[cfg(target_os = "macos")]
fn test_autosync_status_shows_disabled_after_disable() {
    use std::path::PathBuf;

    // Clean up any existing plist and launchd job first
    let plist_path: PathBuf = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.heimdal.autosync.plist");
    let _ = std::process::Command::new("launchctl")
        .args(["remove", "com.heimdal.autosync"])
        .output();
    let _ = std::fs::remove_file(&plist_path);

    // Enable then disable
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not enabled") || combined.contains("disabled"),
        "Status should show not enabled/disabled, got: {}",
        combined
    );
}

// ============================================================================
// Linux systemd integration tests (Task 17)
// ============================================================================

/// Test that enable creates service and timer files at correct location
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_enable_creates_systemd_files() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable autosync with 10m interval
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Verify service file exists
    assert!(
        service_path.exists(),
        "Service file should be created at {}",
        service_path.display()
    );

    // Verify timer file exists
    assert!(
        timer_path.exists(),
        "Timer file should be created at {}",
        timer_path.display()
    );

    // Clean up
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);
}

/// Test that service file contains correct ExecStart path and arguments
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_service_file_contains_correct_exec() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Read service file and verify ExecStart
    let content = std::fs::read_to_string(&service_path).unwrap();
    assert!(
        content.contains("ExecStart="),
        "Service file should contain ExecStart"
    );
    assert!(
        content.contains(" sync"),
        "Service file should contain 'sync' argument"
    );
    assert!(
        content.contains("Type=oneshot"),
        "Service file should be oneshot type"
    );

    // Clean up
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);
}

/// Test that timer file contains correct OnUnitActiveSec interval
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_timer_file_contains_correct_interval() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable autosync with 10m interval (600 seconds)
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");

    cmd.assert().success();

    // Read timer file and verify interval
    let content = std::fs::read_to_string(&timer_path).unwrap();
    assert!(
        content.contains("OnUnitActiveSec="),
        "Timer file should contain OnUnitActiveSec"
    );
    assert!(
        content.contains("OnUnitActiveSec=600s"),
        "Timer file should contain 600 seconds (10 minutes)"
    );
    assert!(
        content.contains("Unit=heimdal-autosync.service"),
        "Timer file should reference the service"
    );

    // Clean up
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);
}

/// Test that status shows "enabled" after enable
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_status_shows_enabled_after_enable_systemd() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("enabled") || combined.contains("active"),
        "Status should show enabled/active, got: {}",
        combined
    );

    // Clean up
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);
}

/// Test that disable removes service and timer files
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_disable_removes_systemd_files() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable autosync first
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    // Verify files exist
    assert!(
        service_path.exists(),
        "Service file should exist after enable"
    );
    assert!(timer_path.exists(), "Timer file should exist after enable");

    // Disable autosync
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Verify files are removed
    assert!(
        !service_path.exists(),
        "Service file should be removed after disable"
    );
    assert!(
        !timer_path.exists(),
        "Timer file should be removed after disable"
    );
}

/// Test that status shows "disabled" after disable
#[test]
#[serial]
#[cfg(target_os = "linux")]
fn test_autosync_status_shows_disabled_after_disable_systemd() {
    skip_if_no_systemd!();

    use std::path::PathBuf;

    // Clean up any existing systemd files and timer first
    let systemd_dir: PathBuf = dirs::home_dir().unwrap().join(".config/systemd/user");
    let service_path = systemd_dir.join("heimdal-autosync.service");
    let timer_path = systemd_dir.join("heimdal-autosync.timer");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "heimdal-autosync.timer"])
        .output();
    let _ = std::fs::remove_file(&service_path);
    let _ = std::fs::remove_file(&timer_path);

    // Enable then disable
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync")
        .arg("enable")
        .arg("--interval")
        .arg("10m");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("disable");
    cmd.assert().success();

    // Check status
    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("auto-sync").arg("status");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("not enabled") || combined.contains("disabled"),
        "Status should show not enabled/disabled, got: {}",
        combined
    );
}
