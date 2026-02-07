#!/usr/bin/env bash
# Phase 2: Configuration & Profiles Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 2: Configuration & Profiles" "2"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-config"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Initialize heimdal
heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for config tests"
    phase_summary
}

# ==============================================
# Test 2.1: List Available Profiles
# ==============================================
test_header "Test 2.1: List available profiles"

if output=$(heimdal profile list 2>&1); then
    test_pass "Profile list command succeeded"
    
    # Check for expected profiles from test repo
    if echo "$output" | grep -q "test"; then
        test_pass "Found 'test' profile"
    else
        test_fail "'test' profile not found in list"
    fi
    
    if echo "$output" | grep -q "development"; then
        test_pass "Found 'development' profile"
    else
        test_fail "'development' profile not found in list"
    fi
else
    test_fail "Profile list command failed"
fi

# ==============================================
# Test 2.2: Show Profile Details
# ==============================================
test_header "Test 2.2: Show profile details"

if output=$(heimdal profile show test 2>&1); then
    test_pass "Profile show command succeeded"
    
    # Check for expected content
    if echo "$output" | grep -q "git\|vim\|curl"; then
        test_pass "Profile contains expected packages"
    fi
else
    test_fail "Profile show command failed"
fi

# ==============================================
# Test 2.3: Config Validation
# ==============================================
test_header "Test 2.3: Config validation"

if heimdal config validate > /dev/null 2>&1; then
    test_pass "Config validation passed"
else
    test_fail "Config validation failed"
fi

# ==============================================
# Test 2.4: View Current Config
# ==============================================
test_header "Test 2.4: View current configuration"

if output=$(heimdal config show 2>&1); then
    test_pass "Config show command succeeded"
    
    # Check for expected sections
    check_string_in_file <(echo "$output") "heimdal:" "heimdal section in config"
    check_string_in_file <(echo "$output") "profiles:" "profiles section in config"
else
    test_fail "Config show command failed"
fi

# ==============================================
# Test 2.5: Profile with Stow Compatibility
# ==============================================
test_header "Test 2.5: Stow compatibility enabled in config"

if output=$(heimdal config show 2>&1); then
    if echo "$output" | grep -q "stow_compat.*true"; then
        test_pass "Stow compatibility is enabled"
    else
        test_fail "Stow compatibility not found or disabled"
    fi
fi

# ==============================================
# Test 2.6: Check Ignore Patterns
# ==============================================
test_header "Test 2.6: Ignore patterns configured"

CONFIG_FILE="$DOTFILES_DIR/heimdal.yaml"

if [ -f "$CONFIG_FILE" ]; then
    check_string_in_file "$CONFIG_FILE" "ignore:" "ignore section"
    check_string_in_file "$CONFIG_FILE" ".git" ".git in ignore list"
    check_string_in_file "$CONFIG_FILE" "heimdal.yaml" "heimdal.yaml in ignore list"
fi

# ==============================================
# Test 2.7: Profile Sources Validation
# ==============================================
test_header "Test 2.7: Profile sources validation"

if output=$(heimdal profile show test 2>&1); then
    # Check that sources section exists and has expected types
    if echo "$output" | grep -q "packages\|symlinks"; then
        test_pass "Profile has valid sources section"
    else
        test_fail "Profile sources section invalid or missing"
    fi
fi

# ==============================================
# Test 2.8: Multiple Profiles Support
# ==============================================
test_header "Test 2.8: Multiple profiles supported"

# Count profiles
profile_count=0
if output=$(heimdal profile list 2>&1); then
    profile_count=$(echo "$output" | grep -c "test\|development" || echo 0)
    
    if [ "$profile_count" -ge 2 ]; then
        test_pass "Multiple profiles found ($profile_count)"
    else
        test_fail "Expected at least 2 profiles, found $profile_count"
    fi
fi

# ==============================================
# Test 2.9: Error Case - Invalid Profile Name
# ==============================================
test_header "Test 2.9: Show non-existent profile fails gracefully"

EXPECTED_ERROR="not found\|does not exist\|unknown" run_failure heimdal profile show nonexistent-profile-xyz

# ==============================================
# Test 2.10: Config Path Detection
# ==============================================
test_header "Test 2.10: Config file path detection"

# Heimdal should find the config in the dotfiles directory
if [ -f "$DOTFILES_DIR/heimdal.yaml" ]; then
    test_pass "Config file exists at expected location"
else
    test_fail "Config file not found at expected location"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
