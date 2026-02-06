# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Week 4: Git Integration & Lifecycle Hooks

- **Git Tracking & Commit System** (Day 1)
  - File change tracking with 6 change types (Modified, Added, Deleted, Renamed, TypeChanged, Untracked)
  - `heimdal commit` command with smart commit message generation
  - Auto-staging of files before commit
  - Commit message templates based on change patterns
  - Optional `--push` flag to push after commit
  - Support for committing specific files
  - `heimdal push` and `heimdal pull` commands
  - Push with optional remote/branch specification
  - Pull with optional rebase
  - 14 comprehensive tests for git operations

- **Git Sync Improvements & Branch Management** (Day 2)
  - Advanced sync module with conflict detection
  - Auto-stash support for uncommitted changes
  - Tracking info with ahead/behind counts
  - Dry-run mode for sync operations
  - Enhanced `heimdal sync` with better error handling
  - Branch management commands:
    - `heimdal branch current` - Show current branch
    - `heimdal branch list` - List all branches with highlight
    - `heimdal branch create` - Create and switch to new branch
    - `heimdal branch switch` - Switch to existing branch
    - `heimdal branch info` - Show tracking information
  - Beautiful formatted branch info with colors
  - 3 new tests for sync and branch operations

- **Lifecycle Hooks System** (Day 3)
  - Shared hooks module with `HookContext` enum
  - Global lifecycle hooks in `HeimdallConfig`:
    - `pre_apply`, `post_apply` - Before/after applying configuration
    - `pre_sync`, `post_sync` - Before/after syncing from remote
  - Profile-specific hooks:
    - Override or extend global hooks per profile
    - Same lifecycle events as global hooks
  - Dotfile-specific hooks:
    - `post_link` - Runs after creating symlink
    - `pre_unlink` - Runs before removing symlink
  - Hook execution features:
    - OS filtering (run only on specific operating systems)
    - Condition checking (file_exists, directory_exists)
    - Dry-run support
    - Error handling (continue or fail on error)
    - Output capture and display
    - Shell expansion support
  - Integration with apply and sync workflows
  - 4 comprehensive tests for hook execution
  - Refactored package hooks to use shared module

- **Git Remote Management** (Day 4)
  - Remote management commands:
    - `heimdal remote list` - List all remotes (with `-v` for URLs)
    - `heimdal remote add` - Add new remote with validation
    - `heimdal remote remove` - Remove remote with existence check
    - `heimdal remote set-url` - Change remote URL
    - `heimdal remote show` - Display remote details
    - `heimdal remote setup` - Interactive remote setup wizard
  - Interactive remote setup features:
    - Shows existing remotes
    - Prompts for remote name (default: origin)
    - Prompts for remote URL (SSH or HTTPS)
    - Handles replacing existing remotes
    - Optional first-time push after setup
    - User-friendly dialoguer-based prompts
  - Enhanced push method with remote/branch specification
  - 7 new remote management methods in GitRepo

### Changed

- Enhanced `cmd_sync()` to use new sync module with conflict handling
- Enhanced `cmd_apply()` to run lifecycle hooks at appropriate stages
- Package module now uses shared hooks module instead of local implementation
- All package managers updated to use `HookContext` enum

### Week 3: Enhanced Status & Package Commands

- **Enhanced Status Command** - Beautiful, informative status display
  - Color-coded output with status icons (✓✗⚠)
  - Organized sections: Configuration, Dotfiles, Packages, Git Status, Warnings
  - Dotfile tracking with symlink status (Synced, Modified, Missing, WrongTarget)
  - Package tracking with installation status and versions
  - Git integration (current branch, uncommitted changes)
  - Smart timestamps with relative time formatting ("2 hours ago")
  - Intelligent warnings system:
    - Modified dotfiles warning
    - Missing symlinks warning
    - Uninstalled packages warning
    - Uncommitted changes warning
    - Stale sync warning (>7 days)
  - Verbose mode for detailed information
  - 7 comprehensive tests

- **Diff Command** - Interactive change management for dotfiles
  - Git-based change detection using `git status --porcelain`
  - Tracks 5 change types: Modified, Added, Deleted, Renamed, TypeChanged
  - Line change statistics (+lines/-lines) in verbose mode
  - Beautiful colored output with change indicators
  - Grouped changes by type with counts
  - Interactive mode (`--interactive` flag):
    - View detailed git diff
    - Commit all or specific files (multi-select)
    - Discard all or specific changes (multi-select)
    - Optional push to remote after commit
    - Confirmation prompts for destructive actions
  - Safety features:
    - Verifies git repository exists
    - Only allows discarding modified/deleted files
    - Confirmation for "discard all"
  - 6 comprehensive tests

