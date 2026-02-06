# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Nothing yet

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
