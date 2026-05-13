use assert_cmd::Command;
use heimdal::lock::HeimdallLock;
use predicates::prelude::*;
use serial_test::serial;

/// Helper to ensure no lock exists before test
fn cleanup_lock() {
    let _ = HeimdallLock::force_unlock();
}

#[test]
#[serial]
fn test_state_lock_info_no_lock() {
    cleanup_lock();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("state").arg("lock-info");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No active lock."));
}

#[test]
#[serial]
fn test_state_lock_info_with_active_lock() {
    cleanup_lock();

    // Create a lock
    let _lock = HeimdallLock::acquire().unwrap();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("state").arg("lock-info");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Lock held by PID"))
        .stdout(predicate::str::contains("Started:"))
        .stdout(predicate::str::contains("Status: active"));
}

#[test]
#[serial]
fn test_state_unlock_without_force() {
    cleanup_lock();

    // Create a lock
    let _lock = HeimdallLock::acquire().unwrap();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("state").arg("unlock");

    cmd.assert().success().stdout(predicate::str::contains(
        "Use --force to remove a lock file.",
    ));

    // Lock should still exist
    assert!(HeimdallLock::info().unwrap().is_some());
}

#[test]
#[serial]
fn test_state_unlock_with_force() {
    cleanup_lock();

    // Create a lock
    let _lock = HeimdallLock::acquire().unwrap();

    // Verify lock exists
    assert!(HeimdallLock::info().unwrap().is_some());

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("state").arg("unlock").arg("--force");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Lock removed."));

    // Lock should be gone
    assert!(HeimdallLock::info().unwrap().is_none());
}

#[test]
#[serial]
fn test_state_unlock_force_when_no_lock() {
    cleanup_lock();

    let mut cmd = Command::cargo_bin("heimdal").unwrap();
    cmd.arg("state").arg("unlock").arg("--force");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Lock removed."));
}
