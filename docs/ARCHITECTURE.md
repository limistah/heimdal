# Architecture Overview

> **Status:** This document is being developed as part of the documentation overhaul (Week 3).

This document provides a comprehensive overview of Heimdal's system architecture, design decisions, and component interactions.

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Data Flow](#data-flow)
4. [Design Principles](#design-principles)
5. [Module Structure](#module-structure)

## System Overview

Heimdal is a universal dotfile and system configuration manager built in Rust. The architecture is designed around several key principles:

- **Cross-platform compatibility** - Works on macOS, Linux (Debian, Ubuntu, Fedora, Arch, Alpine)
- **Declarative configuration** - YAML-based configuration with profile support
- **State management** - Terraform-inspired state locking and conflict resolution
- **Package abstraction** - Universal interface across multiple package managers
- **Git-centric** - Built around Git workflows for syncing configurations

## Core Components

### System Architecture Diagram

```mermaid
graph TB
    subgraph CLI["Heimdal CLI"]
        Main[main.rs<br/>Entry Point]
        Parser[CLI Parser<br/>clap]
        Wizard[Interactive<br/>Wizard]
    end

    subgraph Core["Core Services Layer"]
        Config[Configuration<br/>Loader]
        Profile[Profile<br/>Manager]
        Package[Package<br/>Management]
        Dotfile[Dotfile<br/>Manager]
        State[State<br/>Management]
        Secret[Secret<br/>Manager]
        Sync[Git Sync<br/>Engine]
        Template[Template<br/>Engine]
    end

    subgraph External["External Integrations"]
        subgraph PM["Package Managers"]
            Brew[Homebrew]
            APT[APT/dpkg]
            DNF[DNF/yum]
            Pacman[Pacman]
            MAS[Mac App<br/>Store]
        end
        
        subgraph Storage["Storage Layer"]
            GitRepo[Git Repository<br/>dotfiles]
            PackageDB[Package Database<br/>limistah/heimdal-packages]
            Keychain[OS Keychain<br/>macOS/Linux]
            StateFile[State File<br/>~/.heimdal/state.json]
        end
    end

    Main --> Parser
    Parser --> Wizard
    Parser --> Config
    
    Config --> Profile
    Config --> Package
    Config --> Dotfile
    
    Wizard --> Package
    Wizard --> Config
    
    Package --> PM
    Package --> PackageDB
    Package --> State
    
    Dotfile --> Template
    Dotfile --> State
    Dotfile --> GitRepo
    
    Profile --> Config
    Profile --> State
    
    Sync --> GitRepo
    Sync --> State
    
    Secret --> Keychain
    Template --> Secret
    
    State --> StateFile
    
    style CLI fill:#e1f5ff
    style Core fill:#fff4e1
    style External fill:#f0f0f0
    style PM fill:#e8f5e9
    style Storage fill:#fce4ec
```

## Data Flow

### Package Installation Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Config
    participant Profile
    participant PackageDB
    participant Mapper
    participant PM as Package Manager
    participant State

    User->>CLI: heimdal install <package>
    CLI->>Config: Load heimdal.yaml
    Config-->>CLI: Configuration
    
    CLI->>Profile: Resolve active profile
    Profile-->>CLI: Profile config
    
    CLI->>PackageDB: Load database (cache/download)
    PackageDB-->>CLI: Package metadata
    
    CLI->>Mapper: Map package name
    Note over Mapper: nodejs → node<br/>cross-platform resolution
    Mapper-->>CLI: Platform-specific name
    
    CLI->>PM: Detect package manager
    Note over PM: brew/apt/dnf/pacman
    PM-->>CLI: Detected manager
    
    CLI->>PM: Check if installed
    PM-->>CLI: Installation status
    
    alt Package not installed
        CLI->>PM: Execute install command
        PM-->>CLI: Installation result
        CLI->>State: Record installed package
        State-->>CLI: State updated
    else Already installed
        CLI-->>User: Package already installed
    end
    
    CLI-->>User: Installation complete
```

### Dotfile Sync Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Config
    participant State
    participant Template
    participant Symlink
    participant Git
    participant FS as Filesystem

    User->>CLI: heimdal apply
    CLI->>Config: Load configuration
    Config-->>CLI: Dotfile config
    
    CLI->>State: Acquire state lock
    Note over State: Prevent concurrent<br/>modifications
    State-->>CLI: Lock acquired
    
    loop For each dotfile
        CLI->>Template: Process templates
        Note over Template: Substitute {{variables}}<br/>from config/env/secrets
        Template-->>CLI: Processed content
        
        CLI->>Symlink: Check target location
        Symlink->>FS: Check if file exists
        FS-->>Symlink: File status
        
        alt File exists (conflict)
            Symlink->>User: Prompt resolution
            User-->>Symlink: backup/overwrite/skip
            
            alt Backup chosen
                Symlink->>FS: Backup existing file
            else Overwrite chosen
                Symlink->>FS: Remove existing file
            else Skip chosen
                Symlink-->>CLI: Skip this file
            end
        end
        
        alt No conflict or resolved
            Symlink->>FS: Create symlink
            FS-->>Symlink: Symlink created
            Symlink->>State: Record applied dotfile
        end
    end
    
    CLI->>State: Update state
    State->>FS: Write state.json
    
    CLI->>State: Release lock
    State-->>CLI: Lock released
    
    opt Auto-sync enabled
        CLI->>Git: Commit changes
        CLI->>Git: Push to remote
    end
    
    CLI-->>User: Apply complete
```

## Design Principles

### 1. **State Management**
Inspired by Terraform, Heimdal maintains a state file (`~/.heimdal/heimdal.state.json`) that tracks:
- Active profile
- Installed packages
- Applied dotfiles
- Last sync timestamp
- Conflict history

See [State Management Documentation](STATE_MANAGEMENT.md) for details.

#### State Management Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Unlocked: System Start
    
    Unlocked --> Acquiring: acquire_lock()
    Acquiring --> Locked: Lock acquired
    Acquiring --> WaitForLock: Lock held by another process
    WaitForLock --> Acquiring: Retry after timeout
    WaitForLock --> Failed: Max retries exceeded
    
    Locked --> Modifying: Begin operation
    
    state Modifying {
        [*] --> ReadState
        ReadState --> ValidateState
        ValidateState --> DetectConflicts
        DetectConflicts --> ResolveConflicts: Conflicts found
        DetectConflicts --> ApplyChanges: No conflicts
        ResolveConflicts --> ApplyChanges: Resolved
        ResolveConflicts --> Aborted: Cannot resolve
        ApplyChanges --> WriteState
        WriteState --> [*]
    }
    
    Modifying --> Releasing: Operation complete
    Modifying --> ReleaseOnError: Operation failed
    
    Releasing --> Unlocked: release_lock()
    ReleaseOnError --> Unlocked: release_lock()
    
    Failed --> [*]: Error reported
    Unlocked --> [*]: System shutdown
```

#### State File Structure

```mermaid
graph LR
    State[State File] --> Active[Active Profile]
    State --> Packages[Installed Packages]
    State --> Dotfiles[Applied Dotfiles]
    State --> Lock[Lock Info]
    State --> History[Change History]
    
    Packages --> PkgList[Package List<br/>name, version, manager]
    Dotfiles --> DotList[Dotfile List<br/>source, target, hash]
    Lock --> LockData[Lock ID<br/>Process PID<br/>Timestamp<br/>Hostname]
    History --> Changes[Change Records<br/>timestamp, action, user]
    
    style State fill:#e1f5ff
    style Lock fill:#ffcdd2
    style History fill:#fff9c4
```

### 2. **Package Database**
A separate repository ([heimdal-packages](https://github.com/limistah/heimdal-packages)) maintains package metadata:
- YAML source definitions
- Compiled binary format (Bincode)
- Distributed via GitHub Releases
- Auto-updated every 7 days

See [Package Database Documentation](PACKAGE_DATABASE.md) for details.

### 3. **Profile-Based Configuration**
Users can define multiple profiles (work, personal, server) with:
- Different package sets
- Different dotfile targets
- Different template variables
- Profile inheritance support

### 4. **Cross-Platform Compatibility**
- Package name mapping (e.g., `nodejs` → `node`)
- Platform detection at runtime
- Conditional configuration based on OS/platform
- Graceful degradation when features unavailable

### Git Sync Workflow

Heimdal integrates Git deeply for versioning and syncing dotfiles across machines.

```mermaid
graph TD
    Start([User: heimdal sync]) --> Check{Check<br/>Git Status}
    
    Check -->|Has changes| Stage[Stage Changes]
    Check -->|No changes| Pull
    
    Stage --> Commit[Create Commit]
    Commit --> Push[Push to Remote]
    
    Push --> Pull[Pull from Remote]
    Pull -->|Success| Merge{Merge<br/>Conflicts?}
    Pull -->|Network Error| Retry{Retry?}
    
    Retry -->|Yes| Pull
    Retry -->|No| End([Sync Failed])
    
    Merge -->|No conflicts| Apply[Apply Changes]
    Merge -->|Conflicts| Resolve{Auto-resolve?}
    
    Resolve -->|Yes| Apply
    Resolve -->|No| Manual[Manual Resolution<br/>Required]
    
    Manual --> UserFix[User Resolves]
    UserFix --> Apply
    
    Apply --> UpdateState[Update State File]
    UpdateState --> Success([Sync Complete])
    
    style Start fill:#e1f5ff
    style Success fill:#c8e6c9
    style End fill:#ffcdd2
    style Manual fill:#fff9c4
```

### Conflict Resolution Strategies

```mermaid
graph LR
    Conflict{Conflict<br/>Detected} --> Type{Conflict<br/>Type}
    
    Type -->|File exists| FileConflict[File Conflict]
    Type -->|State mismatch| StateConflict[State Conflict]
    Type -->|Git merge| GitConflict[Git Conflict]
    
    FileConflict --> FileOpts{Resolution}
    FileOpts -->|Backup| Backup[Backup & Replace<br/>file.bak]
    FileOpts -->|Overwrite| Overwrite[Replace File]
    FileOpts -->|Skip| Skip[Keep Existing]
    
    StateConflict --> StateOpts{Resolution}
    StateOpts -->|Remote wins| RemoteWins[Use Remote State]
    StateOpts -->|Local wins| LocalWins[Use Local State]
    StateOpts -->|Merge| StateMerge[Merge States]
    
    GitConflict --> GitOpts{Resolution}
    GitOpts -->|Auto-merge| AutoMerge[Automatic Merge]
    GitOpts -->|Manual| ManualMerge[User Intervention]
    
    Backup --> Resolved([Resolved])
    Overwrite --> Resolved
    Skip --> Resolved
    RemoteWins --> Resolved
    LocalWins --> Resolved
    StateMerge --> Resolved
    AutoMerge --> Resolved
    ManualMerge --> Resolved
    
    style Conflict fill:#ffcdd2
    style Resolved fill:#c8e6c9
```

## Module Structure

```
src/
├── main.rs              # CLI entry point
├── commands/            # CLI command handlers
│   ├── packages/        # Package management commands
│   └── state/           # State management commands
├── config/              # Configuration loading and validation
├── git/                 # Git operations
├── hooks/               # Hook system (pre/post scripts)
├── import/              # Import from other dotfile managers
├── package/             # Package management
│   ├── database/        # Package database handling
│   └── profiles/        # Package profiles
├── profile/             # Profile management
├── secrets/             # Secret management (keychain)
├── state/               # State management (locking, conflicts)
├── symlink/             # Symlink creation (GNU Stow)
├── sync/                # Git sync operations
├── templates/           # Template engine
├── utils/               # Shared utilities
└── wizard/              # Interactive setup wizard
```

See [Module Guide](MODULE_GUIDE.md) for detailed module documentation.

## Key Design Decisions

### Why Rust?
- **Performance** - Fast startup time, efficient resource usage
- **Reliability** - Memory safety, no runtime errors
- **Cross-platform** - Single binary for all platforms
- **Type safety** - Catch errors at compile time

### Why Binary Package Database?
- **Speed** - Bincode deserialization is ~100x faster than JSON
- **Size** - Binary format is more compact (~20KB vs ~80KB JSON)
- **Type safety** - Schema enforced at compile time
- **Offline-first** - Cached locally, auto-updates in background

### Why GNU Stow Compatibility?
- **Ecosystem** - Leverage existing Stow knowledge and tools
- **Simplicity** - Stow's directory structure is intuitive
- **Migration** - Easy migration path from Stow
- **Flexibility** - Users can use Stow and Heimdal interchangeably

### Why OS Keychain Integration?
- **Security** - Secrets never stored in Git
- **Native** - Uses platform-provided secure storage
- **Encrypted** - OS-level encryption
- **Accessible** - Easy to use from CLI and scripts

## Future Architecture Improvements

### Planned Enhancements

#### 1. Plugin System
Enable community-contributed extensions:
- Custom package managers
- Additional import formats
- Custom template functions
- Hook integrations

```mermaid
graph LR
    Heimdal[Heimdal Core] --> API[Plugin API]
    API --> PM[Package Manager<br/>Plugins]
    API --> Import[Import<br/>Plugins]
    API --> Template[Template<br/>Plugins]
    API --> Hook[Hook<br/>Plugins]
    
    style Heimdal fill:#e1f5ff
    style API fill:#fff4e1
```

#### 2. Remote State Backend
Support for team collaboration:
- S3-backed state storage
- Shared team configurations
- Conflict resolution across team members
- Audit logging

#### 3. Parallel Operations
Improve performance for large installations:
- Concurrent package installations
- Parallel dotfile processing
- Background database updates

#### 4. Advanced Templating
Enhanced template capabilities:
- Conditional blocks
- Loops and iterations
- External data sources
- Computed values

#### 5. Drift Detection
Monitor configuration changes:
- Detect manual package installations
- Alert on dotfile modifications
- Auto-remediation options
- Compliance reporting

---

**Related Documentation:**
- [State Management](STATE_MANAGEMENT.md)
- [Package Database](PACKAGE_DATABASE.md)
- [Module Guide](MODULE_GUIDE.md)
- [Contributing Guide](dev/CONTRIBUTING.md)
