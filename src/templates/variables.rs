use std::collections::HashMap;
use std::env;

/// Get system-provided variables that are always available
pub fn get_system_variables() -> HashMap<String, String> {
    let mut vars = HashMap::new();

    // System information
    vars.insert("os".to_string(), env::consts::OS.to_string());
    vars.insert("arch".to_string(), env::consts::ARCH.to_string());
    vars.insert("family".to_string(), env::consts::FAMILY.to_string());

    // User information
    if let Ok(user) = env::var("USER") {
        vars.insert("user".to_string(), user);
    } else if let Ok(username) = env::var("USERNAME") {
        // Windows fallback
        vars.insert("user".to_string(), username);
    }

    // Home directory
    if let Some(home_dir) = dirs::home_dir() {
        vars.insert("home".to_string(), home_dir.display().to_string());
    }

    // Hostname
    match hostname::get() {
        Ok(hostname) => {
            if let Some(hostname_str) = hostname.to_str() {
                vars.insert("hostname".to_string(), hostname_str.to_string());
            } else {
                eprintln!(
                    "Warning: Failed to convert hostname to UTF-8 string; 'hostname' variable will not be set."
                );
            }
        }
        Err(err) => {
            eprintln!(
                "Warning: Failed to retrieve hostname: {}; 'hostname' variable will not be set.",
                err
            );
        }
    }

    vars
}

/// Merge multiple variable maps with priority
/// Priority: profile > config > system (profile variables override config, which override system)
pub fn merge_variables(
    system: HashMap<String, String>,
    config: HashMap<String, String>,
    profile: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = system;
    merged.extend(config);
    merged.extend(profile); // Profile has highest priority
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_variables() {
        let vars = get_system_variables();

        // These should always be available
        assert!(vars.contains_key("os"));
        assert!(vars.contains_key("arch"));
        assert!(vars.contains_key("family"));

        // os should be one of the known values
        let os = vars.get("os").unwrap();
        assert!(matches!(
            os.as_str(),
            "linux" | "macos" | "windows" | "freebsd" | "openbsd"
        ));

        // arch should be one of the known values
        let arch = vars.get("arch").unwrap();
        assert!(matches!(
            arch.as_str(),
            "x86_64" | "aarch64" | "x86" | "arm"
        ));
    }

    #[test]
    fn test_merge_variables() {
        let mut system = HashMap::new();
        system.insert("os".to_string(), "linux".to_string());
        system.insert("user".to_string(), "system_user".to_string());

        let mut config = HashMap::new();
        config.insert("user".to_string(), "config_user".to_string());
        config.insert("email".to_string(), "config@example.com".to_string());

        let mut profile = HashMap::new();
        profile.insert("user".to_string(), "profile_user".to_string());
        profile.insert("name".to_string(), "Profile Name".to_string());

        let merged = merge_variables(system, config, profile);

        // Profile should override config and system
        assert_eq!(merged.get("user").unwrap(), "profile_user");
        // Config should override system
        assert_eq!(merged.get("email").unwrap(), "config@example.com");
        // System value should be present if not overridden
        assert_eq!(merged.get("os").unwrap(), "linux");
        // Profile-only value should be present
        assert_eq!(merged.get("name").unwrap(), "Profile Name");
    }

    #[test]
    fn test_merge_variables_empty() {
        let system = get_system_variables();
        let config = HashMap::new();
        let profile = HashMap::new();

        let merged = merge_variables(system.clone(), config, profile);

        // Should have all system variables
        assert_eq!(merged.len(), system.len());
    }
}
