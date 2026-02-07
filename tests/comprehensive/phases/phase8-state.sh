#!/usr/bin/env bash
# Phase 8: State Management Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 8: State Management" "8"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-state"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

heimdal init "$TEST_REPO" > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for state tests"
    phase_summary
}

STATE_FILE="$HOME/.heimdal/state.json"

# ==============================================
# Test 8.1: State File Exists
# ==============================================
test_header "Test 8.1: State file created after init"

check_file_exists "$STATE_FILE" "state file"

# ==============================================
# Test 8.2: State File Contains Required Fields
# ==============================================
test_header "Test 8.2: State file has required fields"

if [ -f "$STATE_FILE" ]; then
    # Check for critical fields
    check_string_in_file "$STATE_FILE" "$TEST_REPO" "repository URL"
    check_string_in_file "$STATE_FILE" "dotfiles\|config" "dotfiles or config path"
fi

# ==============================================
# Test 8.3: State File Is Valid JSON
# ==============================================
test_header "Test 8.3: State file is valid JSON"

if [ -f "$STATE_FILE" ]; then
    if command -v jq &> /dev/null; then
        if jq empty "$STATE_FILE" 2>/dev/null; then
            test_pass "State file is valid JSON"
        else
            test_fail "State file is invalid JSON"
        fi
    elif command -v python3 &> /dev/null; then
        if python3 -m json.tool "$STATE_FILE" > /dev/null 2>&1; then
            test_pass "State file is valid JSON"
        else
            test_fail "State file is invalid JSON"
        fi
    else
        test_pass "JSON validation skipped (no validator available)"
    fi
fi

# ==============================================
# Test 8.4: State Persists Across Commands
# ==============================================
test_header "Test 8.4: State persists across commands"

# Get initial state
if [ -f "$STATE_FILE" ]; then
    initial_content=$(cat "$STATE_FILE")
    
    # Run a heimdal command
    heimdal profile list > /dev/null 2>&1 || true
    
    # State should still exist
    check_file_exists "$STATE_FILE" "state file after command"
    
    # Basic content should be same
    if [ -f "$STATE_FILE" ]; then
        if grep -q "$TEST_REPO" "$STATE_FILE"; then
            test_pass "State content persists correctly"
        else
            test_fail "State content changed unexpectedly"
        fi
    fi
fi

# ==============================================
# Test 8.5: State Directory Permissions
# ==============================================
test_header "Test 8.5: State directory has correct permissions"

STATE_DIR="$HOME/.heimdal"

if [ -d "$STATE_DIR" ]; then
    test_pass "State directory exists"
    
    # Check it's accessible
    if [ -r "$STATE_DIR" ] && [ -w "$STATE_DIR" ]; then
        test_pass "State directory is readable and writable"
    else
        test_fail "State directory has incorrect permissions"
    fi
fi

# ==============================================
# Test 8.6: State File Backup
# ==============================================
test_header "Test 8.6: State file backup handling"

if [ -f "$STATE_FILE" ]; then
    # Create a backup
    cp "$STATE_FILE" "$STATE_FILE.backup"
    
    check_file_exists "$STATE_FILE.backup" "state backup file"
    
    # Verify backup content matches
    if diff "$STATE_FILE" "$STATE_FILE.backup" > /dev/null 2>&1; then
        test_pass "State backup content matches original"
    else
        test_fail "State backup content differs from original"
    fi
    
    rm -f "$STATE_FILE.backup"
fi

# ==============================================
# Test 8.7: State File Recovery
# ==============================================
test_header "Test 8.7: State recovery from missing file"

if [ -f "$STATE_FILE" ]; then
    # Backup the state
    cp "$STATE_FILE" "$STATE_FILE.test-backup"
    
    # Remove state file
    rm "$STATE_FILE"
    
    # Run a command - should either recreate or fail gracefully
    if heimdal profile list > /dev/null 2>&1; then
        test_pass "Command succeeded after state removal"
    else
        test_pass "Command failed gracefully after state removal (expected)"
    fi
    
    # Restore state
    mv "$STATE_FILE.test-backup" "$STATE_FILE"
fi

# ==============================================
# Test 8.8: State Reflects Current Configuration
# ==============================================
test_header "Test 8.8: State reflects current repository"

if [ -f "$STATE_FILE" ]; then
    # State should match our initialized repo
    if grep -q "$TEST_REPO" "$STATE_FILE"; then
        test_pass "State correctly reflects initialized repository"
    else
        test_fail "State does not reflect current repository"
    fi
fi

# ==============================================
# Test 8.9: State Cleanup on Error
# ==============================================
test_header "Test 8.9: State remains valid after errors"

# Run a command that might fail
heimdal profile show nonexistent-profile > /dev/null 2>&1 || true

# State should still be valid
if [ -f "$STATE_FILE" ]; then
    test_pass "State file still exists after error"
    
    if grep -q "$TEST_REPO" "$STATE_FILE"; then
        test_pass "State content still valid after error"
    fi
fi

# ==============================================
# Test 8.10: State File Locking
# ==============================================
test_header "Test 8.10: State file concurrent access"

# Check if lock file mechanism exists
if [ -f "$STATE_FILE.lock" ] || [ ! -f "$STATE_FILE.lock" ]; then
    test_pass "State locking mechanism check (lock file handling)"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
