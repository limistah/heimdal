# Release Process

This document describes how to release a new version of Heimdal.

## Semantic Versioning

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR** version: Incompatible API changes
- **MINOR** version: New functionality (backward compatible)
- **PATCH** version: Bug fixes (backward compatible)

## Pre-Release Checklist

Before creating a release:

1. **Update Version Numbers**
   ```bash
   # Update Cargo.toml
   vim Cargo.toml  # Change version = "1.0.0"
   ```

2. **Update CHANGELOG.md**
   - Move items from `[Unreleased]` to a new version section
   - Add release date
   - Update comparison links at bottom

3. **Run Tests**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Test Build on Multiple Platforms**
   ```bash
   # On macOS
   cargo build --release
   
   # On Linux (if available)
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

5. **Test Installation Script**
   ```bash
   bash install.sh
   ```

6. **Update Documentation**
   - Ensure README.md is up to date
   - Check all example configurations work
   - Verify links are not broken

## Release Process

### 1. Prepare Release Branch

```bash
# Create release branch
git checkout -b release/v1.0.0

# Make final changes
vim Cargo.toml
vim CHANGELOG.md

# Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "Prepare release v1.0.0"

# Push branch
git push origin release/v1.0.0
```

### 2. Create Pull Request

- Create PR from `release/v1.0.0` to `main`
- Review all changes
- Ensure CI passes
- Merge when ready

### 3. Create Git Tag

```bash
# Pull latest main
git checkout main
git pull origin main

# Create tag
git tag -a v1.0.0 -m "Release version 1.0.0"

# Push tag (this triggers release workflow)
git push origin v1.0.0
```

### 4. GitHub Actions Automation

When you push a tag, GitHub Actions will automatically:
1. Run CI tests
2. Build binaries for all platforms:
   - Linux (x86_64-gnu, x86_64-musl)
   - macOS (x86_64-darwin, aarch64-darwin)
3. Create GitHub release
4. Upload binaries to release
5. Publish to crates.io (if CARGO_TOKEN is set)

### 5. Manual Steps After Automation

#### A. Update Homebrew Tap

```bash
# Clone your tap (create if doesn't exist)
git clone https://github.com/limistah/homebrew-tap.git
cd homebrew-tap

# Create or update formula
mkdir -p Formula
cp ../heimdal/.github/homebrew/heimdal.rb Formula/

# Calculate SHA256
curl -L https://github.com/limistah/heimdal/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256

# Update the formula with correct SHA256
vim Formula/heimdal.rb

# Commit and push
git add Formula/heimdal.rb
git commit -m "Update heimdal to v1.0.0"
git push
```

Users can then install with:
```bash
brew tap limistah/tap
brew install heimdal
```

#### B. Publish to AUR (Arch User Repository)

```bash
# Clone AUR repo (or create new)
git clone ssh://aur@aur.archlinux.org/heimdal.git
cd heimdal

# Update PKGBUILD
cp ../heimdal/.github/aur/PKGBUILD .
vim PKGBUILD  # Update version and sha256

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to v1.0.0"
git push
```

Users can then install with:
```bash
yay -S heimdal
# or
paru -S heimdal
```

#### C. Announce Release

1. **GitHub Discussions**
   - Post announcement in Discussions
   - Highlight key features/changes

2. **Social Media** (optional)
   - Tweet about release
   - Post on Reddit (r/rust, r/commandline)
   - Post on Hacker News

3. **Update Documentation Sites** (if any)
   - Update docs with new features
   - Update installation instructions

### 6. Post-Release

1. **Monitor Issues**
   - Watch for bug reports
   - Respond to user feedback

2. **Plan Next Release**
   - Update project board
   - Plan features for next version

3. **Update Main Branch**
   ```bash
   git checkout main
   git pull origin main
   
   # Create new unreleased section in CHANGELOG
   vim CHANGELOG.md
   
   git add CHANGELOG.md
   git commit -m "Prepare for next development iteration"
   git push origin main
   ```

## Hotfix Process

For critical bugs in production:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/v1.0.1 v1.0.0

# Fix the bug
# ... make changes ...

# Update version
vim Cargo.toml  # v1.0.0 -> v1.0.1
vim CHANGELOG.md

# Commit
git add .
git commit -m "Fix critical bug in package installation"

# Merge to main
git checkout main
git merge hotfix/v1.0.1

# Tag and push
git tag -a v1.0.1 -m "Hotfix: Fix critical bug"
git push origin main
git push origin v1.0.1

# Clean up
git branch -d hotfix/v1.0.1
```

## Version Management

Current version locations:
- `Cargo.toml` - Main version source
- `CHANGELOG.md` - Release notes
- Git tags - Release markers

## Secrets Required

For automated releases, set these secrets in GitHub:
- `CARGO_TOKEN` - Token from crates.io for publishing
- `GITHUB_TOKEN` - Automatically provided by GitHub Actions

To get CARGO_TOKEN:
1. Go to https://crates.io/settings/tokens
2. Create new token
3. Add to GitHub repository secrets

## Troubleshooting

### GitHub Actions Failed

- Check the Actions tab for logs
- Common issues:
  - Forgot to update version in Cargo.toml
  - CARGO_TOKEN not set or expired
  - Build failure on specific platform

### Homebrew Formula Issues

- Test locally: `brew install --build-from-source Formula/heimdal.rb`
- Check SHA256 is correct
- Ensure dependencies are listed

### AUR Package Issues

- Test locally: `makepkg -si`
- Verify .SRCINFO is up to date
- Check package() function installs all files

## Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run tests (`cargo test`)
- [ ] Run linter (`cargo clippy`)
- [ ] Format code (`cargo fmt`)
- [ ] Create release branch
- [ ] Create and merge PR
- [ ] Create and push tag
- [ ] Verify GitHub release created
- [ ] Verify binaries uploaded
- [ ] Update Homebrew tap
- [ ] Update AUR package
- [ ] Announce release
- [ ] Monitor for issues

## Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Homebrew Tap Creation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [AUR Submission Guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
