# Heimdal

[![CI](https://github.com/limistah/heimdal/workflows/CI/badge.svg)](https://github.com/limistah/heimdal/actions)
[![Release](https://img.shields.io/github/v/release/limistah/heimdal)](https://github.com/limistah/heimdal/releases)
[![Crates.io](https://img.shields.io/crates/v/heimdal)](https://crates.io/crates/heimdal)
[![License](https://img.shields.io/github/license/limistah/heimdal)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Documentation](https://img.shields.io/badge/docs-wiki-blue)](https://github.com/limistah/heimdal/wiki)

**A universal dotfile and system configuration manager built in Rust**

Heimdal automatically manages your dotfiles, installs packages, and keeps your development environment in sync across multiple machines. Built with Rust for performance and reliability.

📖 **[Full Documentation](https://github.com/limistah/heimdal/wiki)** |
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

##  Key Features

-  **Universal Package Management** - One config for Homebrew, APT, DNF, Pacman, and Mac App Store
-  **Intelligent Symlinking** - GNU Stow-compatible with automatic conflict resolution
-  **Smart Package Discovery** - Native OS package manager search (brew, apt, dnf, pacman, apk)
-  **Secret Management** - Secure storage using OS keychains (macOS Keychain, Linux Secret Service)
-  **Template System** - Machine-specific configs with variable substitution
-  **Git-Based Sync** - Keep configs in sync across machines with automatic conflict detection
-  **Profile System** - Different configs for work, personal, and server machines
-  **Interactive Wizard** - Guided setup with smart defaults
-  **Import Support** - Migrate from Stow, dotbot, chezmoi, yadm, homesick
-  **Rollback Support** - Easily revert to previous configurations
-  **Hooks System** - Run custom scripts before/after installation
-  **Dry-Run Mode** - Preview changes before applying them

[Explore all features →](https://github.com/limistah/heimdal/wiki/Features)

---

## Package System

Packages are declared directly in `heimdal.yaml` using a `packages:` section. Use `common:` for packages with the same name across all OS package managers, or per-manager keys (`homebrew:`, `apt:`, `dnf:`, `pacman:`, `apk:`) when names differ or a package is OS-specific. A top-level `packages:` section applies to all profiles; each profile can also define its own `packages:`.

```yaml
heimdal:
  version: "1"
  repo: "git@github.com:you/dotfiles.git"

packages:
  common: [git, curl, vim, tmux]  # same name everywhere, every profile
  homebrew: [mas]                  # mac only, every profile

profiles:
  default:
    packages:
      common: [ripgrep, fzf]
      homebrew: [iterm2]
      apt: [build-essential]

  work:
    extends: default
    packages:
      homebrew: [slack, zoom]
```

A `common:` entry can also be an object instead of a plain string, for packages whose name differs by ecosystem (VS Code is `visual-studio-code` on a Homebrew cask but `code` on apt/dnf/pacman). `default` is used whenever the manager that actually runs has no override of its own:

```yaml
packages:
  common:
    - zsh                    # same name everywhere
    - default: docker-desktop
      homebrew_casks: docker-desktop
      apt: docker-ce
      dnf: docker-ce
      pacman: docker
      apk: docker
```

### Search & Install
```bash
# Search via the native OS package manager
heimdal packages search neovim

# Add a package to heimdal.yaml
heimdal packages add neovim
```

### Supported Package Managers
- **macOS**: Homebrew (formulae + casks), Mac App Store (via `mas`)
- **Debian/Ubuntu**: APT
- **Fedora/RHEL/CentOS**: DNF/YUM
- **Arch/Manjaro**: Pacman
- **Alpine**: apk

### Smart Features
- ✅ Dependency detection and suggestions
- ✅ Installation status checking

## Toolchain Packages

Language-toolchain-level global installs — npm, cargo, `go install`, gem, and pip — are declared separately in a `toolchains:` section, because they're a genuinely different axis from `packages:`: only one of homebrew/apt/dnf/pacman/apk/mas ever applies on a given OS, but a single machine can legitimately have npm *and* cargo *and* go tools installed at once, and these package names don't vary by OS. Like `packages:`, a top-level `toolchains:` section applies to all profiles, and each profile can add its own.

```yaml
toolchains:
  npm: [typescript, eslint]          # npm install -g <pkg>
  cargo: [ripgrep, fd-find]          # cargo install <pkg>
  gem: [bundler]                     # gem install <pkg>
  pip: [black, httpie]               # installed via `pipx install <pkg>`
  go:
    - golang.org/x/tools/cmd/goimports@latest
    - github.com/bufbuild/buf/cmd/buf@v1.32.0

profiles:
  default:
    toolchains:
      cargo: [cargo-audit]
```

`npm`/`cargo`/`gem`/`pip` entries are plain package names installed at latest, same as the flat lists under `packages:`. `pip` installs go through [`pipx`](https://pipx.pypa.io/) rather than raw `pip install --user`, since pipx isolates each CLI tool in its own venv and avoids one global install breaking another.

`go` entries are the one exception: each must be a full module import path *with* an explicit `@version` (`...@latest` or `...@vX.Y.Z`), because `go install` itself requires one — `heimdal apply`/`heimdal validate` reject a bare import path with a clear error rather than mis-invoking `go install`.

At `apply` time, every toolchain manager actually present on the machine runs its own list — unlike `packages:`, there's no "pick the first one available" behavior here. Each manager that supports a real "what's installed" query (`npm ls -g --depth=0 --json`, `cargo install --list`, `gem list --local`, `pipx list --json`) is checked before reinstalling, same as `packages:`. `go` is the one exception: `go install` keeps no record of what it has installed system-side, so heimdal tracks each go-installed module path and version itself (in `~/.heimdal/state.json`) and treats that record — not a live query — as the "already installed" check.

### Supported Toolchain Managers
- **npm** — Node.js global packages
- **cargo** — Rust binaries
- **go** — Go tools, via `go install` (state-tracked; see above)
- **gem** — Ruby gems
- **pip** — Python CLI tools, via pipx

---

## Architecture

### [`limistah/heimdal`](https://github.com/limistah/heimdal) (This repo)
The main CLI tool - installation, configuration, and management commands.

**Contribute here for:**
- Bug fixes and issue reports
- New CLI features and commands
- Core functionality improvements
- Documentation updates

[Development Guide →](https://github.com/limistah/heimdal/wiki/CLI-Development)

### How It Works

```
User → Heimdal CLI → Package Manager (brew/apt/dnf/pacman/apk)
                    ├─→ Dotfile Management (GNU Stow compatible)
                    ├─→ Secret Management (OS Keychain)
                    └─→ Git Sync (with state management)
```

---

## Documentation

### For Users
- [Quick Start Guide](https://github.com/limistah/heimdal/wiki/Quick-Start)
- [Package Management](https://github.com/limistah/heimdal/wiki/Package-Management)
- [Profile System](https://github.com/limistah/heimdal/wiki/Profile-System)
- [Dotfile Management](https://github.com/limistah/heimdal/wiki/Dotfile-Management)
- [Template System](https://github.com/limistah/heimdal/wiki/Template-System)
- [Secret Management](https://github.com/limistah/heimdal/wiki/Secret-Management)
- [Git Sync](https://github.com/limistah/heimdal/wiki/Git-Sync)
- [Configuration Reference](https://github.com/limistah/heimdal/wiki/Configuration)
- [Troubleshooting](https://github.com/limistah/heimdal/wiki/Troubleshooting)

### For Contributors
- [CLI Development Guide](https://github.com/limistah/heimdal/wiki/CLI-Development)
- [Testing Guide](docs/dev/TESTING.md)
- [Contributing Guide](docs/dev/CONTRIBUTING.md)

### Technical Docs
- [Architecture Overview](docs/ARCHITECTURE.md)
- [State Management](docs/STATE_MANAGEMENT.md) - Locking, conflict resolution
- [Module Guide](docs/MODULE_GUIDE.md) - Codebase structure

---

## Example Configuration

Here's a minimal `heimdal.yaml` to get started:

```yaml
heimdal:
  version: "1"
  repo: "git@github.com:you/dotfiles.git"

packages:
  common: [git, curl, vim, tmux]  # same name everywhere, every profile
  homebrew: [mas]                  # mac only, every profile

profiles:
  default:
    packages:
      common: [ripgrep, fzf]
      homebrew: [iterm2]
      apt: [build-essential]

  work:
    extends: default
    packages:
      homebrew: [slack, zoom]
```

**More examples:**
- [Minimal configuration](examples/minimal.yaml)
- [Full-featured setup](examples/full.yaml)
- [Cross-platform configuration](examples/multi-platform.yaml)

---

## Usage Examples

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
# Search for packages via the native OS package manager
heimdal packages search ripgrep

# Add a package to heimdal.yaml
heimdal packages add ripgrep

# List installed packages
heimdal packages list --installed
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

## Community & Support

- **Bug Reports:** [GitHub Issues](https://github.com/limistah/heimdal/issues)
- **Feature Requests:** [GitHub Discussions](https://github.com/limistah/heimdal/discussions)
- **Questions:** [GitHub Discussions](https://github.com/limistah/heimdal/discussions)

---

## Contributing

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

---

## Directory Structure

Heimdal uses the following directories:

- `~/.heimdal/` - Heimdal state and data
  - `state.json` - Current state (active profile, last sync, etc.)
  - `backups/` - Backup of overwritten files
- `~/.dotfiles/` - Default dotfiles directory (customizable via config)
- `/usr/local/bin/heimdal` - Heimdal binary (or `~/.cargo/bin/heimdal`)

---

## License

MIT License - see [LICENSE](LICENSE) for details

---

## Acknowledgments

- Inspired by [GNU Stow](https://www.gnu.org/software/stow/), [Homebrew](https://brew.sh/), and various dotfile management tools
- Built with [Rust](https://www.rust-lang.org/) for performance and reliability
- Thanks to all [contributors](https://github.com/limistah/heimdal/graphs/contributors) and users!

---

## Links

- **Main Repository:** https://github.com/limistah/heimdal
- **Documentation:** https://github.com/limistah/heimdal/wiki
- **Crates.io:** https://crates.io/crates/heimdal
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)

---
