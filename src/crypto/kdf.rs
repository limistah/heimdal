// Context strings are fixed. Changing them rotates all derived keys.
// Bump the version suffix (v2, v3, ...) if you need to rotate without changing the master key.
const HISTORY_CTX: &str = "heimdal bifrost history v1";
const MANIFEST_CTX: &str = "heimdal bifrost manifest v1";

pub fn history_key(bifrost: &[u8; 32]) -> [u8; 32] {
    blake3::derive_key(HISTORY_CTX, bifrost)
}

pub fn manifest_key(bifrost: &[u8; 32]) -> [u8; 32] {
    blake3::derive_key(MANIFEST_CTX, bifrost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_and_manifest_keys_differ() {
        let bifrost = [7u8; 32];
        assert_ne!(history_key(&bifrost), manifest_key(&bifrost));
    }

    #[test]
    fn derivation_is_deterministic() {
        let bifrost = [7u8; 32];
        assert_eq!(history_key(&bifrost), history_key(&bifrost));
    }

    #[test]
    fn different_master_gives_different_subkey() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        assert_ne!(history_key(&a), history_key(&b));
    }
}
