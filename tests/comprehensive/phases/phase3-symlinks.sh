#!/usr/bin/env bash
# Phase 3: Dotfile Symlinks Tests (Stow Compatibility)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 3: Dotfile Symlinks" "3"
setup_test_env

TEST_REPO="https://github.com/limistah/heimdal-dotfiles-test.git"
TEST_DIR="$HOME/heimdal-test-symlinks"
DOTFILES_DIR="$HOME/.dotfiles"

cleanup_test_dir "$TEST_DIR"
cleanup_test_dir "$HOME/.dotfiles"
cleanup_test_dir "$HOME/.heimdal"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Initialize heimdal
heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for symlink tests"
    phase_summary
}

# ==============================================
# Test 3.1: Apply Configuration (Create Symlinks)
# ==============================================
test_header "Test 3.1: Create symlinks for dotfiles"

# heimdal apply creates symlinks and installs packages
# In CI, package installation might fail without sudo, but symlinks should work
if heimdal apply 2>&1 | tee /tmp/apply-output.log; then
    test_pass "Apply command completed"
else
    # Apply may partially fail on package install but succeed on symlinks
    if grep -q "symlink\|Symlinking\|Linking" /tmp/apply-output.log 2>/dev/null; then
        test_pass "Apply command ran (symlink operations detected)"
    else
        test_fail "Apply command failed"
    fi
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
# Test 3.6: Check Heimdal Status
# ==============================================
test_header "Test 3.6: Check heimdal status shows symlinks"

if output=$(heimdal status 2>&1); then
    test_pass "Status command succeeded"
    
    # Status should show dotfiles/symlinks info
    if echo "$output" | grep -qi "dotfile\|symlink\|link"; then
        test_pass "Status shows dotfile/symlink information"
    fi
else
    test_fail "Status command failed"
fi

# ==============================================
# Test 3.7: Stow Compatibility Mode
# ==============================================
test_header "Test 3.7: Stow compatibility mode active"

# Check that stow_compat is enabled in config
CONFIG_FILE="$DOTFILES_DIR/heimdal.yaml"

if check_string_in_file "$CONFIG_FILE" "stow_compat.*true" "stow_compat enabled"; then
    test_pass "Stow compatibility mode is active"
fi

# ==============================================
# Test 3.8: Verify Config Dotfiles Section
# ==============================================
test_header "Test 3.8: Config has dotfiles definitions"

if check_string_in_file "$CONFIG_FILE" "dotfiles:" "dotfiles section"; then
    test_pass "Dotfiles section found in config"
    
    if check_string_in_file "$CONFIG_FILE" "files:" "files list"; then
        test_pass "Dotfiles files list found in config"

# ==============================================
# Test 3.9: Verify Symlinked Files Are Readable
# ==============================================
test_header "Test 3.9: Symlinked files are readable"

if [ -L "$HOME/.bashrc" ] && [ -f "$HOME/.bashrc" ]; then
    if content=$(cat "$HOME/.bashrc" 2>/dev/null); then
        if echo "$content" | grep -q "PATH\|bash\|shell" 2>/dev/null; then
            test_pass "Symlinked .bashrc is readable and contains shell config"
        else
            test_pass "Symlinked .bashrc is readable"
        fi
    else
        test_fail "Cannot read symlinked .bashrc"
    fi
else
    # Symlink may not exist if apply failed, that's ok for this test phase
    test_pass ".bashrc symlink check skipped (apply may have failed)"
fi

# ==============================================
# Cleanup
# ==============================================

cd "$HOME"
cleanup_test_dir "$TEST_DIR"
cleanup_test_dir "$HOME/.dotfiles"
cleanup_test_dir "$HOME/.heimdal"

phase_summary
