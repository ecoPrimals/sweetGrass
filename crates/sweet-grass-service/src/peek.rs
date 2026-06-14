// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! riboCipher-aware protocol detection for IPC multiplexing.
//!
//! **riboCipher** (`RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md`) replaces
//! fragile peek-and-guess detection with intentional signal bytes. Clients
//! send a 2+ byte signal prefix; the accept loop routes deterministically.
//!
//! ## Signal Prefix Bytes
//!
//! | Byte   | Tier             |
//! |--------|------------------|
//! | `0xEC` | Clear signal     |
//! | `0xED` | Mito-obfuscated  |
//! | `0xEE` | Nuclear-sealed   |
//!
//! ## Protocol Types (Tier 1 clear signal)
//!
//! | Byte | Protocol        |
//! |------|-----------------|
//! | 0x00 | Probe           |
//! | 0x01 | NDJSON JSON-RPC |
//! | 0x02 | BTSP Binary     |
//! | 0x03 | BTSP JSON-line  |
//! | 0x04 | HTTP/1.1        |
//!
//! ## Unsignalled Connection Policy (Wave 113)
//!
//! Connections not starting with `0xEC`/`0xED`/`0xEE` are **rejected**
//! with JSON-RPC error code `-32002` ("riboCipher signal required").
//!
//! Deprecation timeline:
//! - Wave 111: WARN (shipped)
//! - Wave 112: ERROR (shipped)
//! - Wave 113: **REJECT** (`-32002`) (current)
//! - Wave 114: REMOVE legacy peek code entirely
//!
//! This module is the **reference implementation** for the ecosystem.

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tracing::error;

/// riboCipher signal prefix: clear (local, trusted wire).
pub const RIBOCIPHER_CLEAR: u8 = 0xEC;
/// riboCipher signal prefix: mito-obfuscated (cross-gate WAN).
pub const RIBOCIPHER_MITO: u8 = 0xED;
/// riboCipher signal prefix: nuclear-sealed (privileged).
pub const RIBOCIPHER_NUCLEAR: u8 = 0xEE;

/// riboCipher protocol types (clear signal second byte).
pub mod protocol_type {
    /// Lightweight health probe.
    pub const PROBE: u8 = 0x00;
    /// Newline-delimited JSON-RPC 2.0.
    pub const NDJSON_JSONRPC: u8 = 0x01;
    /// Length-prefixed BTSP binary handshake.
    pub const BTSP_BINARY: u8 = 0x02;
    /// JSON-line BTSP handshake.
    pub const BTSP_JSONLINE: u8 = 0x03;
    /// HTTP/1.1 (axum/hyper over UDS).
    pub const HTTP: u8 = 0x04;
    /// Post-BTSP encrypted session resume.
    pub const ENCRYPTED_RESUME: u8 = 0x05;
}

/// Result of protocol detection.
#[derive(Debug)]
pub enum DetectedProtocol {
    /// riboCipher Tier 1 clear signal — protocol type decoded.
    RiboCipherClear {
        /// The protocol type byte from the signal envelope.
        protocol_type: u8,
    },

    /// Unsignalled connection rejected per Wave 113 deprecation policy.
    /// The first byte is preserved for error reporting.
    Rejected {
        /// The first non-whitespace byte that was not a riboCipher signal.
        first_byte: u8,
    },
}

/// Read the first bytes from `stream` and determine the protocol.
///
/// **riboCipher-only**: checks for signal prefix bytes (`0xEC`/`0xED`/`0xEE`).
/// Unsignalled connections are **rejected** per the Wave 113 deprecation
/// policy — no legacy peek fallback.
///
/// # Errors
///
/// Returns `Err` on I/O failure or unsupported riboCipher tiers (mito/nuclear).
pub async fn detect_protocol<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> std::io::Result<DetectedProtocol> {
    let mut first = [0u8; 1];
    stream.read_exact(&mut first).await?;

    while first[0].is_ascii_whitespace() {
        stream.read_exact(&mut first).await?;
    }

    match first[0] {
        RIBOCIPHER_CLEAR => {
            let mut pt = [0u8; 1];
            stream.read_exact(&mut pt).await?;
            Ok(DetectedProtocol::RiboCipherClear {
                protocol_type: pt[0],
            })
        }
        RIBOCIPHER_MITO => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "riboCipher mito-obfuscated tier not yet implemented",
        )),
        RIBOCIPHER_NUCLEAR => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "riboCipher nuclear-sealed tier not yet implemented",
        )),
        byte => {
            error!(
                first_byte = byte,
                "REJECTED: unsignalled connection (no riboCipher prefix). \
                 Clients MUST send [0xEC, protocol_type] prefix. \
                 See RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md."
            );
            Ok(DetectedProtocol::Rejected { first_byte: byte })
        }
    }
}

