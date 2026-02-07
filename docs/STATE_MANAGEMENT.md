# State Management

Heimdal uses a hybrid state management system inspired by Terraform to coordinate configuration changes across multiple machines safely and efficiently.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [State File Structure](#state-file-structure)
- [Locking Mechanism](#locking-mechanism)
- [Conflict Detection & Resolution](#conflict-detection--resolution)
- [CLI Commands](#cli-commands)
- [Multi-Machine Workflows](#multi-machine-workflows)
- [Troubleshooting](#troubleshooting)

---

## Overview

### Why State Management?

When managing dotfiles across multiple machines, several challenges arise:

1. **Concurrent Modifications**: Two machines updating state simultaneously can lead to lost changes
2. **Version Drift**: Different machines may have different versions of the state
3. **Conflict Resolution**: How to merge divergent states without losing data
4. **Binary Compatibility**: Ensuring different Heimdal versions can work together

### Solution: Hybrid State Management

Heimdal implements a **hybrid locking system** that combines:

- **Local file locks** for fast, single-machine operations
- **Git-based coordination** for multi-machine synchronization
- **State versioning** with automatic migration
- **Conflict detection** using lineage tracking

```
Local Machine                    Git Repository (Remote)
┌──────────────┐                ┌─────────────────────┐
│ state.json   │───sync via────▶│ state.json          │
│ state.lock   │      git       │ lock metadata       │
│ versioning   │◀───────────────│ conflict detection  │
└──────────────┘                └─────────────────────┘
```

---

## Core Concepts

### 1. State Versioning

Heimdal uses schema versioning to evolve the state file format safely.

**Current Version**: V2

**Key Features**:
- Automatic V1→V2 migration
- Backward compatibility
- Version tracking for binary compatibility

**Schema Evolution**:
```
V1 (Legacy)              V2 (Current)
├── Basic state          ├── Enhanced state
├── Simple tracking      ├── Machine metadata
└── No versioning        ├── State lineage
                         ├── Operation history
                         ├── File checksums
                         └── Lock coordination
```

### 2. Machine Metadata

Each machine is uniquely identified:

```json
{
  "machine": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "hostname": "macbook-pro",
    "os": "darwin",
    "arch": "aarch64",
    "user": "john"
  }
}
```

### 3. State Lineage

Tracks state evolution to detect conflicts:

```json
{
  "lineage": {
    "id": "lineage-abc123",
    "serial": 42,
    "parent_serial": 41,
    "git_commit": "a1b2c3d",
    "machines": ["machine-1", "machine-2"]
  }
}
```

**Serial Numbers**:
- Increment with each state write
- Used to detect concurrent modifications
- Parent serial tracks the previous version

### 4. Operation History

Maintains an audit trail of the last 50 operations:

```json
{
  "history": [
    {
      "operation": "apply",
      "timestamp": "2024-01-15T10:30:00Z",
      "machine_id": "machine-1",
      "user": "john",
      "serial": 42
    }
  ]
}
```

### 5. File Checksums

Detects out-of-band file modifications:

```json
{
  "checksums": {
    ".bashrc": "5d41402abc4b2a76b9719d911017c592",
    ".vimrc": "098f6bcd4621d373cade4e832627b4f6"
  }
}
```

---

## State File Structure

### Location

**macOS**: `~/.heimdal/state.json`  
**Linux**: `~/.heimdal/state.json`  
**Windows**: `%USERPROFILE%\.heimdal\state.json`

### Complete V2 Schema

```json
{
  "version": 2,
  "active_profile": "personal",
  "dotfiles_path": "/Users/john/.dotfiles",
  "repo_url": "git@github.com:john/dotfiles.git",
  "last_sync": "2024-01-15T10:30:00Z",
  "last_apply": "2024-01-15T10:35:00Z",
  
  "machine": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "hostname": "macbook-pro",
    "os": "darwin",
    "arch": "aarch64",
    "user": "john"
  },
  
  "heimdal_version": "1.0.0",
  
  "lineage": {
    "id": "lineage-abc123",
    "serial": 42,
    "parent_serial": 41,
    "git_commit": "a1b2c3d4e5f6",
    "machines": ["machine-1", "machine-2", "machine-3"]
  },
  
  "history": [
    {
      "operation": "apply",
      "timestamp": "2024-01-15T10:30:00Z",
      "machine_id": "machine-1",
      "user": "john",
      "serial": 42
    }
  ],
  
  "checksums": {
    ".bashrc": "5d41402abc4b2a76b9719d911017c592",
    ".vimrc": "098f6bcd4621d373cade4e832627b4f6"
  }
}
```

---

## Locking Mechanism

### Lock Types

1. **Local Lock** (Fast, single-machine)
   - File-based lock at `~/.heimdal/state.lock`
   - Prevents concurrent operations on the same machine
   - Automatic stale process detection

2. **Hybrid Lock** (Multi-machine coordination)
   - Acquires local lock first
   - Then coordinates with remote Git repository
   - Detects conflicts before allowing operations
   - Falls back to local-only if remote unavailable

3. **Disabled** (Dangerous!)
   - No locking at all
   - Only use for debugging/testing
   - Can lead to data loss

### Lock File Structure

```json
{
  "id": "lock-uuid-123",
  "lock_type": "hybrid",
  "operation": "apply",
  "machine": {
    "id": "machine-1",
    "hostname": "macbook-pro",
    "pid": 12345,
    "user": "john"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "expected_duration_seconds": 300,
  "reason": "Applying configuration",
  "state_serial": 42
}
```

### Lock Acquisition Flow

```
┌─────────────────┐
│ Operation Start │
└────────┬────────┘
         │
         ▼
    ┌─────────┐
    │ Dry-run?│───Yes──▶ Skip lock
    └────┬────┘
         │No
         ▼
┌──────────────────┐
│ Acquire Local    │
│ Lock             │
└────────┬─────────┘
         │
         ▼
    ┌─────────────┐
    │ Lock Config │
    │ = Hybrid?   │
    └──────┬──────┘
           │Yes
           ▼
   ┌───────────────┐
   │ Fetch Remote  │
   │ State         │
   └───────┬───────┘
           │
           ▼
   ┌───────────────┐
   │ Check         │
   │ Conflicts?    │
   └───┬───────────┘
       │
       ├─Yes─▶ Release lock, error
       │
       └─No──▶ Proceed with operation
               │
               ▼
       ┌───────────────┐
       │ RAII Guard    │
       │ Auto-releases │
       └───────────────┘
```

### Automatic Lock Release

Heimdal uses **RAII (Resource Acquisition Is Initialization)** pattern:

```rust
{
    // Lock acquired
    let _guard = StateGuard::acquire(...)?;
    
    // Do work...
    apply_configuration()?;
    
    // Lock automatically released when _guard goes out of scope
}
```

Benefits:
- **No forgotten unlocks** - impossible to forget to release
- **Exception-safe** - releases even if operation fails
- **Panic-safe** - releases even if code panics

### Stale Lock Detection

A lock is considered stale if:

1. **Process died**: The PID no longer exists
2. **Timeout exceeded**: Lock age > configured timeout (default: 5 minutes)
3. **Machine offline**: Remote machine hasn't been seen in 24 hours

**Automatic cleanup**:
```bash
# Manual check
heimdal state lock-info

# Force remove stale lock
heimdal state unlock --force
```

---

## Conflict Detection & Resolution

### Conflict Types

#### 1. Serial Divergence (HIGH severity)

**Cause**: Two machines modified state from the same parent

```
Machine A (serial 41)          Machine B (serial 41)
        │                              │
        ├─ apply ────▶ serial 42       ├─ apply ────▶ serial 42
        │                              │
        └──────── CONFLICT! ───────────┘
```

**Detection**:
```bash
heimdal state check-conflicts
```

**Output**:
```
🔴 State diverged: local serial 42 vs remote serial 42 (common parent: 41)
```

#### 2. Profile Mismatch (MEDIUM severity)

**Cause**: Different active profiles on different machines

```
Machine A: active_profile = "personal"
Machine B: active_profile = "work"
```

**Impact**: Usually benign, but can cause confusion

#### 3. Path Mismatch (HIGH severity)

**Cause**: Dotfiles path differs between machines

```
Machine A: /Users/john/.dotfiles
Machine B: /home/john/.dotfiles
```

**Impact**: Symlinks and file operations will fail

#### 4. File Drift (LOW severity)

**Cause**: File modified outside of Heimdal

```bash
# User manually edited .bashrc
vim ~/.bashrc

# Heimdal detects the change
heimdal state check-drift
```

### Resolution Strategies

#### 1. Use Local (Overwrite Remote)

```bash
heimdal state resolve --use-local
```

**When to use**:
- You know local state is correct
- Remote changes should be discarded
- Single source of truth needed

**Effect**:
- Local state is kept
- Remote state will be overwritten on next push

#### 2. Use Remote (Overwrite Local)

```bash
heimdal state resolve --use-remote
```

**When to use**:
- Remote state is more up-to-date
- Local changes should be discarded
- Trust the remote version

**Effect**:
- Remote state replaces local
- Local changes are lost

#### 3. Merge (Intelligent Combination)

```bash
heimdal state resolve --merge
```

**When to use**:
- Both states have valid changes
- Want to combine the best of both
- Automatic resolution preferred

**Merge Logic**:
1. Use higher serial number
2. Combine machine lists
3. Use most recent timestamps
4. Merge operation history
5. Prefer remote checksums for conflicts

**Example**:
```
Local:                    Remote:                  Merged:
  serial: 42                serial: 43               serial: 43
  machines: [A, B]          machines: [B, C]         machines: [A, B, C]
  last_sync: 10:00          last_sync: 10:30         last_sync: 10:30
```

#### 4. Manual (Edit by Hand)

```bash
heimdal state resolve --manual
```

**When to use**:
- Conflicts are complex
- Need careful inspection
- Automated resolution might be wrong

**Steps**:
1. Edit: `vim ~/.heimdal/state.json`
2. Validate: `heimdal state version`
3. Commit: `heimdal commit -m "Fix state conflicts"`
4. Push: `heimdal push`

### Automatic Resolution

The `sync` command automatically attempts to resolve conflicts:

```bash
heimdal sync
```

**Flow**:
1. Pulls from Git
2. Detects state conflicts
3. Displays conflicts to user
4. Attempts automatic merge
5. Falls back to manual if merge fails

**Output**:
```
✓ Git pull completed successfully!
⚠️  State Conflicts Detected

1. 🔴 State diverged: local serial 42 vs remote serial 43 (common parent: 41)
2. 🟢 File '.bashrc' has been modified

Attempting automatic merge...
✓ States merged successfully
  - Combined 3 machine(s)
  - Resolved 2 conflict(s)
✓ State conflicts resolved automatically
```

---

## CLI Commands

### State Information

#### Check Current State Version
```bash
heimdal state version
```

**Output**:
```
State Version: 2
Machine ID: 550e8400-e29b-41d4-a716-446655440000
Serial: 42
```

#### View Operation History
```bash
heimdal state history [--limit 10]
```

**Output**:
```
Recent State Operations (last 10):

42. apply  2024-01-15 10:35:00  macbook-pro  john
41. sync   2024-01-15 10:30:00  macbook-pro  john
40. apply  2024-01-14 15:20:00  work-laptop  john
39. sync   2024-01-14 15:15:00  work-laptop  john
```

### Lock Management

#### Show Current Lock
```bash
heimdal state lock-info
```

**Output (when locked)**:
```
State Lock Information:
  Operation: apply
  Machine: macbook-pro (550e8400-...)
  User: john
  PID: 12345
  Acquired: 2 minutes ago
  Lock ID: lock-uuid-123
```

**Output (when unlocked)**:
```
State is not locked
```

#### Force Unlock (Dangerous!)
```bash
heimdal state unlock --force
```

**Warning**: Only use if you're sure the lock is stale!

### Conflict Management

#### Detect Conflicts
```bash
heimdal state check-conflicts
```

**Output**:
```
⚠️  State Conflicts Detected

1. 🔴 State diverged: local serial 42 vs remote serial 42 (common parent: 41)
2. 🟡 Active profile differs: local 'personal' vs remote 'work'

Resolution options:
  1. Use local state:  heimdal state resolve --use-local
  2. Use remote state: heimdal state resolve --use-remote
  3. Merge states:     heimdal state resolve --merge
  4. Manual edit:      heimdal state resolve --manual
```

#### Resolve Conflicts
```bash
# Use local state
heimdal state resolve --use-local

# Use remote state
heimdal state resolve --use-remote

# Intelligent merge
heimdal state resolve --merge

# Manual resolution
heimdal state resolve --manual
```

### File Drift Detection

#### Check for Modified Files
```bash
heimdal state check-drift [--all]
```

**Output**:
```
Checking for file drift...

Modified Files (2):
  • .bashrc
    Expected: 5d41402abc4b2a76b9719d911017c592
    Actual:   098f6bcd4621d373cade4e832627b4f6
    
  • .vimrc
    Expected: 7d793037a0760186574b0282f2f435e7
    Actual:   9d4e1e23bd5b727046a9e3b4b7db57bd

Tip: Run 'heimdal apply' to restore tracked files
     or 'heimdal commit' to accept the changes
```

### Migration

#### Migrate from V1 to V2
```bash
heimdal state migrate [--no-backup]
```

**Output**:
```
Migrating state from V1 to V2...
✓ Backup created: ~/.heimdal/backups/state_v1_20240115_103000.json
✓ State migrated successfully
✓ New state saved

Migration Summary:
  - Added machine metadata
  - Initialized state lineage (serial: 1)
  - Created empty operation history
  - Initialized file checksums
```

**Backup Location**: `~/.heimdal/backups/`

---

## Multi-Machine Workflows

### Scenario 1: Adding a New Machine

**Goal**: Set up Heimdal on a second machine

**Steps**:

1. **On Machine A** (existing):
   ```bash
   # Ensure state is committed and pushed
   heimdal commit -m "Current state"
   heimdal push
   ```

2. **On Machine B** (new):
   ```bash
   # Clone dotfiles repo
   git clone git@github.com:you/dotfiles.git ~/.dotfiles
   
   # Initialize Heimdal
   cd ~/.dotfiles
   heimdal init --profile personal --repo git@github.com:you/dotfiles.git
   
   # Sync and apply
   heimdal sync
   ```

3. **Verification**:
   ```bash
   # Check that Machine B is in the lineage
   heimdal state history
   
   # You should see Machine B's machine ID
   ```

### Scenario 2: Concurrent Modifications

**Problem**: Both machines modified state while offline

**Symptoms**:
```bash
heimdal sync
# Error: State conflicts detected
```

**Resolution**:

1. **View conflicts**:
   ```bash
   heimdal state check-conflicts
   ```

2. **Choose resolution strategy**:
   
   **Option A** - Automatic merge (recommended):
   ```bash
   heimdal state resolve --merge
   heimdal push
   ```
   
   **Option B** - Keep local changes:
   ```bash
   heimdal state resolve --use-local
   heimdal push --force
   ```
   
   **Option C** - Keep remote changes:
   ```bash
   heimdal state resolve --use-remote
   heimdal apply
   ```

3. **Sync other machine**:
   ```bash
   # On the other machine
   heimdal pull
   heimdal apply
   ```

### Scenario 3: Removing a Machine

**Goal**: Stop using Heimdal on a machine

**Steps**:

1. **On machine to remove**:
   ```bash
   # Commit any pending changes
   heimdal commit -m "Final changes from old-machine"
   heimdal push
   
   # Remove Heimdal (optional)
   rm -rf ~/.heimdal
   ```

2. **On remaining machines**:
   ```bash
   # Pull the final changes
   heimdal pull
   
   # The old machine will remain in history but won't make new changes
   ```

**Note**: The old machine ID will remain in `lineage.machines` for historical tracking, but won't interfere with operations.

### Scenario 4: Working Offline

**Goal**: Use Heimdal without internet connection

**How it works**:

1. **Lock acquisition falls back to local-only**:
   ```bash
   heimdal apply
   # ⚠️  Warning: Cannot reach remote repository: connection refused
   # Proceeding with local-only lock.
   # Remember to sync when connection is restored.
   ```

2. **When back online**:
   ```bash
   heimdal sync
   # This will detect and resolve any conflicts
   ```

**Best Practice**:
- Commit changes before going offline
- Sync immediately when back online
- Resolve conflicts promptly

---

## Troubleshooting

### Problem: "State is locked" Error

**Symptom**:
```
Error: State is currently locked

Operation: apply
Machine: macbook-pro (550e8400-...)
User: john
PID: 12345
Acquired: 10 minutes ago

This means another Heimdal operation is in progress.
```

**Solutions**:

1. **Wait for operation to complete** (preferred)
   - Check if another terminal has Heimdal running
   - Wait for that operation to finish

2. **Check if process is still alive**:
   ```bash
   ps aux | grep 12345
   ```

3. **If process died, force unlock**:
   ```bash
   heimdal state unlock --force
   ```

### Problem: "Serial Divergence" Conflict

**Symptom**:
```
🔴 State diverged: local serial 42 vs remote serial 43 (common parent: 41)
```

**Cause**: Two machines modified state concurrently

**Solution**:

1. **If you know which is correct**:
   ```bash
   # Keep local
   heimdal state resolve --use-local
   
   # Or keep remote
   heimdal state resolve --use-remote
   ```

2. **If both have valid changes**:
   ```bash
   heimdal state resolve --merge
   ```

3. **If conflicts are complex**:
   ```bash
   heimdal state resolve --manual
   # Then edit ~/.heimdal/state.json
   ```

### Problem: "Cannot Reach Remote" Warning

**Symptom**:
```
⚠️  Warning: Cannot reach remote repository: connection refused
Proceeding with local-only lock.
Remember to sync when connection is restored.
```

**Cause**: No internet connection or Git remote unavailable

**Impact**: Operations proceed with local lock only

**Solution**:
- This is normal when offline
- Sync when connection is restored:
  ```bash
  heimdal sync
  ```

### Problem: File Drift Detected

**Symptom**:
```
Modified Files (1):
  • .bashrc
    Expected: 5d41402abc4b2a76b9719d911017c592
    Actual:   098f6bcd4621d373cade4e832627b4f6
```

**Cause**: File was modified outside of Heimdal

**Solutions**:

1. **Accept the changes** (keep modified version):
   ```bash
   heimdal commit -m "Accept .bashrc changes"
   heimdal push
   ```

2. **Restore from dotfiles** (revert changes):
   ```bash
   heimdal apply --force
   ```

### Problem: State Migration Failed

**Symptom**:
```
Error: Failed to migrate state from V1 to V2: ...
```

**Solutions**:

1. **Check backup exists**:
   ```bash
   ls ~/.heimdal/backups/
   ```

2. **Restore from backup if needed**:
   ```bash
   cp ~/.heimdal/backups/state_v1_*.json ~/.heimdal/state.json
   ```

3. **Try migration again with verbose output**:
   ```bash
   heimdal --verbose state migrate
   ```

4. **Manual migration** (last resort):
   - View backup: `cat ~/.heimdal/backups/state_v1_*.json`
   - Create V2 manually following the schema
   - Validate: `heimdal state version`

### Problem: Lock File Corruption

**Symptom**:
```
Error: Failed to read lock file: invalid JSON
```

**Solution**:
```bash
# Remove corrupted lock file
rm ~/.heimdal/state.lock

# Try operation again
heimdal apply
```

### Problem: Git Conflicts After Sync

**Symptom**:
```
Merge conflicts detected in:
  - state.json
  
Please resolve conflicts manually and run 'heimdal sync' again
```

**Solution**:

1. **Check Git status**:
   ```bash
   cd ~/.dotfiles
   git status
   ```

2. **Resolve Git conflicts**:
   ```bash
   # Edit conflicted files
   vim state.json
   
   # Mark as resolved
   git add state.json
   git commit -m "Resolve state conflicts"
   ```

3. **Sync again**:
   ```bash
   heimdal sync
   ```

---

## Advanced Topics

### Custom Lock Configuration

Create `~/.heimdal/lock_config.json`:

```json
{
  "lock_type": "Hybrid",
  "timeout": 600,
  "detect_stale": true,
  "retry": {
    "attempts": 5,
    "delay_seconds": 3,
    "exponential_backoff": true
  }
}
```

### State Lineage Inspection

```bash
# View lineage details
cat ~/.heimdal/state.json | jq '.lineage'
```

**Output**:
```json
{
  "id": "lineage-abc123",
  "serial": 42,
  "parent_serial": 41,
  "git_commit": "a1b2c3d4e5f6",
  "machines": [
    "machine-1",
    "machine-2",
    "machine-3"
  ]
}
```

### Debugging State Issues

Enable verbose logging:

```bash
heimdal --verbose apply
heimdal --verbose sync
heimdal --verbose state check-conflicts
```

### State Backup and Restore

**Manual backup**:
```bash
cp ~/.heimdal/state.json ~/.heimdal/state.backup.json
```

**Restore backup**:
```bash
cp ~/.heimdal/state.backup.json ~/.heimdal/state.json
heimdal state version  # Verify
```

**Automatic backups** are created:
- Before migration: `~/.heimdal/backups/state_v1_*.json`
- By default: Not implemented (use Git for versioning)

---

## Best Practices

### 1. Commit Often
```bash
# After making changes
heimdal commit -m "Add new shell aliases"
heimdal push
```

### 2. Sync Before Major Changes
```bash
# Before modifying dotfiles
heimdal sync
# Make changes...
heimdal apply
heimdal commit -m "Updated vim config"
heimdal push
```

### 3. Use Meaningful Commit Messages
```bash
# Good
heimdal commit -m "Add tmux configuration for split pane navigation"

# Bad
heimdal commit -m "update"
```

### 4. Resolve Conflicts Immediately
```bash
# Don't ignore conflicts
heimdal sync
# If conflicts detected:
heimdal state check-conflicts
heimdal state resolve --merge
heimdal push
```

### 5. Monitor State Health
```bash
# Regular checks
heimdal state version
heimdal state check-drift
heimdal state history --limit 5
```

### 6. Keep Heimdal Updated
```bash
# Check version
heimdal --version

# Update (method depends on installation)
brew upgrade heimdal  # macOS
cargo install --force heimdal  # From source
```

---

## Architecture Notes

### Design Principles

1. **Safety First**: Locks prevent data loss from concurrent operations
2. **Offline-capable**: Works without internet connection
3. **Git-native**: Leverages Git's distributed nature
4. **Transparent**: State file is readable JSON
5. **Recoverable**: Backups and manual editing possible

### Comparison to Other Tools

| Feature | Heimdal | Terraform | Chezmoi | Stow |
|---------|---------|-----------|---------|------|
| State Locking | ✅ Hybrid | ✅ Remote | ❌ None | ❌ None |
| Conflict Resolution | ✅ Yes | ✅ Yes | ❌ Manual | ❌ Manual |
| Multi-machine | ✅ Yes | ✅ Yes | ⚠️ Limited | ❌ No |
| Versioning | ✅ Yes | ✅ Yes | ❌ No | ❌ No |
| Offline Support | ✅ Yes | ⚠️ Limited | ✅ Yes | ✅ Yes |

### Future Enhancements

Potential improvements for future versions:

1. **Remote State Backend**: Store state in cloud (S3, GCS, etc.)
2. **State Encryption**: Encrypt sensitive state data
3. **Webhook Integration**: Notify on state changes
4. **State Diff**: View detailed state differences
5. **State Pruning**: Remove old history entries
6. **Conflict Visualization**: GUI for conflict resolution

---

## Support

### Getting Help

1. **Documentation**: https://github.com/limistah/heimdal/blob/main/STATE_MANAGEMENT.md
2. **Issues**: https://github.com/limistah/heimdal/issues
3. **Discussions**: https://github.com/limistah/heimdal/discussions

### Reporting Bugs

When reporting state management issues, include:

1. **State version**: `heimdal state version`
2. **Lock status**: `heimdal state lock-info`
3. **Recent history**: `heimdal state history --limit 10`
4. **Conflict details**: `heimdal state check-conflicts`
5. **Verbose logs**: `heimdal --verbose <command>`

### Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

**Version**: 1.0.0  
**Last Updated**: 2024-01-15  
**Author**: Heimdal Team
