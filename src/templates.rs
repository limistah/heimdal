use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

// Matches {{ variable }}, {{ env.VAR }}, and {{ secret:name }}
static VAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{\s*([\w.:]+)\s*\}\}").unwrap());

/// Substitute {{ variable }} placeholders. Unknown vars are preserved + warned.
/// Also resolves {{ secret:name }} directly from the OS keychain.
pub fn render_string(content: &str, vars: &HashMap<String, String>) -> String {
    VAR_RE
        .replace_all(content, |caps: &regex::Captures| {
            let key = &caps[1];
            // Resolve secret: references directly from keychain
            if let Some(secret_name) = key.strip_prefix("secret:") {
                return match crate::secrets::get_secret(secret_name) {
                    Ok(val) => val,
                    Err(_) => {
                        crate::utils::warning(&format!(
                            "Secret '{}' not found in keychain — placeholder preserved",
                            secret_name
                        ));
                        caps[0].to_string()
                    }
                };
            }
            match vars.get(key) {
                Some(val) => val.clone(),
                None => {
                    crate::utils::warning(&format!("Undefined template variable: {}", key));
                    caps[0].to_string()
                }
            }
        })
        .to_string()
}

/// System variables: hostname, username, os, home
pub fn system_vars() -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("hostname".to_string(), crate::utils::hostname());
    vars.insert("username".to_string(), whoami::username());
    vars.insert("os".to_string(), crate::utils::os_name().to_string());
    vars.insert(
        "home".to_string(),
        dirs::home_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );
    vars
}

/// Build combined variable map. env_prefix is "env" so env vars are {{ env.HOME }}.
pub fn build_vars(explicit: &HashMap<String, String>, env_prefix: &str) -> HashMap<String, String> {
    let mut vars = system_vars();
    for (k, v) in std::env::vars() {
        vars.insert(format!("{}.{}", env_prefix, k), v);
    }
    // Explicit vars override system/env
    for (k, v) in explicit {
        vars.insert(k.clone(), v.clone());
    }
    // Resolve {{ secret:name }} values
    let keys: Vec<String> = vars.keys().cloned().collect();
    for k in keys {
        let v = vars[&k].clone();
        if let Some(secret_name) = v
            .trim()
            .strip_prefix("{{")
            .and_then(|s| s.strip_suffix("}}"))
            .map(str::trim)
            .and_then(|s| s.strip_prefix("secret:"))
            .map(str::trim)
        {
            match crate::secrets::get_secret(secret_name) {
                Ok(resolved) => {
                    vars.insert(k, resolved);
                }
                Err(_) => {
                    crate::utils::warning(&format!(
                        "Secret '{}' not found in keychain — variable '{}' left as placeholder",
                        secret_name, k
                    ));
                }
            }
        }
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_var_substitution_unchanged() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        let result = render_string("Hello {{ name }}", &vars);
        assert_eq!(result, "Hello Alice");
    }

    #[test]
    fn secret_placeholder_preserved_when_secret_not_found() {
        // A secret that doesn't exist in the keychain preserves the placeholder.
        let vars = std::collections::HashMap::new();
        let result = render_string(
            "email: {{ secret:_heimdal_test_nonexistent_secret_ }}",
            &vars,
        );
        assert!(result.contains("secret:_heimdal_test_nonexistent_secret_"));
    }
}

/// Render a template file to a destination.
pub fn render_file(
    src: &Path,
    dest: &Path,
    vars: &HashMap<String, String>,
    dry_run: bool,
) -> Result<()> {
    let content = std::fs::read_to_string(src)
        .map_err(|e| anyhow::anyhow!("Cannot read template '{}': {}", src.display(), e))?;
    let rendered = render_string(&content, vars);

    if dry_run {
        println!("--- [dry-run] Would write: {} ---", dest.display());
        print!("{}", rendered);
        return Ok(());
    }

    crate::utils::ensure_parent_exists(dest)?;
    std::fs::write(dest, rendered)?;
    Ok(())
}
