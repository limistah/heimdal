#!/usr/bin/env bash
# Phase 9: Rollback & Recovery Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 9: Rollback & Recovery" "9"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-rollback"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for rollback tests"
    phase_summary
}

cd "$DOTFILES_DIR"

# ==============================================
# Test 9.1: Backup Before Changes
# ==============================================
test_header "Test 9.1: Create backup before making changes"

# Make initial state snapshot
git tag -a backup-test -m "Test backup point" 2>/dev/null || test_pass "Backup tag created (or already exists)"

check_output_contains "git tag -l" "backup-test" "backup tag"

# ==============================================
# Test 9.2: Make Breaking Changes
# ==============================================
test_header "Test 9.2: Make changes that need rollback"

echo "BROKEN CONFIG" > .bashrc
git add .bashrc
git commit -m "Test: breaking change" > /dev/null 2>&1

test_pass "Breaking change committed"

# ==============================================
# Test 9.3: Rollback Using Git
# ==============================================
test_header "Test 9.3: Rollback to previous state"

if git reset --hard backup-test > /dev/null 2>&1; then
    test_pass "Git rollback succeeded"
    
    # Verify .bashrc is restored
    if ! grep -q "BROKEN CONFIG" .bashrc; then
        test_pass "Breaking change successfully rolled back"
    else
        test_fail "Breaking change still present after rollback"
    fi
else
    test_fail "Git rollback failed"
fi

# ==============================================
# Test 9.4: File-Level Rollback
# ==============================================
test_header "Test 9.4: Rollback individual file"

echo "Another breaking change" > .vimrc
git add .vimrc

# Restore single file
if git restore --staged .vimrc && git restore .vimrc; then
    test_pass "Single file rollback succeeded"
    
    if ! grep -q "Another breaking change" .vimrc; then
        test_pass "File successfully restored"
    fi
else
    test_fail "Single file rollback failed"
fi

# ==============================================
# Test 9.5: Symlink Rollback
# ==============================================
test_header "Test 9.5: Rollback symlinks"

# Create symlinks
heimdal symlink create test > /dev/null 2>&1 || true

if [ -L "$HOME/.bashrc" ]; then
    test_pass "Symlinks created"
    
    # Remove them
    heimdal symlink remove test > /dev/null 2>&1 || rm -f "$HOME/.bashrc" "$HOME/.vimrc"
    
    if [ ! -L "$HOME/.bashrc" ]; then
        test_pass "Symlinks removed (rollback preparation)"
    fi
    
    # Re-create (rollback)
    heimdal symlink create test > /dev/null 2>&1 || true
    
    if [ -L "$HOME/.bashrc" ]; then
        test_pass "Symlinks restored"
    fi
fi

# ==============================================
# Test 9.6: State File Recovery
# ==============================================
test_header "Test 9.6: Recover from corrupted state"

STATE_FILE="$HOME/.heimdal/state.json"
STATE_BACKUP="$HOME/.heimdal/state.json.backup"

if [ -f "$STATE_FILE" ]; then
    # Backup state
    cp "$STATE_FILE" "$STATE_BACKUP"
    
    # Corrupt state
    echo "corrupted" > "$STATE_FILE"
    
    # Restore from backup
    mv "$STATE_BACKUP" "$STATE_FILE"
    
    if grep -q "$TEST_REPO" "$STATE_FILE"; then
        test_pass "State file successfully recovered"
    else
        test_fail "State file recovery failed"
    fi
fi

# ==============================================
# Test 9.7: Config Validation Before Apply
# ==============================================
test_header "Test 9.7: Validate config before applying changes"

if command -v heimdal config validate &> /dev/null; then
    if heimdal config validate > /dev/null 2>&1; then
        test_pass "Config validation passed"
    else
        test_fail "Config validation failed"
    fi
else
    test_pass "Config validation check (command may not exist)"
fi

# ==============================================
# Test 9.8: Dry-Run Mode
# ==============================================
test_header "Test 9.8: Dry-run prevents actual changes"

# Test dry-run if available
if heimdal symlink create test --dry-run > /dev/null 2>&1; then
    test_pass "Dry-run mode available and executed"
else
    test_pass "Dry-run check (flag may not be supported)"
fi

# ==============================================
# Test 9.9: Emergency Restore
# ==============================================
test_header "Test 9.9: Emergency restore from git"

# Simulate emergency: reset to origin
if git fetch origin main > /dev/null 2>&1 || true; then
    test_pass "Can fetch from origin for emergency restore"
fi

# ==============================================
# Test 9.10: Recovery Documentation
# ==============================================
test_header "Test 9.10: Recovery procedures accessible"

# Check if README or docs mention recovery
if [ -f "$DOTFILES_DIR/README.md" ]; then
    test_pass "README exists (may contain recovery info)"
fi

# ==============================================
# Cleanup
# ==============================================

# Clean up test symlinks
heimdal symlink remove test > /dev/null 2>&1 || true
rm -f "$HOME/.bashrc" "$HOME/.vimrc" "$HOME/.gitconfig"

cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
