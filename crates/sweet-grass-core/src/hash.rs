// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Shared hashing and hex encoding utilities.
//!
//! Consolidates hex encode/decode and SHA-256 hashing that were duplicated
//! across `braid`, `entity`, and `factory` modules.

use crate::braid::ContentHash;

/// Error from strict hex decoding.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HexDecodeError {
    /// Input has odd length and cannot represent whole bytes.
    #[error("odd length hex string (length: {0})")]
    OddLength(usize),

    /// A non-hex character was encountered during decoding.
    #[error("invalid hex character at byte offset {position}")]
    InvalidChar {
        /// Byte offset of the invalid character pair.
        position: usize,
    },
}

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

/// Hex-decode a string to bytes, returning a typed error on failure.
///
/// # Errors
///
/// Returns [`HexDecodeError::OddLength`] if the string has odd length,
/// or [`HexDecodeError::InvalidChar`] if a non-hex character is found.
pub fn hex_decode_strict(s: &str) -> Result<Vec<u8>, HexDecodeError> {
    if !s.len().is_multiple_of(2) {
        return Err(HexDecodeError::OddLength(s.len()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| HexDecodeError::InvalidChar { position: i })
        })
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
        assert_eq!(hex_decode_strict("abc"), Err(HexDecodeError::OddLength(3)));
        assert_eq!(
            hex_decode_strict("zzzz"),
            Err(HexDecodeError::InvalidChar { position: 0 })
        );
    }

    #[test]
    fn test_hex_decode_error_display() {
        let err = HexDecodeError::OddLength(5);
        assert!(err.to_string().contains("odd length"));
        assert!(err.to_string().contains('5'));

        let err = HexDecodeError::InvalidChar { position: 4 };
        assert!(err.to_string().contains("invalid hex"));
        assert!(err.to_string().contains('4'));
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
