# Quick Start: Release Heimdal v1.0.0

This is your step-by-step guide to release the first version of Heimdal.

## Prerequisites

- [x] All code is complete
- [x] Documentation is complete
- [x] CI/CD is set up
- [ ] GitHub repository created
- [ ] Code pushed to GitHub

## Step-by-Step Release Process

### 1. Create GitHub Repository (If Not Done)

```bash
# On GitHub: Create new repository "heimdal"
# Then:
cd /Users/aleemisiaka/Library/Work/obsp/heimdal

# Initialize git if not already done
git init
git add .
git commit -m "Initial commit: Heimdal v1.0.0"

# Add remote and push
git remote add origin git@github.com:limistah/heimdal.git
git branch -M main
git push -u origin main
```

### 2. Set Up GitHub Secrets

Go to: `https://github.com/limistah/heimdal/settings/secrets/actions`

**Add CARGO_TOKEN:**
1. Visit https://crates.io/settings/tokens
2. Click "New Token"
3. Name: "GitHub Actions - Heimdal"
4. Scope: "publish-update"
5. Copy the token
6. Add to GitHub Secrets as `CARGO_TOKEN`

### 3. Test CI Pipeline

```bash
# Push a small change to trigger CI
git commit --allow-empty -m "Test CI pipeline"
git push

# Check: https://github.com/limistah/heimdal/actions
# Verify: ✓ CI tests pass on Ubuntu and macOS
```

### 4. Create First Release

```bash
# Ensure main branch is up to date
git checkout main
git pull origin main

# Verify version is 1.0.0 in Cargo.toml
cat Cargo.toml | grep version

# Create and push tag
git tag -a v1.0.0 -m "Release version 1.0.0 - Initial release"
git push origin v1.0.0
```

### 5. Monitor Automated Release

Watch the release workflow:
```bash
# Open in browser:
https://github.com/limistah/heimdal/actions

# The workflow will:
# ✓ Build binaries for 4 platforms
# ✓ Create GitHub release
# ✓ Upload binaries
# ✓ Publish to crates.io
```

Expected duration: ~10-15 minutes

### 6. Verify Release

Check these URLs:

1. **GitHub Release:**
   https://github.com/limistah/heimdal/releases/tag/v1.0.0
   
   Should have 4 binary downloads:
   - heimdal-linux-amd64.tar.gz
   - heimdal-linux-amd64-musl.tar.gz
   - heimdal-darwin-amd64.tar.gz
   - heimdal-darwin-arm64.tar.gz

2. **Crates.io:**
   https://crates.io/crates/heimdal
   
   Should show version 1.0.0

3. **Test Installation:**
   ```bash
   # Test from crates.io
   cargo install heimdal
   heimdal --version  # Should show "heimdal 1.0.0"
   
   # Test from GitHub release
   curl -L https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-darwin-amd64.tar.gz | tar xz
   ./heimdal --version
   ```

### 7. Set Up Homebrew Tap

```bash
# Create new repository on GitHub: "homebrew-tap"
# Then:

cd /tmp
git clone https://github.com/limistah/homebrew-tap.git
cd homebrew-tap

# Create Formula directory
mkdir -p Formula

# Copy formula
cp /Users/aleemisiaka/Library/Work/obsp/heimdal/.github/homebrew/heimdal.rb Formula/

# Calculate SHA256
curl -L https://github.com/limistah/heimdal/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256
# Copy the hash

# Update formula with correct SHA256
vim Formula/heimdal.rb
# Replace "REPLACE_WITH_ACTUAL_SHA256" with the hash

# Commit and push
git add Formula/heimdal.rb
git commit -m "Add heimdal formula v1.0.0"
git push origin main

# Test installation
brew tap limistah/tap
brew install heimdal
heimdal --version
```

### 8. Set Up AUR Package (Optional - If You Use Arch)

