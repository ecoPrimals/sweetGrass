// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! JSON-RPC 2.0 client for loamSpine ledger operations.
//!
//! Provides the sweetGrass → loamSpine IPC path, closing the Provenance
//! Trio triangle (Nest Atomic G3). Uses newline-delimited JSON-RPC over
//! UDS (Unix) or TCP (Windows) — consistent with the service handler layer.
//!
//! Socket resolution chain:
//! 1. `LOAMSPINE_SOCKET` env var (explicit override)
//! 2. `{BIOMEOS_SOCKET_DIR}/loamspine-{FAMILY_ID}.sock` (family-scoped)
//! 3. `{BIOMEOS_SOCKET_DIR}/loamspine.sock` (standalone)

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::debug;

use sweet_grass_core::primal_names::{env_vars, socket_env_var};

use crate::transport_connect::{TransportStream, connect_transport};
use sweet_grass_core::transport::TransportEndpoint;

const LEDGER_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors from ledger client operations.
#[derive(Debug, Error)]
pub enum LedgerClientError {
    /// loamSpine socket is not reachable.
    #[error("ledger unavailable: {0}")]
    Unavailable(String),
    /// loamSpine returned a JSON-RPC error.
    #[error("ledger error: {0}")]
    LedgerError(String),
    /// Response parsing failed.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// I/O error communicating with loamSpine.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Response from `braid.commit` on loamSpine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerCommitResponse {
    /// Spine/partition identifier.
    pub spine_id: String,
    /// Entry hash in the ledger.
    pub entry_hash: String,
    /// Entry index/sequence number.
    pub index: u64,
    /// Whether the entry was sealed.
    #[serde(default)]
    pub sealed: bool,
}

/// Response from `certificate.verify` on loamSpine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Whether the certificate/anchor is valid.
    pub valid: bool,
    /// Verification detail message.
    #[serde(default)]
    pub detail: Option<String>,
}

/// JSON-RPC 2.0 client for loamSpine ledger operations.
///
/// Discovered at runtime via capability-based socket resolution.
/// Never hardcodes primal names in logic — uses env-configurable paths.
#[derive(Debug, Clone)]
pub struct LedgerClient {
    endpoint: TransportEndpoint,
    request_id: std::sync::Arc<AtomicU64>,
}

impl LedgerClient {
    /// Create a new ledger client with the given transport endpoint.
    #[must_use]
    pub fn new(endpoint: TransportEndpoint) -> Self {
        Self {
            endpoint,
            request_id: std::sync::Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create from a socket path (UDS convenience constructor).
    #[must_use]
    pub fn from_socket_path(path: &std::path::Path) -> Self {
        Self::new(TransportEndpoint::uds(path.to_string_lossy()))
    }

    /// The resolved transport endpoint.
    #[must_use]
    pub const fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Commit a braid payload to the loamSpine ledger.
    ///
    /// Calls `braid.commit` on loamSpine with the provided payload.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerClientError`] if the socket is unreachable, loamSpine
    /// returns an error, or the response cannot be parsed.
    pub async fn commit_braid(
        &self,
        payload: serde_json::Value,
    ) -> Result<LedgerCommitResponse, LedgerClientError> {
        let result = self.call_jsonrpc("braid.commit", payload).await?;
        serde_json::from_value(result)
            .map_err(|e| LedgerClientError::InvalidResponse(format!("parse commit response: {e}")))
    }

    /// Verify a certificate/anchor via loamSpine.
    ///
    /// Calls `certificate.verify` with the given certificate ID.
    ///
    /// # Errors
    ///
    /// Returns [`LedgerClientError`] if the socket is unreachable, loamSpine
    /// returns an error, or the response cannot be parsed.
    pub async fn verify_certificate(
        &self,
        cert_id: &str,
    ) -> Result<VerifyResponse, LedgerClientError> {
        let params = serde_json::json!({ "certificate_id": cert_id });
        let result = self.call_jsonrpc("certificate.verify", params).await?;
        serde_json::from_value(result)
            .map_err(|e| LedgerClientError::InvalidResponse(format!("parse verify response: {e}")))
    }

    /// Resolve the loamSpine socket from environment.
    ///
    /// Resolution order:
    /// 1. `LOAMSPINE_SOCKET` — explicit override
    /// 2. `{BIOMEOS_SOCKET_DIR}/loamspine-{FAMILY_ID}.sock` — family-scoped
    /// 3. `{BIOMEOS_SOCKET_DIR}/loamspine.sock` — standalone
    #[must_use]
    pub fn resolve_from_env() -> Option<Self> {
        let path = Self::resolve_socket_path()?;
        debug!(socket = %path.display(), "ledger client resolved");
        Some(Self::from_socket_path(&path))
    }

    fn resolve_socket_path() -> Option<PathBuf> {
        let env_name = socket_env_var("loamspine");
        if let Ok(p) = std::env::var(&env_name) {
            let path = PathBuf::from(p);
            if path.exists() {
                return Some(path);
            }
        }

        let socket_dir = resolve_socket_dir()?;

        if let Ok(family_id) = std::env::var(env_vars::FAMILY_ID) {
            let scoped = socket_dir.join(format!("loamspine-{family_id}.sock"));
            if scoped.exists() {
                return Some(scoped);
            }
        }

        let standalone = socket_dir.join("loamspine.sock");
        if standalone.exists() {
            return Some(standalone);
        }

        None
    }

    async fn call_jsonrpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LedgerClientError> {
        let stream = tokio::time::timeout(LEDGER_TIMEOUT, connect_transport(&self.endpoint))
            .await
            .map_err(|_| LedgerClientError::Unavailable("connection timeout".into()))?
            .map_err(|e| LedgerClientError::Unavailable(format!("{e}")))?;

        self.send_request(stream, method, params).await
    }

    async fn send_request(
        &self,
        stream: TransportStream,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, LedgerClientError> {
        let id = self.request_id.fetch_add(1, Ordering::Relaxed);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let mut payload = serde_json::to_string(&request)
            .map_err(|e| LedgerClientError::InvalidResponse(e.to_string()))?;
        payload.push('\n');

        let (reader, mut writer) = tokio::io::split(stream);
        writer.write_all(payload.as_bytes()).await?;
        writer.flush().await?;

        let mut lines = BufReader::new(reader).lines();
        let response_line = tokio::time::timeout(LEDGER_TIMEOUT, lines.next_line())
            .await
            .map_err(|_| LedgerClientError::Unavailable("response timeout".into()))?
            .map_err(LedgerClientError::Io)?
            .ok_or_else(|| LedgerClientError::Unavailable("empty response from ledger".into()))?;

        let response: serde_json::Value = serde_json::from_str(&response_line)
            .map_err(|e| LedgerClientError::InvalidResponse(e.to_string()))?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error");
            return Err(LedgerClientError::LedgerError(msg.to_owned()));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| LedgerClientError::InvalidResponse("no result field".into()))
    }
}

fn resolve_socket_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var(env_vars::BIOMEOS_SOCKET_DIR) {
        return Some(PathBuf::from(dir));
    }
    if let Ok(xdg) = std::env::var(env_vars::XDG_RUNTIME_DIR) {
        return Some(PathBuf::from(xdg).join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR));
    }
    None
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn resolve_returns_none_without_env() {
        temp_env::with_vars(
            [
                ("LOAMSPINE_SOCKET", None::<&str>),
                (env_vars::BIOMEOS_SOCKET_DIR, None::<&str>),
                (env_vars::XDG_RUNTIME_DIR, None::<&str>),
                (env_vars::FAMILY_ID, None::<&str>),
            ],
            || {
                assert!(LedgerClient::resolve_from_env().is_none());
            },
        );
    }

