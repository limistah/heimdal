//! Integration tests for macOS defaults sync.
//!
//! Note: Most tests are macOS-only and will be skipped on other platforms.

#[cfg(target_os = "macos")]
mod macos_tests {
    use heimdal::config::DefaultsConfig;
    use heimdal::defaults::{
        diff_domain, export_all, get_defaults_dir, list_filtered_domains, plist_path_for_domain,
        should_include_domain,
    };
    use tempfile::TempDir;

    fn test_config() -> DefaultsConfig {
        DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        }
    }

    #[test]
    fn test_list_filtered_domains_returns_results() {
        let config = test_config();
        let domains = list_filtered_domains(&config).unwrap();
        // Should have at least some domains on a real macOS system
        assert!(!domains.is_empty());
    }

    #[test]
    fn test_list_filtered_domains_excludes_defaults() {
        let config = test_config();
        let domains = list_filtered_domains(&config).unwrap();
        // Should not include default-excluded domains
        assert!(!domains.contains(&"com.apple.Safari.SandboxBroker".to_string()));
    }

    #[test]
    fn test_list_filtered_domains_with_include_filter() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };
        let domains = list_filtered_domains(&config).unwrap();
        // When include list is set, only those domains should appear
        assert!(domains.iter().all(|d| d == "com.apple.dock"));
    }

    #[test]
    fn test_export_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()], // Just one domain for speed
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };

        let results = export_all(tmp.path(), &config, false).unwrap();

        // Should have created the directory
        let defaults_dir = get_defaults_dir(tmp.path(), &config);
        assert!(defaults_dir.exists());

        // Should have at least one result
        assert!(!results.is_empty());
    }

    #[test]
    fn test_export_creates_plist_file() {
        let tmp = TempDir::new().unwrap();
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };

        let results = export_all(tmp.path(), &config, false).unwrap();
        let defaults_dir = get_defaults_dir(tmp.path(), &config);

        // All successful exports should have plist files
        for result in &results {
            if result.success {
                let plist_path = plist_path_for_domain(&defaults_dir, &result.domain);
                assert!(
                    plist_path.exists(),
                    "Expected plist file: {}",
                    plist_path.display()
                );
            }
        }
    }

    #[test]
    fn test_export_dry_run_no_files() {
        let tmp = TempDir::new().unwrap();
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };

        let results = export_all(tmp.path(), &config, true).unwrap();

        // Dry run should succeed but create no files
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.success));

        let plist_path =
            plist_path_for_domain(&get_defaults_dir(tmp.path(), &config), "com.apple.dock");
        assert!(!plist_path.exists());
    }

    #[test]
    fn test_diff_domain_with_no_dotfiles() {
        let tmp = TempDir::new().unwrap();
        let config = test_config();

        // Diff a real domain with no exported file
        let diff = diff_domain("com.apple.dock", tmp.path(), &config).unwrap();

        // All keys should be OnlyLocal since there's no dotfiles version
        assert!(!diff.is_empty());
        assert!(!diff.has_conflicts()); // no conflicts when one side is missing entirely
    }

    #[test]
    fn test_diff_domain_with_exported_file() {
        let tmp = TempDir::new().unwrap();
        let config = DefaultsConfig {
            enabled: true,
            include: vec!["com.apple.dock".to_string()],
            exclude: vec![],
            path: "macos-defaults".to_string(),
        };

        // First export to get a plist file
        export_all(tmp.path(), &config, false).unwrap();

        // Now diff — should be empty (just exported, so identical)
        let diff = diff_domain("com.apple.dock", tmp.path(), &config).unwrap();
        assert!(diff.is_empty(), "Diff should be empty right after export");
    }

    #[test]
    fn test_should_include_domain_basic() {
        let config = test_config();
        assert!(should_include_domain("com.apple.dock", &config));
        assert!(should_include_domain("com.apple.finder", &config));
    }

    #[test]
    fn test_should_include_domain_default_excludes() {
        let config = test_config();
        assert!(!should_include_domain(
            "com.apple.Safari.SandboxBroker",
            &config
        ));
        assert!(!should_include_domain("com.apple.xpc.activity2", &config));
        assert!(!should_include_domain("MobileMeAccounts", &config));
    }

    #[test]
    fn test_should_include_domain_explicit_exclude() {
        let config = DefaultsConfig {
            enabled: true,
            include: vec![],
            exclude: vec!["com.apple.dock".to_string()],
            path: "macos-defaults".to_string(),
        };
        assert!(!should_include_domain("com.apple.dock", &config));
        assert!(should_include_domain("com.apple.finder", &config));
    }

    #[test]
    fn test_get_defaults_dir() {
        let config = test_config();
        let tmp = TempDir::new().unwrap();
        let dir = get_defaults_dir(tmp.path(), &config);
        assert_eq!(dir, tmp.path().join("macos-defaults"));
    }

    #[test]
    fn test_plist_path_for_domain() {
        use std::path::PathBuf;
        let defaults_dir = PathBuf::from("/dotfiles/macos-defaults");
        let path = plist_path_for_domain(&defaults_dir, "com.apple.dock");
        assert_eq!(
            path,
            PathBuf::from("/dotfiles/macos-defaults/com.apple.dock.plist")
        );
    }
}

// Tests that work on all platforms
#[test]
fn test_defaults_is_supported() {
    let supported = heimdal::defaults::is_supported();

    #[cfg(target_os = "macos")]
    assert!(supported);

    #[cfg(not(target_os = "macos"))]
    assert!(!supported);
}
