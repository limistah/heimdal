//! Compare local macOS defaults with exported plist files.

use crate::config::DefaultsConfig;
use crate::defaults::export::plist_path_for_domain;
use crate::defaults::paths::get_defaults_dir;
use anyhow::Result;
use defaults_rs::{Domain, PrefValue, Preferences};
use std::collections::HashMap;
use std::path::Path;

/// Represents a difference in a single key between local and dotfiles.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyDiff {
    /// Key exists only in local (system)
    OnlyLocal(PrefValue),
    /// Key exists only in dotfiles
    OnlyDotfiles(PrefValue),
    /// Key exists in both but values differ
    Changed {
        local: PrefValue,
        dotfiles: PrefValue,
    },
}

/// All differences for a single domain.
#[derive(Debug, Clone)]
pub struct DomainDiff {
    pub domain: String,
    pub keys: HashMap<String, KeyDiff>,
}

impl DomainDiff {
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn has_conflicts(&self) -> bool {
        self.keys
            .values()
            .any(|d| matches!(d, KeyDiff::Changed { .. }))
    }
}

/// Read a plist file and return it as a HashMap of key -> plist::Value.
fn read_plist_file(path: &Path) -> Result<HashMap<String, plist::Value>> {
    let value = plist::Value::from_file(path)?;
    match value {
        plist::Value::Dictionary(dict) => Ok(dict.into_iter().collect()),
        _ => Ok(HashMap::new()),
    }
}

/// Convert a plist::Value to a defaults_rs::PrefValue for comparison.
fn plist_to_prefvalue(value: &plist::Value) -> PrefValue {
    match value {
        plist::Value::Boolean(b) => PrefValue::Boolean(*b),
        plist::Value::Integer(i) => PrefValue::Integer(i.as_signed().unwrap_or(0)),
        plist::Value::Real(f) => PrefValue::Float(*f),
        plist::Value::String(s) => PrefValue::String(s.clone()),
        plist::Value::Data(d) => PrefValue::Data(d.clone().into()),
        plist::Value::Date(d) => {
            use std::time::{SystemTime, UNIX_EPOCH};
            let system_time = SystemTime::from(*d);
            let unix_secs = system_time
                .duration_since(UNIX_EPOCH)
                .map(|dur| dur.as_secs_f64())
                .unwrap_or(0.0);
            // Apple epoch starts Jan 1, 2001 (978307200 seconds after Unix epoch)
            PrefValue::Date(unix_secs - 978_307_200.0)
        }
        plist::Value::Array(arr) => PrefValue::Array(arr.iter().map(plist_to_prefvalue).collect()),
        plist::Value::Dictionary(dict) => PrefValue::Dictionary(
            dict.iter()
                .map(|(k, v)| (k.clone(), plist_to_prefvalue(v)))
                .collect(),
        ),
        _ => PrefValue::String("<unknown>".to_string()),
    }
}

/// Compare local system defaults with the exported plist file for one domain.
pub fn diff_domain(
    domain: &str,
    dotfiles_path: &Path,
    config: &DefaultsConfig,
) -> Result<DomainDiff> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);
    let plist_path = plist_path_for_domain(&defaults_dir, domain);

    // Read dotfiles version (may not exist)
    let dotfiles_data: HashMap<String, PrefValue> = if plist_path.exists() {
        read_plist_file(&plist_path)?
            .into_iter()
            .map(|(k, v)| (k, plist_to_prefvalue(&v)))
            .collect()
    } else {
        HashMap::new()
    };

    // Read local system version
    let local_data: HashMap<String, PrefValue> =
        match Preferences::read_domain(Domain::User(domain.to_string())) {
            Ok(PrefValue::Dictionary(dict)) => dict,
            _ => HashMap::new(),
        };

    // Collect all unique keys from both sources
    let all_keys: std::collections::HashSet<&String> =
        local_data.keys().chain(dotfiles_data.keys()).collect();

    let mut keys = HashMap::new();
    for key in all_keys {
        match (local_data.get(key), dotfiles_data.get(key)) {
            (Some(local), None) => {
                keys.insert(key.clone(), KeyDiff::OnlyLocal(local.clone()));
            }
            (None, Some(dotfiles)) => {
                keys.insert(key.clone(), KeyDiff::OnlyDotfiles(dotfiles.clone()));
            }
            (Some(local), Some(dotfiles)) => {
                if local != dotfiles {
                    keys.insert(
                        key.clone(),
                        KeyDiff::Changed {
                            local: local.clone(),
                            dotfiles: dotfiles.clone(),
                        },
                    );
                }
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(DomainDiff {
        domain: domain.to_string(),
        keys,
    })
}

/// Diff all domains that have exported plist files in the defaults directory.
pub fn diff_all(dotfiles_path: &Path, config: &DefaultsConfig) -> Result<Vec<DomainDiff>> {
    let defaults_dir = get_defaults_dir(dotfiles_path, config);
    if !defaults_dir.exists() {
        return Ok(vec![]);
    }

    let mut diffs = Vec::new();
    for entry in std::fs::read_dir(&defaults_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "plist").unwrap_or(false) {
            if let Some(stem) = path.file_stem() {
                let domain = stem.to_string_lossy().to_string();
                let diff = diff_domain(&domain, dotfiles_path, config)?;
                if !diff.is_empty() {
                    diffs.push(diff);
                }
            }
        }
    }
    diffs.sort_by(|a, b| a.domain.cmp(&b.domain));
    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_diff_is_empty() {
        let diff = DomainDiff {
            domain: "com.apple.dock".to_string(),
            keys: HashMap::new(),
        };
        assert!(diff.is_empty());
        assert!(!diff.has_conflicts());
    }

    #[test]
    fn test_domain_diff_has_conflicts() {
        let mut keys = HashMap::new();
        keys.insert(
            "autohide".to_string(),
            KeyDiff::Changed {
                local: PrefValue::Boolean(true),
                dotfiles: PrefValue::Boolean(false),
            },
        );
        let diff = DomainDiff {
            domain: "com.apple.dock".to_string(),
            keys,
        };
        assert!(!diff.is_empty());
        assert!(diff.has_conflicts());
    }

    #[test]
    fn test_domain_diff_no_conflicts_only_additions() {
        let mut keys = HashMap::new();
        keys.insert(
            "newkey".to_string(),
            KeyDiff::OnlyLocal(PrefValue::Boolean(true)),
        );
        let diff = DomainDiff {
            domain: "com.apple.dock".to_string(),
            keys,
        };
        assert!(!diff.is_empty());
        assert!(!diff.has_conflicts());
    }

    #[test]
    fn test_domain_diff_only_dotfiles() {
        let mut keys = HashMap::new();
        keys.insert(
            "removed_key".to_string(),
            KeyDiff::OnlyDotfiles(PrefValue::Integer(42)),
        );
        let diff = DomainDiff {
            domain: "com.apple.dock".to_string(),
            keys,
        };
        assert!(!diff.is_empty());
        assert!(!diff.has_conflicts());
    }
}
