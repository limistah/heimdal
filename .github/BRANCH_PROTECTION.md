# Branch Protection Rules

This document describes the required branch protection rules for the heimdal repository.

## Setup Instructions

Go to: `https://github.com/limistah/heimdal/settings/branches`

## Main Branch Protection

### Required Settings for `main` branch:

1. **Require a pull request before merging**
   - ✅ Enable
   - Require approvals: 1 (recommended)
   - Dismiss stale pull request approvals when new commits are pushed: ✅

2. **Require status checks to pass before merging**
   - ✅ Enable
   - Require branches to be up to date before merging: ✅
   - Status checks that are required:
     - `Test (ubuntu-latest, stable)`
     - `Test (macos-latest, stable)`
     - `Build (ubuntu-latest)`
     - `Build (macos-latest)`

3. **Require conversation resolution before merging**
   - ✅ Enable

4. **Require linear history**
   - ✅ Enable (prevents merge commits, requires rebase or squash)

5. **Do not allow bypassing the above settings**
   - ✅ Enable

6. **Restrict who can push to matching branches**
   - ✅ Enable
   - Allowed to push: (empty - no one can push directly)
   - Include administrators: ❌ (admins also must follow rules)

7. **Allow force pushes**
   - ❌ Disable (CRITICAL: prevents `git push --force`)

8. **Allow deletions**
   - ❌ Disable

## Dev Branch Protection

### Required Settings for `dev` branch:

1. **Require status checks to pass before merging**
   - ✅ Enable
   - Require branches to be up to date before merging: ✅
   - Status checks that are required:
     - `Test (ubuntu-latest, stable)`
     - `Test (macos-latest, stable)`

2. **Require linear history**
   - ✅ Enable

3. **Allow force pushes**
   - ❌ Disable

4. **Allow deletions**
   - ❌ Disable

## Workflow

```
feature-branch → dev → main
     ↓          ↓      ↓
   commits    PR+CI  PR+CI
              (auto) (manual)
```

### Development Flow:

1. **Feature Development**
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b feature/my-feature
   # Make changes
   git commit -m "feat: add new feature"
   git push origin feature/my-feature
   ```

2. **Create PR to dev**
   - CI runs automatically (tests on Ubuntu + macOS)
   - Requires all checks to pass
   - Auto-merge to `dev` after approval (optional)

3. **Release to main**
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b release/v1.x.x
   # Update version in Cargo.toml
   git push origin release/v1.x.x
   ```
   - Create PR from `release/v1.x.x` → `main`
   - CI runs full build matrix (4 platforms)
   - Requires manual approval
   - Triggers release workflow on merge

## Enforcement

- ❌ **NEVER** `git push --force` to `main` or `dev`
- ❌ **NEVER** push directly to `main`
- ✅ **ALWAYS** use PRs for merging to `main`
- ✅ **ALWAYS** ensure CI passes before merging
- ✅ `dev` is the default branch for development
- ✅ `main` is protected for releases only

## CI Checks Required

All PRs must pass:

### For PRs to `dev`:
1. **Rust Formatting** (`cargo fmt --check`)
2. **Clippy Lints** (`cargo clippy -- -D warnings`)
3. **Build** (`cargo build --verbose`)
4. **Tests** (`cargo test --verbose`)
5. Run on: Ubuntu + macOS

### For PRs to `main`:
All of the above, plus:
6. **Multi-platform Builds**:
   - `x86_64-unknown-linux-gnu`
   - `x86_64-unknown-linux-musl`
   - `x86_64-apple-darwin`
   - `aarch64-apple-darwin`
7. **Release Artifacts** generated

## Special Considerations

### Database Loading
- Heimdal loads package database from `heimdal-packages` repo
- Ensure `heimdal-packages` releases are available before deploying
- Test database loading in CI (auto-downloads on first run)

### Version Bumping
When creating a release PR:
1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`
3. Tag after merging: `git tag v1.x.x`
4. Release workflow creates GitHub release automatically

## Violations

If branch protection is bypassed:

1. Revert the commit immediately
2. Investigate how it happened
3. Review and strengthen protection rules
4. Document the incident

## Dependencies Between Repositories

```
heimdal-packages (database) → heimdal (consumer)
```

**Important**: 
- Changes to `heimdal-packages` database schema require coordinated release
- Test database loading in heimdal CI before releasing
- Consider versioning for backward compatibility

---

**Important**: These rules ensure code quality, prevent breaking changes, and maintain a clean git history.
