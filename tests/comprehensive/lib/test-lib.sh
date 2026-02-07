#!/usr/bin/env bash
# Test library for Heimdal comprehensive testing
# Provides common test functions and utilities

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_ERRORS=0

# Test phase information
PHASE_NAME=""
PHASE_NUMBER=""
PHASE_START_TIME=""

# Log file paths
LOG_DIR="${LOG_DIR:-./logs}"
RESULT_DIR="${RESULT_DIR:-./results}"
REPORT_DIR="${REPORT_DIR:-./reports}"

# Initialize test phase
init_test_phase() {
    local phase_name="$1"
    local phase_num="$2"
    
    PHASE_NAME="$phase_name"
    PHASE_NUMBER="$phase_num"
    PHASE_START_TIME=$(date +%s)
    
    # Create log directories
    mkdir -p "$LOG_DIR" "$RESULT_DIR" "$REPORT_DIR"
    
    # Print phase header
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $PHASE_NAME${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo ""
}

# Print test header
test_header() {
    local test_name="$1"
    echo -e "${BLUE}>>> $test_name${NC}"
}

# Print success message
test_pass() {
    local message="$1"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✓ PASS${NC}: $message"
}

# Print failure message
test_fail() {
    local message="$1"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}✗ FAIL${NC}: $message"
}

# Print error message
test_error() {
    local message="$1"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_ERRORS=$((TESTS_ERRORS + 1))
    echo -e "${RED}✗ ERROR${NC}: $message"
}

