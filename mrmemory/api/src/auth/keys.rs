use rand::Rng;
use sha2::{Digest, Sha256};

const KEY_PREFIX: &str = "amr_sk_";
const KEY_BYTES: usize = 32;

/// Base62 alphabet for key encoding.
const BASE62: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Generate a new raw API key: `amr_sk_<43 base62 chars>`.
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let raw_bytes: Vec<u8> = (0..KEY_BYTES).map(|_| rng.gen()).collect();
    let encoded = base62_encode(&raw_bytes);
    format!("{}{}", KEY_PREFIX, encoded)
}

/// SHA-256 hash of the full key string. This is what we store.
pub fn hash_api_key(key: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.finalize().to_vec()
}

/// Extract the first 8 chars after the prefix for identification.
pub fn key_prefix(key: &str) -> String {
    let without_prefix = key.strip_prefix(KEY_PREFIX).unwrap_or(key);
    without_prefix.chars().take(8).collect()
}

fn base62_encode(bytes: &[u8]) -> String {
    let mut result = String::new();
    for &b in bytes {
        // Simple per-byte encoding: 2 base62 chars per byte (covers 0-255)
        result.push(BASE62[(b as usize / 62) % 62] as char);
        result.push(BASE62[(b as usize) % 62] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_api_key();
        assert!(key.starts_with("amr_sk_"));
        assert!(key.len() > 20);
    }

    #[test]
    fn test_key_hashing_deterministic() {
        let key = "amr_sk_test123";
        let h1 = hash_api_key(key);
        let h2 = hash_api_key(key);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_key_hashing_different_keys() {
        let h1 = hash_api_key("amr_sk_aaa");
        let h2 = hash_api_key("amr_sk_bbb");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_key_prefix_extraction() {
        let key = "amr_sk_ABCDEFGHIJKLMNOP";
        assert_eq!(key_prefix(key), "ABCDEFGH");
    }
}
