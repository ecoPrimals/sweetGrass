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
    #![expect(clippy::unwrap_used, reason = "test file")]

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    #[tokio::test]
    async fn connect_uds_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("transport-test.sock");
        let listener = tokio::net::UnixListener::bind(&sock).unwrap();

        let ep = TransportEndpoint::uds(sock.to_str().unwrap());

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut stream = connect_transport(&ep).await.unwrap();
        stream.write_all(b"hello").await.unwrap();

        let mut buf = [0u8; 5];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");

        server.await.unwrap();
    }

    #[tokio::test]
    async fn connect_tcp_roundtrip() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let ep = TransportEndpoint::tcp("127.0.0.1", port);

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.unwrap();
            stream.write_all(&buf).await.unwrap();
        });

        let mut stream = connect_transport(&ep).await.unwrap();
        stream.write_all(b"ping").await.unwrap();

        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"ping");

        server.await.unwrap();
    }

    #[tokio::test]
    async fn transport_stream_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TransportStream>();
    }

    #[tokio::test]
    async fn transport_stream_is_unpin() {
        fn assert_unpin<T: Unpin>() {}
        assert_unpin::<TransportStream>();
    }
}
