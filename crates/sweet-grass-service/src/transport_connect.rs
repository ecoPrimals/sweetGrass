// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Transport-aware connection — opens a stream from a [`TransportEndpoint`].
//!
//! [`TransportEndpoint`]: sweet_grass_core::transport::TransportEndpoint

use std::pin::Pin;
use std::task::{Context, Poll};

use sweet_grass_core::transport::TransportEndpoint;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A transport-agnostic stream returned by [`connect_transport`].
#[derive(Debug)]
pub enum TransportStream {
    /// UDS connection.
    Uds(tokio::net::UnixStream),
    /// TCP connection.
    Tcp(tokio::net::TcpStream),
}

impl AsyncRead for TransportStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TransportStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Uds(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a service via its resolved transport endpoint.
///
/// Returns a [`TransportStream`] — the caller does not need to know the
/// underlying transport.
///
/// # Errors
///
/// Returns `io::Error` if the connection fails (socket not found, connection
/// refused, etc.). `MeshRelay` is not yet supported.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Uds(stream))
        }
        TransportEndpoint::Tcp { host, port } => {
            let stream = tokio::net::TcpStream::connect((host.as_str(), *port)).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "mesh_relay transport not yet implemented (peer={peer_id}, cap={capability})"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_uds_nonexistent_fails() {
        let ep = TransportEndpoint::uds("/tmp/sweetgrass-transport-test-nonexistent.sock");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn connect_tcp_refused_fails() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 1);
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn connect_mesh_relay_unsupported() {
        let ep = TransportEndpoint::mesh_relay("test-peer", "test-cap");
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    }
}
