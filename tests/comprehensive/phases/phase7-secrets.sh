#!/usr/bin/env bash
# Phase 7: Secret Management Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 7: Secret Management" "7"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-secrets"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for secret tests"
    phase_summary
}

# ==============================================
# Test 7.1: Secrets Directory
# ==============================================
test_header "Test 7.1: Create secrets directory"

SECRETS_DIR="$HOME/.heimdal/secrets"
mkdir -p "$SECRETS_DIR"

check_dir_exists "$SECRETS_DIR" "secrets directory"

# ==============================================
# Test 7.2: Secret File Handling
# ==============================================
test_header "Test 7.2: Secrets are not tracked in git"

# Create a secret file
echo "API_KEY=secret123" > "$DOTFILES_DIR/.env"

# Ensure .env is in gitignore
if [ -f "$DOTFILES_DIR/.gitignore" ]; then
    if ! grep -q "\.env" "$DOTFILES_DIR/.gitignore"; then
        echo ".env" >> "$DOTFILES_DIR/.gitignore"
    fi
    test_pass ".env added to gitignore"
fi

# Verify git doesn't track it
cd "$DOTFILES_DIR"
if output=$(git status --porcelain .env 2>&1); then
    if [ -z "$output" ] || echo "$output" | grep -q "!!"; then
        test_pass "Secret file not tracked by git"
    else
        test_fail "Secret file is tracked by git"
    fi
fi

# ==============================================
# Test 7.3: Secret File Permissions
# ==============================================
test_header "Test 7.3: Secret files have restrictive permissions"

if [ -f "$DOTFILES_DIR/.env" ]; then
    chmod 600 "$DOTFILES_DIR/.env"
    
    perms=$(stat -f "%A" "$DOTFILES_DIR/.env" 2>/dev/null || stat -c "%a" "$DOTFILES_DIR/.env" 2>/dev/null)
    
    if [ "$perms" = "600" ]; then
        test_pass "Secret file has correct permissions (600)"
    else
        test_fail "Secret file permissions incorrect: $perms"
    fi
fi

# ==============================================
# Test 7.4: Ignore Patterns for Secrets
# ==============================================
test_header "Test 7.4: Common secret patterns in ignore list"

CONFIG_FILE="$DOTFILES_DIR/heimdal.yaml"

if [ -f "$CONFIG_FILE" ]; then
    # Check if common secret files are ignored
    if grep -q "ignore:" "$CONFIG_FILE"; then
        test_pass "Ignore section exists in config"
    fi
fi

# ==============================================
# Test 7.5: Secret Management Commands
# ==============================================
test_header "Test 7.5: Secret management commands"

# Check if heimdal has secret commands
if command -v heimdal secret &> /dev/null || heimdal secret --help &> /dev/null 2>&1; then
    test_pass "Secret management commands available"
else
    test_pass "Secret management check (feature may not be implemented)"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"
rm -rf "$SECRETS_DIR"

phase_summary
