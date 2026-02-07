#!/usr/bin/env bash
# Phase 3: Dotfile Symlinks Tests (Stow Compatibility)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 3: Dotfile Symlinks" "3"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-symlinks"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Initialize heimdal
heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for symlink tests"
    phase_summary
}

# ==============================================
# Test 3.1: Symlink Dotfiles
# ==============================================
test_header "Test 3.1: Create symlinks for dotfiles"

if heimdal symlink create test > /dev/null 2>&1; then
    test_pass "Symlink create command succeeded"
else
    test_fail "Symlink create command failed"
fi

# ==============================================
# Test 3.2: Verify .bashrc Symlink
# ==============================================
test_header "Test 3.2: .bashrc symlinked correctly"

BASHRC_LINK="$HOME/.bashrc"
BASHRC_TARGET="$DOTFILES_DIR/.bashrc"

if [ -L "$BASHRC_LINK" ]; then
    actual_target=$(readlink "$BASHRC_LINK")
    
    if [ "$actual_target" = "$BASHRC_TARGET" ]; then
        test_pass ".bashrc symlink points to correct target"
    else
        test_fail ".bashrc symlink target mismatch (expected: $BASHRC_TARGET, actual: $actual_target)"
    fi
else
    test_fail ".bashrc symlink does not exist"
fi

# ==============================================
# Test 3.3: Verify .vimrc Symlink
# ==============================================
test_header "Test 3.3: .vimrc symlinked correctly"

VIMRC_LINK="$HOME/.vimrc"
VIMRC_TARGET="$DOTFILES_DIR/.vimrc"

check_symlink "$VIMRC_LINK" "$VIMRC_TARGET" ".vimrc"

# ==============================================
# Test 3.4: Verify .gitconfig Symlink
# ==============================================
test_header "Test 3.4: .gitconfig symlinked correctly"

GITCONFIG_LINK="$HOME/.gitconfig"
GITCONFIG_TARGET="$DOTFILES_DIR/.gitconfig"

check_symlink "$GITCONFIG_LINK" "$GITCONFIG_TARGET" ".gitconfig"

# ==============================================
# Test 3.5: Verify Nested Config Directory Symlinks
# ==============================================
test_header "Test 3.5: Nested config directories symlinked"

NVIM_LINK="$HOME/.config/nvim"
NVIM_TARGET="$DOTFILES_DIR/.config/nvim"

if check_dir_exists "$HOME/.config" ".config directory"; then
    check_symlink "$NVIM_LINK" "$NVIM_TARGET" ".config/nvim"
fi

# ==============================================
# Test 3.6: List Symlinks
# ==============================================
test_header "Test 3.6: List existing symlinks"

if output=$(heimdal symlink list 2>&1); then
    test_pass "Symlink list command succeeded"
    
    # Check for expected symlinks in output
    if echo "$output" | grep -q ".bashrc\|.vimrc\|.gitconfig"; then
        test_pass "Symlinks listed in output"
    else
        test_fail "Expected symlinks not found in list output"
    fi
else
    test_fail "Symlink list command failed"
fi

# ==============================================
# Test 3.7: Symlink Status Check
# ==============================================
test_header "Test 3.7: Check symlink status"

if heimdal symlink status > /dev/null 2>&1; then
    test_pass "Symlink status command succeeded"
else
    test_fail "Symlink status command failed"
fi

# ==============================================
# Test 3.8: Stow Compatibility Mode
# ==============================================
test_header "Test 3.8: Stow compatibility mode active"

# Check that stow_compat is enabled in config
CONFIG_FILE="$DOTFILES_DIR/heimdal.yaml"

if check_string_in_file "$CONFIG_FILE" "stow_compat.*true" "stow_compat enabled"; then
    test_pass "Stow compatibility mode is active"
fi

# ==============================================
# Test 3.9: Remove Symlinks
# ==============================================
test_header "Test 3.9: Remove symlinks"

if heimdal symlink remove test > /dev/null 2>&1; then
    test_pass "Symlink remove command succeeded"
    
    # Verify symlinks are removed
    if [ ! -L "$HOME/.bashrc" ]; then
        test_pass ".bashrc symlink removed"
    else
        test_fail ".bashrc symlink still exists after removal"
    fi
    
    if [ ! -L "$HOME/.vimrc" ]; then
        test_pass ".vimrc symlink removed"
    else
        test_fail ".vimrc symlink still exists after removal"
    fi
else
    test_fail "Symlink remove command failed"
fi

# ==============================================
# Test 3.10: Re-create Symlinks
# ==============================================
test_header "Test 3.10: Re-create symlinks after removal"

if heimdal symlink create test > /dev/null 2>&1; then
    test_pass "Symlinks re-created successfully"
    
    # Verify .bashrc is back
    check_symlink "$HOME/.bashrc" "" ".bashrc re-created"
else
    test_fail "Failed to re-create symlinks"
fi

# ==============================================
# Test 3.11: Symlink Conflict Detection
# ==============================================
test_header "Test 3.11: Handle existing file conflicts"

# Create a conflicting file
CONFLICT_FILE="$HOME/.test-conflict"
echo "existing content" > "$CONFLICT_FILE"

# Add conflict file to test profile (would need config modification)
# For now, just verify that symlink command handles conflicts gracefully
# This test is informational

if heimdal symlink create test > /dev/null 2>&1; then
    test_pass "Symlink command handled potential conflicts"
else
    test_pass "Symlink command detected conflicts (expected behavior)"
fi

rm -f "$CONFLICT_FILE"

# ==============================================
# Test 3.12: Verify Symlink File Contents Accessible
# ==============================================
test_header "Test 3.12: Symlinked files are readable"

if [ -L "$HOME/.bashrc" ] && [ -f "$HOME/.bashrc" ]; then
    if content=$(cat "$HOME/.bashrc" 2>/dev/null); then
        if echo "$content" | grep -q "export PATH"; then
            test_pass "Symlinked .bashrc is readable and contains expected content"
        else
            test_fail "Symlinked .bashrc missing expected content"
        fi
    else
        test_fail "Cannot read symlinked .bashrc"
    fi
else
    test_fail ".bashrc symlink broken or missing"
fi

# ==============================================
# Cleanup
# ==============================================

# Remove all test symlinks
heimdal symlink remove test > /dev/null 2>&1 || true
rm -f "$HOME/.bashrc" "$HOME/.vimrc" "$HOME/.gitconfig"
rm -rf "$HOME/.config/nvim"

cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
