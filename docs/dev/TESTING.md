# Testing Guide

> **Status:** This document is being developed as part of the documentation overhaul (Week 3).

This guide explains how to write, run, and maintain tests for Heimdal.

## Table of Contents

1. [Testing Philosophy](#testing-philosophy)
2. [Running Tests](#running-tests)
3. [Test Organization](#test-organization)
4. [Writing Tests](#writing-tests)
5. [Test Patterns](#test-patterns)
6. [Continuous Integration](#continuous-integration)

## Testing Philosophy

Heimdal follows these testing principles:

- **Unit tests** - Test individual functions and modules in isolation
- **Integration tests** - Test interactions between modules
- **No flaky tests** - Tests must be deterministic and reliable
- **Fast tests** - Test suite should complete in under 2 minutes
- **Readable tests** - Tests should serve as documentation

## Running Tests

### Run All Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4
```

### Run Specific Tests

```bash
# Run tests in a specific module
cargo test package::database

# Run a specific test
cargo test test_package_search

# Run tests matching a pattern
cargo test search
```

### Run with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## Test Organization

### Unit Tests

Located in the same file as the code they test, using `#[cfg(test)]`:

```rust
// src/package/mapper.rs
pub fn map_package_name(name: &str, platform: &str) -> Option<String> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_package_name() {
        assert_eq!(
            map_package_name("nodejs", "brew"),
            Some("node".to_string())
        );
    }
}
```

### Integration Tests

Located in `tests/` directory at the project root:

```
tests/
├── integration_test.rs
├── package_management_test.rs
└── wizard_test.rs
```

### Test Fixtures

Located in test-specific directories:

```
test-dotfiles/          # Sample dotfiles for testing
test-heimdal.yaml       # Sample config for testing
```

## Writing Tests

### Basic Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange - Set up test data
        let input = "test";
        
        // Act - Execute the function
        let result = function_to_test(input);
        
        // Assert - Verify the result
        assert_eq!(result, expected_value);
    }
}
```

### Testing with Results

```rust
#[test]
fn test_error_handling() -> Result<()> {
    let config = load_config(Path::new("invalid.yaml"))?;
    assert!(config.is_valid());
    Ok(())
}
```

### Testing Panics

```rust
#[test]
#[should_panic(expected = "Invalid configuration")]
fn test_invalid_config_panics() {
    validate_config(&invalid_config);
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Test Patterns

### Mocking External Dependencies

Use dependency injection to mock external systems:

```rust
pub trait PackageManager {
    fn install(&self, package: &str) -> Result<()>;
}

pub struct RealPackageManager;
impl PackageManager for RealPackageManager {
    fn install(&self, package: &str) -> Result<()> {
        // Real implementation
    }
}

pub struct MockPackageManager {
    pub installed: Vec<String>,
}
impl PackageManager for MockPackageManager {
    fn install(&self, package: &str) -> Result<()> {
        // Mock implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_install_with_mock() {
        let mut mock = MockPackageManager { installed: vec![] };
        let result = install_package(&mock, "neovim");
        assert!(result.is_ok());
    }
}
```

### Testing with Temporary Files

```rust
use tempfile::TempDir;

#[test]
fn test_file_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("heimdal.yaml");
    
    // Write test file
    fs::write(&config_path, "test: value")?;
    
    // Test function
    let config = load_config(&config_path)?;
    
    // temp_dir is automatically cleaned up
    Ok(())
}
```

### Testing Environment Variables

```rust
#[test]
fn test_env_variable() {
    std::env::set_var("HEIMDAL_TEST", "value");
    let result = get_env_config();
    assert_eq!(result, Some("value"));
    std::env::remove_var("HEIMDAL_TEST");
}
```

### Parameterized Tests

```rust
#[test]
fn test_package_search() {
    let test_cases = vec![
        ("neovim", Some("neovim")),
        ("neovom", Some("neovim")), // Fuzzy match
        ("unknown", None),
    ];
    
    for (input, expected) in test_cases {
        let result = search_package(input);
        assert_eq!(result, expected);
    }
}
```

## Test Coverage

### Current Coverage Status

```
Module                   Coverage
------------------------  --------
package/database         95%
config                   90%
state                    85%
symlink                  80%
wizard                   75%
git                      70%
Overall                  82%
```

### Improving Coverage

Focus on:
1. Error paths
2. Edge cases
3. Platform-specific code
4. Integration between modules

## Continuous Integration

### GitHub Actions Workflow

Tests run automatically on:
- Every push to `main` and `dev`
- Every pull request
- Nightly builds

### CI Test Matrix

Tests run on multiple platforms:
- **OS:** Ubuntu, macOS, Windows
- **Rust:** Stable, Beta, Nightly

### Failing Tests in CI

Common CI-specific issues:

1. **Network Tests** - May fail due to rate limiting
   - Solution: Add test fallback database
   
2. **File System Tests** - May fail due to permissions
   - Solution: Use temp directories with proper cleanup
   
3. **Environment Tests** - May fail due to missing tools
   - Solution: Check for tool availability, skip if missing

### Example CI-Safe Test

```rust
#[test]
fn test_package_database() {
    let db = match PackageDatabase::new() {
        Ok(db) => db,
        Err(_) => {
            // Fallback for CI environment
            PackageDatabase::populate_test_fallback()
        }
    };
    
    let result = db.search("neovim");
    assert!(!result.is_empty());
}
```

## Test Utilities

### Helper Functions

Create reusable test helpers:

```rust
#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    pub fn create_test_config() -> Config {
        Config {
            global: GlobalConfig::default(),
            profiles: HashMap::new(),
        }
    }
    
    pub fn setup_test_environment() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        // Set up test fixtures
        temp_dir
    }
}
```

### Test Fixtures

```rust
#[cfg(test)]
const TEST_CONFIG: &str = r#"
global:
  dotfiles_dir: ~/.dotfiles
profiles:
  test:
    packages: ["git", "vim"]
"#;

#[test]
fn test_config_parsing() {
    let config: Config = serde_yaml::from_str(TEST_CONFIG).unwrap();
    assert_eq!(config.profiles.len(), 1);
}
```

## Best Practices

### Do's
- ✅ Write tests for new features
- ✅ Test error paths
- ✅ Use descriptive test names
- ✅ Keep tests focused and simple
- ✅ Clean up resources (files, env vars)
- ✅ Use assertions with helpful messages

### Don'ts
- ❌ Don't test external services directly
- ❌ Don't write flaky tests
- ❌ Don't use sleep/delays
- ❌ Don't share state between tests
- ❌ Don't commit commented-out tests

## Debugging Tests

### Run a Single Test with Output

```bash
cargo test test_name -- --nocapture
```

### Run Tests with Backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

### Run Tests with Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Known Test Issues

### Ignored Tests

Some tests are ignored due to:
- Requiring specific OS features
- Needing external services
- Being slow (benchmarks)

Run ignored tests with:
```bash
cargo test -- --ignored
```

### Platform-Specific Tests

```rust
#[test]
#[cfg(target_os = "macos")]
fn test_keychain_macos() {
    // macOS-specific test
}

#[test]
#[cfg(target_os = "linux")]
fn test_keychain_linux() {
    // Linux-specific test
}
```

## Future Improvements

This section will be expanded with:
- Property-based testing (proptest)
- Mutation testing
- Performance benchmarks
- End-to-end testing strategy

---

**Related Documentation:**
- [Contributing Guide](CONTRIBUTING.md)
- [Architecture Overview](../ARCHITECTURE.md)
- [Module Guide](../MODULE_GUIDE.md)

**Running Tests:**
```bash
# Quick test run
cargo test

# Full test suite with coverage
cargo tarpaulin

# CI-style test (matches GitHub Actions)
cargo test --all-targets --all-features
cargo clippy --all-targets
```
