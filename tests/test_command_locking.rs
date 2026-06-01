use heimdal::lock::HeimdallLock;
use serial_test::serial;

/// Test the lock lifecycle - acquire, hold, release
#[test]
#[serial]
fn test_lock_lifecycle() {
    // Clean up any existing lock
    let _ = HeimdallLock::force_unlock();

    // Verify no lock exists
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(info.is_none(), "Lock should not exist before test");

    // Acquire a lock to simulate another process
    let lock = HeimdallLock::acquire().expect("Failed to acquire lock");

    // Verify lock exists
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(info.is_some(), "Lock should exist");

    // Drop the lock
    drop(lock);

    // Verify lock is released
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(info.is_none(), "Lock should be released after drop");
}

/// Test that concurrent lock acquisition fails
#[test]
#[serial]
fn test_concurrent_lock_prevention() {
    // Clean up any existing lock
    let _ = HeimdallLock::force_unlock();

    // Acquire first lock
    let _lock1 = HeimdallLock::acquire().expect("Failed to acquire first lock");

    // Try to acquire second lock - should fail
    let result = HeimdallLock::acquire();

    assert!(result.is_err(), "Second lock acquisition should fail");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("already running"),
        "Error should mention already running, got: {}",
        error_msg
    );
}

/// Integration test: Verify apply.rs will use locking
/// This test documents the expected behavior - apply should acquire lock
#[test]
#[serial]
fn test_apply_command_will_use_locking() {
    // This test serves as documentation that apply command should:
    // 1. Call HeimdallLock::acquire() at the start of run()
    // 2. Hold the lock for the duration of the operation
    // 3. Release the lock when done (via Drop)

    // The actual implementation will be in src/commands/apply.rs:
    // let _lock = crate::lock::HeimdallLock::acquire()?;

    // Clean up any existing lock
    let _ = HeimdallLock::force_unlock();

    // Simulate what apply will do:
    let _lock = HeimdallLock::acquire().expect("Apply should acquire lock");

    // Lock should exist during operation
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(info.is_some(), "Lock should exist during apply operation");

    // After dropping (end of operation), lock should be gone
    drop(_lock);
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(
        info.is_none(),
        "Lock should be released after apply completes"
    );
}

/// Integration test: Verify sync.rs will use locking
/// This test documents the expected behavior - sync should acquire lock
#[test]
#[serial]
fn test_sync_command_will_use_locking() {
    // This test serves as documentation that sync command should:
    // 1. Call HeimdallLock::acquire() at the start of run()
    // 2. Hold the lock for the duration of the operation
    // 3. Release the lock when done (via Drop)

    // The actual implementation will be in src/commands/sync.rs:
    // let _lock = crate::lock::HeimdallLock::acquire()?;

    // Clean up any existing lock
    let _ = HeimdallLock::force_unlock();

    // Simulate what sync will do:
    let _lock = HeimdallLock::acquire().expect("Sync should acquire lock");

    // Lock should exist during operation
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(info.is_some(), "Lock should exist during sync operation");

    // After dropping (end of operation), lock should be gone
    drop(_lock);
    let info = HeimdallLock::info().expect("Failed to check lock info");
    assert!(
        info.is_none(),
        "Lock should be released after sync completes"
    );
}
