//! Interactive conflict resolution for defaults sync.

use crate::defaults::diff::{DomainDiff, KeyDiff};
use crate::utils::info;
use colored::Colorize;
use defaults_rs::PrefValue;

/// User's choice for resolving a domain conflict.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolution {
    /// Use local (system) values — export to dotfiles
    UseLocal,
    /// Use dotfiles values — import to system
    UseDotfiles,
    /// Skip this domain, make no changes
    Skip,
}

/// Format a PrefValue for display (truncated for large values).
pub fn format_pref_value(value: &PrefValue) -> String {
    match value {
        PrefValue::Boolean(b) => format!("{}", b),
        PrefValue::Integer(i) => format!("{}", i),
        PrefValue::Float(f) => format!("{:.4}", f),
        PrefValue::String(s) => {
            if s.len() > 50 {
                format!("\"{}...\"", &s[..47])
            } else {
                format!("\"{}\"", s)
            }
        }
        PrefValue::Data(d) => format!("<data: {} bytes>", d.len()),
        PrefValue::Date(ts) => format!("<date: {:.0}>", ts),
        PrefValue::Array(arr) => format!("<array: {} items>", arr.len()),
        PrefValue::Dictionary(dict) => format!("<dict: {} keys>", dict.len()),
        PrefValue::Url(u) => format!("<url: {}>", u),
        PrefValue::Uuid(u) => format!("<uuid: {}>", u),
        PrefValue::Uid(u) => format!("<uid: {}>", u),
    }
}

/// Print a summary of differences for a domain.
pub fn print_domain_diff(diff: &DomainDiff) {
    println!();
    println!("{}", format!("Domain: {}", diff.domain).bold());
    println!("{}", "-".repeat(60));

    let mut only_local: Vec<&String> = Vec::new();
    let mut only_dotfiles: Vec<&String> = Vec::new();
    let mut changed: Vec<&String> = Vec::new();

    for (key, diff_type) in &diff.keys {
        match diff_type {
            KeyDiff::OnlyLocal(_) => only_local.push(key),
            KeyDiff::OnlyDotfiles(_) => only_dotfiles.push(key),
            KeyDiff::Changed { .. } => changed.push(key),
        }
    }

    only_local.sort();
    only_dotfiles.sort();
    changed.sort();

    if !only_local.is_empty() {
        println!("{}", "  Only in local (system):".green());
        for key in &only_local {
            if let Some(KeyDiff::OnlyLocal(val)) = diff.keys.get(*key) {
                println!("    + {} = {}", key, format_pref_value(val));
            }
        }
    }

    if !only_dotfiles.is_empty() {
        println!("{}", "  Only in dotfiles:".yellow());
        for key in &only_dotfiles {
            if let Some(KeyDiff::OnlyDotfiles(val)) = diff.keys.get(*key) {
                println!("    - {} = {}", key, format_pref_value(val));
            }
        }
    }

    if !changed.is_empty() {
        println!("{}", "  Changed:".red());
        for key in &changed {
            if let Some(KeyDiff::Changed { local, dotfiles }) = diff.keys.get(*key) {
                println!("    ~ {}", key);
                println!("      Local:    {}", format_pref_value(local));
                println!("      Dotfiles: {}", format_pref_value(dotfiles));
            }
        }
    }

    println!();
}

/// Prompt user to resolve a domain conflict.
pub fn prompt_resolution(diff: &DomainDiff) -> Resolution {
    print_domain_diff(diff);

    let choices = vec![
        "Use Local (export to dotfiles)",
        "Use Dotfiles (import to system)",
        "Skip (no changes)",
    ];

    let selection = dialoguer::Select::new()
        .with_prompt("How do you want to resolve this domain?")
        .items(&choices)
        .default(0)
        .interact()
        .unwrap_or(2);

    match selection {
        0 => Resolution::UseLocal,
        1 => Resolution::UseDotfiles,
        _ => Resolution::Skip,
    }
}

/// Resolve all conflicting domains interactively.
pub fn resolve_all(diffs: Vec<DomainDiff>) -> Vec<(String, Resolution)> {
    if diffs.is_empty() {
        info("No differences found");
        return vec![];
    }

    let conflict_count = diffs.iter().filter(|d| d.has_conflicts()).count();
    let total = diffs.len();

    info(&format!(
        "Found {} domains with differences ({} with conflicts)",
        total, conflict_count
    ));

    let mut resolutions = Vec::new();

    for diff in diffs {
        let resolution = prompt_resolution(&diff);
        resolutions.push((diff.domain, resolution));
    }

    resolutions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resolution_variants() {
        assert_ne!(Resolution::UseLocal, Resolution::UseDotfiles);
        assert_ne!(Resolution::UseLocal, Resolution::Skip);
        assert_ne!(Resolution::UseDotfiles, Resolution::Skip);
    }

    #[test]
    fn test_format_pref_value_bool() {
        let formatted = format_pref_value(&PrefValue::Boolean(true));
        assert!(formatted.contains("true"));
    }

    #[test]
    fn test_format_pref_value_bool_false() {
        let formatted = format_pref_value(&PrefValue::Boolean(false));
        assert!(formatted.contains("false"));
    }

    #[test]
    fn test_format_pref_value_string() {
        let formatted = format_pref_value(&PrefValue::String("hello".to_string()));
        assert!(formatted.contains("hello"));
    }

    #[test]
    fn test_format_pref_value_int() {
        let formatted = format_pref_value(&PrefValue::Integer(42));
        assert!(formatted.contains("42"));
    }

    #[test]
    fn test_format_pref_value_float() {
        let formatted = format_pref_value(&PrefValue::Float(3.14));
        assert!(formatted.contains("3.14"));
    }

    #[test]
    fn test_format_pref_value_long_string_truncated() {
        let long = "a".repeat(60);
        let formatted = format_pref_value(&PrefValue::String(long));
        assert!(formatted.contains("..."));
        // should be capped at < 60 chars visible + quotes + ellipsis
        assert!(formatted.len() < 60 + 10);
    }

    #[test]
    fn test_format_pref_value_array() {
        let arr = vec![PrefValue::Boolean(true), PrefValue::Integer(1)];
        let formatted = format_pref_value(&PrefValue::Array(arr));
        assert!(formatted.contains("2 items"));
    }

    #[test]
    fn test_format_pref_value_dict() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), PrefValue::Boolean(true));
        let formatted = format_pref_value(&PrefValue::Dictionary(map));
        assert!(formatted.contains("1 keys"));
    }
}
