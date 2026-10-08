// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Transport-aware connection — opens a stream from a [`TransportEndpoint`].
//!
//! Provides platform-agnostic connection and JSON-RPC utilities. All transport
//! dispatch happens via [`TransportEndpoint`] — callers never need `#[cfg]`
//! gates for UDS vs TCP.
//!
//! [`TransportEndpoint`]: sweet_grass_core::transport::TransportEndpoint

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use sweet_grass_core::transport::TransportEndpoint;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};

/// A transport-agnostic stream returned by [`connect_transport`].
#[derive(Debug)]
pub enum TransportStream {
    /// UDS connection (Unix only).
    #[cfg(unix)]
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
            #[cfg(unix)]
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
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
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
        #[cfg(unix)]
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Uds(stream))
        },
        #[cfg(not(unix))]
        TransportEndpoint::Uds { path } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("UDS transport not available on this platform: {path}"),
        )),
        TransportEndpoint::Tcp { host, port } => {
            let stream = tokio::net::TcpStream::connect((host.as_str(), *port)).await?;
            Ok(TransportStream::Tcp(stream))
        },
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => {
            // Gravitational mesh relay: route through local songBird.
            // songBird discovers the peer's socket via mesh.peers and
            // forwards the connection. The gravitational distance between
            // this node and the peer determines relay latency.
            mesh_relay_via_songbird(peer_id, capability).await
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "unsupported transport endpoint variant",
        )),
    }
}

// ──────────────────────── Mesh relay via songBird ────────────────────────

/// Route a connection through the local songBird mesh relay.
///
/// Discovers the songBird UDS socket, sends a `mesh.relay` request
/// with the target peer_id and capability, and receives a TCP address
/// to connect to. The relay path uses the gravitational mesh — closer
/// peers (higher gravity in swarmVine PeerStats) get faster routing.
#[cfg(unix)]
async fn mesh_relay_via_songbird(peer_id: &str, capability: &str) -> std::io::Result<TransportStream> {
    let songbird_socket = discover_songbird_socket().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "songBird socket not found for mesh relay — cannot reach peer",
        )
    })?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "mesh.resolve",
        "params": {
            "peer_id": peer_id,
            "capability": capability,
        },
        "id": 1,
    });

    let stream = tokio::net::UnixStream::connect(&songbird_socket).await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!("Cannot connect to songBird for mesh relay: {e}"),
        )
    })?;

    let (rd, mut wr) = tokio::io::split(stream);
    let mut reader = BufReader::new(rd);

    // riboCipher clear signal
    wr.write_all(&[0xEC, 0x01]).await?;
    let msg = format!("{request}\n");
    wr.write_all(msg.as_bytes()).await?;

    let mut response = String::new();
    let timeout = Duration::from_secs(5);
    match tokio::time::timeout(timeout, reader.read_line(&mut response)).await {
        Ok(Ok(n)) if n > 0 => {}
        Ok(Ok(_)) => return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Empty response from songBird mesh.resolve",
        )),
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Timeout waiting for songBird mesh.resolve",
        )),
    }

    let resp: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid mesh.resolve response: {e}"))
    })?;

    if let Some(error) = resp.get("error") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("mesh.resolve error: {error}"),
        ));
    }

    // songBird returns the resolved endpoint — connect via TCP
    let result = resp.get("result").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No result in mesh.resolve response")
    })?;

    let addr = result.get("endpoint").and_then(|v| v.as_str()).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "No endpoint in mesh.resolve result")
    })?;

    let tcp_stream = tokio::net::TcpStream::connect(addr).await?;
    Ok(TransportStream::Tcp(tcp_stream))
}

#[cfg(not(unix))]
async fn mesh_relay_via_songbird(peer_id: &str, capability: &str) -> std::io::Result<TransportStream> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!("mesh_relay requires Unix sockets (peer={peer_id}, cap={capability})"),
    ))
}

/// Discover the local songBird UDS socket.
#[cfg(unix)]
fn discover_songbird_socket() -> Option<std::path::PathBuf> {
    // Priority: SONGBIRD_SOCKET env → /run/membrane/songbird.sock
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    let default = std::path::PathBuf::from("/run/membrane/songbird.sock");
    if default.exists() {
        Some(default)
    } else {
        None
    }
}

// ──────────────────────── Server-side: TransportListener ────────────────────────

/// A transport-agnostic server listener.
///
/// Accepts connections and yields [`TransportStream`] — the accept loop
/// never needs to know UDS vs TCP.
#[derive(Debug)]
pub enum TransportListener {
    /// UDS listener (Unix only).
    #[cfg(unix)]
    Uds(tokio::net::UnixListener),
    /// TCP listener.
    Tcp(tokio::net::TcpListener),
}

