// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Newline-delimited `JSON-RPC` 2.0 client for `NestGate` via [`TransportEndpoint`].
//!
//! Phase 2 transport abstraction: the client accepts any `TransportEndpoint`
//! (UDS, TCP, or future `mesh_relay`) and dispatches accordingly. On Unix, UDS
//! is the primary path; on other platforms, TCP is used transparently.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use sweet_grass_core::transport::TransportEndpoint;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, trace};

use crate::error::NestGateStoreError;

/// `JSON-RPC` 2.0 client communicating with `NestGate` via transport endpoint.
///
/// Each RPC call opens a fresh connection (short-lived, no pooling) following
/// the same pattern used across the ecosystem for primal-to-primal IPC.
/// Supports UDS (Unix), TCP (all platforms), and `mesh_relay` (future).
#[derive(Debug)]
pub struct NestGateClient {
    endpoint: TransportEndpoint,
    request_id: AtomicU64,
    family_id: Option<String>,
}

impl NestGateClient {
    /// Create a new client targeting the given transport endpoint.
    pub const fn new(endpoint: TransportEndpoint, family_id: Option<String>) -> Self {
        Self {
            endpoint,
            request_id: AtomicU64::new(1),
            family_id,
        }
    }

    /// Create a client from a UDS socket path (convenience for Unix-primary deployments).
    pub fn from_socket_path(socket_path: &Path, family_id: Option<String>) -> Self {
        let path_str = socket_path.to_string_lossy().into_owned();
        Self::new(TransportEndpoint::uds(path_str), family_id)
    }

    /// The socket path this client targets (if UDS endpoint).
    pub fn socket_path(&self) -> &Path {
        match &self.endpoint {
            TransportEndpoint::Uds { path } => Path::new(path.as_str()),
            _ => Path::new(""),
        }
    }

    /// The transport endpoint this client uses.
    pub const fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Send a JSON-RPC request and return the result value.
    ///
    /// # Errors
    ///
    /// Returns an error if connection, write, read, or JSON-RPC error occurs.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value, NestGateStoreError> {
        let id = self.request_id.fetch_add(1, Ordering::Relaxed);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id,
        });

        let mut request_line = serde_json::to_string(&request)?;
        request_line.push('\n');

        trace!(method, id, endpoint = %self.endpoint, "NestGate RPC call");

        let stream = self.connect().await?;
        let (reader, mut writer) = tokio::io::split(stream);

        writer.write_all(request_line.as_bytes()).await?;
        writer.flush().await?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        buf_reader.read_line(&mut response_line).await?;

        if response_line.is_empty() {
            return Err(NestGateStoreError::Rpc(
                "NestGate closed connection without response".to_string(),
            ));
        }

        let response: Value = serde_json::from_str(&response_line)?;

        debug!(method, id, "NestGate RPC response received");

        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(Value::as_i64).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            return Err(NestGateStoreError::JsonRpcError { code, message });
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Build params with automatic `family_id` injection.
    pub fn with_family(&self, mut params: Value) -> Value {
        if let (Some(fid), Some(obj)) = (&self.family_id, params.as_object_mut())
            && !obj.contains_key("family_id")
        {
            obj.insert("family_id".to_string(), Value::String(fid.clone()));
        }
        params
    }

    /// Open a connection via the configured transport endpoint.
    async fn connect(&self) -> Result<TransportStream, NestGateStoreError> {
        match &self.endpoint {
            #[cfg(unix)]
            TransportEndpoint::Uds { path } => {
                let stream = tokio::net::UnixStream::connect(path)
                    .await
                    .map_err(|e| NestGateStoreError::ConnectionFailed(format!("{path}: {e}")))?;
                Ok(TransportStream::Uds(stream))
            },
            #[cfg(not(unix))]
            TransportEndpoint::Uds { path } => Err(NestGateStoreError::ConnectionFailed(format!(
                "UDS transport not available on this platform: {path}"
            ))),
            TransportEndpoint::Tcp { host, port } => {
                let stream = tokio::net::TcpStream::connect((host.as_str(), *port))
                    .await
                    .map_err(|e| {
                        NestGateStoreError::ConnectionFailed(format!("{host}:{port}: {e}"))
                    })?;
                Ok(TransportStream::Tcp(stream))
            },
            TransportEndpoint::MeshRelay {
                peer_id,
                capability,
            } => Err(NestGateStoreError::ConnectionFailed(format!(
                "mesh_relay not yet implemented (peer={peer_id}, cap={capability})",
            ))),
            _ => Err(NestGateStoreError::ConnectionFailed(String::from(
                "unsupported transport endpoint variant",
            ))),
        }
    }
}

/// Transport-agnostic stream for `NestGateClient` connections.
enum TransportStream {
    #[cfg(unix)]
    Uds(tokio::net::UnixStream),
    Tcp(tokio::net::TcpStream),
}

impl tokio::io::AsyncRead for TransportStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for TransportStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation_from_endpoint() {
        let client = NestGateClient::new(
            TransportEndpoint::uds("/tmp/test.sock"),
            Some("test-family".to_string()),
        );
        assert_eq!(client.socket_path().to_str(), Some("/tmp/test.sock"));
        assert_eq!(client.endpoint().transport_name(), "uds");
    }

    #[test]
    fn test_client_creation_from_socket_path() {
        let client = NestGateClient::from_socket_path(
            Path::new("/tmp/test.sock"),
            Some("test-family".to_string()),
        );
        assert_eq!(client.socket_path().to_str(), Some("/tmp/test.sock"));
    }

    #[test]
    fn test_client_tcp_endpoint() {
        let client = NestGateClient::new(
            TransportEndpoint::tcp("127.0.0.1", 9200),
            Some("test-family".to_string()),
        );
        assert_eq!(client.endpoint().transport_name(), "tcp");
    }

    #[test]
    fn test_with_family_injects_id() {
        let client = NestGateClient::new(
            TransportEndpoint::uds("/tmp/test.sock"),
            Some("fam-123".to_string()),
        );
        let params = serde_json::json!({"key": "test"});
        let result = client.with_family(params);
        assert_eq!(result["family_id"], "fam-123");
    }

    #[test]
    fn test_with_family_preserves_existing() {
        let client = NestGateClient::new(
            TransportEndpoint::uds("/tmp/test.sock"),
            Some("fam-123".to_string()),
        );
        let params = serde_json::json!({"key": "test", "family_id": "existing"});
        let result = client.with_family(params);
        assert_eq!(result["family_id"], "existing");
    }

    #[test]
    fn test_with_family_no_family_configured() {
        let client = NestGateClient::new(TransportEndpoint::uds("/tmp/test.sock"), None);
        let params = serde_json::json!({"key": "test"});
        let result = client.with_family(params);
        assert!(result.get("family_id").is_none());
    }
}
