pub mod backup;

use std::path::Path;

pub const SECRET_NAME: &str = "bifrost";

/// Parse a 64-character hex string into a 32-byte key.
pub fn parse_hex_key(s: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(s.trim())
        .map_err(|_| anyhow::anyhow!("bifrost key must be 64 hex characters (got invalid hex)"))?;
    anyhow::ensure!(
        bytes.len() == 32,
        "bifrost key must be 32 bytes (64 hex chars), got {} bytes",
        bytes.len()
    );
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Generate a random 32-byte key, store it in the OS keychain, and return the raw bytes.
pub fn generate(dotfiles_path: &Path) -> anyhow::Result<[u8; 32]> {
    let mut key = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
    let hex = hex::encode(key);
    crate::secrets::set_secret(dotfiles_path, SECRET_NAME, &hex)?;
    Ok(key)
}

/// Load the bifrost key from the OS keychain.
pub fn load() -> anyhow::Result<[u8; 32]> {
    let hex = crate::secrets::get_secret(SECRET_NAME)?;
    parse_hex_key(&hex)
}

/// Store an existing key (provided as hex) in the OS keychain.
pub fn set(dotfiles_path: &Path, hex: &str) -> anyhow::Result<()> {
    parse_hex_key(hex)?;
    crate::secrets::set_secret(dotfiles_path, SECRET_NAME, hex.trim())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_hex_key() {
        let hex = "a".repeat(64);
        let key = parse_hex_key(&hex).unwrap();
        assert_eq!(key, [0xaa_u8; 32]);
    }

    #[test]
    fn parse_wrong_length_fails() {
        assert!(parse_hex_key("deadbeef").is_err());
    }

    #[test]
    fn parse_invalid_hex_fails() {
        assert!(parse_hex_key(&"g".repeat(64)).is_err());
    }
}
