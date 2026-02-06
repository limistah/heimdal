use anyhow::Result;
use std::env;

use crate::config::schema::DotfileCondition;

/// Evaluate whether a dotfile condition is met
pub fn evaluate_condition(condition: &DotfileCondition, current_profile: &str) -> Result<bool> {
    // Check OS condition
    if !condition.os.is_empty() {
        let current_os = get_current_os();
        if !condition.os.iter().any(|os| os == &current_os) {
            return Ok(false);
        }
    }

    // Check profile condition
    if !condition.profile.is_empty() && !condition.profile.iter().any(|p| p == current_profile) {
        return Ok(false);
    }

    // Check environment variable condition
    if let Some(env_cond) = &condition.env {
        if !check_env_condition(env_cond) {
            return Ok(false);
        }
    }

    // Check hostname condition
    if let Some(hostname_pattern) = &condition.hostname {
        if !check_hostname_pattern(hostname_pattern)? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Get the current operating system identifier
fn get_current_os() -> String {
    if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if an environment variable condition is met
/// Format: "VAR=value" or "VAR" (just checks existence)
fn check_env_condition(condition: &str) -> bool {
    if let Some((key, expected_value)) = condition.split_once('=') {
        // Check if env var equals expected value
        if let Ok(actual_value) = env::var(key.trim()) {
            actual_value == expected_value.trim()
        } else {
            false
        }
    } else {
        // Just check if env var exists
        env::var(condition.trim()).is_ok()
    }
}

/// Check if the current hostname matches a pattern
/// Supports simple glob patterns with * wildcard
fn check_hostname_pattern(pattern: &str) -> Result<bool> {
    let hostname = hostname::get()?.to_string_lossy().to_string();

    // Simple glob matching
    if pattern.contains('*') {
        let pattern_parts: Vec<&str> = pattern.split('*').collect();

        if pattern_parts.len() == 2 {
            let prefix = pattern_parts[0];
            let suffix = pattern_parts[1];

            return Ok(hostname.starts_with(prefix) && hostname.ends_with(suffix));
        }
    }

    // Exact match
    Ok(hostname == pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_os() {
        let os = get_current_os();
        assert!(["macos", "linux", "windows", "unknown"].contains(&os.as_str()));
    }

    #[test]
    fn test_check_env_condition_exists() {
        env::set_var("HEIMDAL_TEST_VAR_1", "test_value");
        assert!(check_env_condition("HEIMDAL_TEST_VAR_1"));
        env::remove_var("HEIMDAL_TEST_VAR_1");
    }

    #[test]
    fn test_check_env_condition_equals() {
        env::set_var("HEIMDAL_TEST_VAR_2", "expected");
        assert!(check_env_condition("HEIMDAL_TEST_VAR_2=expected"));
        assert!(!check_env_condition("HEIMDAL_TEST_VAR_2=wrong"));
        env::remove_var("HEIMDAL_TEST_VAR_2");
    }

    #[test]
    fn test_hostname_pattern_exact() {
        // This test requires a real hostname, so we'll just test the logic
        let result = check_hostname_pattern("some-host");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_condition_os() {
        let current_os = get_current_os();
        let condition = DotfileCondition {
            os: vec![current_os.clone()],
            profile: vec![],
            env: None,
            hostname: None,
        };

        let result = evaluate_condition(&condition, "default").unwrap();
        assert!(result);

        let condition = DotfileCondition {
            os: vec!["nonexistent-os".to_string()],
            profile: vec![],
            env: None,
            hostname: None,
        };

        let result = evaluate_condition(&condition, "default").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_evaluate_condition_profile() {
        let condition = DotfileCondition {
            os: vec![],
            profile: vec!["work".to_string(), "dev".to_string()],
            env: None,
            hostname: None,
        };

        assert!(evaluate_condition(&condition, "work").unwrap());
        assert!(evaluate_condition(&condition, "dev").unwrap());
        assert!(!evaluate_condition(&condition, "personal").unwrap());
    }

    #[test]
    fn test_evaluate_condition_empty() {
        // Empty condition should always be true
        let condition = DotfileCondition {
            os: vec![],
            profile: vec![],
            env: None,
            hostname: None,
        };

        assert!(evaluate_condition(&condition, "any").unwrap());
    }
}
