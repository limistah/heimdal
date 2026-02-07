# Package Database Design

> **Status:** This document is being developed as part of the documentation overhaul (Week 3).

This document describes the design and implementation of Heimdal's package database system.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Data Format](#data-format)
4. [Database Schema](#database-schema)
5. [Compilation Process](#compilation-process)
6. [Distribution & Caching](#distribution--caching)
7. [Search & Indexing](#search--indexing)

## Overview

Heimdal uses a centralized package database maintained in a separate repository ([heimdal-packages](https://github.com/limistah/heimdal-packages)). The database contains metadata for 60+ popular development tools with cross-platform package mappings.

### Key Features

- **Fast Loading** - Binary format (Bincode) loads in ~10ms
- **Small Size** - ~20KB for 60+ packages
- **Rich Metadata** - Descriptions, categories, tags, dependencies
- **Cross-Platform** - Maps package names across Homebrew, APT, DNF, Pacman, MAS
- **Offline-First** - Cached locally, auto-updates every 7 days
- **Community-Maintained** - Anyone can contribute package definitions

## Architecture

```
heimdal-packages repo (YAML source)
    │
    ├─→ Validation (JSON Schema)
    │
    ├─→ Compilation (Rust binary)
    │
    ├─→ Bincode Serialization
    │
    ├─→ SHA256 Checksum
    │
    ├─→ GitHub Release (packages.db + packages.db.sha256)
    │
    └─→ Heimdal CLI Downloads & Caches
```

### Two-Repository Design

**Why separate the package database from the CLI?**

1. **Independent Updates** - Package metadata can be updated without CLI releases
2. **Community Contributions** - Lower barrier to contribute packages vs. CLI code
3. **Faster CI** - Package validation is separate from CLI tests
4. **Versioning** - Database has its own version separate from CLI
5. **Size** - CLI binary stays small, database downloaded separately

## Data Format

### Source Format (YAML)

Package definitions are written in human-readable YAML:

```yaml
name: neovim
description: Vim-fork focused on extensibility and usability
category: editor
popularity: 95

platforms:
  brew: neovim
  apt: neovim
  dnf: neovim
  pacman: neovim

dependencies:
  required:
    - package: git
      reason: Required for plugin management
  optional:
    - package: ripgrep
      reason: Faster file searching
    - package: fzf
      reason: Fuzzy finding in Telescope

tags:
  - editor
  - vim
  - terminal
  - lua

alternatives:
  - vim
  - emacs
  - helix

related:
  - tmux
  - fzf
  - ripgrep

website: https://neovim.io
license: Apache-2.0
source: https://github.com/neovim/neovim
```

### Compiled Format (Bincode)

The YAML files are compiled into a binary format using Rust's `bincode` crate:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledDatabase {
    pub version: u32,
    pub last_updated: String,
    pub packages: Vec<Package>,
    pub groups: Vec<PackageGroup>,
    
    // Indexes for O(1) lookups
    pub index_by_name: HashMap<String, usize>,
    pub index_by_category: HashMap<String, Vec<usize>>,
    pub index_by_tag: HashMap<String, Vec<usize>>,
}
```

## Database Schema

### Package Schema

```rust
pub struct Package {
    pub name: String,
    pub description: String,
    pub category: String,
    pub popularity: u8,              // 0-100
    pub platforms: Platforms,
    pub dependencies: Dependencies,
    pub alternatives: Vec<String>,
    pub related: Vec<String>,
    pub tags: Vec<String>,
    pub website: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
}

pub struct Platforms {
    pub apt: Option<String>,
    pub brew: Option<String>,
    pub dnf: Option<String>,
    pub pacman: Option<String>,
    pub mas: Option<i64>,            // Mac App Store ID
}
```

### Package Group Schema

```rust
pub struct PackageGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub packages: GroupPackages,
    pub platform_overrides: HashMap<String, PlatformOverride>,
}

pub struct GroupPackages {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}
```

## Compilation Process

### 1. Validation

```bash
# Validate all YAML files against JSON schemas
cargo run --bin validate

# Checks:
# - Valid YAML syntax
# - All required fields present
# - Cross-references valid (dependencies, alternatives exist)
# - Platform-specific package names are correct
# - No duplicate package names
```

### 2. Compilation

```bash
# Compile YAML to binary database
cargo run --bin compile

# Process:
# 1. Load all YAML files
# 2. Build indexes (by name, category, tag)
# 3. Serialize to Bincode format
# 4. Generate SHA256 checksum
# 5. Write packages.db and packages.db.sha256
```

### 3. Distribution

Compiled database is released via GitHub Releases:
- **URL:** `https://github.com/limistah/heimdal-packages/releases/latest/download/packages.db`
- **Checksum:** `https://github.com/limistah/heimdal-packages/releases/latest/download/packages.db.sha256`

## Distribution & Caching

### Download & Cache Strategy

```rust
// Cache location: ~/.heimdal/cache/packages.db
pub struct DatabaseCache {
    cache_dir: PathBuf,
}

impl DatabaseCache {
    // Check if cache needs update (age > 7 days)
    pub fn needs_update(&self, max_age_days: i64) -> bool;
    
    // Download from GitHub Releases
    pub fn download(&self) -> Result<()>;
    
    // Verify SHA256 checksum
    pub fn verify_checksum(&self) -> Result<bool>;
    
    // Load cached database
    pub fn load(&self) -> Result<CompiledDatabase>;
}
```

### Auto-Update Logic

```
User Command
    │
    ├─→ Check cache age
    │
    ├─→ If > 7 days old:
    │   ├─→ Try to download new database
    │   ├─→ Verify checksum
    │   ├─→ If success: use new database
    │   └─→ If fail: use cached version (warn user)
    │
    └─→ Load database from cache
```

## Search & Indexing

### Fuzzy Search

Uses `fuzzy-matcher` crate for typo-tolerant search:

```rust
pub enum MatchKind {
    Exact,              // Score: 1000
    NameContains,       // Score: 500
    DescriptionContains,// Score: 200
    TagContains,        // Score: 100
    Fuzzy,              // Score: 0-100 (from fuzzy-matcher)
}
```

### Search Examples

```bash
# Exact match
$ heimdal packages search neovim
✓ neovim (editor) - Vim-fork focused on extensibility

# Fuzzy match (typo)
$ heimdal packages search neovom
Did you mean 'neovim'?

# Tag search
$ heimdal packages search --tag editor
neovim, vim, emacs, helix

# Category search
$ heimdal packages search --category terminal
bat, fd, fzf, htop, jq, ripgrep, tmux
```

### Indexes for Performance

```rust
// O(1) lookup by name
index_by_name: HashMap<String, usize>

// O(1) lookup by category
index_by_category: HashMap<String, Vec<usize>>

// O(1) lookup by tag
index_by_tag: HashMap<String, Vec<usize>>
```

## Performance Metrics

### Database Size

- **YAML Source:** ~150KB (60 packages)
- **Compiled Binary:** ~20KB (87% reduction)
- **JSON Equivalent:** ~80KB (75% reduction vs JSON)

### Load Time

- **Binary (Bincode):** ~10ms average
- **JSON Parsing:** ~1000ms average
- **100x faster** than JSON

### Memory Usage

- **In-Memory Size:** ~50KB (deserialized)
- **Indexes:** ~10KB additional

## Future Improvements

This section will be expanded in Week 3 with:

- **Incremental Updates** - Only download changed packages
- **Compression** - Gzip compression for even smaller size
- **Multiple Databases** - Support for custom/private databases
- **Database Versioning** - Compatibility checking
- **Metrics** - Track popular packages, search trends

---

**Related Documentation:**
- [Architecture Overview](ARCHITECTURE.md)
- [Module Guide](MODULE_GUIDE.md)
- [Package Contribution Guide](https://github.com/limistah/heimdal-packages/blob/main/CONTRIBUTING.md)

**Source Code:**
- [Database Loader](../src/package/database/loader.rs)
- [Database Core](../src/package/database/core.rs)
- [Database Cache](../src/package/database/cache.rs)
