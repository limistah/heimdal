use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SERVICE_NAME: &str = "heimdal";

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecretsManifest {
    names: Vec<String>,
}

fn manifest_path(dotfiles_path: &Path) -> PathBuf {
    dotfiles_path.join(".heimdal").join("secrets_manifest.json")
}

fn load_manifest(dotfiles_path: &Path) -> SecretsManifest {
    // Try encrypted path first
    let enc_path = manifest_path(dotfiles_path).with_extension("json.enc");
    if enc_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&enc_path) {
            // Try to decrypt
            if let Ok(bifrost) = crate::key::load() {
                let key = crate::crypto::kdf::manifest_key(&bifrost);
                if let Ok(blob) = URL_SAFE_NO_PAD.decode(content.trim()) {
                    if let Ok(json) = crate::crypto::decrypt(&key, &blob) {
                        if let Ok(m) = serde_json::from_slice(&json) {
                            return m;
                        }
                    }
                }
            }
            // Fallback: treat as plaintext JSON (unencrypted legacy or no bifrost key)
            if let Ok(m) = serde_json::from_str(content.trim()) {
                return m;
            }
        }
    }
    // Legacy plaintext path
    let plain_path = manifest_path(dotfiles_path);
    if plain_path.exists() {
        return std::fs::read_to_string(&plain_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
    }
    SecretsManifest::default()
}

fn save_manifest(dotfiles_path: &Path, manifest: &SecretsManifest) -> Result<()> {
    let path = manifest_path(dotfiles_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(manifest)?;
    match crate::key::load() {
        Ok(bifrost) => {
            // Bifrost available: encrypt and write to .json.enc, remove legacy plaintext.
            let key = crate::crypto::kdf::manifest_key(&bifrost);
            let blob = crate::crypto::encrypt(&key, &json)?;
            let content = URL_SAFE_NO_PAD.encode(&blob);
            let enc_path = path.with_extension("json.enc");
            let tmp = enc_path.with_extension(format!("tmp.{}", std::process::id()));
            std::fs::write(&tmp, content)?;
            std::fs::rename(&tmp, &enc_path)?;
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
        Err(_) => {
            // No bifrost key: write plaintext to the legacy .json path.
            // Never write plaintext into .json.enc — that would be misleading.
            let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
            std::fs::write(&tmp, json)?;
            std::fs::rename(&tmp, &path)?;
        }
    }
    Ok(())
}

pub fn set_secret(dotfiles_path: &Path, name: &str, value: &str) -> Result<()> {
    let key = format!("{}:{}", whoami::username(), name);
    let entry = keyring::Entry::new(SERVICE_NAME, &key)
        .map_err(|e| crate::error::HeimdallError::Secret(e.to_string()))?;
    entry
        .set_password(value)
        .map_err(|e| crate::error::HeimdallError::Secret(e.to_string()))?;

    let mut manifest = load_manifest(dotfiles_path);
    if !manifest.names.contains(&name.to_string()) {
        manifest.names.push(name.to_string());
        manifest.names.sort();
        save_manifest(dotfiles_path, &manifest)?;
    }
    Ok(())
}

pub fn get_secret(name: &str) -> Result<String> {
    let key = format!("{}:{}", whoami::username(), name);
    let entry = keyring::Entry::new(SERVICE_NAME, &key)
        .map_err(|e| crate::error::HeimdallError::Secret(e.to_string()))?;
    entry.get_password().map_err(|_| {
        crate::error::HeimdallError::Secret(format!(
            "Secret '{}' not found. Add it with: heimdal secret add {}",
            name, name
        ))
        .into()
    })
}

pub fn delete_secret(dotfiles_path: &Path, name: &str) -> Result<()> {
    let key = format!("{}:{}", whoami::username(), name);
    if let Ok(entry) = keyring::Entry::new(SERVICE_NAME, &key) {
        let _ = entry.delete_credential();
    }
    let mut manifest = load_manifest(dotfiles_path);
    manifest.names.retain(|n| n != name);
    save_manifest(dotfiles_path, &manifest)?;
    Ok(())
}

pub fn list_secrets(dotfiles_path: &Path) -> Vec<String> {
    load_manifest(dotfiles_path).names
}
