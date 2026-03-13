// SPDX-License-Identifier: AGPL-3.0-only
//! Shared hashing and hex encoding utilities.
//!
//! Consolidates hex encode/decode and SHA-256 hashing that were duplicated
//! across `braid`, `entity`, and `factory` modules.

use crate::braid::ContentHash;

/// Hex-encode bytes to a lowercase hex string.
#[must_use]
pub fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write;
    bytes.as_ref().iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

/// Hex-decode a string to bytes, returning `None` on invalid input.
#[must_use]
pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Hex-decode a string to bytes, returning a descriptive error on failure.
///
/// # Errors
///
/// Returns an error if the string has odd length or contains non-hex characters.
pub fn hex_decode_strict(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd length hex string".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("invalid hex: {e}")))
        .collect()
}

/// Compute SHA-256 hash of data, returned as `sha256:{hex}` `ContentHash`.
#[must_use]
pub fn sha256(data: &[u8]) -> ContentHash {
    use sha2::{Digest, Sha256};
    let result = Sha256::digest(data);
    ContentHash::new(format!("sha256:{}", hex_encode(result)))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode([0xde, 0xad, 0xbe, 0xef]), "deadbeef");
        assert_eq!(hex_encode([]), "");
        assert_eq!(hex_encode([0x00, 0xff]), "00ff");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("deadbeef"), Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(hex_decode("00ff"), Some(vec![0x00, 0xff]));
        assert_eq!(hex_decode(""), Some(vec![]));
        assert!(hex_decode("abc").is_none());
        assert!(hex_decode("zzzz").is_none());
    }

    #[test]
    fn test_hex_decode_strict() {
        assert_eq!(
            hex_decode_strict("deadbeef"),
            Ok(vec![0xde, 0xad, 0xbe, 0xef])
        );
        assert!(hex_decode_strict("abc").is_err());
        assert!(hex_decode_strict("zzzz").is_err());
    }

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert!(hash.as_str().starts_with("sha256:"));
        assert_eq!(hash.as_str().len(), "sha256:".len() + 64);

        let hash2 = sha256(b"hello");
        assert_eq!(hash, hash2);
    }
}
