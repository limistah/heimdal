# Heimdal

[![CI](https://github.com/limistah/heimdal/workflows/CI/badge.svg)](https://github.com/limistah/heimdal/actions)
[![Release](https://img.shields.io/github/v/release/limistah/heimdal)](https://github.com/limistah/heimdal/releases)
[![Crates.io](https://img.shields.io/crates/v/heimdal)](https://crates.io/crates/heimdal)
[![License](https://img.shields.io/github/license/limistah/heimdal)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://img.shields.io/badge/docs-wiki-blue)](https://github.com/limistah/heimdal/wiki)
[![Packages](https://img.shields.io/badge/packages-60+-green)](https://github.com/limistah/heimdal-packages)

**A universal dotfile and system configuration manager built in Rust**

Heimdal automatically manages your dotfiles, installs packages, and keeps your development environment in sync across multiple machines. Built with Rust for performance and reliability.

📖 **[Full Documentation](https://github.com/limistah/heimdal/wiki)** | 
📦 **[Package Database](https://github.com/limistah/heimdal-packages)** | 
💬 **[Discussions](https://github.com/limistah/heimdal/discussions)**

---

## ⚡ Quick Start

### Installation

**Homebrew (macOS)**
```bash
brew install limistah/tap/heimdal
```

**APT (Debian/Ubuntu)**
```bash
# One-line setup
curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/main/scripts/install-deb.sh | bash

# Or manual
curl -fsSL https://limistah.github.io/heimdal/gpg | sudo gpg --dearmor -o /usr/share/keyrings/heimdal-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/heimdal-archive-keyring.gpg] https://limistah.github.io/heimdal/deb stable main" | sudo tee /etc/apt/sources.list.d/heimdal.list
sudo apt update && sudo apt install heimdal
```

**Cargo (All Platforms)**
```bash
cargo install heimdal
```

**Universal installer**
```bash
curl -fsSL https://raw.githubusercontent.com/limistah/heimdal/main/install.sh | bash
```

**More installation options:** [Installation Guide →](https://github.com/limistah/heimdal/wiki/Installation)

### Setup (2 minutes)

**New to dotfiles?**
```bash
heimdal wizard
```

The interactive wizard guides you through:
- Smart scanning with real-time progress
- Interactive file and package selection
- Smart profile names (auto-generated from hostname/OS)
- Git sync setup with remote configuration

**Migrating from Stow, dotbot, chezmoi, yadm, or homesick?**
```bash
heimdal wizard  # Choose "Import existing dotfiles"
heimdal import --path ~/.dotfiles --preview  # Preview before importing
```

**Cloning to a new machine?**
```bash
git clone <your-dotfiles-repo> ~/.dotfiles
cd ~/.dotfiles
heimdal apply
```

[Learn more →](https://github.com/limistah/heimdal/wiki/Quick-Start)

---

## ✨ Key Features

- 📦 **Universal Package Management** - One config for Homebrew, APT, DNF, Pacman, and Mac App Store
- 🔗 **Intelligent Symlinking** - GNU Stow-compatible with automatic conflict resolution
- 🎯 **Smart Package Discovery** - Fuzzy search, tags, groups, and 60+ curated packages
- 🔐 **Secret Management** - Secure storage using OS keychains (macOS Keychain, Linux Secret Service)
- 🎨 **Template System** - Machine-specific configs with variable substitution
- 🌿 **Git-Based Sync** - Keep configs in sync across machines with automatic conflict detection
- 🎭 **Profile System** - Different configs for work, personal, and server machines
- 🤖 **Interactive Wizard** - Guided setup with smart defaults
- 🚀 **Import Support** - Migrate from Stow, dotbot, chezmoi, yadm, homesick
- 🔄 **Rollback Support** - Easily revert to previous configurations
- 🪝 **Hooks System** - Run custom scripts before/after installation
- 🔍 **Dry-Run Mode** - Preview changes before applying them

[Explore all features →](https://github.com/limistah/heimdal/wiki/Features)

---

## 📦 Package System

Heimdal includes a [powerful package database](https://github.com/limistah/heimdal-packages) with 60+ popular development tools, smart search, and curated groups.

### Search & Install
```bash
# Fuzzy search with intelligent scoring
heimdal packages search neovim
heimdal packages install neovim

# Browse by tags
heimdal packages search --tag editor

# Install pre-configured groups
heimdal packages group install web-dev

# Get package info
heimdal packages info docker

# Check for outdated packages
heimdal packages outdated

# Upgrade all outdated packages
heimdal packages upgrade
```

### Supported Package Managers
- **macOS**: Homebrew (formulae + casks), Mac App Store (via `mas`)
- **Debian/Ubuntu**: APT
- **Fedora/RHEL/CentOS**: DNF/YUM
- **Arch/Manjaro**: Pacman

### Available Package Groups

Pre-configured collections for common workflows:

- **`essential`** - Core CLI tools (git, curl, vim, tmux)
- **`web-dev`** - Modern web development (node, yarn, docker, postgres, redis)
- **`rust-dev`** - Rust ecosystem (rust, cargo, rust-analyzer, ripgrep, fd, bat)
- **`python-dev`** - Python development (python, pip, pipenv, pyenv)
- **`go-dev`** - Go development (go, gopls, docker, kubectl)
- **`devops`** - Infrastructure tools (terraform, ansible, docker, kubectl, helm)
- **`data-science`** - Data analysis (python, jupyter, pandas, postgresql)
- **`terminal`** - Enhanced terminal experience (tmux, fzf, ripgrep, bat, delta)

[See all 15+ groups →](https://github.com/limistah/heimdal/wiki/Package-Groups)

### Smart Features
- ✅ Dependency detection and suggestions
- ✅ Outdated package detection
- ✅ Cross-platform package mapping
- ✅ Installation status checking
- ✅ Fuzzy search with typo tolerance
- ✅ Tag-based filtering
- ✅ Automatic suggestions based on project files

**Want to add a package?** See the [Package Contribution Guide →](https://github.com/limistah/heimdal-packages/blob/main/CONTRIBUTING.md)

---

## 🏗️ Architecture

Heimdal consists of two repositories:

### [`limistah/heimdal`](https://github.com/limistah/heimdal) (This repo)
The main CLI tool - installation, configuration, and management commands.

**Contribute here for:**
- 🐛 Bug fixes and issue reports
- ✨ New CLI features and commands
- 🔧 Core functionality improvements
- 📚 Documentation updates

[Development Guide →](https://github.com/limistah/heimdal/wiki/CLI-Development)

### [`limistah/heimdal-packages`](https://github.com/limistah/heimdal-packages)
The package metadata database - YAML definitions compiled to binary format (20KB, 60+ packages).

**Contribute here for:**
- 📦 Adding new packages
- 🏷️ Updating package metadata
- 📝 Package descriptions and tags
- 🎯 New package groups

[Package Contribution Guide →](https://github.com/limistah/heimdal-packages/blob/main/CONTRIBUTING.md)

### How It Works

```
User → Heimdal CLI → Package Database (60+ packages)
                    ├─→ Homebrew / APT / DNF / Pacman / MAS
                    ├─→ Dotfile Management (GNU Stow compatible)
                    ├─→ Secret Management (OS Keychain)
                    └─→ Git Sync (with state management)
```

The package database is downloaded from GitHub Releases and cached locally (`~/.heimdal/cache/packages.db`). It auto-updates every 7 days.

---

## 📚 Documentation

### For Users
- [📖 Quick Start Guide](https://github.com/limistah/heimdal/wiki/Quick-Start)
- [📦 Package Management](https://github.com/limistah/heimdal/wiki/Package-Management)
- [🎭 Profile System](https://github.com/limistah/heimdal/wiki/Profile-System)
- [📁 Dotfile Management](https://github.com/limistah/heimdal/wiki/Dotfile-Management)
- [🎨 Template System](https://github.com/limistah/heimdal/wiki/Template-System)
- [🔐 Secret Management](https://github.com/limistah/heimdal/wiki/Secret-Management)
- [🌿 Git Sync](https://github.com/limistah/heimdal/wiki/Git-Sync)
- [⚙️ Configuration Reference](https://github.com/limistah/heimdal/wiki/Configuration)
- [🐛 Troubleshooting](https://github.com/limistah/heimdal/wiki/Troubleshooting)

### For Contributors
- [🔧 CLI Development Guide](https://github.com/limistah/heimdal/wiki/CLI-Development)
- [📦 Package Contributions](https://github.com/limistah/heimdal-packages/blob/main/CONTRIBUTING.md)
- [🧪 Testing Guide](docs/dev/TESTING.md)
- [🤝 Contributing Guide](docs/dev/CONTRIBUTING.md)

### Technical Docs
- [🏗️ Architecture Overview](docs/ARCHITECTURE.md)
- [🔒 State Management](docs/STATE_MANAGEMENT.md) - Locking, conflict resolution
- [💾 Package Database Design](docs/PACKAGE_DATABASE.md) - Binary format, indexing
- [🗺️ Module Guide](docs/MODULE_GUIDE.md) - Codebase structure

---

## 🎯 Example Configuration

Here's a minimal `heimdal.yaml` to get started:

```yaml
global:
  dotfiles_dir: ~/.dotfiles
  ignore_patterns:
    - ".git"
    - "*.swp"
    - ".DS_Store"

package_sources:
  packages:
    - git
    - neovim
    - tmux
    - fzf
  groups:
    - rust-dev

profiles:
  work:
    packages:
      - docker
      - kubectl
      - terraform
    dotfiles:
      targets:
        - path: ~/.dotfiles/work
          stow: true
    templates:
      email: "work@example.com"

  personal:
    packages:
      - spotify
    dotfiles:
      targets:
        - path: ~/.dotfiles/personal
          stow: true
    templates:
      email: "personal@example.com"
```

**More examples:**
- [Minimal configuration](examples/minimal.yaml)
- [Full-featured setup](examples/full.yaml)
- [Cross-platform configuration](examples/multi-platform.yaml)

---

## 🚀 Usage Examples

### Basic Workflow
```bash
# Initialize a new dotfiles repository
heimdal wizard

# Apply your configuration
heimdal apply

# Sync changes to/from Git
heimdal sync

# Check status
heimdal status

# Switch profiles
heimdal profile switch personal

# Validate configuration
heimdal validate
```

### Package Management
```bash
# Search for packages
heimdal packages search ripgrep

# Install a package
heimdal packages install ripgrep

# Install a package group
heimdal packages group install terminal

# List installed packages
heimdal packages list --installed

# Check for outdated packages
heimdal packages outdated

# Upgrade all packages
heimdal packages upgrade
```

### Advanced Features
```bash
# Dry-run before applying
heimdal apply --dry-run

# Import from existing dotfiles
heimdal import --path ~/old-dotfiles

# Rollback to previous state
heimdal rollback

# View sync history
heimdal history

# Manage secrets
heimdal secret set API_KEY "secret-value"
heimdal secret get API_KEY
```

[See full CLI reference →](https://github.com/limistah/heimdal/wiki/CLI-Reference)

---

## 💬 Community & Support

- 🐛 **Bug Reports:** [GitHub Issues](https://github.com/limistah/heimdal/issues)
- 💡 **Feature Requests:** [GitHub Discussions](https://github.com/limistah/heimdal/discussions)
- 💬 **Questions:** [GitHub Discussions](https://github.com/limistah/heimdal/discussions)
- 📦 **Package Requests:** [heimdal-packages Issues](https://github.com/limistah/heimdal-packages/issues)

---

## 🤝 Contributing

We welcome contributions! Here's how to get involved:

### CLI Development

1. Fork this repository
2. Clone your fork: `git clone <your-fork-url>`
3. Create a feature branch: `git checkout -b feature/amazing-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy --all-targets`
7. Commit your changes: `git commit -m 'Add amazing feature'`
8. Push to your fork: `git push origin feature/amazing-feature`
9. Open a Pull Request

[See full CLI development guide →](https://github.com/limistah/heimdal/wiki/CLI-Development)

### Package Contributions

Want to add a package to the database?

1. Head to [`limistah/heimdal-packages`](https://github.com/limistah/heimdal-packages)
2. Follow the [contribution guide](https://github.com/limistah/heimdal-packages/blob/main/CONTRIBUTING.md)
3. Create a YAML file for your package
4. Run validation: `cargo run --bin validate`
5. Submit a Pull Request

Package contributions are quick and easy - most packages take <5 minutes to add!

---

## 🔧 Directory Structure

Heimdal uses the following directories:

- `~/.heimdal/` - Heimdal state and data
  - `state.json` - Current state (active profile, last sync, etc.)
  - `cache/packages.db` - Cached package database
  - `backups/` - Backup of overwritten files
- `~/.dotfiles/` - Default dotfiles directory (customizable via config)
- `/usr/local/bin/heimdal` - Heimdal binary (or `~/.cargo/bin/heimdal`)

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

---

## 🙏 Acknowledgments

- Inspired by [GNU Stow](https://www.gnu.org/software/stow/), [Homebrew](https://brew.sh/), and various dotfile management tools
- Built with [Rust](https://www.rust-lang.org/) for performance and reliability
- Thanks to all [contributors](https://github.com/limistah/heimdal/graphs/contributors) and users!

---

## 🔗 Links

- **Main Repository:** https://github.com/limistah/heimdal
- **Package Database:** https://github.com/limistah/heimdal-packages
- **Documentation:** https://github.com/limistah/heimdal/wiki
- **Crates.io:** https://crates.io/crates/heimdal
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)

---

**Built by [@limistah](https://github.com/limistah)**
