//! Export macOS defaults to plist files.

use crate::config::DefaultsConfig;
use crate::defaults::domains::{list_filtered_domains, should_include_domain};
use crate::defaults::paths::get_defaults_dir;
use crate::utils::{info, step, verbose};
use anyhow::Result;
use defaults_rs::{Domain, Preferences};
use std::path::{Path, PathBuf};

/// Result of exporting a single domain.
#[derive(Debug, Clone)]
pub struct ExportResult {
    pub domain: String,
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

/// Get the plist file path for a domain within the defaults directory.
pub fn plist_path_for_domain(defaults_dir: &Path, domain: &str) -> PathBuf {
    defaults_dir.join(format!("{}.plist", domain))
}

/// Export a single domain to a plist file. Returns ExportResult.
fn export_domain(domain: &str, defaults_dir: &Path, dry_run: bool) -> ExportResult {
    let path = plist_path_for_domain(defaults_dir, domain);
    if dry_run {
        return ExportResult {
            domain: domain.to_string(),
            path,
            success: true,
            error: None,
        };
    }
    let path_str = match path.to_str() {
        Some(s) => s,
        None => {
            return ExportResult {
                domain: domain.to_string(),
                path,
                success: false,
                error: Some("export path contains non-UTF-8 characters".to_string()),
            }
        }
    };
    match Preferences::export(Domain::User(domain.to_string()), path_str) {
        Ok(()) => ExportResult {
            domain: domain.to_string(),
            path,
            success: true,
            error: None,
        },
        Err(e) => ExportResult {
            domain: domain.to_string(),
            path,
            success: false,
            error: Some(e.to_string()),
        },
    }
}

/// Export all matching domains to the defaults directory.
pub fn export_all(
    dotfiles_path: &Path,
    config: &DefaultsConfig,
    dry_run: bool,
) -> Result<Vec<ExportResult>> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);
    if !dry_run {
        std::fs::create_dir_all(&defaults_dir)?;
    }
    let domains = list_filtered_domains(config)?;
    verbose(&format!("Found {} domains to export", domains.len()));
    let mut results = Vec::new();
    for domain in &domains {
        verbose(&format!("Exporting: {}", domain));
        let result = export_domain(domain, &defaults_dir, dry_run);
        if !result.success {
            if let Some(ref err) = result.error {
                verbose(&format!("  Failed: {}", err));
            }
        }
        results.push(result);
    }
    let success_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.len() - success_count;
    if fail_count > 0 {
        info(&format!(
            "Exported {} domains ({} failed)",
            success_count, fail_count
        ));
    } else {
        step(&format!("Exported {} domains", success_count));
    }
    Ok(results)
}

/// Export specific domains by name.
pub fn export_domains(
    dotfiles_path: &Path,
    config: &DefaultsConfig,
    domains: &[String],
    dry_run: bool,
) -> Result<Vec<ExportResult>> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);
    if !dry_run {
        std::fs::create_dir_all(&defaults_dir)?;
    }
    let mut results = Vec::new();
    for domain in domains {
        if !should_include_domain(domain, config) {
            verbose(&format!("Skipping excluded domain: {}", domain));
            continue;
        }
        verbose(&format!("Exporting: {}", domain));
        let result = export_domain(domain, &defaults_dir, dry_run);
        results.push(result);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plist_path_for_domain() {
        let defaults_dir = PathBuf::from("/dotfiles/macos-defaults");
        let result = plist_path_for_domain(&defaults_dir, "com.apple.dock");
        assert_eq!(
            result,
            PathBuf::from("/dotfiles/macos-defaults/com.apple.dock.plist")
        );
    }

    #[test]
    fn test_plist_path_for_domain_nested() {
        let defaults_dir = PathBuf::from("/home/user/.dotfiles/prefs");
        let result = plist_path_for_domain(&defaults_dir, "com.googlecode.iterm2");
        assert_eq!(
            result,
            PathBuf::from("/home/user/.dotfiles/prefs/com.googlecode.iterm2.plist")
        );
    }

    #[test]
    fn test_export_result_success() {
        let result = ExportResult {
            domain: "com.apple.dock".to_string(),
            path: PathBuf::from("/test/com.apple.dock.plist"),
            success: true,
            error: None,
        };
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_export_result_failure() {
        let result = ExportResult {
            domain: "com.apple.dock".to_string(),
            path: PathBuf::from("/test/com.apple.dock.plist"),
            success: false,
            error: Some("Permission denied".to_string()),
        };
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
