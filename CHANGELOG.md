# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Interactive Setup Wizard** (`heimdal wizard`) - Onboarding in under 2 minutes
  - Three setup flows: Start fresh, Import existing, Clone repo
  - Automatic dotfile scanning in home directory
  - Package detection across all supported package managers
  - Configuration generation with preview
  - Git repository setup assistance

- **Import System** - Effortless migration from other dotfile managers
  - Import command: `heimdal import --path ~/dotfiles`
  - Auto-detection of existing tools (Stow, dotbot, manual)
  - Direct conversion to Heimdal format
  - Preserves compatibility settings (e.g., Stow compatibility mode)
  - Extracts package information from dotbot shell commands

- **Stow Importer** - Full GNU Stow support
  - Detects `.stowrc` and Stow directory structure
  - Scans all packages automatically
  - Maps files to home directory destinations
  - Maintains Stow compatibility mode in generated config
  
- **dotbot Importer** - dotbot configuration conversion
  - Parses `install.conf.yaml`
  - Converts link directives to Heimdal format
  - Extracts package installations from shell commands
  - Supports brew and apt package extraction

- **Enhanced Package Detection**
  - Automatic categorization (Essential, Development, Terminal, Editor, Application)
  - Support for 5 package managers: Homebrew, APT, DNF, Pacman, mas
  - Filters system packages (APT/DNF) to show only user-installed
  - Smart grouping and display in wizard

- **Improved Error Messages**
  - Helpful error formatting with causes and solutions
  - Symlink error helpers with actionable advice
  - Package installation error guidance
  - Configuration error details

### Changed
- Updated README with wizard quick start guide
- Added import examples and migration documentation
- Enhanced installation instructions

### Fixed
- Package categorization now checks editors before essential tools (fixes neovim classification)
- All tests passing (30/30)

## [1.0.0] - 2026-02-06

### Added
- Initial release of Heimdal
- Core commands: `init`, `apply`, `sync`, `status`, `profiles`, `validate`
- Package manager support:
  - Homebrew (formulae and casks)
  - Mac App Store (via mas)
  - APT (Debian/Ubuntu)
  - DNF (Fedora/RHEL/CentOS)
  - Pacman (Arch/Manjaro)
- GNU Stow-compatible symlink management
- Profile-based configuration with inheritance
- Package name mapping across platforms
- Pre/post install hooks with conditional execution
- Conflict resolution for symlinks (prompt/backup/force/skip)
- Git integration with submodule support
- State management in `~/.heimdal/`
- Auto-sync functionality via cron jobs
- Rollback to previous configurations
- History command to view past changes
- Dry-run mode for all operations
- Comprehensive documentation and examples
- Installation script for easy setup

### Package Manager Features
- Automatic package name translation across platforms
- Batch installation for efficiency
- Installation status checking
- Built-in mappings for 20+ common tools
- Custom mapping support in configuration

### Symlink Features
- GNU Stow compatibility
- Reads `.stowrc` configuration
- Global and profile-specific ignore patterns
- Conflict detection and resolution strategies
- Backup of overwritten files to `~/.heimdal/backups/`
- Support for nested directory structures

### Configuration Features
- YAML-based configuration
- Profile inheritance (additive merging)
- Conditional hooks based on OS
- Multiple source types (simple packages, platform-specific)
- Validation command to check configuration syntax

### Git Features
- Clone with `--recurse-submodules`
- Pull and sync workflow
- Rollback to any commit or tag
- History viewing with colored output
- Automatic reapplication after sync

### Auto-Sync Features
- Cron-based background synchronization
- Flexible intervals: hourly, daily, weekly, custom (e.g., `30m`, `2h`)
- Quiet mode for background execution
- Status checking

### Documentation
- Comprehensive README with examples
- Example configurations (minimal, full, multi-platform)
- Contributing guidelines
- Installation script
- MIT License

[Unreleased]: https://github.com/limistah/heimdal/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/limistah/heimdal/releases/tag/v1.0.0
