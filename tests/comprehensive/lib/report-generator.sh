#!/usr/bin/env bash
# Report generator for comprehensive test results
# Aggregates test results and creates summary reports

set -euo pipefail

RESULT_DIR="${1:-./results}"
REPORT_DIR="${2:-./reports}"
PLATFORM="${PLATFORM:-unknown}"

echo "Generating test report from $RESULT_DIR..."

mkdir -p "$REPORT_DIR"

# Output files
SUMMARY_REPORT="$REPORT_DIR/summary-${PLATFORM}.md"
JSON_REPORT="$REPORT_DIR/summary-${PLATFORM}.json"

# Initialize counters
total_phases=0
passed_phases=0
failed_phases=0
total_tests=0
total_passed=0
total_failed=0
total_errors=0
total_duration=0

# Find all summary files
summary_files=$(find "$RESULT_DIR" -name "summary-phase*-${PLATFORM}.json" 2>/dev/null | sort || true)

if [ -z "$summary_files" ]; then
    echo "No test results found for platform: $PLATFORM"
    exit 0
fi

# Create markdown report header
cat > "$SUMMARY_REPORT" <<EOF
# Heimdal Comprehensive Test Report

**Platform:** ${PLATFORM}  
**Date:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Summary

EOF

# Process each phase summary
phase_details=""
for summary in $summary_files; do
    total_phases=$((total_phases + 1))
    
    # Parse JSON
    phase=$(grep '"phase"' "$summary" | sed 's/.*: "\(.*\)".*/\1/' | tr -d ',')
    phase_name=$(grep '"phase_name"' "$summary" | sed 's/.*: "\(.*\)".*/\1/')
    tests_run=$(grep '"tests_run"' "$summary" | sed 's/.*: \(.*\),/\1/')
    tests_passed=$(grep '"tests_passed"' "$summary" | sed 's/.*: \(.*\),/\1/')
    tests_failed=$(grep '"tests_failed"' "$summary" | sed 's/.*: \(.*\),/\1/')
    tests_errors=$(grep '"tests_errors"' "$summary" | sed 's/.*: \(.*\),/\1/')
    duration=$(grep '"duration"' "$summary" | sed 's/.*: \(.*\),/\1/')
    
    # Update totals
    total_tests=$((total_tests + tests_run))
    total_passed=$((total_passed + tests_passed))
    total_failed=$((total_failed + tests_failed))
    total_errors=$((total_errors + tests_errors))
    total_duration=$((total_duration + duration))
    
    # Determine phase status
    if [ "$tests_failed" -eq 0 ] && [ "$tests_errors" -eq 0 ]; then
        passed_phases=$((passed_phases + 1))
        status="✅ PASS"
    else
        failed_phases=$((failed_phases + 1))
        status="❌ FAIL"
    fi
    
    # Add to phase details
    phase_details+="| Phase ${phase} | ${phase_name} | ${tests_run} | ${tests_passed} | ${tests_failed} | ${tests_errors} | ${duration}s | ${status} |"$'\n'
done

# Calculate pass rate
if [ "$total_tests" -gt 0 ]; then
    pass_rate=$((total_passed * 100 / total_tests))
else
    pass_rate=0
fi

# Write summary statistics
cat >> "$SUMMARY_REPORT" <<EOF
- **Total Phases:** ${total_phases}
- **Passed Phases:** ${passed_phases}
- **Failed Phases:** ${failed_phases}
- **Total Tests:** ${total_tests}
- **Tests Passed:** ${total_passed}
- **Tests Failed:** ${total_failed}
- **Tests Errors:** ${total_errors}
- **Pass Rate:** ${pass_rate}%
- **Total Duration:** ${total_duration}s

## Phase Results

| Phase | Name | Tests | Passed | Failed | Errors | Duration | Status |
|-------|------|-------|--------|--------|--------|----------|--------|
${phase_details}

EOF

# Add failure details if any
if [ "$failed_phases" -gt 0 ]; then
    cat >> "$SUMMARY_REPORT" <<EOF
## Failure Details

The following phases failed:

EOF
    
    # Find failure reports
    failure_reports=$(find "$RESULT_DIR" -name "failures-*-${PLATFORM}.json" 2>/dev/null || true)
    
    if [ -n "$failure_reports" ]; then
        for report in $failure_reports; do
            phase=$(grep '"phase"' "$report" | sed 's/.*: "\(.*\)".*/\1/' | tr -d ',')
            phase_name=$(grep '"phase_name"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
            test_name=$(grep '"test_name"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
            error_message=$(grep '"error_message"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
            
            cat >> "$SUMMARY_REPORT" <<EOF
### Phase ${phase}: ${phase_name}

**Test:** ${test_name}

**Error:**
\`\`\`
${error_message}
\`\`\`

EOF
        done
    fi
fi

# Add conclusion
if [ "$failed_phases" -eq 0 ]; then
    cat >> "$SUMMARY_REPORT" <<EOF
## Conclusion

✅ **All tests passed successfully!**

All ${total_phases} test phases completed successfully with ${total_passed} tests passing.

EOF
else
    cat >> "$SUMMARY_REPORT" <<EOF
## Conclusion

❌ **Some tests failed.**

${failed_phases} out of ${total_phases} phases failed. Please review the failure details above and the test logs for more information.

EOF
fi

# Create JSON report
cat > "$JSON_REPORT" <<EOF
{
  "platform": "${PLATFORM}",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "summary": {
    "total_phases": ${total_phases},
    "passed_phases": ${passed_phases},
    "failed_phases": ${failed_phases},
    "total_tests": ${total_tests},
    "total_passed": ${total_passed},
    "total_failed": ${total_failed},
    "total_errors": ${total_errors},
    "pass_rate": ${pass_rate},
    "total_duration": ${total_duration}
  }
}
EOF

echo "Report generated:"
echo "  - Markdown: $SUMMARY_REPORT"
echo "  - JSON: $JSON_REPORT"

# Display summary
echo ""
echo "=========================================="
echo "  Test Report Summary - ${PLATFORM}"
echo "=========================================="
echo "Phases:   ${passed_phases}/${total_phases} passed"
echo "Tests:    ${total_passed}/${total_tests} passed (${pass_rate}%)"
echo "Duration: ${total_duration}s"
echo ""

if [ "$failed_phases" -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ ${failed_phases} phase(s) failed"
    exit 1
fi
