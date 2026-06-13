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
//! ## Legacy Fallback (deprecation period)
//!
//! Connections not starting with `0xEC`/`0xED`/`0xEE` are classified via
//! the legacy peek path with an ERROR log. Deprecation timeline:
//! - Wave 111: WARN (shipped)
//! - Wave 112: **ERROR** (current)
//! - Wave 113: REJECT (`-32002`)
//! - Wave 114: REMOVE legacy peek code
//!
//! This module is the **reference implementation** for the ecosystem.

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tracing::error;

use crate::btsp::protocol::ClientHello;

/// First byte of any JSON object (opening brace).
pub const JSON_FIRST_BYTE: u8 = b'{';

/// Maximum size for the first line during auto-detection (64 KiB).
const MAX_FIRST_LINE: usize = 64 * 1024;

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

    /// Length-prefixed BTSP — legacy unsignalled connection.
    /// The byte is stored for re-presentation via [`PeekedStream`].
    LengthPrefixedBtsp(u8),

    /// JSON-line BTSP — first line contained `"protocol":"btsp"`.
    /// The `ClientHello` was extracted from the parsed line.
    JsonLineBtsp(ClientHello),

    /// JSON-RPC 2.0 — first line contained `"jsonrpc":"2.0"`.
    /// The parsed request is carried for immediate processing.
    JsonRpc(serde_json::Value),

    /// Unrecognized first line (valid JSON but no protocol marker).
    Unknown(serde_json::Value),
}

