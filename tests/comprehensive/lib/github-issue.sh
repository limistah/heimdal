#!/usr/bin/env bash
# GitHub issue creator for test failures
# Automatically creates issues from test failure reports

set -euo pipefail

RESULT_DIR="${1:-./results}"
REPO="${GITHUB_REPOSITORY:-limistah/heimdal}"
WORKFLOW_RUN="${GITHUB_RUN_ID:-unknown}"
WORKFLOW_URL="${GITHUB_SERVER_URL:-https://github.com}/${REPO}/actions/runs/${WORKFLOW_RUN}"

echo "Scanning for test failures in $RESULT_DIR..."

# Find all failure reports
failure_reports=$(find "$RESULT_DIR" -name "failures-*.json" 2>/dev/null || true)

if [ -z "$failure_reports" ]; then
    echo "No test failures found. All tests passed!"
    exit 0
fi

echo "Found failure reports. Creating GitHub issues..."

# Process each failure report
for report in $failure_reports; do
    echo "Processing failure report: $report"
    
    # Parse JSON (using basic grep/sed since jq might not be available)
    platform=$(grep '"platform"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    phase=$(grep '"phase"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    phase_name=$(grep '"phase_name"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    test_name=$(grep '"test_name"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    error_message=$(grep '"error_message"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    timestamp=$(grep '"timestamp"' "$report" | sed 's/.*: "\(.*\)".*/\1/')
    tests_run=$(grep '"tests_run"' "$report" | sed 's/.*: \(.*\),/\1/')
    tests_passed=$(grep '"tests_passed"' "$report" | sed 's/.*: \(.*\),/\1/')
    tests_failed=$(grep '"tests_failed"' "$report" | sed 's/.*: \(.*\),/\1/')
    
    # Create issue title
    issue_title="[Automated Test] Phase ${phase} failed on ${platform}"
    
    # Create issue body
    issue_body=$(cat <<EOF
## Test Failure Report

**Phase:** ${phase_name}  
**Platform:** ${platform}  
**Timestamp:** ${timestamp}

### Failure Summary

- **Tests Run:** ${tests_run}
- **Tests Passed:** ${tests_passed}
- **Tests Failed:** ${tests_failed}

### Failed Test

**Test Name:** ${test_name}

**Error Message:**
\`\`\`
${error_message}
\`\`\`

### Environment

- **Platform:** ${platform}
- **Workflow Run:** [View workflow run](${WORKFLOW_URL})
- **Phase:** ${phase} - ${phase_name}

### Reproduction Steps

1. Check out the repository at the failing commit
2. Run the comprehensive test suite for ${platform}:
   \`\`\`bash
   cd tests/comprehensive
   PLATFORM=${platform} ./phases/phase${phase}-*.sh
   \`\`\`
3. Review the test output and logs

### Artifacts

Test logs and detailed reports are available in the [workflow artifacts](${WORKFLOW_URL}).

### Next Steps

- [ ] Review the test failure
- [ ] Identify the root cause
- [ ] Create a fix
- [ ] Verify fix with local testing
- [ ] Re-run comprehensive tests

---

*This issue was automatically created by the comprehensive test suite.*
*Workflow Run: ${WORKFLOW_RUN}*
EOF
)
    
    # Create the GitHub issue
    echo "Creating issue: $issue_title"
    
    if command -v gh &> /dev/null; then
        # Use GitHub CLI if available
        gh issue create \
            --title "$issue_title" \
            --body "$issue_body" \
            --label "bug,automated-test,ci-failure,phase-${phase},platform-${platform}" \
            --repo "$REPO" || {
                echo "Warning: Failed to create issue for $report"
            }
    else
        echo "Warning: GitHub CLI (gh) not available. Cannot create issue."
        echo "Issue would have been created with title: $issue_title"
    fi
    
    # Rate limit: wait 2 seconds between issue creations
    sleep 2
done

echo "GitHub issue creation complete."
