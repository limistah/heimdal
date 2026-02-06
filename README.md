# Heimdal

[![CI](https://github.com/limistah/heimdal/workflows/CI/badge.svg)](https://github.com/limistah/heimdal/actions)
[![Release](https://img.shields.io/github/v/release/limistah/heimdal)](https://github.com/limistah/heimdal/releases)
[![Crates.io](https://img.shields.io/crates/v/heimdal)](https://crates.io/crates/heimdal)
[![License](https://img.shields.io/github/license/limistah/heimdal)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**A universal dotfile and system configuration manager built in Rust**

Heimdal is a powerful, cross-platform tool that automatically manages your dotfiles, installs packages, and keeps your development environment in sync across multiple machines. Say goodbye to manual configuration and hello to automated, declarative system management.

## 🚀 Quick Start

### New to dotfiles?

```bash
heimdal wizard
```

The interactive wizard will guide you through:
- **Smart scanning** with real-time progress indicators
- **Interactive selection** - Choose exactly which files and packages to track
- **Smart profile names** - Auto-generated based on your hostname and OS  
- **Package detection** with helpful empty state guidance
- **Git sync setup** with remote configuration

**Done in under 2 minutes!**

### Migrating from Stow or dotbot?

```bash
heimdal wizard
# Choose "Import existing dotfiles"
```

Heimdal automatically detects and converts:
- ✅ **GNU Stow** - Maintains Stow compatibility
- ✅ **dotbot** - Converts `install.conf.yaml`
- ✅ **chezmoi** - Parses chezmoi naming conventions
- ✅ **yadm** - Imports yadm tracked files
- ✅ **homesick** - Converts castle structures
- ✅ **Manual setups** - Smart scanning and detection

**Preview before importing:**
```bash
heimdal import --path ~/dotfiles --preview
```

No manual conversion needed!

## Features

### Core Features
- **Universal Package Management** - Install packages across Homebrew, APT, DNF, Pacman, and Mac App Store from a single configuration
- **Intelligent Symlinking** - GNU Stow-compatible symlink management with conflict resolution
- **Template System** - Simple variable substitution for machine-specific configs (user, hostname, email, etc.)
- **Secret Management** - Secure API keys and tokens using OS-native keychains (never stored in git)
- **Git-Based Sync** - Keep your configuration in sync across machines using Git
- **Profile-Based Configuration** - Different configurations for different machines (work, personal, servers)
- **Interactive Wizard** - Smart setup with progress indicators and interactive selection
- **Auto-Sync** - Background synchronization via cron jobs
- **Rollback Support** - Easily revert to previous configurations
- **Hooks System** - Run custom scripts before/after installation
- **Dry-Run Mode** - Preview changes before applying them

### ✨ Recent Improvements (v1.4.0 - Week 10)
- **Smart Package Management**
  - Fuzzy search with typo tolerance and intelligent scoring
  - Installation status detection across all package managers
  - Tag-based filtering for refined searches
  - Automatic package suggestions based on detected project technologies
  - 15 curated package groups for common workflows (web-dev, rust-dev, python-dev, etc.)
  - Package version management with outdated detection and bulk upgrades
  - Rich terminal output with relevance indicators and metadata

### Previous Improvements
- **v1.3.0 - Enhanced Import System**
  - Support for 3 additional dotfile managers: chezmoi, yadm, homesick
  - Intelligent conflict resolution with 4 strategies (Skip, Overwrite, Backup, Ask)
  - Import preview mode to see what would be imported
  - Better file categorization and destination mapping
- **v1.2.0 - Wizard UX & Performance**
  - Real-time progress indicators for scanning operations
  - Interactive file/package selection with multi-select support
  - Smart profile name generation (e.g., "work-mac", "personal-linux")
  - 30-50% faster template rendering with cached regex
  - Eliminated critical `.unwrap()` calls with proper error handling
  - Added logging macros to reduce boilerplate

### 🎯 Smart Package Intelligence (New in v1.1.0)
- **Package Profiles** - Pre-configured package sets for common workflows
  - 10 built-in profiles: Minimal, Developer, Web Dev, Rust Dev, Python Dev, Go Dev, DevOps, Data Science, Designer, Writer
  - Automatic platform-aware package selection
- **Dependency Detection** - Automatically detects and suggests missing dependencies
  - Required dependencies (e.g., neovim needs git)
  - Optional enhancements (e.g., neovim works better with ripgrep, fzf)
  - 50+ package relationships mapped
- **Package Database** - Rich metadata for 60+ popular packages
  - Descriptions, categories, popularity scores
  - Alternative package suggestions
  - Related package recommendations
- **Enhanced Package Mapper** - Intelligent cross-platform package name handling
  - Fuzzy matching for typos (e.g., 'ripgrap' → suggests 'ripgrep')
  - Name normalization (e.g., 'nodejs' → 'node', 'golang' → 'go')
  - 60+ packages mapped across all platforms
  - Automatic translation between platform-specific names

## Supported Package Managers

- **macOS**: Homebrew (formulae + casks), Mac App Store (via `mas`)
- **Debian/Ubuntu**: APT
- **Fedora/RHEL/CentOS**: DNF
- **Arch/Manjaro**: Pacman

## Installation

Choose your preferred installation method based on your operating system and package manager:

### Homebrew (macOS)

```bash
brew tap limistah/tap
brew install heimdal
```

### APT (Debian/Ubuntu)

**From limistah's APT Repository (Recommended):**

```bash
# One-line setup (adds repository only)
curl -fsSL https://limistah.github.io/apt-repo/setup.sh | sudo bash

# Then install Heimdal
sudo apt install heimdal
```

Or manually add the repository:

```bash
# Add repository
echo "deb [trusted=yes] https://limistah.github.io/apt-repo stable main" | sudo tee /etc/apt/sources.list.d/limistah.list

# Update and install Heimdal
sudo apt update
sudo apt install heimdal
```

**Direct DEB Package:**

```bash
# Download and install the latest .deb package
curl -LO https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal_1.0.0_amd64.deb
sudo dpkg -i heimdal_1.0.0_amd64.deb
```

Or use the automated script:

```bash
curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/master/.github/debian/install-deb.sh | bash
```

### DNF/YUM (Fedora/RHEL/CentOS)

```bash
# Download and install the latest RPM package
curl -LO https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-1.0.0-1.x86_64.rpm
sudo rpm -i heimdal-1.0.0-1.x86_64.rpm

# Or using DNF
sudo dnf install https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-1.0.0-1.x86_64.rpm
```

Or use the automated script:

```bash
curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/master/.github/rpm/install-rpm.sh | bash
```

### Pacman (Arch Linux)

Install from AUR:

```bash
# Using yay
yay -S heimdal-bin

# Using paru
paru -S heimdal-bin

# Manual installation
git clone https://aur.archlinux.org/heimdal-bin.git
cd heimdal-bin
makepkg -si
```

### APK (Alpine Linux)

**From limistah's APK Repository (Recommended):**

```bash
# One-line setup (adds repository only)
wget -qO- https://limistah.github.io/apk-repo/setup.sh | sudo sh

# Then install Heimdal
sudo apk add heimdal --allow-untrusted
```

Or manually add the repository:

```bash
# Add repository
echo "https://limistah.github.io/apk-repo/stable" | sudo tee -a /etc/apk/repositories

# Update and install Heimdal
sudo apk update
sudo apk add heimdal --allow-untrusted
```

**Direct APK Package:**

```bash
# Download and install the latest APK package
wget https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-1.0.0-r0.apk
apk add --allow-untrusted heimdal-1.0.0-r0.apk
```

Or use the automated script:

```bash
wget -O - https://raw.githubusercontent.com/limistah/heimdal/master/.github/apk/install-apk.sh | sh
```

### Cargo (All Platforms)

```bash
cargo install heimdal
```

### Universal Install Script

Works on macOS and Linux, automatically detects your platform and package manager:

```bash
curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/master/install.sh | bash
```

### From Source

```bash
git clone https://github.com/limistah/heimdal.git
cd heimdal
cargo build --release
sudo mv target/release/heimdal /usr/local/bin/
```

### Manual Download

Download pre-built binaries from the [releases page](https://github.com/limistah/heimdal/releases/latest):

- **DEB Package** (Debian/Ubuntu): `heimdal_1.0.0_amd64.deb`
- **RPM Package** (Fedora/RHEL/CentOS): `heimdal-1.0.0-1.x86_64.rpm`
- **APK Package** (Alpine): `heimdal-1.0.0-r0.apk`
- **Linux (GNU)**: `heimdal-linux-amd64.tar.gz`
- **Linux (MUSL)**: `heimdal-linux-amd64-musl.tar.gz`
- **macOS (Intel)**: `heimdal-darwin-amd64.tar.gz`
- **macOS (Apple Silicon)**: `heimdal-darwin-arm64.tar.gz`

```bash
# Example for Linux
curl -L https://github.com/limistah/heimdal/releases/download/v1.0.0/heimdal-linux-amd64.tar.gz | tar xz
sudo mv heimdal /usr/local/bin/
```

## Manual Setup (Advanced)

If you prefer manual setup instead of the wizard:

### 1. Create a Dotfiles Repository

```bash
mkdir ~/dotfiles && cd ~/dotfiles && git init
```

### 2. Create `heimdal.yaml`

```yaml
heimdal:
  version: "1.0"
  repo: "git@github.com:yourusername/dotfiles.git"
  stow_compat: true

# Global ignore patterns
ignore:
  - .git
  - .gitignore
  - heimdal.yaml
  - README.md

# Simple package list (auto-mapped across package managers)
sources:
  packages:
    - git
    - vim
    - tmux
    - fzf
    - ripgrep

  # Platform-specific packages with hooks
  homebrew:
    packages:
      - neovim
      - zoxide
    casks:
      - iterm2
      - docker
    hooks:
      post_install:
        - command: "brew cleanup"
          description: "Clean up Homebrew cache"

  apt:
    packages:
      - build-essential
    hooks:
      pre_install:
        - command: "sudo apt-get update"
          description: "Update package lists"

# Machine profiles
profiles:
  base:
    sources:
      - packages
      - homebrew
      - apt
    dotfiles:
      use_stowrc: true
  
  work-laptop:
    extends: base
    sources:
      - name: homebrew
        packages:
          - kubectl
          - docker-compose

# Auto-sync configuration
sync:
  enabled: true
  interval: "1h"
```

### 3. Add Your Dotfiles

```bash
# Add your dotfiles
cp ~/.bashrc .
cp ~/.vimrc .
mkdir -p .config/nvim
cp -r ~/.config/nvim/* .config/nvim/

# Create .stowrc for symlink configuration
cat > .stowrc << EOF
--target=$HOME
--ignore=.git
--ignore=heimdal.yaml
--ignore=.stowrc
EOF

# Commit and push
git add .
git commit -m "Initial commit"
git push origin main
```

### 4. Initialize on a New Machine

```bash
# Clone and initialize
heimdal init --profile work-laptop --repo git@github.com:yourusername/dotfiles.git

# Apply configuration
heimdal apply
```

## Usage

### Setup Wizard (Recommended)

The easiest way to get started:

```bash
heimdal wizard
```

The wizard offers three setup flows:

#### 1. Start Fresh
- Creates a new dotfiles repository
- Choose from 10 pre-configured package profiles or customize your own
- Scans your home directory for existing dotfiles
- Detects installed packages (supports Homebrew, APT, DNF, Pacman, mas)
- Automatically detects missing dependencies and suggests additions
- Generates a complete `heimdal.yaml` configuration
- Optionally sets up Git remote

**Available Package Profiles:**
- **Minimal** - Essential tools only (git, curl, vim, tmux)
- **Developer** - Full development environment (editors, git tools, build tools)
- **Web Dev** - Modern web development (node, yarn, docker, postgres)
- **Rust Dev** - Rust ecosystem (rust, cargo, rust-analyzer, ripgrep, fd, bat)
- **Python Dev** - Python development (python, pip, pipenv, pyenv)
- **Go Dev** - Go development (go, gopls, docker, kubectl)
- **DevOps** - Infrastructure tools (terraform, ansible, docker, kubectl, helm)
- **Data Science** - Data analysis (python, jupyter, pandas, postgresql)
- **Designer** - Design and media tools
- **Writer** - Documentation and writing tools (pandoc, markdown tools)

#### 2. Import Existing Dotfiles
- Automatically detects your setup (GNU Stow, dotbot, chezmoi, or manual)
- Converts configuration to Heimdal format
- Preserves compatibility (e.g., Stow compatibility mode)
- Extracts package information from dotbot shell commands

**Supported Tools:**
- ✅ **GNU Stow** - Detects `.stowrc` or Stow directory structure
- ✅ **dotbot** - Parses `install.conf.yaml`
- ✅ **chezmoi** - Coming soon
- ✅ **Manual** - Smart scanning fallback

#### 3. Clone Existing Heimdal Repo
- Clone your existing Heimdal dotfiles repository
- Select profile to apply
- Initialize on a new machine

**Example: Importing from Stow**

```bash
$ heimdal wizard
? What would you like to do?
  > Import existing dotfiles

? Where are your dotfiles? ~/dotfiles

→ Analyzing directory structure...
✓ Detected: GNU Stow setup

? Convert GNU Stow configuration to Heimdal? Yes

→ Importing from GNU Stow...
✓ Found 12 files

Dotfiles to track:
  1. vim/.vimrc → ~/.vimrc
  2. zsh/.zshrc → ~/.zshrc
  3. tmux/.tmux.conf → ~/.tmux.conf
  ... and 9 more

? Generate heimdal.yaml configuration? Yes
? Profile name: personal

→ Generating configuration...
✓ Saved to ~/dotfiles/heimdal.yaml

✓ Import complete!
```

### Direct Import (Without Wizard)

For quick, non-interactive imports:

```bash
# Auto-detect tool and import
heimdal import --path ~/dotfiles

# Import from specific tool
heimdal import --path ~/dotfiles --from stow
heimdal import --path ~/dotfiles --from dotbot

# Specify output location
heimdal import --path ~/dotfiles --output ~/my-config.yaml
```

The `import` command will:
1. Detect the dotfile management tool (or use specified tool)
2. Parse the existing configuration
3. Generate `heimdal.yaml`
4. Preserve compatibility settings (e.g., Stow compatibility mode)

### Initialize Heimdal

Initialize Heimdal on a new machine by cloning your dotfiles repository:

```bash
heimdal init --profile work-laptop --repo git@github.com:yourusername/dotfiles.git

# Or specify a custom path
heimdal init --profile work-laptop --repo git@github.com:yourusername/dotfiles.git --path ~/my-dotfiles
```

### Apply Configuration

Install packages and create symlinks based on your configuration:

```bash
# Apply with dry-run (preview changes)
heimdal apply --dry-run

# Apply for real
heimdal apply

# Force overwrite existing files
heimdal apply --force
```

### Sync Configuration

Pull latest changes from Git and reapply configuration:

```bash
# Interactive sync
heimdal sync

# Quiet mode (for cron jobs)
heimdal sync --quiet

# Dry-run
heimdal sync --dry-run
```

### Check Status

View current Heimdal state and git status:

```bash
# Basic status
heimdal status

# Verbose (includes git status)
heimdal status --verbose
```

### View Local Changes

Show differences between local dotfiles and repository:

```bash
# Show summary of changes
heimdal diff

# Show detailed changes (line counts)
heimdal diff --verbose

# Interactive mode (commit or discard changes)
heimdal diff --interactive
```

The diff command shows:
- **Modified files** - Files with content changes
- **Added files** - New files staged for commit
- **Deleted files** - Files that were removed
- **Renamed files** - Files that were moved
- **Untracked files** - New files not yet in git

**Interactive mode** offers actions:
- View detailed git diff
- Commit all or specific files
- Discard all or specific changes
- Push to remote after commit

### Manage Packages

Heimdal provides powerful package management commands that work across all package managers:

#### Add Packages

```bash
# Add package with auto-detection
heimdal packages add git

# Specify package manager
heimdal packages add docker --manager homebrew

# Add to specific profile
heimdal packages add python --profile work-laptop

# Add without installing (just update config)
heimdal packages add nodejs --no-install
```

The add command:
- Auto-detects available package managers
- Shows package metadata from database
- Detects and suggests dependencies
- Updates `heimdal.yaml` configuration
- Optionally installs immediately

#### Remove Packages

```bash
# Remove package
heimdal packages remove docker

# Remove from specific profile
heimdal packages remove python --profile work-laptop

# Force removal (skip dependency warnings)
heimdal packages remove nodejs --force

# Remove without uninstalling
heimdal packages remove vim --no-uninstall
```

The remove command:
- Finds package in configuration
- Checks for dependent packages
- Updates `heimdal.yaml` configuration
- Optionally uninstalls from system

#### Search Packages

```bash
# Fuzzy search by name or description
heimdal packages search neovim

# Filter by category
heimdal packages search editor --category editor

# Filter by tag
heimdal packages search docker --tag container
```

The search command features:
- **Fuzzy matching** - Finds packages even with typos
- **Smart scoring** - Prioritizes exact matches over fuzzy matches
- **Installation status** - Shows which packages are already installed (✓/○)
- **Relevance indicators** - ★ highly relevant / ☆ relevant / · fuzzy match
- **Rich metadata** - Displays descriptions, popularity, alternatives

Available categories: development, editor, terminal, language, container, infrastructure, database, network, application

#### Smart Package Suggestions

```bash
# Suggest packages for current directory
heimdal packages suggest

# Suggest for specific directory
heimdal packages suggest --directory ~/projects/my-app
```

The suggest command:
- **Auto-detects technologies** - Scans for `package.json`, `Cargo.toml`, `requirements.txt`, etc.
- **Context-aware recommendations** - Suggests tools based on detected project types
- **Relevance scoring** - Prioritizes essential tools for detected technologies
- **Installation status** - Shows which suggested packages are already installed

Supports 15+ tool patterns: Node.js, Python, Rust, Go, Docker, Kubernetes, and more.

#### Package Groups

```bash
# List all package groups
heimdal packages list-groups

# List groups by category
heimdal packages list-groups --category development

# Show group details
heimdal packages show-group web-dev

# Search for groups
heimdal packages search-groups rust

# Install a package group
heimdal packages add-group web-dev

# Include optional packages
heimdal packages add-group web-dev --include-optional

# Dry-run to preview
heimdal packages add-group rust-dev --dry-run
```

**Available Groups** (15 curated collections):
- `essential` - Core system utilities
- `web-dev` - Web development stack (Node.js, TypeScript, etc.)
- `rust-dev` - Rust development tools
- `python-dev` - Python environment
- `go-dev` - Go development
- `docker` - Container ecosystem
- `editors` - Modern text editors
- `shells` - Shell enhancements
- `terminal-tools` - CLI productivity tools
- `network` - Network debugging
- `monitoring` - System monitoring
- `security` - Security and encryption tools
- `cloud` - Cloud provider CLIs
- `database` - Database management tools
- `media` - Media processing tools

#### Package Version Management

```bash
# Check for outdated packages
heimdal packages outdated

# Check all packages (not just profile)
heimdal packages outdated --all

# Upgrade specific package
heimdal packages upgrade docker

# Upgrade all outdated packages
heimdal packages upgrade --all

# Dry-run upgrade
heimdal packages upgrade --all --dry-run
```

The version management system:
- **Cross-platform** - Works with Homebrew, APT, DNF, Pacman
- **Version comparison** - Shows installed vs. available versions
- **Colored output** - Red for outdated, green for up-to-date
- **Bulk operations** - Upgrade multiple packages at once

#### List Packages

```bash
# List packages in current profile
heimdal packages list

# List packages in specific profile
heimdal packages list --profile work-laptop
```

Available categories: development, editor, terminal, language, container, infrastructure, database, network, application

#### Get Package Info

```bash
# Show detailed package information
heimdal packages info neovim
```

Shows:
- Description and category
- Alternative packages
- Related packages
- Dependencies (required and optional)
- Tags and popularity

#### List Packages

```bash
# List all packages in current profile
heimdal packages list

# Show only installed packages
heimdal packages list --installed

# List packages in specific profile
heimdal packages list --profile work-laptop
```

### Auto-Sync

Enable automatic background synchronization:

```bash
# Enable auto-sync every hour
heimdal auto-sync enable 1h

# Enable auto-sync every 30 minutes
heimdal auto-sync enable 30m

# Enable daily sync
heimdal auto-sync enable daily

# Check auto-sync status
heimdal auto-sync status

# Disable auto-sync
heimdal auto-sync disable
```

### Rollback

Revert to a previous configuration:

```bash
# Rollback to previous commit
heimdal rollback

# Rollback to specific commit
heimdal rollback abc123

# Rollback to tag
heimdal rollback v1.0.0
```

### View History

See your configuration change history:

```bash
# Show last 10 commits
heimdal history

# Show last 20 commits
heimdal history 20
```

### Git Operations

Heimdal provides comprehensive git workflow management for your dotfiles:

#### Commit Changes

```bash
# Commit with auto-generated message
heimdal commit --auto

# Commit with custom message
heimdal commit -m "Add nvim config"

# Commit and push
heimdal commit -m "Update zshrc" --push

# Commit specific files
heimdal commit -m "Update vim config" vim/.vimrc
```

The commit command:
- Auto-stages files before committing
- Generates smart commit messages based on changes
- Optional push to remote after commit
- Shows git status before committing

#### Push and Pull

```bash
# Push to default remote
heimdal push

# Push to specific remote
heimdal push --remote upstream

# Push specific branch
heimdal push --branch main

# Pull from remote
heimdal pull

# Pull with rebase
heimdal pull --rebase
```

#### Branch Management

```bash
# Show current branch
heimdal branch current

# List all branches
heimdal branch list

# Create and switch to new branch
heimdal branch create feature-branch

# Switch to existing branch
heimdal branch switch main

# Show tracking information
heimdal branch info
```

The branch info shows:
- Current branch name
- Upstream tracking branch
- Commits ahead/behind remote
- Sync status

#### Remote Management

```bash
# List remotes
heimdal remote list
heimdal remote list -v  # Show URLs

# Add new remote
heimdal remote add origin git@github.com:user/dotfiles.git

# Remove remote
heimdal remote remove upstream

# Change remote URL
heimdal remote set-url origin https://github.com/user/dotfiles.git

# Show remote details
heimdal remote show origin

# Interactive remote setup (recommended for first-time setup)
heimdal remote setup
```

The interactive remote setup:
- Shows existing remotes if any
- Prompts for remote name (default: origin)
- Prompts for remote URL (SSH or HTTPS)
- Handles replacing existing remotes
- Optionally pushes after adding remote

### Profile Management

Heimdal's powerful profile system allows you to manage different configurations for different machines or use cases (work laptop, personal desktop, server, etc.).

#### Switch Profiles

Switch between different profiles with automatic configuration reapply:

```bash
# Switch to a profile (auto-applies configuration)
heimdal profile switch work

# Switch without auto-applying
heimdal profile switch work --no-apply
```

#### View Profile Information

```bash
# Show currently active profile
heimdal profile current

# Show details about a specific profile
heimdal profile show work

# Show resolved configuration (after inheritance)
heimdal profile show work --resolved

# List all available profiles
heimdal profile list

# List with details
heimdal profile list --verbose
```

#### Compare Profiles

Compare dotfiles and packages between profiles:

```bash
# Compare current profile with another
heimdal profile diff work

# Compare two specific profiles
heimdal profile diff personal work
```

The diff shows:
- **Common items** - Shared between both profiles
- **Only in profile 1** - Unique to first profile
- **Only in profile 2** - Unique to second profile

#### Profile Templates

Create new profiles from built-in templates:

```bash
# List available templates
heimdal profile templates
```

**Available templates:**
- `minimal` - Basic shell config only
- `developer` - Dev tools and editor configs
- `devops` - Infrastructure and deployment tools
- `macos-desktop` - macOS GUI and window management
- `linux-server` - Server configuration
- `workstation` - Comprehensive setup with all features

**Create from template:**

```bash
# Create a new profile from a template
heimdal profile create my-work --template developer

# The new profile will be added to heimdal.yaml
# Edit it to customize packages and dotfiles
```

#### Clone Profiles

Duplicate an existing profile for customization:

```bash
# Clone a profile
heimdal profile clone work work-laptop

# The cloned profile will have the same configuration
# Edit heimdal.yaml to customize it
```

#### Conditional Dotfiles

Apply dotfiles conditionally based on OS, profile, environment, or hostname:

```yaml
profiles:
  work:
    dotfiles:
      files:
        - source: vim/.vimrc
          target: ~/.vimrc
          # This file will only be linked on macOS or Linux
          when:
            os: ["macos", "linux"]
        
        - source: work/.ssh/config
          target: ~/.ssh/config
          # Only link for work profile
          when:
            profile: ["work"]
        
        - source: aws/.aws/config
          target: ~/.aws/config
          # Only link when WORK_ENV variable is set
          when:
            env: "WORK_ENV=true"
        
        - source: vpn/vpn.conf
          target: /etc/vpn.conf
          # Only link on machines with hostname starting with "work-"
          when:
            hostname: "work-*"
```

**Condition types:**
- `os`: ["macos", "linux", "windows"] - Operating system
- `profile`: ["work", "personal"] - Profile names
- `env`: "VAR=value" or just "VAR" - Environment variables
- `hostname`: "pattern" - Hostname with glob support

### List Profiles

View available profiles in your configuration:

```bash
heimdal profiles
```

### Validate Configuration

Check if your `heimdal.yaml` is valid:

```bash
# Validate current directory
heimdal validate

# Validate specific file
heimdal validate --config ~/dotfiles/heimdal.yaml
```

## Configuration Reference

### Root Configuration

```yaml
heimdal:
  version: "1.0"              # Config version
  repo: "git@github..."       # Git repository URL
  stow_compat: true           # Use .stowrc for symlinking

# Global ignore patterns (applies to all profiles)
ignore:
  - .git
  - "*.md"
  - .DS_Store

# Package name mappings (override defaults)
mappings:
  docker:
    apt: "docker.io"
    brew: "docker"
    dnf: "docker"
    pacman: "docker"
```

### Package Sources

```yaml
sources:
  # Simple packages (auto-mapped)
  packages:
    - git
    - vim
    - tmux

  # Homebrew (macOS)
  homebrew:
    packages:      # Formulae
      - neovim
      - fzf
    casks:         # Applications
      - firefox
      - slack
    hooks:
      pre_install:
        - command: "brew update"
      post_install:
        - command: "brew cleanup"

  # Mac App Store
  mas:
    packages:
      - id: 497799835
        name: "Xcode"
      - id: 1352778147
        name: "Bitwarden"
    hooks:
      pre_install:
        - command: "mas account"
          description: "Check MAS login"

  # APT (Debian/Ubuntu)
  apt:
    packages:
      - build-essential
      - python3-pip
    hooks:
      pre_install:
        - command: "sudo apt-get update"

  # DNF (Fedora/RHEL/CentOS)
  dnf:
    packages:
      - gcc
      - make
    hooks:
      pre_install:
        - command: "sudo dnf check-update"

  # Pacman (Arch/Manjaro)
  pacman:
    packages:
      - base-devel
      - linux-headers
    hooks:
      pre_install:
        - command: "sudo pacman -Sy"
```

### Profiles

```yaml
profiles:
  # Base profile
  base:
    sources:
      - packages
      - homebrew
    dotfiles:
      use_stowrc: true        # Use .stowrc for configuration
      ignore:                 # Profile-specific ignore patterns
        - "*.swp"
        - "*.tmp"
    hooks:
      post_apply:
        - command: "echo 'Setup complete!'"

  # Derived profile (inherits from base)
  work-laptop:
    extends: base
    sources:
      - name: homebrew        # Override homebrew source
        packages:
          - kubectl
          - aws-cli
        casks:
          - docker
```

### Hooks

Hooks can run commands before/after installations:

```yaml
hooks:
  pre_install:
    # Simple command
    - "brew update"
    
    # Detailed command with options
    - command: "sudo apt-get update"
      description: "Update package lists"
      os: ["linux"]           # Only on Linux
      when: "[ -f /usr/bin/apt-get ]"  # Conditional
      fail_on_error: true     # Stop if fails (default: true)
```

### Dotfiles Configuration

```yaml
profiles:
  myprofile:
    dotfiles:
      # Use .stowrc file for configuration
      use_stowrc: true
      
      # OR specify explicit file mappings
      symlink_all: false
      files:
        - source: ".bashrc"
          target: "~/.bashrc"
        - source: ".config/nvim"
          target: "~/.config/nvim"
```

### Sync Configuration

```yaml
sync:
  enabled: true               # Enable sync feature
  interval: "1h"             # Default interval for auto-sync
  auto_apply: true           # Auto-apply after sync
  notify:
    desktop: true            # Desktop notifications
    log: true                # Log notifications
  rollback_on_error: true    # Rollback if sync fails
```

## Advanced Features

### Package Name Mapping

Heimdal automatically maps tool names to package names across different package managers. The enhanced mapper includes:

**60+ Built-in Mappings:**
- Core tools: git, vim, neovim, tmux, curl, wget, tree, make
- Terminal utilities: ripgrep, bat, fd, fzf, htop, zsh, starship, jq
- Programming languages: node, python, go, rust
- Containers: docker, docker-compose, kubectl, helm, k9s
- Infrastructure: terraform, ansible
- Databases: postgresql, redis, mysql, sqlite
- Git tools: gh, delta, lazygit
- And many more...

**Smart Name Normalization:**
```bash
# Common aliases are automatically normalized
nodejs  → node
golang  → go
postgres → postgresql
k8s     → kubectl
rg      → ripgrep
nvim    → neovim
```

**Fuzzy Matching for Typos:**
```bash
# Heimdal suggests corrections for misspelled packages
ripgrap  → suggests 'ripgrep'
dokcer   → suggests 'docker'
kubctl   → suggests 'kubectl'
```

**Platform-Specific Translations:**
```yaml
# Same package, different names across platforms
docker:
  apt: docker.io       # Debian/Ubuntu
  brew: docker         # macOS
  dnf: docker          # Fedora
  pacman: docker       # Arch

node:
  apt: nodejs          # Debian/Ubuntu
  brew: node           # macOS
  dnf: nodejs          # Fedora
  pacman: nodejs       # Arch
```

You can still override any mapping in your config:

```yaml
mappings:
  mytool:
    apt: "mytool-deb"
    brew: "mytool"
    dnf: "mytool-rpm"
    pacman: "mytool-arch"
```

### Template System

Heimdal includes a simple template system for machine-specific configurations. Use `{{ variable }}` syntax to substitute values based on the current machine and profile.

**Built-in System Variables:**
- `{{ os }}` - Operating system (linux, macos, windows)
- `{{ arch }}` - Architecture (x86_64, aarch64, arm)
- `{{ family }}` - OS family (unix, windows)
- `{{ hostname }}` - Machine hostname
- `{{ user }}` - Current user
- `{{ home }}` - Home directory path

**Example `.gitconfig.tmpl`:**
```ini
[user]
    name = {{ name }}
    email = {{ email }}

[core]
    editor = vim

# Machine-specific settings
[http]
    proxy = {{ http_proxy }}
```

**Configuration in `heimdal.yaml`:**
```yaml
# Global variables
templates:
  variables:
    name: "John Doe"
    http_proxy: ""

profiles:
  work-laptop:
    templates:
      variables:
        email: "john@company.com"      # Override for work
        http_proxy: "proxy.company.com"
      files:
        - src: .gitconfig.tmpl
          dest: .gitconfig

  personal:
    templates:
      variables:
        email: "john@personal.com"     # Override for personal
      files:
        - src: .gitconfig.tmpl
          dest: .gitconfig
```

**Auto-Detection:**
Heimdal automatically detects `.tmpl` files in your dotfiles directory:
```bash
~/.dotfiles/
├── .gitconfig.tmpl   # Auto-detected
├── .zshrc.tmpl       # Auto-detected
└── heimdal.yaml
```

**Template Commands:**
```bash
# Preview how a template will be rendered
heimdal template preview .gitconfig.tmpl

# List all template files
heimdal template list -v

# Show all available variables
heimdal template variables
```

**Variable Priority:**
Profile variables > Config variables > System variables

This means profile-specific variables override global config variables, which override built-in system variables.

### Conflict Resolution

When creating symlinks, Heimdal can handle conflicts in several ways:

- **Prompt** (default): Ask what to do
- **Backup**: Back up existing files to `~/.heimdal/backups/`
- **Force**: Overwrite existing files
- **Skip**: Don't create the symlink

Use `heimdal apply --force` to always overwrite.

### Profile Inheritance

Profiles can extend other profiles additively:

```yaml
profiles:
  base:
    sources:
      - packages

  work-laptop:
    extends: base    # Includes all sources from base
    sources:
      - name: homebrew
        packages:
          - kubectl  # Adds kubectl to the packages
```

## Directory Structure

Heimdal uses the following directories:

- `~/.heimdal/` - Heimdal state and data
  - `state.json` - Current state (active profile, last sync, etc.)
  - `backups/` - Backup of overwritten files
- `~/.dotfiles/` - Default dotfiles directory (customizable)
- `/usr/local/bin/heimdal` - Heimdal binary

## Troubleshooting

### Check Configuration

```bash
heimdal validate --config ~/dotfiles/heimdal.yaml
```

### Dry-Run First

Always test with `--dry-run` before applying:

```bash
heimdal apply --dry-run
```

### Check Status

```bash
heimdal status --verbose
```

### View Logs

Heimdal outputs to stdout. Redirect to a file for logging:

```bash
heimdal sync 2>&1 | tee ~/.heimdal/sync.log
```

## Examples

See the `examples/` directory for sample configurations:

- `examples/minimal.yaml` - Minimal configuration
- `examples/full.yaml` - Full-featured configuration
- `examples/multi-platform.yaml` - Cross-platform setup

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by GNU Stow, Homebrew, and various dotfile management tools
- Built with Rust for performance and reliability
- Thanks to all contributors and users!

## Links

- **Repository**: https://github.com/limistah/heimdal
- **Issues**: https://github.com/limistah/heimdal/issues
- **Discussions**: https://github.com/limistah/heimdal/discussions

## Author

Aleem Isiaka - [@limistah](https://github.com/limistah)
