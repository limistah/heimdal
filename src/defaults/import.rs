//! Import plist files back to macOS defaults.

use crate::config::DefaultsConfig;
use crate::defaults::export::plist_path_for_domain;
use crate::defaults::paths::get_defaults_dir;
use crate::utils::{step, verbose};
use anyhow::Result;
use defaults_rs::{Domain, Preferences};
use std::path::Path;

/// Result of importing a single domain.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportResult {
    pub domain: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Import a single domain from a plist file into the system.
fn import_domain(domain: &str, defaults_dir: &Path, dry_run: bool) -> ImportResult {
    let path = plist_path_for_domain(defaults_dir, domain);

    if !path.exists() {
        return ImportResult {
            domain: domain.to_string(),
            success: false,
            error: Some(format!("Plist file not found: {}", path.display())),
        };
    }

    if dry_run {
        return ImportResult {
            domain: domain.to_string(),
            success: true,
            error: None,
        };
    }

    let path_str = match path.to_str() {
        Some(s) => s,
        None => {
            return ImportResult {
                domain: domain.to_string(),
                success: false,
                error: Some("import path contains non-UTF-8 characters".to_string()),
            }
        }
    };

    match Preferences::import(Domain::User(domain.to_string()), path_str) {
        Ok(()) => ImportResult {
            domain: domain.to_string(),
            success: true,
            error: None,
        },
        Err(e) => ImportResult {
            domain: domain.to_string(),
            success: false,
            error: Some(e.to_string()),
        },
    }
}

/// Import specific domains from dotfiles plist files into the system.
pub fn import_domains(
    dotfiles_path: &Path,
    config: &DefaultsConfig,
    domains: &[String],
    dry_run: bool,
) -> Result<Vec<ImportResult>> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);

    let mut results = Vec::new();

    for domain in domains {
        verbose(&format!("Importing: {}", domain));
        let result = import_domain(domain, &defaults_dir, dry_run);

        if !result.success {
            if let Some(ref err) = result.error {
                verbose(&format!("  Failed: {}", err));
            }
        }

        results.push(result);
    }

    let success_count = results.iter().filter(|r| r.success).count();
    step(&format!("Imported {} domains", success_count));

    Ok(results)
}

/// Import all domains that have exported plist files in the defaults directory.
pub fn import_all(
    dotfiles_path: &Path,
    config: &DefaultsConfig,
    dry_run: bool,
) -> Result<Vec<ImportResult>> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);

    if !defaults_dir.exists() {
        return Ok(vec![]);
    }

    let mut domains = Vec::new();

    for entry in std::fs::read_dir(&defaults_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "plist").unwrap_or(false) {
            if let Some(stem) = path.file_stem() {
                domains.push(stem.to_string_lossy().to_string());
            }
        }
    }

    domains.sort();
    import_domains(dotfiles_path, config, &domains, dry_run)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_result_success() {
        let result = ImportResult {
            domain: "com.apple.dock".to_string(),
            success: true,
            error: None,
        };
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_import_result_with_error() {
        let result = ImportResult {
            domain: "com.apple.dock".to_string(),
            success: false,
            error: Some("Permission denied".to_string()),
        };
        assert!(!result.success);
        assert_eq!(result.error, Some("Permission denied".to_string()));
    }

    #[test]
    fn test_import_domain_missing_file() {
        use std::path::PathBuf;
        let tmp_dir = PathBuf::from("/tmp/nonexistent_heimdal_test_dir");
        let result = import_domain("com.apple.dock", &tmp_dir, false);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Plist file not found"));
    }
}
