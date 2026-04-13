// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Newline-delimited `JSON-RPC` 2.0 client for `NestGate` over UDS.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, trace};

use crate::error::NestGateStoreError;

/// `JSON-RPC` 2.0 client communicating with `NestGate` over a Unix Domain Socket.
///
/// Each RPC call opens a fresh connection (short-lived, no pooling) following
/// the same pattern used across the ecosystem for primal-to-primal IPC.
#[derive(Debug)]
pub struct NestGateClient {
    socket_path: PathBuf,
    request_id: AtomicU64,
    family_id: Option<String>,
}

impl NestGateClient {
    /// Create a new client targeting the given `NestGate` socket.
    pub const fn new(socket_path: PathBuf, family_id: Option<String>) -> Self {
        Self {
            socket_path,
            request_id: AtomicU64::new(1),
            family_id,
        }
    }

    /// The socket path this client targets.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
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

        trace!(method, id, socket = %self.socket_path.display(), "NestGate RPC call");

        let stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            NestGateStoreError::ConnectionFailed(format!("{}: {}", self.socket_path.display(), e,))
        })?;

        let (reader, mut writer) = stream.into_split();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = NestGateClient::new(
            PathBuf::from("/tmp/test.sock"),
            Some("test-family".to_string()),
        );
        assert_eq!(client.socket_path().to_str(), Some("/tmp/test.sock"));
    }

    #[test]
    fn test_with_family_injects_id() {
        let client =
            NestGateClient::new(PathBuf::from("/tmp/test.sock"), Some("fam-123".to_string()));
        let params = serde_json::json!({"key": "test"});
        let result = client.with_family(params);
        assert_eq!(result["family_id"], "fam-123");
    }

    #[test]
    fn test_with_family_preserves_existing() {
        let client =
            NestGateClient::new(PathBuf::from("/tmp/test.sock"), Some("fam-123".to_string()));
        let params = serde_json::json!({"key": "test", "family_id": "existing"});
        let result = client.with_family(params);
        assert_eq!(result["family_id"], "existing");
    }

    #[test]
    fn test_with_family_no_family_configured() {
        let client = NestGateClient::new(PathBuf::from("/tmp/test.sock"), None);
        let params = serde_json::json!({"key": "test"});
        let result = client.with_family(params);
        assert!(result.get("family_id").is_none());
    }
}
