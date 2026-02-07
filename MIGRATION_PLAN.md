# Migration Plan: Heimdal Package Database Externalization

**Status**: In Progress  
**Started**: February 7, 2026  
**Target Completion**: February 28, 2026

## Overview

Migrate ~2,500 lines of embedded static package data from Heimdal codebase to external YAML-based database (`heimdal-packages` repository).

## Progress Tracker

### Phase 1: Infrastructure ✅ COMPLETED
- [x] Create `heimdal-packages` repository
- [x] Define JSON schemas for validation
- [x] Write YAML → Bincode compiler
- [x] Create initial directory structure
- [x] Add example packages (neovim, git)
- [x] Add example group (web-dev)
- [x] Write documentation (README, CONTRIBUTING, ARCHITECTURE)

### Phase 2: Data Migration 🔄 IN PROGRESS
- [ ] **Priority 1: Packages** (40+ items)
  - [x] neovim.yaml
  - [x] git.yaml
  - [ ] Remaining 38 packages from `src/package/database.rs`
  
- [ ] **Priority 2: Mappings** (80+ items)
  - [ ] Extract from `src/package/mapper.rs` to `mappings/*.yaml`
  
- [ ] **Priority 3: Dependencies** (50+ items)
  - [ ] Extract from `src/package/dependencies.rs` to `dependencies/*.yaml`
  
- [ ] **Priority 4: Groups** (15 items)
  - [x] web-dev.yaml
  - [ ] Remaining 14 groups from `src/package/groups.rs`
  
- [ ] **Priority 5: Profiles** (10 items)
  - [ ] Extract from `src/package/profiles.rs` to `profiles/*.yaml`
  
- [ ] **Priority 6: Suggestions** (15+ items)
  - [ ] Extract from `src/package/suggestions.rs` to `suggestions/*.yaml`

### Phase 3: Heimdal Code Updates 📋 PENDING
- [ ] Add database loader module
  - [ ] `src/package/database/loader.rs`
  - [ ] `src/package/database/updater.rs`
  - [ ] `src/package/database/cache.rs`
  
- [ ] Update all code references
  - [ ] `src/commands/packages/search.rs`
  - [ ] `src/commands/packages/suggest.rs`
  - [ ] `src/wizard/mod.rs`
  - [ ] `src/package/mod.rs`
  
- [ ] Add `heimdal packages update` command
- [ ] Integrate auto-update during `heimdal sync`
- [ ] Remove embedded database from code

### Phase 4: Testing & Release 📋 PENDING
- [ ] Integration testing
- [ ] First database release (v1.0.0)
- [ ] Update Heimdal to use external database
- [ ] Migration guide for users
- [ ] Release notes

## Current Status: Phase 2 - Data Migration

### Next Steps (This Week)
1. ✅ Complete critical code fixes (unwrap() calls)
2. 🔄 Migrate remaining 38 packages to YAML
3. 🔄 Migrate package mappings
4. 🔄 Migrate dependencies
5. ⏳ Compile first complete database

### Blockers
None currently.

### Files Modified
- ✅ `src/main.rs` - Fixed unwrap() calls
- ⏳ `src/package/database.rs` - Will be replaced by loader
- ⏳ `src/package/mapper.rs` - Data to be extracted
- ⏳ `src/package/dependencies.rs` - Data to be extracted
- ⏳ `src/package/groups.rs` - Data to be extracted

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| Week 1 (Feb 7-14) | Infrastructure + Data Migration | ✅ Repo setup, 🔄 50% data migrated |
| Week 2 (Feb 14-21) | Complete Data Migration | 100% data in YAML, first compiled DB |
| Week 3 (Feb 21-28) | Code Updates | Database loader, update command |
| Week 4 (Feb 28-Mar 7) | Testing & Polish | Integration tests, release v1.0.0 |

## Success Metrics

- [ ] Binary size reduced by 300-400 KB
- [ ] All 40+ packages in YAML format
- [ ] Database compiles successfully
- [ ] Heimdal can download and use database
- [ ] Auto-update works during sync
- [ ] Zero breaking changes for users

## Links

- Main Review: [DISTINGUISHED_ENGINEER_REVIEW.md](./DISTINGUISHED_ENGINEER_REVIEW.md)
- Package Repo: `../heimdal-packages/`
- Architecture Doc: `../heimdal-packages/ARCHITECTURE.md`
