# Heimdal Release Strategy

## Overview

This document outlines the complete release management strategy for Heimdal, including versioning, distribution channels, and automation.

## Distribution Channels

### 1. GitHub Releases (Primary)
- **Platform**: All (Linux, macOS, Windows)
- **Format**: Pre-compiled binaries
- **Automation**: GitHub Actions
- **Artifacts**:
  - `heimdal-linux-amd64.tar.gz` (Linux x86_64 GNU)
  - `heimdal-linux-amd64-musl.tar.gz` (Linux x86_64 MUSL)
  - `heimdal-darwin-amd64.tar.gz` (macOS Intel)
  - `heimdal-darwin-arm64.tar.gz` (macOS Apple Silicon)

### 2. Crates.io
- **Platform**: All (source)
- **Installation**: `cargo install heimdal`
- **Automation**: GitHub Actions on tag push
- **Requirements**: `CARGO_TOKEN` secret in GitHub

### 3. Homebrew (macOS/Linux)
- **Platform**: macOS, Linux
- **Installation**: `brew install limistah/tap/heimdal`
- **Repository**: `https://github.com/limistah/homebrew-tap`
- **Update**: Manual (update formula after release)

### 4. AUR (Arch Linux)
- **Platform**: Arch Linux, Manjaro
- **Installation**: `yay -S heimdal` or `paru -S heimdal`
- **Repository**: `https://aur.archlinux.org/heimdal.git`
- **Update**: Manual (update PKGBUILD after release)

### 5. Direct Download (install.sh)
- **Platform**: All
- **Installation**: `curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/main/install.sh | bash`
- **Method**: Clones repo and builds from source
- **Requirements**: Rust toolchain

## Release Workflow

### Automated (via GitHub Actions)

```
Push Tag (v1.0.0)
    ↓
CI Tests Run
    ↓
Build Binaries (4 platforms)
    ↓
Create GitHub Release
    ↓
Upload Binaries
    ↓
Publish to Crates.io
```

### Manual Steps

1. **Homebrew Tap Update**
   - Calculate SHA256 of release tarball
   - Update `Formula/heimdal.rb`
   - Push to homebrew-tap repo

2. **AUR Package Update**
   - Update `PKGBUILD` version and sha256
   - Generate new `.SRCINFO`
   - Push to AUR

3. **Announcement**
   - GitHub Discussions
   - Social media (optional)
   - Reddit, Hacker News (optional)

## Version Numbers

### Format: MAJOR.MINOR.PATCH

- **MAJOR**: Breaking changes (e.g., CLI command changes, config format changes)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Examples:
- `1.0.0` → `1.0.1`: Bug fix
- `1.0.0` → `1.1.0`: New package manager support
- `1.0.0` → `2.0.0`: Config format change

## Release Cadence

### Recommended Schedule:
- **Patch releases**: As needed (critical bugs)
- **Minor releases**: Every 1-2 months
- **Major releases**: Every 6-12 months

## Pre-Release Process

1. **Code Freeze**
   - Stop accepting new features
   - Focus on bug fixes and documentation

2. **Testing Phase**
   - Test on all supported platforms
   - Run full integration tests
   - Test installation methods

3. **Documentation Review**
   - Update README
   - Update CHANGELOG
   - Check example configurations

4. **Version Bump**
   - Update `Cargo.toml`
   - Update `CHANGELOG.md`
   - Create release branch

## Post-Release Process

1. **Monitoring** (First 48 hours)
   - Watch for critical bugs
   - Monitor GitHub issues
   - Check installation reports

2. **Distribution Updates** (Within 1 week)
   - Update Homebrew formula
   - Update AUR package
   - Verify all download links work

3. **Announcement** (Within 24 hours)
   - Post to GitHub Discussions
   - Optional: Social media, forums

4. **Feedback Collection**
   - Monitor user feedback
   - Track feature requests
   - Plan next release

## Hotfix Process

For critical production bugs:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/v1.0.1 v1.0.0

# Fix bug, update version, update CHANGELOG
# ...

# Merge and tag
git checkout main
git merge hotfix/v1.0.1
git tag v1.0.1
git push origin main v1.0.1
```

## Beta/RC Releases

For major changes, consider release candidates:

```bash
# Tag as release candidate
git tag v2.0.0-rc.1

# Users can test with:
cargo install heimdal --version 2.0.0-rc.1

# After testing, release final:
git tag v2.0.0
```

## GitHub Secrets Setup

Required secrets in GitHub repository settings:

1. **CARGO_TOKEN**
   - Get from: https://crates.io/settings/tokens
   - Name: "GitHub Actions - heimdal"
   - Scope: "publish-update"
   - Add to: Settings → Secrets → Actions

2. **GITHUB_TOKEN**
   - Automatically provided
   - No setup needed

## Troubleshooting Releases

### Build Fails on Specific Platform
- Check GitHub Actions logs
- Test locally with same target
- May need platform-specific fixes

### Crates.io Publish Fails
- Verify CARGO_TOKEN is valid
- Check crate name isn't taken
- Ensure all metadata is correct in Cargo.toml

### Homebrew Formula Fails
- Test locally: `brew install --build-from-source Formula/heimdal.rb`
- Common issues: wrong SHA256, missing dependencies
- Check formula syntax

### Download Links Broken
- Verify GitHub release created successfully
- Check binary names match expectations
- Test download manually

## Metrics to Track

1. **Downloads**
   - GitHub release downloads
   - Crates.io downloads
   - Homebrew installs (via analytics)

2. **Issues**
   - Bug reports per release
   - Time to fix critical bugs
   - User satisfaction

3. **Adoption**
   - Stars on GitHub
   - Forks
   - Contributors

## Documentation Checklist

Before each release, ensure:

- [ ] README.md is up to date
- [ ] CHANGELOG.md includes all changes
- [ ] Examples work with new version
- [ ] API documentation is current
- [ ] Installation instructions are tested
- [ ] Migration guide (for breaking changes)

## Communication Channels

### Where to Announce:
1. **Primary**
   - GitHub Releases (automatic)
   - GitHub Discussions

2. **Secondary** (optional)
   - Twitter/X
   - Reddit (r/rust, r/commandline, r/unixporn)
   - Hacker News
   - Lobsters

3. **Community**
   - Discord/Slack (if created)
   - Mailing list (if created)

## Long-Term Goals

### Future Distribution Channels:
- [ ] Debian/Ubuntu PPA
- [ ] Fedora COPR
- [ ] Snap package
- [ ] Flatpak
- [ ] Windows (Scoop, Chocolatey)
- [ ] Docker image

### Automation Improvements:
- [ ] Automated Homebrew formula updates
- [ ] Automated AUR updates
- [ ] Automated changelog generation
- [ ] Binary verification/signing

## Resources

- **GitHub Actions Docs**: https://docs.github.com/en/actions
- **Cargo Publishing**: https://doc.rust-lang.org/cargo/reference/publishing.html
- **Homebrew Formula**: https://docs.brew.sh/Formula-Cookbook
- **AUR Guidelines**: https://wiki.archlinux.org/title/AUR_submission_guidelines
- **Semantic Versioning**: https://semver.org/

## Questions?

For questions about releases, open a GitHub Discussion or contact the maintainers.
