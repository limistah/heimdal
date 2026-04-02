use crate::utils::{info, success};
use anyhow::Result;

/// Re-encrypt all history files in the dotfiles repo with a freshly generated bifrost key.
///
/// Workflow:
/// 1. Load the current bifrost key and derive the old history + manifest subkeys.
/// 2. Generate a new bifrost key and derive the new subkeys.
/// 3. For every `*.jsonl.enc` file in `dotfiles_path/history/`:
///    - Decrypt all entries with the old key.
///    - Re-encrypt them with the new key, writing atomically.
/// 4. Rekey the secrets manifest using the new manifest subkey.
/// 5. Store the new bifrost key in the OS keychain, replacing the old one.
///
/// After rekey completes, export the new key: `heimdal key export`.
pub fn run() -> Result<()> {
    let state = crate::state::State::load()?;

    // --- Load old key material ---
    let old_bifrost = crate::key::load()
        .map_err(|_| anyhow::anyhow!("No bifrost key found. Run `heimdal key gen` first."))?;
    let old_history_key = crate::crypto::kdf::history_key(&old_bifrost);
    let old_manifest_key = crate::crypto::kdf::manifest_key(&old_bifrost);

    // --- Generate new key material ---
    let mut new_bifrost = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut new_bifrost);
    let new_history_key = crate::crypto::kdf::history_key(&new_bifrost);
    let new_manifest_key = crate::crypto::kdf::manifest_key(&new_bifrost);

    // --- Rekey history files ---
    let history_dir = state.dotfiles_path.join("history");
    let mut rekeyed = 0usize;

    if history_dir.exists() {
        for entry in std::fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "enc").unwrap_or(false) {
                rekey_file(&path, &old_history_key, &new_history_key)?;
                rekeyed += 1;
                info(&format!(
                    "Rekeyed {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }

    // --- Rekey secrets manifest ---
    rekey_manifest(&state.dotfiles_path, &old_manifest_key, &new_manifest_key)?;

    // --- Commit new key to keychain ---
    crate::key::set(&state.dotfiles_path, &hex::encode(new_bifrost))?;

    success(&format!(
        "Rekey complete: {} history file(s) re-encrypted.",
        rekeyed
    ));
    crate::utils::info("Your old bifrost key is no longer valid.");
    crate::utils::info("Back up the new key now:  heimdal key export");
    Ok(())
}

/// Decrypt all entries in `path` with `old_key`, re-encrypt with `new_key`, write atomically.
fn rekey_file(path: &std::path::Path, old_key: &[u8; 32], new_key: &[u8; 32]) -> Result<()> {
    let entries = crate::history::store::read_encrypted(path, old_key)?;

    // Write to a temp file alongside the original, then rename atomically.
    let tmp = path.with_extension(format!("rekeying.{}", std::process::id()));
    for entry in &entries {
        crate::history::store::append_encrypted(&tmp, entry, new_key)?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read the encrypted secrets manifest, decrypt with `old_key`, re-encrypt with `new_key`.
fn rekey_manifest(
    dotfiles_path: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<()> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    let manifest_enc = dotfiles_path
        .join(".heimdal")
        .join("secrets_manifest.json.enc");

    if !manifest_enc.exists() {
        return Ok(()); // nothing to rekey
    }

    let content = std::fs::read_to_string(&manifest_enc)?;
    let blob = URL_SAFE_NO_PAD
        .decode(content.trim())
        .map_err(|e| anyhow::anyhow!("manifest decode failed: {e}"))?;
    let json = crate::crypto::decrypt(old_key, &blob).map_err(|_| {
        anyhow::anyhow!("manifest decrypt failed — is the old bifrost key correct?")
    })?;

    let new_blob = crate::crypto::encrypt(new_key, &json)?;
    let new_content = URL_SAFE_NO_PAD.encode(&new_blob);

    let tmp = manifest_enc.with_extension(format!("rekeying.{}", std::process::id()));
    std::fs::write(&tmp, new_content)?;
    std::fs::rename(&tmp, &manifest_enc)?;
    Ok(())
}
