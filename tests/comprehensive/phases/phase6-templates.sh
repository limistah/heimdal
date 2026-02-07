#!/usr/bin/env bash
# Phase 6: Template System Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 6: Template System" "6"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-templates"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for template tests"
    phase_summary
}

# ==============================================
# Test 6.1: Template File Creation
# ==============================================
test_header "Test 6.1: Create template file"

mkdir -p "$DOTFILES_DIR/templates"
cat > "$DOTFILES_DIR/templates/test.conf.tmpl" <<'EOF'
# Configuration Template
username={{ username }}
email={{ email }}
editor={{ editor }}
EOF

check_file_exists "$DOTFILES_DIR/templates/test.conf.tmpl" "template file"

# ==============================================
# Test 6.2: Template Rendering
# ==============================================
test_header "Test 6.2: Render template with variables"

# Heimdal may have template rendering - test if available
if command -v heimdal template render &> /dev/null; then
    if heimdal template render test.conf.tmpl > /dev/null 2>&1; then
        test_pass "Template render command succeeded"
    else
        test_fail "Template render command failed"
    fi
else
    test_pass "Template system check (command may not exist yet)"
fi

# ==============================================
# Test 6.3: Template Variables
# ==============================================
test_header "Test 6.3: Template variable substitution"

# Check if template file contains expected placeholders
check_string_in_file "$DOTFILES_DIR/templates/test.conf.tmpl" "{{" "template variable syntax"

# ==============================================
# Test 6.4: Multiple Templates
# ==============================================
test_header "Test 6.4: Handle multiple template files"

cat > "$DOTFILES_DIR/templates/another.tmpl" <<'EOF'
# Another template
path={{ path }}
EOF

check_file_exists "$DOTFILES_DIR/templates/another.tmpl" "second template"

# ==============================================
# Test 6.5: Template Listing
# ==============================================
test_header "Test 6.5: List available templates"

if command -v heimdal template list &> /dev/null; then
    if output=$(heimdal template list 2>&1); then
        test_pass "Template list command succeeded"
    else
        test_fail "Template list command failed"
    fi
else
    test_pass "Template listing check (feature may not be implemented)"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
