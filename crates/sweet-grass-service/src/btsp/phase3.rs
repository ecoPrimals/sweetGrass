// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP Phase 3 — encrypted channel negotiation and framing.
//!
//! After a successful Phase 1–2 handshake, the client may send a
//! `btsp.negotiate` JSON-RPC request to upgrade the connection to
//! ChaCha20-Poly1305 AEAD framing.
//!
//! # Session Key Derivation
//!
//! ```text
//! salt = client_nonce || server_nonce
//! c2s_key = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-c2s")
//! s2c_key = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-s2c")
//! ```
//!
//! # Wire Format (encrypted channel)
//!
//! Each frame is a length-prefixed encrypted blob:
//! ```text
//! [4 bytes: length (big-endian u32)] [12 bytes: nonce] [length bytes: ciphertext + tag]
//! ```

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::protocol::BtspError;

/// HKDF info string for client-to-server direction.
const HKDF_INFO_C2S: &[u8] = b"btsp-session-v1-c2s";

/// HKDF info string for server-to-client direction.
const HKDF_INFO_S2C: &[u8] = b"btsp-session-v1-s2c";

/// Nonce size for ChaCha20-Poly1305 (12 bytes).
const NONCE_SIZE: usize = 12;

/// Server nonce size for key derivation salt (32 bytes).
const KEY_DERIVATION_NONCE_SIZE: usize = 32;

/// Minimum encrypted frame size: 12-byte nonce + 16-byte Poly1305 tag.
const MIN_ENCRYPTED_FRAME: usize = NONCE_SIZE + 16;

/// Cipher suites supported in Phase 3 negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase3Cipher {
    /// ChaCha20-Poly1305 AEAD (encrypted + authenticated).
    #[serde(rename = "chacha20-poly1305")]
    ChaCha20Poly1305,
    /// Plaintext (no encryption, no integrity).
    #[serde(rename = "null")]
    Null,
}

impl Phase3Cipher {
    /// Wire-format name for this cipher.
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ChaCha20Poly1305 => "chacha20-poly1305",
            Self::Null => "null",
        }
    }
}

/// Client → Server: Phase 3 negotiate request params.
#[derive(Debug, Deserialize)]
pub struct NegotiateParams {
    /// Session ID from the Phase 1 handshake.
    pub session_id: String,
    /// Ciphers the client supports, ordered by preference.
    pub ciphers: Vec<String>,
    /// Client-generated random nonce for session key derivation (base64).
    pub client_nonce: String,
}

/// Server → Client: Phase 3 negotiate response.
#[derive(Debug, Serialize)]
pub struct NegotiateResult {
    /// The cipher selected by the server.
    pub cipher: String,
    /// Server-generated random nonce for session key derivation (base64).
    pub server_nonce: String,
}

/// Directional session keys for encrypted BTSP framing.
///
/// `encrypt_key` protects outbound frames; `decrypt_key` decrypts inbound
/// frames.  Both are zeroed from memory on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    encrypt_key: [u8; 32],
    decrypt_key: [u8; 32],
}

impl std::fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKeys")
            .field("encrypt_key", &"[REDACTED]")
            .field("decrypt_key", &"[REDACTED]")
            .finish()
    }
}

impl SessionKeys {
    /// Derive directional session keys via HKDF-SHA256.
    ///
    /// Matches primalSpring's `SessionKeys::derive` — salt is
    /// `client_nonce || server_nonce`, info strings are directional.
    ///
    /// # Errors
    ///
    /// Returns [`BtspError::HandshakeFailed`] if HKDF expansion fails
    /// (should not happen with valid 32-byte inputs).
    pub fn derive(
        handshake_key: &[u8; 32],
        client_nonce: &[u8],
        server_nonce: &[u8],
        is_server: bool,
    ) -> Result<Self, BtspError> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut client_to_server = [0u8; 32];
        hk.expand(HKDF_INFO_C2S, &mut client_to_server)
            .map_err(|e| BtspError::HandshakeFailed {
                reason: format!("HKDF c2s expand: {e}"),
            })?;

        let mut server_to_client = [0u8; 32];
        hk.expand(HKDF_INFO_S2C, &mut server_to_client)
            .map_err(|e| BtspError::HandshakeFailed {
                reason: format!("HKDF s2c expand: {e}"),
            })?;

