use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SERVICE_NAME: &str = "heimdal";
const METADATA_KEY: &str = "heimdal_secrets_metadata";

/// A secret stored in the system keychain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub value: String,
    pub created_at: String,
}

/// Cross-platform secret storage using system keychains
/// - macOS: Keychain
/// - Linux: Secret Service (libsecret)
/// - Windows: Credential Manager
pub struct SecretStore;

impl SecretStore {
    /// Create a new secret store
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Add or update a secret
    pub fn set(&self, name: &str, value: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, name)
            .with_context(|| format!("Failed to create keychain entry for '{}'", name))?;

        entry
            .set_password(value)
            .with_context(|| {
                format!(
                    "Failed to store secret '{}'. Note: On Linux, you may need to install and run gnome-keyring or another Secret Service provider.",
                    name
                )
            })?;

        // Update metadata
        self.update_metadata(name)?;

        Ok(())
    }

    /// Get a secret by name
    pub fn get(&self, name: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, name)
            .with_context(|| format!("Failed to create keychain entry for '{}'", name))?;

        entry
            .get_password()
            .with_context(|| format!("Failed to retrieve secret '{}'", name))
    }

    /// Remove a secret
    pub fn remove(&self, name: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, name)
            .with_context(|| format!("Failed to create keychain entry for '{}'", name))?;

        entry
            .delete_credential()
            .with_context(|| format!("Failed to delete secret '{}'", name))?;

        // Update metadata
        self.remove_from_metadata(name)?;

        Ok(())
    }

    /// List all secret names (not values)
    pub fn list(&self) -> Result<Vec<Secret>> {
        let metadata = self.load_metadata()?;
        let mut secrets = Vec::new();

        for (name, created_at) in metadata {
            // Try to get the secret to verify it still exists
            if let Ok(value) = self.get(&name) {
                secrets.push(Secret {
                    name,
                    value,
                    created_at,
                });
            }
        }

        Ok(secrets)
    }

    /// Check if a secret exists
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_ok()
    }

    // Metadata management - we store a list of secret names in the keychain
    // This allows us to list all secrets without platform-specific APIs
    fn load_metadata(&self) -> Result<HashMap<String, String>> {
        let entry = Entry::new(SERVICE_NAME, METADATA_KEY)?;

        match entry.get_password() {
            Ok(json) => {
                let metadata: HashMap<String, String> =
                    serde_json::from_str(&json).context("Failed to parse secrets metadata")?;
                Ok(metadata)
            }
            Err(_) => Ok(HashMap::new()), // No metadata yet
        }
    }

    fn save_metadata(&self, metadata: &HashMap<String, String>) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, METADATA_KEY)?;
        let json = serde_json::to_string(metadata)?;
        entry.set_password(&json)?;
        Ok(())
    }

    fn update_metadata(&self, name: &str) -> Result<()> {
        let mut metadata = self.load_metadata()?;
        metadata.insert(name.to_string(), chrono::Utc::now().to_rfc3339());
        self.save_metadata(&metadata)
    }

    fn remove_from_metadata(&self, name: &str) -> Result<()> {
        let mut metadata = self.load_metadata()?;
        metadata.remove(name);
        self.save_metadata(&metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_store_operations() {
        let store = SecretStore::new().unwrap();
        let test_name = "heimdal_test_secret";
        let test_value = "test_value_123";

        // Clean up first
        let _ = store.remove(test_name);

        // Test set
        match store.set(test_name, test_value) {
            Ok(_) => println!("✓ Set secret successfully"),
            Err(e) => {
                eprintln!("✗ Failed to set secret: {:?}", e);
                panic!("Failed to set secret: {}", e);
            }
        }

        // Test exists - try to get and see detailed error
        println!("Checking if secret exists...");
        match store.get(test_name) {
            Ok(val) => {
                println!("✓ Got secret: {}", val);
                assert_eq!(val, test_value);
            }
            Err(e) => {
                eprintln!("✗ Get failed with error: {:?}", e);
                eprintln!("Error chain:");
                let mut current: &dyn std::error::Error = &*e;
                loop {
                    eprintln!("  - {}", current);
                    match current.source() {
                        Some(source) => current = source,
                        None => break,
                    }
                }
                panic!("Failed to get secret after setting it");
            }
        }

        // Test list
        let secrets = store.list().unwrap();
        println!("Listed {} secrets", secrets.len());
        assert!(secrets.iter().any(|s| s.name == test_name));

        // Test remove
        store.remove(test_name).unwrap();
        assert!(!store.exists(test_name));

        println!("✓ All tests passed");
    }
}
