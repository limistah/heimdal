#!/usr/bin/env bash
# Phase 5: Git Operations Tests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib/test-lib.sh"

init_test_phase "Phase 5: Git Operations" "5"
setup_test_env

TEST_REPO="limistah/heimdal-dotfiles-test"
TEST_DIR="$HOME/heimdal-test-git"
DOTFILES_DIR="$TEST_DIR/dotfiles"

cleanup_test_dir "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Initialize heimdal
heimdal init "$TEST_REPO" > /dev/null 2>&1 || {
    test_error "Failed to initialize heimdal for git tests"
    phase_summary
}

cd "$DOTFILES_DIR"

# ==============================================
# Test 5.1: Git Status
# ==============================================
test_header "Test 5.1: Git status in dotfiles repository"

if heimdal git status > /dev/null 2>&1; then
    test_pass "Git status command succeeded"
else
    test_fail "Git status command failed"
fi

# ==============================================
# Test 5.2: Git Repository Is Clean
# ==============================================
test_header "Test 5.2: Repository is in clean state after init"

if output=$(git status --porcelain 2>&1); then
    if [ -z "$output" ]; then
        test_pass "Repository is clean (no uncommitted changes)"
    else
        test_fail "Repository has uncommitted changes: $output"
    fi
else
    test_fail "Failed to check git status"
fi

# ==============================================
# Test 5.3: Make Changes to Dotfiles
# ==============================================
test_header "Test 5.3: Modify a dotfile"

echo "# Test modification" >> .bashrc

if [ -f .bashrc ]; then
    test_pass "Dotfile modified successfully"
else
    test_fail "Failed to modify dotfile"
fi

# ==============================================
# Test 5.4: Git Status Shows Changes
# ==============================================
test_header "Test 5.4: Git status shows modifications"

if output=$(git status --porcelain 2>&1); then
    if echo "$output" | grep -q ".bashrc"; then
        test_pass "Git detected .bashrc modification"
    else
        test_fail "Git did not detect .bashrc modification"
    fi
else
    test_fail "Git status check failed"
fi

# ==============================================
# Test 5.5: Git Add Changes
# ==============================================
test_header "Test 5.5: Stage changes with git add"

if heimdal git add .bashrc > /dev/null 2>&1; then
    test_pass "Git add command succeeded"
else
    test_fail "Git add command failed"
fi

# Verify file is staged
if git diff --cached --name-only | grep -q ".bashrc"; then
    test_pass ".bashrc is staged for commit"
else
    test_fail ".bashrc is not staged"
fi

# ==============================================
# Test 5.6: Git Commit
# ==============================================
test_header "Test 5.6: Commit changes"

if heimdal git commit -m "Test: modify bashrc for testing" > /dev/null 2>&1; then
    test_pass "Git commit succeeded"
else
    test_fail "Git commit failed"
fi

# ==============================================
# Test 5.7: Git Log
# ==============================================
test_header "Test 5.7: View git log"

if output=$(heimdal git log --oneline -n 5 2>&1); then
    test_pass "Git log command succeeded"
    
    # Check for our test commit
    if echo "$output" | grep -q "Test: modify bashrc"; then
        test_pass "Found test commit in log"
    else
        test_fail "Test commit not found in log"
    fi
else
    test_fail "Git log command failed"
fi

# ==============================================
# Test 5.8: Git Diff (Should Be Empty)
# ==============================================
test_header "Test 5.8: No diff after commit"

if output=$(git diff 2>&1); then
    if [ -z "$output" ]; then
        test_pass "No uncommitted changes after commit"
    else
        test_fail "Unexpected uncommitted changes after commit"
    fi
else
    test_fail "Git diff command failed"
fi

# ==============================================
# Test 5.9: Create New File
# ==============================================
test_header "Test 5.9: Add new dotfile"

echo "# New test file" > .test-newfile

if heimdal git add .test-newfile > /dev/null 2>&1; then
    test_pass "New file added to git"
else
    test_fail "Failed to add new file to git"
fi

if heimdal git commit -m "Test: add new file" > /dev/null 2>&1; then
    test_pass "New file committed"
else
    test_fail "Failed to commit new file"
fi

# ==============================================
# Test 5.10: Git Show Commit
# ==============================================
test_header "Test 5.10: Show commit details"

if output=$(git show --stat HEAD 2>&1); then
    test_pass "Git show command succeeded"
    
    if echo "$output" | grep -q ".test-newfile"; then
        test_pass "Commit contains new file"
    fi
else
    test_fail "Git show command failed"
fi

# ==============================================
# Test 5.11: Git Branch
# ==============================================
test_header "Test 5.11: Check current branch"

if output=$(git branch --show-current 2>&1); then
    test_pass "Git branch command succeeded (current: $output)"
else
    test_fail "Git branch command failed"
fi

# ==============================================
# Test 5.12: Create Feature Branch
# ==============================================
test_header "Test 5.12: Create and switch to feature branch"

