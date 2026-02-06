# Heimdal

[![CI](https://github.com/limistah/heimdal/workflows/CI/badge.svg)](https://github.com/limistah/heimdal/actions)
[![Release](https://img.shields.io/github/v/release/limistah/heimdal)](https://github.com/limistah/heimdal/releases)
[![Crates.io](https://img.shields.io/crates/v/heimdal)](https://crates.io/crates/heimdal)
[![License](https://img.shields.io/github/license/limistah/heimdal)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

**A universal dotfile and system configuration manager built in Rust**

Heimdal is a powerful, cross-platform tool that automatically manages your dotfiles, installs packages, and keeps your development environment in sync across multiple machines. Say goodbye to manual configuration and hello to automated, declarative system management.

## Features

- **Universal Package Management** - Install packages across Homebrew, APT, DNF, Pacman, and Mac App Store from a single configuration
- **Intelligent Symlinking** - GNU Stow-compatible symlink management with conflict resolution
- **Git-Based Sync** - Keep your configuration in sync across machines using Git
- **Profile-Based Configuration** - Different configurations for different machines (work, personal, servers)
- **Auto-Sync** - Background synchronization via cron jobs
- **Rollback Support** - Easily revert to previous configurations
- **Hooks System** - Run custom scripts before/after installation
- **Package Name Mapping** - Automatic translation of package names across different package managers
- **Dry-Run Mode** - Preview changes before applying them

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

**From APT Repository (Recommended):**

```bash
# One-line setup
curl -fsSL https://limistah.github.io/apt-repo/setup.sh | sudo bash
```

Or manually add the repository:

```bash
# Add repository
echo "deb [trusted=yes] https://limistah.github.io/apt-repo stable main" | sudo tee /etc/apt/sources.list.d/heimdal.list

# Install
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

**From APK Repository (Recommended):**

```bash
# One-line setup
wget -qO- https://limistah.github.io/apk-repo/setup.sh | sudo sh
```

Or manually add the repository:

```bash
# Add repository
echo "https://limistah.github.io/apk-repo/stable" | sudo tee -a /etc/apk/repositories

# Install
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

## Quick Start

### 1. Create a Dotfiles Repository

Create a new Git repository for your dotfiles:

```bash
mkdir ~/dotfiles
cd ~/dotfiles
git init
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

Heimdal automatically maps tool names to package names across different package managers. For example, `docker` becomes `docker.io` on APT but stays `docker` on Homebrew.

Built-in mappings include: git, vim, neovim, docker, gcc, fd, ripgrep, bat, fzf, zoxide, and many more.

You can override mappings in your config:

```yaml
mappings:
  mytool:
    apt: "mytool-deb"
    brew: "mytool"
    dnf: "mytool-rpm"
    pacman: "mytool-arch"
```

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
