use anyhow::Result;
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
    let path = manifest_path(dotfiles_path);
    if !path.exists() {
        return SecretsManifest::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_manifest(dotfiles_path: &Path, manifest: &SecretsManifest) -> Result<()> {
    let path = manifest_path(dotfiles_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string_pretty(manifest)?)?;
    std::fs::rename(&tmp, &path)?;
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