# Run command and capture output
run_command() {
    local cmd="$*"
    local output
    local exit_code
    
    echo -e "${YELLOW}Running:${NC} $cmd"
    
    if output=$(eval "$cmd" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi
    
    echo "$output"
    return $exit_code
}

# Run command expecting success
run_success() {
    local cmd="$*"
    local output
    local exit_code
    
    if output=$(run_command "$cmd"); then
        test_pass "Command succeeded: $cmd"
        return 0
    else
        exit_code=$?
        test_fail "Command failed (exit $exit_code): $cmd"
        echo "$output"
        return 1
    fi
}

# Run command expecting failure
run_failure() {
    local cmd="$*"
    local expected_pattern="${EXPECTED_ERROR:-}"
    local output
    local exit_code
    
    if output=$(eval "$cmd" 2>&1); then
        exit_code=0
        test_fail "Command should have failed but succeeded: $cmd"
        return 1
    else
        exit_code=$?
        
        if [ -n "$expected_pattern" ]; then
            if echo "$output" | grep -q "$expected_pattern"; then
                test_pass "Command failed as expected with pattern '$expected_pattern': $cmd"
                return 0
            else
                test_fail "Command failed but error message doesn't match pattern '$expected_pattern': $cmd"
                echo "$output"
                return 1
            fi
        else
            test_pass "Command failed as expected (exit $exit_code): $cmd"
            return 0
        fi
    fi
}

# Check if file exists
check_file_exists() {
    local file="$1"
    local description="${2:-$file}"
    
    if [ -f "$file" ]; then
        test_pass "File exists: $description"
        return 0
    else
        test_fail "File does not exist: $description"
        return 1
    fi
}

# Check if directory exists
check_dir_exists() {
    local dir="$1"
    local description="${2:-$dir}"
    
    if [ -d "$dir" ]; then
        test_pass "Directory exists: $description"
        return 0
    else
        test_fail "Directory does not exist: $description"
        return 1
    fi
}

# Check if symlink exists and points to correct target
check_symlink() {
    local link="$1"
    local expected_target="${2:-}"
    local description="${3:-$link}"
    
    if [ ! -L "$link" ]; then
        test_fail "Symlink does not exist: $description"
        return 1
    fi
    
    if [ -n "$expected_target" ]; then
        local actual_target
        actual_target=$(readlink "$link")
        
        if [ "$actual_target" = "$expected_target" ]; then
            test_pass "Symlink correct: $description -> $expected_target"
            return 0
        else
            test_fail "Symlink target mismatch: $description (expected: $expected_target, actual: $actual_target)"
            return 1
        fi
    else
        test_pass "Symlink exists: $description"
        return 0
    fi
}

# Check if string is in file
check_string_in_file() {
    local file="$1"
    local pattern="$2"
    local description="${3:-pattern '$pattern' in $file}"
    
    if [ ! -f "$file" ]; then
        test_fail "File does not exist: $file"
        return 1
    fi
    
    if grep -q "$pattern" "$file"; then
        test_pass "Found $description"
        return 0
    else
        test_fail "Not found $description"
        return 1
    fi
}

# Check if string is NOT in file
check_string_not_in_file() {
    local file="$1"
    local pattern="$2"
    local description="${3:-pattern '$pattern' not in $file}"
    
    if [ ! -f "$file" ]; then
        test_fail "File does not exist: $file"
        return 1
    fi
    
    if grep -q "$pattern" "$file"; then
        test_fail "Unexpectedly found $description"
        return 1
    else
        test_pass "Correctly absent: $description"
        return 0
    fi
}

# Check command output contains pattern
check_output_contains() {
    local cmd="$1"
    local pattern="$2"
    local description="${3:-output contains '$pattern'}"
    local output
    
    if output=$(eval "$cmd" 2>&1); then
        if echo "$output" | grep -q "$pattern"; then
            test_pass "Command output contains pattern: $description"
            return 0
        else
            test_fail "Command output missing pattern: $description"
            echo "Output was: $output"
            return 1
        fi
    else
        test_fail "Command failed: $cmd"
        return 1
    fi
}

# Create failure report for GitHub issue creation
create_failure_report() {
    local platform="${PLATFORM:-unknown}"
    local test_name="$1"
    local error_message="$2"
    local log_file="${3:-}"
    
    local report_file="$RESULT_DIR/failures-phase${PHASE_NUMBER}-${platform}.json"
    
    # Create JSON report
    cat > "$report_file" <<EOF
{
  "platform": "$platform",
  "phase": "$PHASE_NUMBER",
  "phase_name": "$PHASE_NAME",
  "test_name": "$test_name",
  "error_message": "$error_message",
  "log_file": "$log_file",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "tests_run": $TESTS_RUN,
  "tests_passed": $TESTS_PASSED,
  "tests_failed": $TESTS_FAILED,
  "tests_errors": $TESTS_ERRORS
}
EOF
    
    echo "Failure report created: $report_file"
}

# Print phase summary and exit
phase_summary() {
    local phase_end_time
    local phase_duration
    
    phase_end_time=$(date +%s)
    phase_duration=$((phase_end_time - PHASE_START_TIME))
    
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $PHASE_NAME - Summary${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo -e "Tests run:    $TESTS_RUN"
    echo -e "${GREEN}Tests passed: $TESTS_PASSED${NC}"
    
    if [ "$TESTS_FAILED" -gt 0 ]; then
        echo -e "${RED}Tests failed: $TESTS_FAILED${NC}"
    else
        echo -e "Tests failed: $TESTS_FAILED"
    fi
    
    if [ "$TESTS_ERRORS" -gt 0 ]; then
        echo -e "${RED}Tests errors: $TESTS_ERRORS${NC}"
    else
        echo -e "Tests errors: $TESTS_ERRORS"
    fi
    
    echo -e "Duration:     ${phase_duration}s"
    echo ""
    
    # Write summary to JSON
    local platform="${PLATFORM:-unknown}"
    local summary_file="$RESULT_DIR/summary-phase${PHASE_NUMBER}-${platform}.json"
    
    cat > "$summary_file" <<EOF
{
  "platform": "$platform",
  "phase": "$PHASE_NUMBER",
  "phase_name": "$PHASE_NAME",
  "tests_run": $TESTS_RUN,
  "tests_passed": $TESTS_PASSED,
  "tests_failed": $TESTS_FAILED,
  "tests_errors": $TESTS_ERRORS,
  "duration": $phase_duration,
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
    
    # Exit with appropriate code
    if [ "$TESTS_FAILED" -gt 0 ] || [ "$TESTS_ERRORS" -gt 0 ]; then
        echo -e "${RED}Phase FAILED${NC}"
        exit 1
    else
        echo -e "${GREEN}Phase PASSED${NC}"
        exit 0
    fi
}

# Utility: Clean up test directory
cleanup_test_dir() {
    local dir="$1"
    if [ -d "$dir" ]; then
        rm -rf "$dir"
    fi
}

# Utility: Setup test environment
setup_test_env() {
    export HEIMDAL_TEST=1
    export HOME="${HOME:-/root}"
    
    # Configure git for testing
    git config --global user.name "Heimdal Test" || true
    git config --global user.email "test@heimdal.test" || true
    git config --global init.defaultBranch main || true
}

# Export all functions
export -f init_test_phase test_header test_pass test_fail test_error
export -f run_command run_success run_failure
export -f check_file_exists check_dir_exists check_symlink
export -f check_string_in_file check_string_not_in_file check_output_contains
export -f create_failure_report phase_summary
export -f cleanup_test_dir setup_test_env
