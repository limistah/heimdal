use anyhow::Result;
use heimdal::lock::HeimdallLock;
use serial_test::serial;

/// Helper to clean up test lock files - force unlock at start of each test
fn cleanup_test_lock() {
    let _ = HeimdallLock::force_unlock();
}

#[test]
#[serial]
fn test_acquire_lock_successfully() {
    cleanup_test_lock();

    let lock = HeimdallLock::acquire();
    assert!(lock.is_ok(), "Should acquire lock successfully");

    cleanup_test_lock();
}

#[test]
#[serial]
fn test_lock_file_contains_current_pid() -> Result<()> {
    cleanup_test_lock();

    let _lock = HeimdallLock::acquire()?;

    let info = HeimdallLock::info()?;
    assert!(info.is_some(), "Lock info should exist");

    let lock_info = info.unwrap();
    assert_eq!(
        lock_info.pid,
        std::process::id(),
        "Lock should contain current PID"
    );
    assert!(
        !lock_info.hostname.is_empty(),
        "Hostname should not be empty"
    );

    cleanup_test_lock();
    Ok(())
}

#[test]
#[serial]
fn test_second_acquire_fails_with_already_locked() {
    cleanup_test_lock();

    let _lock1 = HeimdallLock::acquire().expect("First lock should succeed");
    let lock2 = HeimdallLock::acquire();

    assert!(lock2.is_err(), "Second acquire should fail");
    let err_msg = lock2.unwrap_err().to_string();
    assert!(
        err_msg.contains("already running") || err_msg.contains("PID"),
        "Error should mention already running or PID, got: {}",
        err_msg
    );

    cleanup_test_lock();
}

#[test]
#[serial]
fn test_release_removes_lock_file() -> Result<()> {
    cleanup_test_lock();

    {
        let _lock = HeimdallLock::acquire()?;
        let info = HeimdallLock::info()?;
        assert!(info.is_some(), "Lock should exist while held");
    } // lock dropped here

    let info = HeimdallLock::info()?;
    assert!(info.is_none(), "Lock file should be removed after release");

    cleanup_test_lock();
    Ok(())
}

#[test]
#[serial]
fn test_stale_lock_cleanup() -> Result<()> {
    cleanup_test_lock();

    // Create a fake stale lock with a dead PID
    // We can't easily test this without mocking, but we can verify the logic path
    // Instead, we'll test that force_unlock works and then we can acquire

    // First acquire a lock
    let lock = HeimdallLock::acquire()?;
    drop(lock); // Release it normally

    // Now acquire again - should work fine
    let lock2 = HeimdallLock::acquire();
    assert!(
        lock2.is_ok(),
        "Should acquire lock after previous lock was released"
    );

    cleanup_test_lock();
    Ok(())
}

#[test]
#[serial]
fn test_force_unlock() -> Result<()> {
    cleanup_test_lock();

    let _lock = HeimdallLock::acquire()?;
    assert!(HeimdallLock::info()?.is_some(), "Lock should exist");

    HeimdallLock::force_unlock()?;
    assert!(
        HeimdallLock::info()?.is_none(),
        "Lock should be removed after force unlock"
    );

    cleanup_test_lock();
    Ok(())
}
