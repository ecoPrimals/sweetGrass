// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! First-byte protocol auto-detection for BTSP/JSON-RPC multiplexing.
//!
//! When BTSP is required (`FAMILY_ID` set), connections may arrive as either:
//! - Raw JSON-RPC (first byte `{` / 0x7B) — health probes, biomeOS, springs
//! - BTSP length-prefixed (first byte is part of 4-byte big-endian frame length)
//!
//! These are unambiguous: a `{` first byte would imply a BTSP frame of ~2 GiB,
//! which far exceeds the 16 MiB maximum. [`PeekedStream`] wraps an async stream
//! after consuming one byte, re-presenting it transparently so the downstream
//! handler sees the complete byte sequence.
//!
//! This matches the ecosystem pattern established by `BearDog` (PG-35) and
//! `Squirrel` (PG-30) for BTSP Phase 2 auto-detection.

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// First byte of any JSON-RPC 2.0 message (object opening brace).
pub const JSON_RPC_FIRST_BYTE: u8 = b'{';

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

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
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
}
