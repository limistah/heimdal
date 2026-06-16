//! Domain discovery and filtering.

use crate::config::DefaultsConfig;
use anyhow::Result;
use defaults_rs::{Domain, Preferences};

/// Check if a domain should be included based on config filters.
///
/// Priority: explicit exclude > default exclude > explicit include > include-all
pub fn should_include_domain(domain: &str, config: &DefaultsConfig) -> bool {
    // Explicit exclude always wins
    if config.exclude.iter().any(|e| e == domain) {
        return false;
    }
    // Check default excludes
    if DEFAULT_EXCLUDE.iter().any(|&e| e == domain) {
        return false;
    }
    // If include list is specified, domain must be in it
    if !config.include.is_empty() {
        return config.include.iter().any(|i| i == domain);
    }
    true
}

/// List all user domains filtered by config.
pub fn list_filtered_domains(config: &DefaultsConfig) -> Result<Vec<String>> {
    let all_domains = Preferences::list_domains()?;
    let mut filtered: Vec<String> = all_domains
        .into_iter()
        .filter_map(|d| {
            if let Domain::User(name) = d {
                if should_include_domain(&name, config) {
                    return Some(name);
                }
            }
            None
        })
        .collect();
    filtered.sort();
    Ok(filtered)
}

/// Well-known domains to exclude by default (ephemeral/cache/internal).
const DEFAULT_EXCLUDE: &[&str] = &[
    "com.apple.Safari.SandboxBroker",
    "com.apple.security.cloudkeychainproxy3",
    "com.apple.security.cloudkeychainproxy3.keysToRegister",
    "com.apple.xpc.activity2",
    "com.apple.pluginkit.pkd",
    "com.apple.iBooksX",
    "com.apple.CoreSimulator.CoreSimulatorService",
    "com.apple.Spotlight",
    "com.apple.systempreferences.cache",
    "com.apple.LaunchServices",
    "com.apple.identityservices.idstatuscache",
    "MobileMeAccounts",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_include_domain_basic() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };
        assert!(should_include_domain("com.apple.dock", &config));
        assert!(should_include_domain("com.apple.finder", &config));
    }

    #[test]
    fn test_should_include_domain_with_default_exclude() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };
        assert!(!should_include_domain(
            "com.apple.Safari.SandboxBroker",
            &config
        ));
        assert!(!should_include_domain("com.apple.xpc.activity2", &config));
    }

    #[test]
    fn test_should_include_domain_with_explicit_include() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string(), "com.apple.finder".to_string()],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };
        assert!(should_include_domain("com.apple.dock", &config));
        assert!(should_include_domain("com.apple.finder", &config));
        assert!(!should_include_domain("com.apple.Safari", &config));
    }

    #[test]
    fn test_should_include_domain_with_explicit_exclude() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec!["com.apple.Safari".to_string()],
            path: "macos-defaults".to_string(),
        };
        assert!(should_include_domain("com.apple.dock", &config));
        assert!(!should_include_domain("com.apple.Safari", &config));
    }

    #[test]
    fn test_should_include_domain_exclude_takes_precedence() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()],
            exclude: vec!["com.apple.dock".to_string()],
            path: "macos-defaults".to_string(),
        };
        // Explicit exclude beats explicit include
        assert!(!should_include_domain("com.apple.dock", &config));
    }
}
