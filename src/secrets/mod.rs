pub mod store;

pub use store::SecretStore;

use anyhow::Result;
use std::collections::HashMap;

/// Get all secrets as template variables
/// Returns a HashMap with secret names as keys
pub fn get_secret_variables() -> Result<HashMap<String, String>> {
    let store = SecretStore::new()?;
    let secrets = store.list()?;

    let mut vars = HashMap::new();
    for secret in secrets {
        // Secrets are accessed as {{ secrets.name }}
        // So we prefix them with "secrets."
        vars.insert(format!("secrets.{}", secret.name), secret.value);
    }

    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_template_integration() {
        let store = SecretStore::new().unwrap();
        let test_name = "heimdal_template_test";

        // Clean up first
        let _ = store.remove(test_name);

        // Add a test secret
        store.set(test_name, "secret_value_789").unwrap();

        // Get secret variables
        let vars = get_secret_variables().unwrap();

        // Should be prefixed with "secrets."
        assert!(vars.contains_key(&format!("secrets.{}", test_name)));
        assert_eq!(
            vars.get(&format!("secrets.{}", test_name)).unwrap(),
            "secret_value_789"
        );

        // Clean up
        store.remove(test_name).unwrap();
    }
}
