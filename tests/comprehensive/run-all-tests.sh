#!/usr/bin/env bash
# Master test runner for Heimdal comprehensive tests
# Executes all 13 test phases and generates summary

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PHASES_DIR="$SCRIPT_DIR/phases"
LOG_DIR="${LOG_DIR:-$SCRIPT_DIR/logs}"
RESULT_DIR="${RESULT_DIR:-$SCRIPT_DIR/results}"
REPORT_DIR="${REPORT_DIR:-$SCRIPT_DIR/reports}"
PLATFORM="${PLATFORM:-$(uname -s | tr '[:upper:]' '[:lower:]')}"

# Create directories
mkdir -p "$LOG_DIR/$PLATFORM" "$RESULT_DIR" "$REPORT_DIR"

# Export variables for test scripts
export LOG_DIR="$LOG_DIR/$PLATFORM"
export RESULT_DIR
export REPORT_DIR
export PLATFORM

echo "=========================================="
echo "  Heimdal Comprehensive Test Suite"
echo "=========================================="
echo "Platform: $PLATFORM"
echo "Date: $(date)"
echo "=========================================="
echo ""

# Track overall results
phases_run=0
phases_passed=0
phases_failed=0
start_time=$(date +%s)

# Run all 13 phases
for phase_num in {1..13}; do
    phase_script=$(find "$PHASES_DIR" -name "phase${phase_num}-*.sh" | head -n 1)
    
    if [ ! -f "$phase_script" ]; then
        echo "ERROR: Phase $phase_num script not found"
        continue
    fi
    
    phase_name=$(basename "$phase_script" .sh)
    phases_run=$((phases_run + 1))
    
    echo ""
    echo "=========================================="
    echo "  Running: $phase_name"
    echo "=========================================="
    
    # Run phase and capture output
    log_file="$LOG_DIR/${phase_name}.log"
    
    if bash "$phase_script" 2>&1 | tee "$log_file"; then
        phases_passed=$((phases_passed + 1))
        echo "✅ Phase $phase_num PASSED"
    else
        phases_failed=$((phases_failed + 1))
        echo "❌ Phase $phase_num FAILED"
    fi
done

# Calculate total duration
end_time=$(date +%s)
total_duration=$((end_time - start_time))

echo ""
echo "=========================================="
echo "  Test Suite Summary"
echo "=========================================="
echo "Phases run:    $phases_run"
echo "Phases passed: $phases_passed"
echo "Phases failed: $phases_failed"
echo "Duration:      ${total_duration}s"
echo "=========================================="
echo ""

# Generate comprehensive report
if [ -f "$SCRIPT_DIR/lib/report-generator.sh" ]; then
    bash "$SCRIPT_DIR/lib/report-generator.sh" "$RESULT_DIR" "$REPORT_DIR"
fi

# Create GitHub issues for failures
if [ "$phases_failed" -gt 0 ] && [ -f "$SCRIPT_DIR/lib/github-issue.sh" ]; then
    echo "Creating GitHub issues for test failures..."
    bash "$SCRIPT_DIR/lib/github-issue.sh" "$RESULT_DIR"
fi

# Exit with appropriate code
if [ "$phases_failed" -gt 0 ]; then
    echo "❌ TEST SUITE FAILED"
    exit 1
else
    echo "✅ TEST SUITE PASSED"
    exit 0
fi
