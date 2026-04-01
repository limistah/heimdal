use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;

// Production Argon2id parameters: 64 MB memory, 3 iterations, 1 thread
fn argon2_params() -> anyhow::Result<Params> {
    #[cfg(not(test))]
    let (m, t) = (65536, 3);
    #[cfg(test)]
    let (m, t) = (4096, 1); // reduced for test speed
    Params::new(m, t, 1, Some(32)).map_err(|e| anyhow::anyhow!("argon2 params: {e}"))
}

/// Wrap the 32-byte bifrost key with a passphrase and return a portable base64url string.
/// Layout: base64url([32-byte salt][version=0x01][24-byte nonce][encrypted bifrost + tag])
pub fn export_with_passphrase(bifrost: &[u8; 32], passphrase: &str) -> anyhow::Result<String> {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let wrap_key = derive_wrap_key(passphrase, &salt)?;
    let ciphertext = crate::crypto::encrypt(&wrap_key, bifrost)?;

    let mut payload = Vec::with_capacity(32 + ciphertext.len());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&payload))
}

/// Recover a bifrost key from a blob produced by `export_with_passphrase`.
pub fn import_with_passphrase(blob: &str, passphrase: &str) -> anyhow::Result<[u8; 32]> {
    let payload = URL_SAFE_NO_PAD
        .decode(blob.trim())
        .map_err(|_| anyhow::anyhow!("invalid backup blob — not valid base64url"))?;

    anyhow::ensure!(payload.len() > 32, "backup blob is too short");

    let (salt, ciphertext) = payload.split_at(32);
    let salt: [u8; 32] = salt.try_into().unwrap();

    let wrap_key = derive_wrap_key(passphrase, &salt)?;
    let plaintext = crate::crypto::decrypt(&wrap_key, ciphertext)?;

    anyhow::ensure!(plaintext.len() == 32, "decrypted key has wrong length");
    let mut key = [0u8; 32];
    key.copy_from_slice(&plaintext);
    Ok(key)
}

fn derive_wrap_key(passphrase: &str, salt: &[u8; 32]) -> anyhow::Result<[u8; 32]> {
    let params = argon2_params()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_import_roundtrip() {
        let key = [55u8; 32];
        let blob = export_with_passphrase(&key, "hunter2").unwrap();
        let recovered = import_with_passphrase(&blob, "hunter2").unwrap();
        assert_eq!(recovered, key);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let key = [55u8; 32];
        let blob = export_with_passphrase(&key, "correct").unwrap();
        assert!(import_with_passphrase(&blob, "wrong").is_err());
    }

    #[test]
    fn blob_is_valid_base64url() {
        let key = [1u8; 32];
        let blob = export_with_passphrase(&key, "pass").unwrap();
        assert!(URL_SAFE_NO_PAD.decode(&blob).is_ok());
    }
}