- **Package Management Commands** - Complete package lifecycle management
  
  - **`packages add <name>`** - Interactive package addition
    - Auto-detect available package managers
    - Interactive manager selection if multiple available
    - Package database lookup with metadata display
    - Package name normalization
    - Dependency detection with interactive prompts
    - Auto-update `heimdal.yaml` configuration
    - Optional `--no-install` flag
    - Support for `--manager` and `--profile` flags
  
  - **`packages remove <name>`** - Interactive package removal
    - Find package across all managers
    - Reverse dependency checking (warns if other packages depend on it)
    - Interactive removal confirmation
    - Auto-update `heimdal.yaml` configuration
    - Optional system uninstallation with confirmation
    - `--force` flag to skip dependency warnings
    - `--no-uninstall` flag to skip system removal
  
  - **`packages search <query>`** - Package database search
    - Search by name, description, or tags
    - Filter by category with `--category` flag
    - Display results with metadata (description, category, tags)
    - Results sorted by popularity
  
  - **`packages info <name>`** - Detailed package information
    - Show package metadata (description, category, tags)
    - Display alternative packages
    - Show related packages
    - List dependencies (required and optional)
    - Beautiful formatted output
  
  - **`packages list`** - List profile packages
    - List all packages in active profile
    - Grouped by package manager
    - Show installation status (✓/✗)
    - `--installed` flag to filter installed-only
    - `--profile` flag for specific profile
    - Total package count summary
  
  - Configuration management:
    - Reads from `heimdal.yaml`
    - Updates `Sources` section
    - Ensures profile references package manager source
    - Initializes sources with default hooks
  
  - 7 comprehensive tests across all package commands

#### Week 2: Package Intelligence & Smart Defaults

- **Package Profiles System** - Pre-configured package sets for common workflows
  - 10 built-in profiles: Minimal, Developer, Web Dev, Rust Dev, Python Dev, Go Dev, DevOps, Data Science, Designer, Writer
  - Platform-aware package resolution (macOS, Debian, Arch, Fedora)
  - Interactive profile selector in wizard
  - Base packages + platform-specific additions for each profile
  - Comprehensive test coverage (14 tests)

- **Dependency Detection & Analysis** - Automatic dependency management
  - Dependency graph with 50+ package relationships
  - Required vs. optional dependency tracking
  - Missing dependency detection with analysis results
  - Smart suggestions based on installed packages
  - Integrated into wizard with automatic prompts
  - Examples:
    - neovim requires git (for plugin management)
    - docker suggests docker-compose and kubectl
    - kubectl suggests helm and k9s
  - Comprehensive test coverage (13 tests)

- **Package Intelligence Database** - Rich metadata for 60+ popular packages
  - Package descriptions and categories (13 categories)
  - Popularity scores for better recommendations
  - Alternative package suggestions (e.g., vim vs neovim vs emacs)
  - Related package recommendations
  - Tag-based filtering and search
  - Category-based browsing
  - Comprehensive test coverage (13 tests)

- **Enhanced Package Mapper** - Intelligent cross-platform package handling
  - Expanded from 20 to 60+ package mappings
  - All database packages now have proper mappings across platforms
  - Added packages: helm, k9s, terraform, ansible, postgresql, redis, yarn, pipenv, zsh, starship, pandoc, and 40+ more
  - **Fuzzy Matching** using Jaro-Winkler distance algorithm
    - Detects typos: 'ripgrap' → suggests 'ripgrep'
    - Handles misspellings: 'dokcer' → suggests 'docker'
    - 0.85 similarity threshold for suggestions
  - **Name Normalization** for common aliases
    - nodejs/node.js → node
    - golang/go-lang → go
    - postgres/pg/psql → postgresql
    - k8s/kubernetes → kubectl
    - rg → ripgrep, nvim → neovim
  - Platform-specific mappings handled correctly:
    - docker.io (APT) ↔ docker (Homebrew/DNF/Pacman)
    - nodejs (APT/DNF) ↔ node (Homebrew)
    - fd-find (APT) ↔ fd (Homebrew/Pacman)
    - kubernetes-client (DNF) ↔ kubectl (others)
    - build-essential (APT) ↔ gcc (others)
  - Comprehensive test coverage (23 tests, up from 3)

#### Week 1: Interactive Wizard & Import System

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
- Updated README with Week 2 package intelligence features
- Added package profiles documentation
- Enhanced package name mapping documentation with examples
- Updated wizard documentation with profile selection flow
- Test suite expanded from 70 to 90 tests (29% increase)

### Technical Details
- Added `strsim` dependency for fuzzy matching (v0.11)
- Total new code: 2,709+ lines across 4 new modules
- Modules: `profiles.rs` (576 lines), `dependencies.rs` (652 lines), `database.rs` (738 lines), `mapper.rs` (expanded to 726 lines)
- All 90 tests passing

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
