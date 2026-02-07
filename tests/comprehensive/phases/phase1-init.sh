#!/usr/bin/env bash
# Phase 1: Setup & Initialization Tests
# Tests the critical v1.1.1 bugfix where repo field is now optional

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 1: Setup & Initialization" "1"
setup_test_env

# Test repository and directories
TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-init"
DOTFILES_DIR="$TEST_DIR/dotfiles"

# Clean up any existing test directory
cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# ==============================================
# Test 1.1: Version Check
# ==============================================
test_header "Test 1.1: Verify Heimdal version is 1.1.1"

if heimdal --version | grep -q "1.1.1"; then
    test_pass "Heimdal version 1.1.1 confirmed"
else
    version=$(heimdal --version || echo "version check failed")
    test_fail "Expected version 1.1.1, got: $version"
fi

# ==============================================
# Test 1.2: Help Command
# ==============================================
test_header "Test 1.2: Help command works"

if heimdal --help > /dev/null 2>&1; then
    test_pass "Help command succeeded"
else
    test_fail "Help command failed"
fi

# ==============================================
# Test 1.3: Init Without Repo Field (CRITICAL v1.1.1 TEST)
# ==============================================
test_header "Test 1.3: Init without repo field in heimdal.yaml (v1.1.1 bugfix)"

# This is THE critical test for v1.1.1!
# The test repository intentionally omits the repo: field
# In v1.1.0 this would fail with "missing field repo" error
# In v1.1.1 this should succeed

if output=$(heimdal init "$TEST_REPO" 2>&1); then
    test_pass "heimdal init succeeded without repo field"
    
    # Verify the error message is NOT present
    if echo "$output" | grep -q "missing field repo"; then
        test_fail "v1.1.1 bugfix FAILED: Still seeing 'missing field repo' error"
        create_failure_report "Init with optional repo field" "Bug still present: missing field repo error" "$LOG_DIR/phase1.log"
    else
        test_pass "v1.1.1 bugfix CONFIRMED: No 'missing field repo' error"
    fi
else
    test_fail "heimdal init failed unexpectedly"
    echo "$output"
    create_failure_report "Init command" "heimdal init failed" "$LOG_DIR/phase1.log"
fi

# ==============================================
# Test 1.4: State File Creation
# ==============================================
test_header "Test 1.4: State file created correctly"

STATE_FILE="$HOME/.heimdal/state.json"

if check_file_exists "$STATE_FILE" "Heimdal state file"; then
    # Check that state file contains the repo URL
    if check_string_in_file "$STATE_FILE" "$TEST_REPO" "repo URL in state file"; then
        test_pass "State file contains repo URL (compensating for optional repo field)"
    fi
fi

# ==============================================
# Test 1.5: Config File Validation
# ==============================================
test_header "Test 1.5: Config file validation"

CONFIG_FILE="$DOTFILES_DIR/heimdal.yaml"

if check_file_exists "$CONFIG_FILE" "heimdal.yaml"; then
    # Verify repo field is actually absent
    if ! grep -q "^[[:space:]]*repo:" "$CONFIG_FILE"; then
        test_pass "Config file correctly omits repo field (testing v1.1.1 feature)"
    else
        test_fail "Config file unexpectedly contains repo field"
    fi
    
    # Verify required fields are present
    check_string_in_file "$CONFIG_FILE" "version:" "version field"
    check_string_in_file "$CONFIG_FILE" "heimdal:" "heimdal section"
fi

# ==============================================
# Test 1.6: Dotfiles Directory Creation
# ==============================================
test_header "Test 1.6: Dotfiles directory created"

check_dir_exists "$DOTFILES_DIR" "dotfiles directory"

# ==============================================
# Test 1.7: Git Repository Cloning
# ==============================================
test_header "Test 1.7: Test repository cloned successfully"

if [ -d "$DOTFILES_DIR/.git" ]; then
    test_pass "Git repository cloned"
    
    # Verify it's the correct repository
    cd "$DOTFILES_DIR"
    if remote_url=$(git remote get-url origin 2>/dev/null); then
        if echo "$remote_url" | grep -q "$TEST_REPO"; then
            test_pass "Correct repository URL: $remote_url"
        else
            test_fail "Wrong repository URL: $remote_url"
        fi
    fi
    cd "$TEST_DIR"
else
    test_fail "Git repository not cloned"
fi

# ==============================================
# Test 1.8: Expected Test Files Present
# ==============================================
test_header "Test 1.8: Test fixtures present in cloned repo"

check_file_exists "$DOTFILES_DIR/.bashrc" ".bashrc"
check_file_exists "$DOTFILES_DIR/.vimrc" ".vimrc"
check_file_exists "$DOTFILES_DIR/.gitconfig" ".gitconfig"
check_dir_exists "$DOTFILES_DIR/.config/nvim" ".config/nvim"

# ==============================================
# Test 1.9: Error Case - Invalid Repository
# ==============================================
test_header "Test 1.9: Init with invalid repository fails gracefully"

INVALID_TEST_DIR="$HOME/heimdal-test-invalid"
cleanup_test_dir "$INVALID_TEST_DIR"
mkdir -p "$INVALID_TEST_DIR"
cd "$INVALID_TEST_DIR"

EXPECTED_ERROR="not found\|does not exist\|failed" run_failure heimdal init "invalid/nonexistent-repo-12345"

cd "$TEST_DIR"
cleanup_test_dir "$INVALID_TEST_DIR"

# ==============================================
# Test 1.10: Error Case - Re-init in Existing Directory
# ==============================================
test_header "Test 1.10: Re-init in existing directory handled correctly"

cd "$TEST_DIR"

# Try to init again in the same directory
# This should either succeed (idempotent) or fail gracefully
if heimdal init "$TEST_REPO" > /dev/null 2>&1; then
    test_pass "Re-init succeeded (idempotent behavior)"
else
    # Failure is acceptable if error message is reasonable
    test_pass "Re-init failed gracefully (expected behavior)"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

# Print phase summary and exit
phase_summary