    #[test]
    fn resolve_explicit_env_override() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("loamspine-explicit.sock");
        std::fs::write(&sock, "").unwrap();

        temp_env::with_vars([("LOAMSPINE_SOCKET", Some(sock.to_str().unwrap()))], || {
            let client = LedgerClient::resolve_from_env().unwrap();
            assert!(matches!(client.endpoint(), TransportEndpoint::Uds { .. }));
        });
    }

    #[test]
    fn resolve_family_scoped() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("loamspine-testfam.sock");
        std::fs::write(&sock, "").unwrap();

        temp_env::with_vars(
            [
                ("LOAMSPINE_SOCKET", None::<&str>),
                (
                    env_vars::BIOMEOS_SOCKET_DIR,
                    Some(dir.path().to_str().unwrap()),
                ),
                (env_vars::FAMILY_ID, Some("testfam")),
            ],
            || {
                let client = LedgerClient::resolve_from_env().unwrap();
                assert!(matches!(client.endpoint(), TransportEndpoint::Uds { .. }));
            },
        );
    }

    #[test]
    fn resolve_standalone_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("loamspine.sock");
        std::fs::write(&sock, "").unwrap();

        temp_env::with_vars(
            [
                ("LOAMSPINE_SOCKET", None::<&str>),
                (
                    env_vars::BIOMEOS_SOCKET_DIR,
                    Some(dir.path().to_str().unwrap()),
                ),
                (env_vars::FAMILY_ID, None::<&str>),
            ],
            || {
                let client = LedgerClient::resolve_from_env().unwrap();
                assert!(matches!(client.endpoint(), TransportEndpoint::Uds { .. }));
            },
        );
    }

    #[test]
    fn commit_response_deserializes() {
        let json = r#"{"spine_id":"default","entry_hash":"sha256:abc","index":42,"sealed":true}"#;
        let resp: LedgerCommitResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.spine_id, "default");
        assert_eq!(resp.index, 42);
        assert!(resp.sealed);
    }

    #[test]
    fn verify_response_deserializes() {
        let json = r#"{"valid":true,"detail":"certificate sealed in ledger"}"#;
        let resp: VerifyResponse = serde_json::from_str(json).unwrap();
        assert!(resp.valid);
        assert_eq!(resp.detail.unwrap(), "certificate sealed in ledger");
    }

    #[test]
    fn verify_response_minimal() {
        let json = r#"{"valid":false}"#;
        let resp: VerifyResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.valid);
        assert!(resp.detail.is_none());
    }
}