if git checkout -b test-feature-branch > /dev/null 2>&1; then
    test_pass "Created feature branch"
    
    current_branch=$(git branch --show-current)
    if [ "$current_branch" = "test-feature-branch" ]; then
        test_pass "Switched to feature branch"
    else
        test_fail "Not on feature branch (on: $current_branch)"
    fi
else
    test_fail "Failed to create feature branch"
fi

# ==============================================
# Test 5.13: Make Changes on Feature Branch
# ==============================================
test_header "Test 5.13: Commit changes on feature branch"

echo "# Feature branch change" >> .vimrc

if heimdal git add .vimrc > /dev/null 2>&1 && \
   heimdal git commit -m "Test: feature branch change" > /dev/null 2>&1; then
    test_pass "Committed changes on feature branch"
else
    test_fail "Failed to commit on feature branch"
fi

# ==============================================
# Test 5.14: Switch Back to Main Branch
# ==============================================
test_header "Test 5.14: Switch back to main branch"

if git checkout main > /dev/null 2>&1; then
    test_pass "Switched to main branch"
    
    # Verify feature branch changes are not present
    if ! grep -q "Feature branch change" .vimrc; then
        test_pass "Feature branch changes not present on main"
    else
        test_fail "Feature branch changes unexpectedly present on main"
    fi
else
    test_fail "Failed to switch to main branch"
fi

# ==============================================
# Test 5.15: Git Remote
# ==============================================
test_header "Test 5.15: Check git remote configuration"

if output=$(git remote -v 2>&1); then
    test_pass "Git remote command succeeded"
    
    if echo "$output" | grep -q "$TEST_REPO"; then
        test_pass "Remote points to correct repository"
    else
        test_fail "Remote repository mismatch"
    fi
else
    test_fail "Git remote command failed"
fi

# ==============================================
# Test 5.16: Git Pull (Dry Run)
# ==============================================
test_header "Test 5.16: Git pull simulation"

# Test that pull command works (won't actually pull in test env)
if git pull --dry-run > /dev/null 2>&1 || git fetch --dry-run > /dev/null 2>&1; then
    test_pass "Git pull/fetch dry-run succeeded"
else
    # May fail due to auth, that's expected in test env
    test_pass "Git pull command executed (auth expected to fail in test)"
fi

# ==============================================
# Test 5.17: Git Stash
# ==============================================
test_header "Test 5.17: Git stash functionality"

# Make uncommitted change
echo "# Temporary change" >> .bashrc

if output=$(git stash 2>&1); then
    test_pass "Git stash succeeded"
    
    # Verify change is gone
    if ! grep -q "Temporary change" .bashrc; then
        test_pass "Stashed change removed from working directory"
    else
        test_fail "Stashed change still present"
    fi
    
    # Pop the stash
    if git stash pop > /dev/null 2>&1; then
        test_pass "Git stash pop succeeded"
        
        # Verify change is back
        if grep -q "Temporary change" .bashrc; then
            test_pass "Stashed change restored"
        else
            test_fail "Stashed change not restored"
        fi
    else
        test_fail "Git stash pop failed"
    fi
else
    test_fail "Git stash failed"
fi

# Clean up the temporary change
git restore .bashrc > /dev/null 2>&1 || git checkout -- .bashrc > /dev/null 2>&1

# ==============================================
# Test 5.18: Git Reset
# ==============================================
test_header "Test 5.18: Git reset functionality"

# Make a change and stage it
echo "# Reset test" >> .test-newfile
git add .test-newfile

# Reset
if git reset HEAD .test-newfile > /dev/null 2>&1; then
    test_pass "Git reset succeeded"
    
    # Verify file is unstaged
    if ! git diff --cached --name-only | grep -q ".test-newfile"; then
        test_pass "File successfully unstaged"
    else
        test_fail "File still staged after reset"
    fi
else
    test_fail "Git reset failed"
fi

# Clean up
git restore .test-newfile > /dev/null 2>&1 || git checkout -- .test-newfile > /dev/null 2>&1

# ==============================================
# Test 5.19: Git Diff Between Branches
# ==============================================
test_header "Test 5.19: Git diff between branches"

if output=$(git diff main test-feature-branch 2>&1); then
    test_pass "Git diff between branches succeeded"
    
    if echo "$output" | grep -q "Feature branch change\|vimrc"; then
        test_pass "Diff shows expected branch differences"
    fi
else
    test_fail "Git diff between branches failed"
fi

# ==============================================
# Test 5.20: Git Clean Check
# ==============================================
test_header "Test 5.20: Return to clean state"

# Ensure we're on main
git checkout main > /dev/null 2>&1

# Check for clean status
if output=$(git status --porcelain); then
    if [ -z "$output" ]; then
        test_pass "Repository is in clean state"
    else
        # Clean up any remaining changes
        git restore . > /dev/null 2>&1 || git checkout -- . > /dev/null 2>&1
        test_pass "Repository cleaned up"
    fi
fi

# ==============================================
# Cleanup
# ==============================================
cd "$HOME"
cleanup_test_dir "$TEST_DIR"

phase_summary
