// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! First-line protocol auto-detection for BTSP/JSON-RPC multiplexing.
//!
//! When BTSP is required (`FAMILY_ID` set), connections may arrive as:
//! - Length-prefixed BTSP (first byte is part of a 4-byte big-endian frame length)
//! - JSON-line BTSP (`{"protocol":"btsp",...}\n`) — primalSpring-compatible handshake
//! - Raw JSON-RPC (`{"jsonrpc":"2.0",...}\n`) — health probes, biomeOS, springs
//!
//! Detection reads the first byte: if not `{`, it's length-prefixed BTSP. If `{`,
//! the rest of the first line is read and the JSON is inspected for `"protocol"`
//! vs `"jsonrpc"` keys. [`PeekedStream`] re-presents the first byte for the
//! length-prefixed path. [`detect_protocol`] performs the full first-line routing.
//!
//! This aligns with Phase 45b wire-format guidance and `BearDog` (PG-35) /
//! `Squirrel` (PG-30) ecosystem patterns.

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};

use crate::btsp::protocol::ClientHello;

/// First byte of any JSON object (opening brace).
pub const JSON_FIRST_BYTE: u8 = b'{';

/// Maximum size for the first line during auto-detection (64 KiB).
const MAX_FIRST_LINE: usize = 64 * 1024;

/// Result of first-line protocol auto-detection.
pub enum DetectedProtocol {
    /// Length-prefixed BTSP — first byte was not `{`.
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

/// Read the first line from `stream` and determine the protocol.
///
/// This consumes bytes from the stream. For length-prefixed BTSP,
/// only one byte is consumed (the caller wraps it in [`PeekedStream`]).
/// For JSON protocols, the entire first line (up to `\n`) is consumed.
///
/// # Errors
///
/// Returns `Err` on I/O failure or if the first line exceeds 64 KiB.
pub async fn detect_protocol<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> std::io::Result<DetectedProtocol> {
    let mut first = [0u8; 1];
    stream.read_exact(&mut first).await?;

    if first[0] != JSON_FIRST_BYTE {
        return Ok(DetectedProtocol::LengthPrefixedBtsp(first[0]));
    }

    let mut line_buf = vec![b'{'];
    loop {
        let mut byte = [0u8; 1];
        match stream.read_exact(&mut byte).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            },
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
            },
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
            },
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
            },
            other => panic!("expected JsonRpc for EOF-terminated line, got {}", variant_name(&other)),
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

    fn variant_name(d: &DetectedProtocol) -> &'static str {
        match d {
            DetectedProtocol::LengthPrefixedBtsp(_) => "LengthPrefixedBtsp",
            DetectedProtocol::JsonLineBtsp(_) => "JsonLineBtsp",
            DetectedProtocol::JsonRpc(_) => "JsonRpc",
            DetectedProtocol::Unknown(_) => "Unknown",
        }
    }
}