impl TransportListener {
    /// Bind a listener according to the endpoint descriptor.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` if bind fails.
    pub async fn bind(endpoint: &TransportEndpoint) -> std::io::Result<Self> {
        match endpoint {
            #[cfg(unix)]
            TransportEndpoint::Uds { path } => {
                let path = std::path::Path::new(path);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
                let listener = tokio::net::UnixListener::bind(path)?;
                Ok(Self::Uds(listener))
            },
            #[cfg(not(unix))]
            TransportEndpoint::Uds { path } => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("UDS listener not available on this platform: {path}"),
            )),
            TransportEndpoint::Tcp { host, port } => {
                let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
                Ok(Self::Tcp(listener))
            },
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "cannot bind a listener on mesh_relay or unknown endpoint",
            )),
        }
    }

    /// Accept a single connection and return a transport-agnostic stream.
    ///
    /// # Errors
    ///
    /// Returns `io::Error` on accept failure.
    pub async fn accept(&self) -> std::io::Result<TransportStream> {
        match self {
            #[cfg(unix)]
            Self::Uds(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Uds(stream))
            },
            Self::Tcp(l) => {
                let (stream, _) = l.accept().await?;
                Ok(TransportStream::Tcp(stream))
            },
        }
    }

    /// Whether this listener is a local-only transport (UDS).
    #[must_use]
    pub const fn is_local(&self) -> bool {
        match self {
            #[cfg(unix)]
            Self::Uds(_) => true,
            Self::Tcp(_) => false,
        }
    }
}

// ──────────────────────── Utilities ────────────────────────

/// Default timeout for JSON-RPC probes.
///
/// Override via `SWEETGRASS_PROBE_TIMEOUT_MS` for tuning without recompilation.
pub fn probe_timeout() -> Duration {
    std::env::var("SWEETGRASS_PROBE_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(Duration::from_secs(3), Duration::from_millis)
}

/// Send a JSON-RPC request via a [`TransportEndpoint`] and return the response.
///
/// Opens a fresh connection, sends the newline-delimited JSON-RPC request,
/// reads one response line, and parses it. Transport-agnostic — works over
/// UDS (Unix), TCP (all platforms), or any future transport variant.
///
/// # Errors
///
/// Returns an error if connection fails, write/read fails, response times out,
/// or the response is not valid JSON.
pub async fn send_jsonrpc(
    endpoint: &TransportEndpoint,
    request: &serde_json::Value,
    timeout: Duration,
) -> std::io::Result<serde_json::Value> {
    let stream = connect_transport(endpoint).await?;
    let (reader, mut writer) = tokio::io::split(stream);

    let mut payload = serde_json::to_string(request).map_err(std::io::Error::other)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    let bytes_read = tokio::time::timeout(timeout, buf_reader.read_line(&mut response_line))
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "JSON-RPC response timeout")
        })??;

    if bytes_read == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "remote closed connection without response",
        ));
    }

    serde_json::from_str(response_line.trim()).map_err(std::io::Error::other)
}

/// Send a `health.liveness` JSON-RPC probe to a capability endpoint.
///
/// Transport-agnostic: resolves via [`connect_transport`] internally.
/// Returns `Ok(())` if the endpoint responds with a valid `result` field,
/// or an error otherwise.
///
/// # Errors
///
/// Returns an error on connection failure, timeout, malformed response,
/// or if the response contains no `result` field.
pub async fn try_liveness_probe(endpoint: &TransportEndpoint) -> std::io::Result<()> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 1,
    });

    let response = send_jsonrpc(endpoint, &request, probe_timeout()).await?;

    if response.get("result").is_some() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no result in liveness response",
        ))
    }
}

/// Resolve a capability domain to a [`TransportEndpoint`].
///
/// Discovery order:
/// 1. Check env var `CAPABILITY_{DOMAIN}_ENDPOINT` for explicit JSON endpoint
/// 2. On Unix: check `{socket_dir}/{domain}.sock` existence → UDS endpoint
/// 3. Returns `None` if no endpoint is discoverable
#[must_use]
pub fn resolve_capability_endpoint(
    domain: &str,
    socket_dir: &std::path::Path,
) -> Option<TransportEndpoint> {
    let env_key = format!(
        "CAPABILITY_{}_ENDPOINT",
        domain.to_uppercase().replace('-', "_")
    );
    if let Ok(json) = std::env::var(&env_key)
        && let Ok(ep) = sweet_grass_core::transport::parse_transport_endpoint(&json)
    {
        return Some(ep);
    }

    #[cfg(unix)]
    {
        let socket = socket_dir.join(format!("{domain}.sock"));
        if socket.exists() {
            return Some(TransportEndpoint::uds(socket.to_string_lossy()));
        }
    }

    #[cfg(not(unix))]
    {
        let _ = socket_dir;
    }

    None
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test file")]

    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[cfg(unix)]
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

    #[cfg(unix)]
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
