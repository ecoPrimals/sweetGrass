// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP wire types and length-prefixed framing.
//!
//! All frames use a 4-byte big-endian length prefix per
//! `BTSP_PROTOCOL_STANDARD.md` §Wire Framing.  Maximum frame size
//! is 16 MiB.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Maximum allowed frame size (16 MiB).
const MAX_FRAME_SIZE: u32 = 16 * 1024 * 1024;

/// BTSP protocol errors.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BtspError {
    /// I/O error during framing.
    #[error("BTSP I/O: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization error.
    #[error("BTSP JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Frame exceeds the 16 MiB limit.
    #[error("BTSP frame too large: {size} bytes (max {MAX_FRAME_SIZE})")]
    FrameTooLarge {
        /// Actual frame size received.
        size: u32,
    },

    /// Handshake verification failed (wrong family).
    #[error("BTSP handshake failed: {reason}")]
    HandshakeFailed {
        /// Human-readable failure reason.
        reason: String,
    },

    /// Crypto provider (`BearDog`) unreachable.
    #[error("BTSP crypto provider unreachable: {0}")]
    CryptoProviderUnavailable(String),

    /// Unexpected message during handshake.
    #[error("BTSP unexpected message: expected {expected}, got {actual}")]
    UnexpectedMessage {
        /// Expected message type.
        expected: String,
        /// Actual message type received.
        actual: String,
    },
}

// -- Handshake wire types per BTSP_PROTOCOL_STANDARD §Handshake Sequence --

/// Client → Server: initiates handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Protocol version (must be 1).
    pub version: u32,
    /// Client's ephemeral X25519 public key (base64).
    pub client_ephemeral_pub: String,
}

/// Server → Client: challenge with server ephemeral key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    /// Protocol version (must be 1).
    pub version: u32,
    /// Server's ephemeral X25519 public key (base64).
    pub server_ephemeral_pub: String,
    /// Random 32-byte challenge (base64).
    pub challenge: String,
}

/// Client → Server: HMAC-SHA256 challenge response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// `HMAC-SHA256(handshake_key, challenge ‖ client_pub ‖ server_pub)` (base64).
    pub response: String,
    /// Client's preferred cipher suite.
    pub preferred_cipher: String,
}

/// Server → Client: handshake succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    /// Negotiated cipher suite.
    pub cipher: String,
    /// Session identifier (hex).
    pub session_id: String,
}

/// Server → Client: handshake failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeError {
    /// Always `"handshake_failed"`.
    pub error: String,
    /// Reason string.
    pub reason: String,
}

// -- Length-prefixed framing per BTSP_PROTOCOL_STANDARD §Wire Framing --

/// Read a single length-prefixed frame from an async reader.
///
/// # Errors
///
/// Returns [`BtspError::FrameTooLarge`] if the declared length exceeds
/// 16 MiB, or [`BtspError::Io`] on read failure.
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Vec<u8>, BtspError> {
    let len = reader.read_u32().await?;
    if len > MAX_FRAME_SIZE {
        return Err(BtspError::FrameTooLarge { size: len });
    }

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Write a single length-prefixed frame to an async writer.
///
/// # Errors
///
/// Returns [`BtspError::FrameTooLarge`] if the payload exceeds 16 MiB,
/// or [`BtspError::Io`] on write failure.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<(), BtspError> {
    let len =
        u32::try_from(payload.len()).map_err(|_| BtspError::FrameTooLarge { size: u32::MAX })?;
    if len > MAX_FRAME_SIZE {
        return Err(BtspError::FrameTooLarge { size: len });
    }

    writer.write_u32(len).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Deserialize a handshake message from a length-prefixed frame.
///
/// # Errors
///
/// Returns framing or JSON errors.
pub async fn read_message<R: AsyncReadExt + Unpin + Send, T: serde::de::DeserializeOwned>(
    reader: &mut R,
) -> Result<T, BtspError> {
    let frame = read_frame(reader).await?;
    Ok(serde_json::from_slice(&frame)?)
}

/// Serialize and write a handshake message as a length-prefixed frame.
///
/// # Errors
///
/// Returns framing or JSON errors.
pub async fn write_message<W: AsyncWriteExt + Unpin + Send, T: Serialize + Sync>(
    writer: &mut W,
    msg: &T,
) -> Result<(), BtspError> {
    let payload = serde_json::to_vec(msg)?;
    write_frame(writer, &payload).await
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test module")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn frame_roundtrip() {
        let mut buf = Vec::new();
        let payload = b"hello btsp";

        write_frame(&mut buf, payload).await.expect("write");

        assert_eq!(buf.len(), 4 + payload.len());
        assert_eq!(
            u32::from_be_bytes(buf[..4].try_into().unwrap()),
            u32::try_from(payload.len()).unwrap()
        );

        let mut cursor = std::io::Cursor::new(buf);
        let read_back = read_frame(&mut cursor).await.expect("read");
        assert_eq!(read_back, payload);
    }

    #[tokio::test]
    async fn frame_too_large_rejected() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(MAX_FRAME_SIZE + 1).to_be_bytes());

        let mut cursor = std::io::Cursor::new(buf);
        let result = read_frame(&mut cursor).await;
        assert!(matches!(result, Err(BtspError::FrameTooLarge { .. })));
    }

    #[tokio::test]
    async fn message_roundtrip() {
        let hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "dGVzdA==".to_string(),
        };

        let mut buf = Vec::new();
        write_message(&mut buf, &hello).await.expect("write");

        let mut cursor = std::io::Cursor::new(buf);
        let read_back: ClientHello = read_message(&mut cursor).await.expect("read");
        assert_eq!(read_back.version, 1);
        assert_eq!(read_back.client_ephemeral_pub, "dGVzdA==");
    }

    #[test]
    fn wire_types_serialize() {
        let hello = ServerHello {
            version: 1,
            server_ephemeral_pub: "c2VydmVy".to_string(),
            challenge: "Y2hhbGxlbmdl".to_string(),
        };
        let json = serde_json::to_string(&hello).expect("serialize");
        assert!(json.contains("server_ephemeral_pub"));
        assert!(json.contains("challenge"));

        let resp = ChallengeResponse {
            response: "cmVzcA==".to_string(),
            preferred_cipher: "chacha20_poly1305".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("preferred_cipher"));

        let complete = HandshakeComplete {
            cipher: "chacha20_poly1305".to_string(),
            session_id: "abcdef0123456789".to_string(),
        };
        let json = serde_json::to_string(&complete).expect("serialize");
        assert!(json.contains("session_id"));

        let err = HandshakeError {
            error: "handshake_failed".to_string(),
            reason: "family_verification".to_string(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("handshake_failed"));
    }
}
