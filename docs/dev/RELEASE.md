# Release Process

> **Status:** This document is being developed as part of the documentation overhaul (Week 3).

This guide documents the process for creating and publishing new Heimdal releases.

## Table of Contents

1. [Versioning](#versioning)
2. [Release Checklist](#release-checklist)
3. [Creating a Release](#creating-a-release)
4. [Publishing](#publishing)
5. [Post-Release Tasks](#post-release-tasks)

## Versioning

Heimdal follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version (1.x.x) - Incompatible API changes
- **MINOR** version (x.1.x) - New features, backwards compatible
- **PATCH** version (x.x.1) - Bug fixes, backwards compatible

### Version Naming Convention

```
v1.4.0 - Major release with new features
v1.4.1 - Bug fix release
v2.0.0 - Breaking changes
```

## Release Checklist

### Pre-Release (1-2 days before)

- [ ] All tests passing on CI
- [ ] No critical bugs in issue tracker
- [ ] All planned features merged
- [ ] Documentation updated
  - [ ] README.md reflects new features
  - [ ] CHANGELOG.md has entry for this version
  - [ ] Wiki updated with new commands/features
- [ ] Version numbers updated
  - [ ] `Cargo.toml` version
  - [ ] CHANGELOG.md version header
  - [ ] README.md version references
- [ ] Run full test suite locally
- [ ] Test on all platforms (macOS, Linux)
- [ ] Package database is up to date

### Release Day

- [ ] Create release branch
- [ ] Final testing
- [ ] Tag the release
- [ ] Build and publish to crates.io
- [ ] Create GitHub Release with notes
- [ ] Update Homebrew formula
- [ ] Update package repositories (APT, etc.)
- [ ] Announce release

### Post-Release

- [ ] Merge release branch back to main
- [ ] Update documentation links
- [ ] Close milestone in GitHub
- [ ] Announce in discussions
- [ ] Monitor for bug reports

## Creating a Release

### Step 1: Prepare Release Branch

```bash
# Create release branch from dev
git checkout dev
git pull origin dev
git checkout -b release/v1.4.0

# Update version in Cargo.toml
# Edit the version field:
# version = "1.4.0"

# Update CHANGELOG.md
# Add release notes for v1.4.0

git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v1.4.0"
git push origin release/v1.4.0
```

### Step 2: Run Final Tests

```bash
# Run full test suite
cargo test --all-targets --all-features

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build for all targets
cargo build --release

# Test installation from source
cargo install --path .
heimdal --version
```

### Step 3: Create Git Tag

```bash
# Create annotated tag
git tag -a v1.4.0 -m "Release v1.4.0

## New Features
- Smart package management with fuzzy search
- 15 curated package groups
- Outdated package detection

## Bug Fixes
- Fixed state locking on Windows
- Improved error messages

## Documentation
- Added comprehensive wiki
- Updated examples
"

# Push tag
git push origin v1.4.0
```

### Step 4: Merge to Main

```bash
# Create PR from release/v1.4.0 to main
gh pr create --base main --head release/v1.4.0 \
  --title "Release v1.4.0" \
  --body "Release PR for v1.4.0"

# After review, merge PR
# Then merge main back to dev
git checkout dev
git pull origin dev
git merge main
git push origin dev
```

## Publishing

### Publish to Crates.io

```bash
# Login to crates.io (one-time)
cargo login

# Dry-run to check for issues
cargo publish --dry-run

# Publish (cannot be undone!)
cargo publish

# Verify on crates.io
open https://crates.io/crates/heimdal
```

### Create GitHub Release

1. Go to: https://github.com/limistah/heimdal/releases/new
2. Select tag: `v1.4.0`
3. Release title: `v1.4.0`
4. Description: Copy from CHANGELOG.md
5. Attach binaries (optional):
   - `heimdal-v1.4.0-x86_64-apple-darwin.tar.gz` (macOS Intel)
   - `heimdal-v1.4.0-aarch64-apple-darwin.tar.gz` (macOS Apple Silicon)
   - `heimdal-v1.4.0-x86_64-unknown-linux-gnu.tar.gz` (Linux)
6. Check "Create a discussion for this release"
7. Click "Publish release"

Or use CLI:

```bash
gh release create v1.4.0 \
  --title "v1.4.0" \
  --notes-file CHANGELOG.md \
  --discussion-category "Releases"
```

### Update Homebrew Formula

```bash
# Fork homebrew-tap if needed
gh repo fork limistah/homebrew-tap

# Clone your fork
git clone https://github.com/<your-username>/homebrew-tap.git
cd homebrew-tap

# Update Formula/heimdal.rb
# - Update version
# - Update sha256 (from GitHub release)
# - Update dependencies if changed

# Test formula
brew install --build-from-source Formula/heimdal.rb
brew test heimdal
brew audit --strict heimdal

# Commit and PR
git add Formula/heimdal.rb
git commit -m "heimdal: update to v1.4.0"
git push origin main

gh pr create --repo limistah/homebrew-tap \
  --title "heimdal: update to v1.4.0" \
  --body "Updates Heimdal to v1.4.0"
```

### Update APT Repository

```bash
# Build .deb package
cargo deb

# Upload to package repository
# (Specific steps depend on hosting setup)

# Update repository metadata
# Test installation
sudo apt update
sudo apt install heimdal
```

## Post-Release Tasks

### Update Documentation

```bash
# Update README badges (if applicable)
# Update wiki with release notes
# Link to new release from homepage
```

### Announce Release

Post announcement in:
- GitHub Discussions
- Twitter/Social media (if applicable)
- Discord/Chat (if applicable)
- Reddit r/rust (for major releases)

### Monitor Issues

- Watch for bug reports related to new release
- Respond quickly to critical issues
- Consider hotfix release if needed

### Plan Next Release

- Create new milestone for next version
- Move incomplete issues to next milestone
- Update roadmap

## Hotfix Releases

For critical bugs in production:

```bash
# Create hotfix branch from main
git checkout main
git pull origin main
git checkout -b hotfix/v1.4.1

# Fix the bug
# Update version to v1.4.1
# Update CHANGELOG.md

# Test thoroughly
cargo test

# Merge to main
git checkout main
git merge hotfix/v1.4.1
git tag v1.4.1
git push origin main --tags

# Merge to dev
git checkout dev
git merge hotfix/v1.4.1
git push origin dev

# Publish immediately
cargo publish
gh release create v1.4.1 --notes "Hotfix: ..."
```

## Release Automation

### GitHub Actions Workflow

Consider automating with:

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
```

## Troubleshooting

### Common Issues

**"Crate already exists"**
- You cannot unpublish from crates.io
- Use `cargo yank` to mark version as unavailable
- Bump version and re-publish

**"Tag already exists"**
- Delete tag: `git tag -d v1.4.0`
- Delete remote tag: `git push --delete origin v1.4.0`
- Create new tag

**"Tests failing on CI"**
- Do not release with failing tests
- Fix tests first, then release

**"Homebrew formula fails"**
- Test locally with `brew install --build-from-source`
- Check sha256 hash matches GitHub release
- Verify all dependencies are available

## Release Schedule

### Regular Releases
- **Major releases** - Every 6-12 months
- **Minor releases** - Every 4-6 weeks
- **Patch releases** - As needed for critical bugs

### Version Support
- **Latest major** - Full support
- **Previous major** - Security fixes only
- **Older versions** - No support

## Changelog Format

Use this template for CHANGELOG.md entries:

```markdown
## [1.4.0] - 2024-02-07

### Added
- New fuzzy search for packages
- Package groups (15 curated groups)
- Outdated package detection

### Changed
- Improved error messages for config validation
- Updated dependency versions

### Fixed
- State locking on Windows
- Template rendering edge cases

### Deprecated
- `heimdal old-command` (use `heimdal new-command`)

### Removed
- Support for legacy config format

### Security
- Updated dependencies with security vulnerabilities
```

---

**Related Documentation:**
- [Contributing Guide](CONTRIBUTING.md)
- [Testing Guide](TESTING.md)
- [Architecture Overview](../ARCHITECTURE.md)

**Quick Release Commands:**
```bash
# Prepare release
git checkout -b release/v1.x.x
# Update Cargo.toml and CHANGELOG.md
git commit -am "chore: prepare release v1.x.x"

# Tag and publish
git tag v1.x.x
cargo publish
gh release create v1.x.x
```