        if is_server {
            Ok(Self {
                encrypt_key: server_to_client,
                decrypt_key: client_to_server,
            })
        } else {
            Ok(Self {
                encrypt_key: client_to_server,
                decrypt_key: server_to_client,
            })
        }
    }

    /// Encrypt a plaintext payload for transmission.
    ///
    /// Returns `nonce(12) || ciphertext || tag(16)`.
    ///
    /// # Errors
    ///
    /// Returns [`BtspError::HandshakeFailed`] on encryption failure.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, BtspError> {
        use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
        use chacha20poly1305::{AeadCore, ChaCha20Poly1305};

        let cipher =
            ChaCha20Poly1305::new_from_slice(&self.encrypt_key).map_err(|e| {
                BtspError::HandshakeFailed {
                    reason: format!("cipher init: {e}"),
                }
            })?;

        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext =
            cipher
                .encrypt(&nonce, plaintext)
                .map_err(|e| BtspError::HandshakeFailed {
                    reason: format!("encrypt: {e}"),
                })?;

        let mut frame = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        frame.extend_from_slice(&nonce);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    /// Decrypt an incoming encrypted frame.
    ///
    /// Expects `nonce(12) || ciphertext || tag(16)`.
    ///
    /// # Errors
    ///
    /// Returns [`BtspError::HandshakeFailed`] on decryption failure or
    /// if the frame is too short.
    pub fn decrypt(&self, frame: &[u8]) -> Result<Vec<u8>, BtspError> {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        if frame.len() < MIN_ENCRYPTED_FRAME {
            return Err(BtspError::HandshakeFailed {
                reason: format!(
                    "encrypted frame too short: {} bytes (min {MIN_ENCRYPTED_FRAME})",
                    frame.len()
                ),
            });
        }

        let (nonce_bytes, ciphertext) = frame.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher =
            ChaCha20Poly1305::new_from_slice(&self.decrypt_key).map_err(|e| {
                BtspError::HandshakeFailed {
                    reason: format!("cipher init: {e}"),
                }
            })?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| BtspError::HandshakeFailed {
                reason: format!("decrypt: {e}"),
            })
    }
}