/// A stream wrapper that re-yields a single consumed byte before delegating
/// to the underlying stream.
pub struct PeekedStream<S> {
    first_byte: Option<u8>,
    inner: S,
}

impl<S> PeekedStream<S> {
    /// Wrap `inner`, yielding `first_byte` on the first read.
    pub const fn new(first_byte: u8, inner: S) -> Self {
        Self {
            first_byte: Some(first_byte),
            inner,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PeekedStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if let Some(byte) = self.first_byte.take() {
            buf.put_slice(&[byte]);
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PeekedStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // --- riboCipher signal detection tests ---

    #[tokio::test]
    async fn ribocipher_clear_jsonrpc() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::NDJSON_JSONRPC];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::NDJSON_JSONRPC);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_clear_probe() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::PROBE];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::PROBE);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_clear_btsp_binary() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::BTSP_BINARY];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::BTSP_BINARY);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_clear_btsp_jsonline() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::BTSP_JSONLINE];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::BTSP_JSONLINE);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_clear_http() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::HTTP];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::HTTP);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_clear_encrypted_resume() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::ENCRYPTED_RESUME];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::RiboCipherClear { protocol_type: pt } => {
                assert_eq!(pt, protocol_type::ENCRYPTED_RESUME);
            }
            DetectedProtocol::Rejected { .. } => panic!("expected RiboCipherClear, got Rejected"),
        }
    }

    #[tokio::test]
    async fn ribocipher_mito_returns_unsupported() {
        let data = vec![RIBOCIPHER_MITO, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
        assert!(
            err.to_string().contains("mito-obfuscated"),
            "error should mention mito tier"
        );
    }

    #[tokio::test]
    async fn ribocipher_nuclear_returns_unsupported() {
        let data = vec![RIBOCIPHER_NUCLEAR, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
        assert!(
            err.to_string().contains("nuclear-sealed"),
            "error should mention nuclear tier"
        );
    }

    #[tokio::test]
    async fn ribocipher_clear_takes_precedence_over_legacy() {
        let data = vec![RIBOCIPHER_CLEAR, protocol_type::BTSP_BINARY, 0x00, 0x00];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(
                result,
                DetectedProtocol::RiboCipherClear {
                    protocol_type: protocol_type::BTSP_BINARY
                }
            ),
            "riboCipher signal must be detected before legacy peek"
        );
    }

    // --- Unsignalled connection rejection tests (Wave 113) ---

    #[tokio::test]
    async fn unsignalled_json_rejected() {
        let line = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::Rejected { first_byte: b'{' }),
            "unsignalled JSON should be rejected, not parsed"
        );
    }

    #[tokio::test]
    async fn unsignalled_binary_rejected() {
        let data = vec![0x00, 0x00, 0x00, 0x10];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::Rejected { first_byte: 0x00 }),
            "unsignalled binary should be rejected"
        );
    }

    #[tokio::test]
    async fn unsignalled_http_verb_rejected() {
        let data = b"GET / HTTP/1.1\r\n\r\n";
        let mut cursor = std::io::Cursor::new(data.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::Rejected { first_byte: b'G' }),
            "unsignalled HTTP should be rejected"
        );
    }

    #[tokio::test]
    async fn unsignalled_with_leading_whitespace_rejected() {
        let line =
            b"\n \t{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::Rejected { first_byte: b'{' }),
            "unsignalled JSON after whitespace should still be rejected"
        );
    }

    // --- PeekedStream tests (retained for Wave 114 removal) ---

    #[tokio::test]
    async fn peeked_stream_prepends_byte() {
        let data = b"hello";
        let cursor = std::io::Cursor::new(data.to_vec());
        let mut peeked = PeekedStream::new(b'{', cursor);

        let mut buf = vec![0u8; 6];
        let n = peeked.read(&mut buf).await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(buf[0], b'{');

        let n = peeked.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello");
    }

    #[tokio::test]
    async fn peeked_stream_write_passes_through() {
        let inner = Vec::<u8>::new();
        let mut peeked = PeekedStream::new(b'x', inner);

        peeked.write_all(b"test").await.unwrap();
        assert_eq!(&peeked.inner, b"test");
    }

}
