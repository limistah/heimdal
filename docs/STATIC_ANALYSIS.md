# Static Analysis

This document describes the static analysis tools used in the Heimdal project.

## Automated Tools

### 1. Clippy (Rust Linter)
**Runs on**: Every push and PR  
**Purpose**: Identifies code quality issues, potential bugs, and style violations

```bash
# Run locally
cargo clippy --all-targets --all-features -- -D warnings
```

### 2. cargo-audit (Security Vulnerability Scanner)
**Runs on**: Every push, PR, and weekly  
**Purpose**: Checks dependencies for known security vulnerabilities

```bash
# Run locally
cargo install cargo-audit
cargo audit
```

### 3. cargo-deny (Dependency Validator)
**Runs on**: Every push and PR  
**Purpose**: Validates licenses, checks for banned crates, detects duplicate dependencies

```bash
# Run locally
cargo install cargo-deny
cargo deny check
```

Configuration: `deny.toml`

### 4. cargo-outdated (Dependency Update Checker)
**Runs on**: Weekly  
**Purpose**: Identifies outdated dependencies

```bash
# Run locally
cargo install cargo-outdated
cargo outdated
```

### 5. cargo-udeps (Unused Dependency Checker)
**Runs on**: Every push and PR  
**Purpose**: Detects dependencies that are declared but not used

```bash
# Run locally (requires nightly)
cargo install cargo-udeps
cargo +nightly udeps --all-targets
```

### 6. cargo-semver-checks (API Compatibility)
**Runs on**: Pull requests  
**Purpose**: Ensures semantic versioning compliance for API changes

```bash
# Run locally
cargo install cargo-semver-checks
cargo semver-checks check-release
```

### 7. cargo-tarpaulin (Code Coverage)
**Runs on**: Every push and PR  
**Purpose**: Measures test coverage

```bash
# Run locally
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

Results uploaded to Codecov.

### 8. cargo-bloat (Binary Size Analysis)
**Runs on**: Every push and PR  
**Purpose**: Identifies which dependencies contribute most to binary size

```bash
# Run locally
cargo install cargo-bloat
cargo bloat --release --crates
```

## Running All Checks Locally

```bash
# Install all tools
cargo install cargo-audit cargo-deny cargo-outdated cargo-udeps cargo-semver-checks cargo-tarpaulin cargo-bloat

# Run complete check suite
./scripts/static-analysis.sh  # (to be created)
```

## CI/CD Integration

The `.github/workflows/static-analysis.yml` workflow runs all these tools automatically on:

- **Push** to main/master/develop/dev branches
- **Pull requests** to main/master/dev branches  
- **Weekly schedule** (Mondays at 00:00 UTC)

## Results and Artifacts

Analysis results are uploaded as GitHub Actions artifacts:

- `clippy-results`: JSON output from Clippy
- `audit-results`: JSON security audit report
- `outdated-results`: JSON outdated dependencies report
- `coverage-results`: HTML and XML coverage reports
- `bloat-analysis`: Binary size analysis

## Thresholds and Policies

### Code Coverage
- **Target**: 70% overall coverage
- **Patch coverage**: 80% for new code
- **Threshold**: ±5% change allowed

### Security
- **Vulnerabilities**: Zero tolerance (deny)
- **Unmaintained crates**: Warning
- **Yanked crates**: Warning

### Dependencies
- **Duplicate versions**: Warning
- **Wildcards**: Denied
- **Unknown sources**: Warning

### Licenses
**Allowed**:
- MIT
- Apache-2.0
- BSD-2-Clause / BSD-3-Clause
- ISC
- Unicode-DFS-2016
- Unlicense

**Copyleft**: Warning (requires review)

## Badge Status

Add these badges to README.md:

```markdown
[![Clippy](https://github.com/limistah/heimdal/actions/workflows/static-analysis.yml/badge.svg)](https://github.com/limistah/heimdal/actions/workflows/static-analysis.yml)
[![codecov](https://codecov.io/gh/limistah/heimdal/branch/main/graph/badge.svg)](https://codecov.io/gh/limistah/heimdal)
[![Security Audit](https://github.com/limistah/heimdal/actions/workflows/static-analysis.yml/badge.svg)](https://github.com/limistah/heimdal/security)
```

## Pre-commit Hooks (Optional)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Run quick checks before commit
cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test || exit 1
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting

### False Positives

If a tool reports a false positive, you can:

1. **Clippy**: Add `#[allow(clippy::lint_name)]` annotation
2. **cargo-deny**: Add to `ignore` list in `deny.toml`
3. **cargo-audit**: Add to `ignore` list in `deny.toml`

### Performance

If analysis is slow:

1. Use `--release` flag for faster builds
2. Enable incremental compilation
3. Use `cargo-cache` to clean old build artifacts

## Contributing

When submitting PRs:

1. Ensure all static analysis checks pass
2. Fix all clippy warnings
3. Add tests for new functionality (maintain coverage)
4. Update dependencies if needed
5. Run `cargo deny check` locally

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.