/// Generate a 32-byte random nonce for Phase 3 key derivation salt.
///
/// # Errors
///
/// Returns [`BtspError::HandshakeFailed`] if the OS CSPRNG is unavailable.
pub fn generate_server_nonce() -> Result<[u8; KEY_DERIVATION_NONCE_SIZE], BtspError> {
    use chacha20poly1305::aead::OsRng;
    use chacha20poly1305::aead::rand_core::RngCore;

    let mut nonce = [0u8; KEY_DERIVATION_NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    Ok(nonce)
}

/// Select the best cipher from the client's offered list.
///
/// Returns `ChaCha20Poly1305` if the client offers it, otherwise `Null`.
pub fn select_cipher(offered: &[String]) -> Phase3Cipher {
    if offered
        .iter()
        .any(|c| c == Phase3Cipher::ChaCha20Poly1305.wire_name())
    {
        Phase3Cipher::ChaCha20Poly1305
    } else {
        Phase3Cipher::Null
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn cipher_wire_names() {
        assert_eq!(Phase3Cipher::ChaCha20Poly1305.wire_name(), "chacha20-poly1305");
        assert_eq!(Phase3Cipher::Null.wire_name(), "null");
    }

    #[test]
    fn cipher_serde_roundtrip() {
        let json = serde_json::to_string(&Phase3Cipher::ChaCha20Poly1305).unwrap();
        assert_eq!(json, "\"chacha20-poly1305\"");

        let parsed: Phase3Cipher = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Phase3Cipher::ChaCha20Poly1305);

        let null_json = serde_json::to_string(&Phase3Cipher::Null).unwrap();
        assert_eq!(null_json, "\"null\"");
    }

    #[test]
    fn session_keys_derive_deterministic() {
        let handshake_key = [0xABu8; 32];
        let client_nonce = [1u8; 32];
        let server_nonce = [2u8; 32];

        let server_keys =
            SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, true).unwrap();
        let client_keys =
            SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false).unwrap();

        assert_eq!(
            server_keys.encrypt_key, client_keys.decrypt_key,
            "server encrypt must equal client decrypt"
        );
        assert_eq!(
            server_keys.decrypt_key, client_keys.encrypt_key,
            "server decrypt must equal client encrypt"
        );
        assert_ne!(
            server_keys.encrypt_key, server_keys.decrypt_key,
            "directional keys must differ"
        );
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let handshake_key = [0x42u8; 32];
        let client_nonce = [3u8; 32];
        let server_nonce = [4u8; 32];

        let server_keys =
            SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, true).unwrap();
        let client_keys =
            SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false).unwrap();

        let plaintext = b"hello encrypted btsp";

        let encrypted = server_keys.encrypt(plaintext).unwrap();
        assert_ne!(&encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len());

        let decrypted = client_keys.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_wrong_key_fails() {
        let key_a = [0x01u8; 32];
        let key_b = [0x02u8; 32];
        let nonce_c = [3u8; 32];
        let nonce_s = [4u8; 32];

        let keys_a = SessionKeys::derive(&key_a, &nonce_c, &nonce_s, true).unwrap();
        let keys_b = SessionKeys::derive(&key_b, &nonce_c, &nonce_s, false).unwrap();

        let encrypted = keys_a.encrypt(b"secret").unwrap();
        assert!(keys_b.decrypt(&encrypted).is_err());
    }

    #[test]
    fn decrypt_frame_too_short() {
        let keys = SessionKeys::derive(&[0u8; 32], &[1u8; 32], &[2u8; 32], true).unwrap();
        let result = keys.decrypt(&[0u8; 10]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn generate_server_nonce_produces_32_bytes() {
        let nonce = generate_server_nonce().unwrap();
        assert_eq!(nonce.len(), 32);
    }

    #[test]
    fn generate_server_nonce_is_random() {
        let a = generate_server_nonce().unwrap();
        let b = generate_server_nonce().unwrap();
        assert_ne!(a, b, "two nonces should differ");
    }

    #[test]
    fn select_cipher_prefers_chacha() {
        let offered = vec!["chacha20-poly1305".to_owned(), "null".to_owned()];
        assert_eq!(select_cipher(&offered), Phase3Cipher::ChaCha20Poly1305);
    }

    #[test]
    fn select_cipher_falls_back_to_null() {
        let offered = vec!["unknown-cipher".to_owned()];
        assert_eq!(select_cipher(&offered), Phase3Cipher::Null);
    }

    #[test]
    fn select_cipher_empty_list() {
        assert_eq!(select_cipher(&[]), Phase3Cipher::Null);
    }

    #[test]
    fn negotiate_params_deserialize() {
        let json = r#"{"session_id":"abc","ciphers":["chacha20-poly1305"],"client_nonce":"AQID"}"#;
        let params: NegotiateParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.session_id, "abc");
        assert_eq!(params.ciphers.len(), 1);
        assert_eq!(params.client_nonce, "AQID");
    }

    #[test]
    fn negotiate_result_serialize() {
        let result = NegotiateResult {
            cipher: "chacha20-poly1305".to_owned(),
            server_nonce: "BAUG".to_owned(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("chacha20-poly1305"));
        assert!(json.contains("BAUG"));
    }

    #[test]
    fn session_keys_debug_redacted() {
        let keys = SessionKeys::derive(&[0u8; 32], &[1u8; 32], &[2u8; 32], true).unwrap();
        let debug = format!("{keys:?}");
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("0"));
    }

    #[test]
    fn encrypt_empty_payload() {
        let keys = SessionKeys::derive(&[0x55u8; 32], &[1u8; 32], &[2u8; 32], true).unwrap();
        let client_keys =
            SessionKeys::derive(&[0x55u8; 32], &[1u8; 32], &[2u8; 32], false).unwrap();

        let encrypted = keys.encrypt(b"").unwrap();
        assert_eq!(encrypted.len(), NONCE_SIZE + 16);

        let decrypted = client_keys.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn encrypt_large_payload() {
        let keys = SessionKeys::derive(&[0x77u8; 32], &[1u8; 32], &[2u8; 32], true).unwrap();
        let client_keys =
            SessionKeys::derive(&[0x77u8; 32], &[1u8; 32], &[2u8; 32], false).unwrap();

        let large = vec![0xFFu8; 64 * 1024];
        let encrypted = keys.encrypt(&large).unwrap();
        let decrypted = client_keys.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, large);
    }
}
