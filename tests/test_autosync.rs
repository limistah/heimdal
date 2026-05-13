use assert_cmd::Command;
use predicates::prelude::*;

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
        !combined.contains("check your cron jobs"),
        "macOS should not reference cron jobs, got: {}",
        combined
    );
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