/// Read the first bytes from `stream` and determine the protocol.
///
/// **riboCipher-first**: checks for signal prefix bytes (`0xEC`/`0xED`/`0xEE`)
/// before falling back to legacy peek logic. Unsignalled connections log a
/// WARN per the Wave 111 deprecation schedule.
///
/// # Errors
///
/// Returns `Err` on I/O failure, unsupported riboCipher tiers (mito/nuclear),
/// or if the first line exceeds 64 KiB.
pub async fn detect_protocol<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> std::io::Result<DetectedProtocol> {
    let mut first = [0u8; 1];
    stream.read_exact(&mut first).await?;

    while first[0].is_ascii_whitespace() {
        stream.read_exact(&mut first).await?;
    }

    // --- riboCipher signal detection (check BEFORE legacy peek) ---

    match first[0] {
        RIBOCIPHER_CLEAR => {
            let mut pt = [0u8; 1];
            stream.read_exact(&mut pt).await?;
            return Ok(DetectedProtocol::RiboCipherClear {
                protocol_type: pt[0],
            });
        }
        RIBOCIPHER_MITO => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "riboCipher mito-obfuscated tier not yet implemented",
            ));
        }
        RIBOCIPHER_NUCLEAR => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "riboCipher nuclear-sealed tier not yet implemented",
            ));
        }
        _ => {}
    }

    // --- Legacy fallback (deprecated — Wave 112 ERROR) ---

    error!(
        first_byte = first[0],
        "DEPRECATED: unsignalled connection (no riboCipher prefix). \
         Falling back to legacy peek detection. \
         Clients MUST send [0xEC, protocol_type] prefix. \
         Wave 113: connections without signal will be REJECTED."
    );

    if first[0] != JSON_FIRST_BYTE {
        return Ok(DetectedProtocol::LengthPrefixedBtsp(first[0]));
    }

    let mut line_buf = vec![b'{'];
    loop {
        let mut byte = [0u8; 1];
        match stream.read_exact(&mut byte).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => return Err(e),
        }
        if byte[0] == b'\n' {
            break;
        }
        line_buf.push(byte[0]);
        if line_buf.len() > MAX_FIRST_LINE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "first line exceeds 64 KiB during protocol detection",
            ));
        }
    }

    let parsed: serde_json::Value = serde_json::from_slice(&line_buf)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if parsed.get("protocol").and_then(serde_json::Value::as_str) == Some("btsp") {
        let hello: ClientHello = serde_json::from_value(parsed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        return Ok(DetectedProtocol::JsonLineBtsp(hello));
    }

    if parsed.get("jsonrpc").is_some() {
        return Ok(DetectedProtocol::JsonRpc(parsed));
    }

    Ok(DetectedProtocol::Unknown(parsed))
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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
            other => panic!("expected RiboCipherClear, got {}", variant_name(&other)),
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

    // --- Legacy fallback tests (unsignalled connections) ---

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
    async fn peeked_stream_read_exact() {
        let data = b"world";
        let cursor = std::io::Cursor::new(data.to_vec());
        let mut peeked = PeekedStream::new(b'W', cursor);

        let mut buf = vec![0u8; 6];
        peeked.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"Wworld");
    }

    #[tokio::test]
    async fn peeked_stream_write_passes_through() {
        let inner = Vec::<u8>::new();
        let mut peeked = PeekedStream::new(b'x', inner);

        peeked.write_all(b"test").await.unwrap();
        assert_eq!(&peeked.inner, b"test");
    }

    #[tokio::test]
    async fn detect_length_prefixed_btsp() {
        let data = vec![0x00, 0x00, 0x00, 0x10];
        let mut cursor = std::io::Cursor::new(data);
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::LengthPrefixedBtsp(0x00)),
            "first byte 0x00 should route to length-prefixed BTSP"
        );
    }

    #[tokio::test]
    async fn detect_jsonline_btsp() {
        let line = b"{\"protocol\":\"btsp\",\"version\":1,\"client_ephemeral_pub\":\"dGVzdA==\"}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::JsonLineBtsp(hello) => {
                assert_eq!(hello.version, 1);
                assert_eq!(hello.client_ephemeral_pub, "dGVzdA==");
            }
            other => panic!("expected JsonLineBtsp, got {}", variant_name(&other)),
        }
    }

    #[tokio::test]
    async fn detect_jsonrpc() {
        let line = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::JsonRpc(val) => {
                assert_eq!(val["method"], "health.check");
            }
            other => panic!("expected JsonRpc, got {}", variant_name(&other)),
        }
    }

    #[tokio::test]
    async fn detect_unknown_json() {
        let line = b"{\"type\":\"something_else\"}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::Unknown(_)),
            "JSON without protocol or jsonrpc should be Unknown"
        );
    }

    #[tokio::test]
    async fn detect_jsonrpc_eof_terminated() {
        let line = b"{\"jsonrpc\":\"2.0\",\"method\":\"braid.create\",\"params\":{},\"id\":1}";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::JsonRpc(val) => {
                assert_eq!(val["method"], "braid.create");
                assert_eq!(val["id"], 1);
            }
            other => panic!(
                "expected JsonRpc for EOF-terminated line, got {}",
                variant_name(&other)
            ),
        }
    }

    #[tokio::test]
    async fn detect_btsp_eof_terminated() {
        let line = b"{\"protocol\":\"btsp\",\"version\":1,\"client_ephemeral_pub\":\"dGVzdA==\"}";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::JsonLineBtsp(_)),
            "EOF-terminated BTSP line should still be detected"
        );
    }

    #[tokio::test]
    async fn detect_jsonrpc_with_leading_whitespace() {
        let line =
            b"\n \t{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"params\":{},\"id\":1}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        match result {
            DetectedProtocol::JsonRpc(val) => {
                assert_eq!(val["method"], "health.check");
            }
            other => panic!(
                "leading whitespace should not misroute to BTSP, got {}",
                variant_name(&other)
            ),
        }
    }

    #[tokio::test]
    async fn detect_btsp_with_leading_newline() {
        let line =
            b"\r\n{\"protocol\":\"btsp\",\"version\":1,\"client_ephemeral_pub\":\"dGVzdA==\"}\n";
        let mut cursor = std::io::Cursor::new(line.to_vec());
        let result = detect_protocol(&mut cursor).await.unwrap();
        assert!(
            matches!(result, DetectedProtocol::JsonLineBtsp(_)),
            "leading CRLF should still detect BTSP handshake"
        );
    }

    fn variant_name(d: &DetectedProtocol) -> &'static str {
        match d {
            DetectedProtocol::RiboCipherClear { .. } => "RiboCipherClear",
            DetectedProtocol::LengthPrefixedBtsp(_) => "LengthPrefixedBtsp",
            DetectedProtocol::JsonLineBtsp(_) => "JsonLineBtsp",
            DetectedProtocol::JsonRpc(_) => "JsonRpc",
            DetectedProtocol::Unknown(_) => "Unknown",
        }
    }
}
