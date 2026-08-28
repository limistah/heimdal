use crate::utils::{info, success};
use anyhow::Result;
use std::path::PathBuf;

/// Re-encrypt all history files in the dotfiles repo with a freshly generated bifrost key.
///
/// Workflow (staged so a failure partway through never leaves data encrypted
/// with a key that isn't recoverable anywhere):
/// 1. Load the current bifrost key and derive the old history + manifest subkeys.
/// 2. Generate a new bifrost key and derive the new subkeys.
/// 3. For every `*.jsonl.enc` file in `dotfiles_path/history/` and the secrets
///    manifest: decrypt with the old key, re-encrypt with the new key, and
///    write the result to a *temp* file — the originals are untouched so far.
/// 4. Only once every file has been staged successfully, persist the new
///    bifrost key to the OS keychain.
/// 5. Only once the new key is safely stored, rename every staged temp file
///    over its original (same-directory rename, effectively atomic).
///
/// If any staging step fails, the temp files are cleaned up and the
/// originals — still readable with the old key, which is still the key in
/// the keychain — are left exactly as they were.
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

    // --- Stage: rekey history files to temp paths, without touching originals ---
    let history_dir = state.dotfiles_path.join("history");
    let mut staged: Vec<(PathBuf, PathBuf)> = Vec::new(); // (tmp, final)

    let stage_result = (|| -> Result<()> {
        if history_dir.exists() {
            for entry in std::fs::read_dir(&history_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "enc").unwrap_or(false) {
                    let tmp = stage_rekey_file(&path, &old_history_key, &new_history_key)?;
                    staged.push((tmp, path));
                }
            }
        }

        if let Some((tmp, final_path)) =
            stage_rekey_manifest(&state.dotfiles_path, &old_manifest_key, &new_manifest_key)?
        {
            staged.push((tmp, final_path));
        }

        Ok(())
    })();

    if let Err(e) = stage_result {
        for (tmp, _) in &staged {
            let _ = std::fs::remove_file(tmp);
        }
        return Err(e);
    }

    // --- Commit new key to keychain first: only after this succeeds do we
    // touch any original file, so a failure here leaves everything decryptable
    // with the still-current old key. ---
    if let Err(e) = crate::key::set(&state.dotfiles_path, &hex::encode(new_bifrost)) {
        for (tmp, _) in &staged {
            let _ = std::fs::remove_file(tmp);
        }
        return Err(e);
    }

    // --- Commit: rename staged files over their originals ---
    let mut rekeyed = 0usize;
    for (tmp, final_path) in &staged {
        std::fs::rename(tmp, final_path)?;
        if final_path.extension().map(|e| e == "enc").unwrap_or(false)
            && final_path.parent() == Some(history_dir.as_path())
        {
            rekeyed += 1;
            info(&format!(
                "Rekeyed {}",
                final_path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    success(&format!(
        "Rekey complete: {} history file(s) re-encrypted.",
        rekeyed
    ));
    crate::utils::info("Your old bifrost key is no longer valid.");
    crate::utils::info("Back up the new key now:  heimdal key export");
    Ok(())
}

/// Decrypt all entries in `path` with `old_key`, re-encrypt with `new_key`, and
/// write the result to a temp file alongside `path`. Does NOT touch `path`
/// itself — the caller renames the temp file into place once every file in
/// the batch has staged successfully.
fn stage_rekey_file(
    path: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<PathBuf> {
    let entries = crate::history::store::read_encrypted(path, old_key)?;

    let tmp = path.with_extension(format!("rekeying.{}", std::process::id()));
    let _ = std::fs::remove_file(&tmp); // in case a previous failed run left one behind
    for entry in &entries {
        crate::history::store::append_encrypted(&tmp, entry, new_key)?;
    }
    Ok(tmp)
}

/// Read the encrypted secrets manifest, decrypt with `old_key`, re-encrypt with
/// `new_key`, and write the result to a temp file. Returns `(tmp, final)` paths
/// for the caller to rename once the whole batch has staged successfully, or
/// `None` if there is no manifest to rekey.
fn stage_rekey_manifest(
    dotfiles_path: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<Option<(PathBuf, PathBuf)>> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;

    let manifest_enc = dotfiles_path
        .join(".heimdal")
        .join("secrets_manifest.json.enc");

    if !manifest_enc.exists() {
        return Ok(None); // nothing to rekey
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
    std::fs::write(&tmp, new_content.as_bytes())?;
    Ok(Some((tmp, manifest_enc)))
}
