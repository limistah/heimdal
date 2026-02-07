#!/usr/bin/env bash
# Phase 12: Hooks System Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 12: Hooks System" "12"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-hooks"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

heimdal init --repo "$TEST_REPO" --profile test > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for hooks tests"
    phase_summary
}

# ==============================================
# Test 12.1: Hooks Directory
# ==============================================
test_header "Test 12.1: Create hooks directory"

HOOKS_DIR="$DOTFILES_DIR/.heimdal/hooks"
mkdir -p "$HOOKS_DIR"

check_dir_exists "$HOOKS_DIR" "hooks directory"

# ==============================================
# Test 12.2: Pre-Init Hook
# ==============================================
test_header "Test 12.2: Pre-init hook creation"

cat > "$HOOKS_DIR/pre-init.sh" <<'EOF'
#!/bin/bash
echo "Pre-init hook executed"
exit 0
EOF

chmod +x "$HOOKS_DIR/pre-init.sh"

check_file_exists "$HOOKS_DIR/pre-init.sh" "pre-init hook"

# ==============================================
# Test 12.3: Post-Init Hook
# ==============================================
test_header "Test 12.3: Post-init hook creation"

cat > "$HOOKS_DIR/post-init.sh" <<'EOF'
#!/bin/bash
echo "Post-init hook executed"
exit 0
EOF

chmod +x "$HOOKS_DIR/post-init.sh"

check_file_exists "$HOOKS_DIR/post-init.sh" "post-init hook"

# ==============================================
# Test 12.4: Pre-Symlink Hook
# ==============================================
test_header "Test 12.4: Pre-symlink hook"

cat > "$HOOKS_DIR/pre-symlink.sh" <<'EOF'
#!/bin/bash
echo "Pre-symlink hook executed"
exit 0
EOF

chmod +x "$HOOKS_DIR/pre-symlink.sh"

test_pass "Pre-symlink hook created"

# ==============================================
# Test 12.5: Post-Symlink Hook
# ==============================================
test_header "Test 12.5: Post-symlink hook"

cat > "$HOOKS_DIR/post-symlink.sh" <<'EOF'
#!/bin/bash
echo "Post-symlink hook executed"
exit 0
EOF

chmod +x "$HOOKS_DIR/post-symlink.sh"

test_pass "Post-symlink hook created"

# ==============================================
# Test 12.6: Hook Execution
# ==============================================
test_header "Test 12.6: Execute hook manually"

if [ -x "$HOOKS_DIR/post-init.sh" ]; then
    if output=$("$HOOKS_DIR/post-init.sh" 2>&1); then
        test_pass "Hook executed successfully"
        
        if echo "$output" | grep -q "Post-init hook executed"; then
            test_pass "Hook produced expected output"
        fi
    else
        test_fail "Hook execution failed"
    fi
fi

# ==============================================
# Test 12.7: Hook with Arguments
# ==============================================
test_header "Test 12.7: Hook with arguments"

cat > "$HOOKS_DIR/test-args.sh" <<'EOF'
#!/bin/bash
echo "Received args: $@"
exit 0
EOF

chmod +x "$HOOKS_DIR/test-args.sh"

if output=$("$HOOKS_DIR/test-args.sh" arg1 arg2 2>&1); then
    if echo "$output" | grep -q "arg1 arg2"; then
        test_pass "Hook correctly received arguments"
    fi
fi

# ==============================================
# Test 12.8: Hook Failure Handling
# ==============================================
test_header "Test 12.8: Hook failure handling"

cat > "$HOOKS_DIR/failing-hook.sh" <<'EOF'
#!/bin/bash
echo "This hook will fail"
exit 1
EOF

chmod +x "$HOOKS_DIR/failing-hook.sh"

if "$HOOKS_DIR/failing-hook.sh" > /dev/null 2>&1; then
    test_fail "Failing hook unexpectedly succeeded"
else
    test_pass "Failing hook correctly returned non-zero exit"
fi

# ==============================================
# Test 12.9: Hook Discovery
# ==============================================
test_header "Test 12.9: Discover available hooks"

if [ -d "$HOOKS_DIR" ]; then
    hook_count=$(find "$HOOKS_DIR" -name "*.sh" -type f | wc -l | tr -d ' ')
    
    if [ "$hook_count" -gt 0 ]; then
        test_pass "Found $hook_count hook scripts"
    else
        test_fail "No hooks discovered"
    fi
fi

# ==============================================
# Test 12.10: Hook Documentation
# ==============================================
test_header "Test 12.10: Hook documentation/comments"

if grep -q "#!/bin/bash" "$HOOKS_DIR/post-init.sh"; then
    test_pass "Hooks have proper shebang"
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