```bash
# Get AUR access: Create account at aur.archlinux.org
# Set up SSH keys

# Clone AUR repo
cd /tmp
git clone ssh://aur@aur.archlinux.org/heimdal.git
cd heimdal

# Copy PKGBUILD
cp /Users/aleemisiaka/Library/Work/obsp/heimdal/.github/aur/PKGBUILD .

# Calculate SHA256
curl -L https://github.com/limistah/heimdal/archive/refs/tags/v1.0.0.tar.gz | shasum -a 256

# Update PKGBUILD with correct SHA256
vim PKGBUILD

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Initial release: heimdal 1.0.0"
git push

# Test (on Arch Linux)
yay -S heimdal
```

### 9. Announce Release

Create announcement in GitHub Discussions:

```markdown
Title: Heimdal v1.0.0 Released! 🎉

I'm excited to announce the first release of Heimdal - a universal dotfile 
and system configuration manager built in Rust!

## What is Heimdal?

Heimdal automatically manages your dotfiles, installs packages, and keeps 
your development environment in sync across multiple machines.

## Key Features

- 📦 Universal package management (Homebrew, APT, DNF, Pacman, MAS)
- 🔗 GNU Stow-compatible symlink management
- 🔄 Git-based synchronization
- 🎯 Profile-based configuration
- ⏰ Auto-sync via cron
- ↩️ Rollback support
- 🎨 Rich CLI with colored output

## Installation

### From Homebrew (macOS/Linux)
```bash
brew tap limistah/tap
brew install heimdal
```

### From Crates.io
```bash
cargo install heimdal
```

### From AUR (Arch Linux)
```bash
yay -S heimdal
```

### From GitHub Releases
Download pre-built binaries from the releases page.

## Quick Start

```bash
heimdal init --profile work-laptop --repo git@github.com:you/dotfiles.git
heimdal apply
```

## Documentation

Full documentation available in the README:
https://github.com/limistah/heimdal#readme

## Feedback

Please report bugs or suggest features by opening an issue!

Enjoy! 🚀
```

### 10. Optional Social Media

Post on:
- Twitter/X
- Reddit (r/rust, r/commandline)
- Hacker News
- Dev.to

Example tweet:
```
Just released Heimdal v1.0.0! 🎉

A universal dotfile manager that:
📦 Works with Homebrew, APT, DNF, Pacman
🔗 GNU Stow compatible
🔄 Git-based sync
⚡ Written in Rust

Check it out: https://github.com/limistah/heimdal

#rustlang #dotfiles #cli
```

## Post-Release Checklist

- [ ] GitHub release created successfully
- [ ] All 4 binaries uploaded
- [ ] Published to crates.io
- [ ] Homebrew tap working
- [ ] AUR package published (if applicable)
- [ ] GitHub Discussions announcement posted
- [ ] Social media posted (optional)
- [ ] Documentation links verified
- [ ] Install script tested

## Next Steps

1. **Monitor Issues**
   - Watch for bug reports
   - Respond to questions
   - Fix critical bugs quickly

2. **Plan v1.1.0**
   - Collect feature requests
   - Create project board
   - Plan next sprint

3. **Community Building**
   - Engage with users
   - Accept contributions
   - Build documentation site (optional)

## Troubleshooting

### GitHub Actions Failed
- Check logs in Actions tab
- Common issue: CARGO_TOKEN not set
- Fix and re-push tag: `git tag -f v1.0.0`

### Crates.io Publish Failed
- Verify CARGO_TOKEN is correct
- Check crate name isn't taken
- Manually publish: `cargo publish --token <token>`

### Homebrew Formula Fails
- Test locally first
- Check SHA256 is correct
- Verify all dependencies listed

## Support

If you need help:
- Open an issue: https://github.com/limistah/heimdal/issues
- Start a discussion: https://github.com/limistah/heimdal/discussions

## Congratulations! 🎉

You've successfully released Heimdal v1.0.0!

Your universal dotfile manager is now available to users worldwide.
