pub mod kdf;

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305,
};

/// Encrypts `plaintext` with a 32-byte key.
/// Output format: [1 byte version=0x01][24 bytes random XNonce][ciphertext + 16-byte Poly1305 tag]
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encrypt failed: {e}"))?;

    let mut out = Vec::with_capacity(1 + 24 + ct.len());
    out.push(0x01u8); // version byte
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypts a blob produced by `encrypt`.
pub fn decrypt(key: &[u8; 32], blob: &[u8]) -> anyhow::Result<Vec<u8>> {
    // minimum: 1 (version) + 24 (nonce) + 16 (tag) = 41 bytes
    anyhow::ensure!(blob.len() >= 41, "blob too short to be a valid ciphertext");
    anyhow::ensure!(
        blob[0] == 0x01,
        "unsupported ciphertext version: {}",
        blob[0]
    );

    let nonce = chacha20poly1305::XNonce::from_slice(&blob[1..25]);
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher
        .decrypt(nonce, &blob[25..])
        .map_err(|_| anyhow::anyhow!("decryption failed — wrong key or corrupted data"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"hello heimdal";
        let blob = encrypt(&key, plaintext).unwrap();
        let recovered = decrypt(&key, &blob).unwrap();
        assert_eq!(recovered, plaintext.as_slice());
    }

    #[test]
    fn wrong_key_fails() {
        let key = [42u8; 32];
        let wrong = [99u8; 32];
        let blob = encrypt(&key, b"secret").unwrap();
        assert!(decrypt(&wrong, &blob).is_err());
    }

    #[test]
    fn nonces_are_random() {
        let key = [42u8; 32];
        let b1 = encrypt(&key, b"same").unwrap();
        let b2 = encrypt(&key, b"same").unwrap();
        assert_ne!(b1, b2);
    }

    #[test]
    fn corrupted_blob_fails() {
        let key = [42u8; 32];
        let mut blob = encrypt(&key, b"data").unwrap();
        blob[30] ^= 0xFF;
        assert!(decrypt(&key, &blob).is_err());
    }
}
