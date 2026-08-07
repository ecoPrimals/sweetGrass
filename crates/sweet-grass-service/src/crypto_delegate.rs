// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability-based crypto delegation via transport-agnostic JSON-RPC.
//!
//! Implements `crypto.sign` delegation per `NUCLEUS_TWO_TIER_CRYPTO_MODEL`.
//! sweetGrass never touches key material — all signing is delegated to
//! whichever primal provides the `Capability::Signing` domain (currently
//! `BearDog`) over the transport layer using newline-delimited JSON-RPC 2.0.
//!
//! G66 evolution: uses `TransportEndpoint` + `connect_transport()` instead of
//! raw `UnixStream`. Works on all platforms without silicon deism.

use std::path::{Path, PathBuf};

use base64::Engine;
use sweet_grass_core::primal_names::{env_vars, paths};
use sweet_grass_core::transport::TransportEndpoint;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::debug;

/// Errors from crypto delegation to the signing capability provider.
#[derive(Debug, Error)]
pub enum CryptoDelegateError {
    /// Signing provider socket is not reachable.
    #[error("crypto provider unavailable: {0}")]
    Unavailable(String),

    /// Signing provider returned a JSON-RPC error.
    #[error("crypto provider error: {0}")]
    ProviderError(String),

    /// Response parsing failed.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// I/O error communicating with signing provider.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Result of a successful `crypto.sign` call.
#[derive(Debug, Clone)]
pub struct CryptoSignResult {
    /// Raw Ed25519 signature bytes.
    pub signature: Vec<u8>,
    /// Algorithm identifier (e.g. `"ed25519"`).
    pub algorithm: String,
    /// Signing provider Ed25519 public key bytes.
    pub public_key: Vec<u8>,
}

/// Capability-based JSON-RPC client for `crypto.sign` / `crypto.verify`.
///
/// Discovers the signing provider at runtime via environment tiers — the
/// primal providing `Capability::Signing` is never hardcoded.
///
/// G66: stores a `TransportEndpoint` instead of a raw socket path.
/// Works on all platforms via `connect_transport()`.
#[derive(Debug, Clone)]
pub struct CryptoDelegate {
    endpoint: TransportEndpoint,
    socket_path: PathBuf,
}

impl CryptoDelegate {
    /// Resolve the signing capability provider from environment.
    ///
    /// Resolution order (first match wins):
    /// 1. `SECURITY_PROVIDER_SOCKET` — explicit capability socket
    /// 2. `BEARDOG_SOCKET` — primal-specific alias (deployment shortcut)
    /// 3. `BIOMEOS_SOCKET_DIR/security.sock` — ecosystem convention
    /// 4. `XDG_RUNTIME_DIR/biomeos/security.sock`
    ///
    /// Returns `None` if no viable socket path can be determined.
    #[must_use]
    pub fn resolve() -> Option<Self> {
        let path = Self::resolve_socket_path()?;
        let endpoint = TransportEndpoint::uds(path.to_string_lossy());
        debug!(socket = %path.display(), "crypto delegate resolved");
        Some(Self {
            endpoint,
            socket_path: path,
        })
    }

    /// Create with an explicit socket path (for testing / DI).
    #[must_use]
    pub fn with_socket(socket_path: PathBuf) -> Self {
        let endpoint = TransportEndpoint::uds(socket_path.to_string_lossy());
        Self {
            endpoint,
            socket_path,
        }
    }

    /// Create with a transport endpoint directly (G66 pattern).
    #[must_use]
    pub fn with_endpoint(endpoint: TransportEndpoint) -> Self {
        let socket_path = match &endpoint {
            TransportEndpoint::Uds { path } => PathBuf::from(path),
            _ => PathBuf::new(),
        };
        Self {
            endpoint,
            socket_path,
        }
    }

    /// The resolved socket path (for backward compat — prefer `endpoint()`).
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// The resolved transport endpoint.
    #[must_use]
    pub const fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Sign a message via `BearDog` `crypto.sign`.
    ///
    /// The message bytes are base64-encoded before sending per
    /// `CRYPTO_WIRE_CONTRACT.md`.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoDelegateError`] if the socket is unreachable, `BearDog`
    /// returns an error, or the response cannot be parsed.
    pub async fn sign(&self, message: &[u8]) -> Result<CryptoSignResult, CryptoDelegateError> {
        let message_b64 = base64::engine::general_purpose::STANDARD.encode(message);

        let result = self
            .call_jsonrpc("crypto.sign", serde_json::json!({ "message": message_b64 }))
            .await?;

        let b64 = base64::engine::general_purpose::STANDARD;

        let sig_b64 = result
            .get("signature")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                CryptoDelegateError::InvalidResponse("missing `signature` field".into())
            })?;

        let algorithm = result
            .get("algorithm")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("ed25519")
            .to_owned();

        let pub_b64 = result
            .get("public_key")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                CryptoDelegateError::InvalidResponse("missing `public_key` field".into())
            })?;

        let signature = b64
            .decode(sig_b64)
            .map_err(|e| CryptoDelegateError::InvalidResponse(format!("signature base64: {e}")))?;

        let public_key = b64
            .decode(pub_b64)
            .map_err(|e| CryptoDelegateError::InvalidResponse(format!("public_key base64: {e}")))?;

        Ok(CryptoSignResult {
            signature,
            algorithm,
            public_key,
        })
    }

    fn resolve_socket_path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var(env_vars::SECURITY_PROVIDER_SOCKET) {
            return Some(PathBuf::from(p));
        }

        if let Ok(p) = std::env::var(env_vars::BEARDOG_SOCKET) {
            return Some(PathBuf::from(p));
        }

        if let Ok(dir) = std::env::var(env_vars::BIOMEOS_SOCKET_DIR) {
            return Some(PathBuf::from(dir).join("security.sock"));
        }

        if let Ok(xdg) = std::env::var(env_vars::XDG_RUNTIME_DIR) {
            return Some(
                PathBuf::from(xdg)
                    .join(paths::BIOMEOS_DIR)
                    .join("security.sock"),
            );
        }

        None
    }

    async fn call_jsonrpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, CryptoDelegateError> {
        let mut stream = crate::transport_connect::connect_transport(&self.endpoint)
            .await
            .map_err(|e| {
                CryptoDelegateError::Unavailable(format!("{}: {e}", self.endpoint))
            })?;

        if crate::btsp_client::btsp_strict_mode_expected() {
            crate::btsp_client::perform_client_handshake(&mut stream)
                .await
                .map_err(|e| CryptoDelegateError::Unavailable(format!("BTSP handshake: {e}")))?;
            debug!("BTSP handshake complete, sending JSON-RPC");
        }

        let (reader, writer) = tokio::io::split(stream);
        let mut writer = writer;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut payload = serde_json::to_string(&request)
            .map_err(|e| CryptoDelegateError::InvalidResponse(e.to_string()))?;
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await?;

        let mut lines = BufReader::new(reader).lines();
        let response_line = lines.next_line().await?.ok_or_else(|| {
            CryptoDelegateError::Unavailable("empty response from crypto provider".into())
        })?;

        let response: serde_json::Value = serde_json::from_str(&response_line)
            .map_err(|e| CryptoDelegateError::InvalidResponse(e.to_string()))?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error");
            return Err(CryptoDelegateError::ProviderError(msg.to_owned()));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| CryptoDelegateError::InvalidResponse("no result field".into()))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn start_mock_beardog(listener: StdUnixListener) -> tokio::task::JoinHandle<()> {
        listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::UnixListener::from_std(listener).unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();

            if let Some(line) = lines.next_line().await.unwrap() {
                let req: serde_json::Value = serde_json::from_str(&line).unwrap();
                let method = req["method"].as_str().unwrap();

                let response = if method == "crypto.sign" {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "result": {
                            "signature": "dGVzdC1zaWduYXR1cmUtYnl0ZXM=",
                            "algorithm": "ed25519",
                            "public_key": "dGVzdC1wdWJsaWMta2V5LWJ5dGVz"
                        }
                    })
                } else {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "error": { "code": -32601, "message": "Method not found" }
                    })
                };

                let mut resp = serde_json::to_string(&response).unwrap();
                resp.push('\n');
                writer.write_all(resp.as_bytes()).await.unwrap();
            }
        })
    }

    #[tokio::test]
    async fn test_crypto_sign_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("beardog-test.sock");

        let std_listener = StdUnixListener::bind(&sock).unwrap();
        let handle = start_mock_beardog(std_listener);

        let delegate = CryptoDelegate::with_socket(sock);
        let result = delegate.sign(b"hello provenance").await.unwrap();

        assert_eq!(result.algorithm, "ed25519");
        assert_eq!(result.signature, b"test-signature-bytes");
        assert_eq!(result.public_key, b"test-public-key-bytes");

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_crypto_sign_unavailable() {
        let delegate = CryptoDelegate::with_socket(PathBuf::from("/nonexistent/beardog.sock"));
        let err = delegate.sign(b"hello").await.unwrap_err();
        assert!(
            matches!(err, CryptoDelegateError::Unavailable(_)),
            "expected Unavailable, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_crypto_sign_error_response() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("beardog-err.sock");

        let std_listener = StdUnixListener::bind(&sock).unwrap();
        std_listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::UnixListener::from_std(std_listener).unwrap();

        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();
            let _ = lines.next_line().await.unwrap();

            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": { "code": -32000, "message": "key not loaded" }
            });
            let mut s = serde_json::to_string(&resp).unwrap();
            s.push('\n');
            writer.write_all(s.as_bytes()).await.unwrap();
        });

        let delegate = CryptoDelegate::with_socket(sock);
        let err = delegate.sign(b"test").await.unwrap_err();
        assert!(
            matches!(err, CryptoDelegateError::ProviderError(_)),
            "expected ProviderError, got: {err}"
        );
        assert!(err.to_string().contains("key not loaded"));

        handle.await.unwrap();
    }

    #[test]
    fn test_resolve_returns_none_without_env() {
        temp_env::with_vars(
            [
                (env_vars::BEARDOG_SOCKET, None::<&str>),
                (env_vars::SECURITY_PROVIDER_SOCKET, None::<&str>),
                (env_vars::BIOMEOS_SOCKET_DIR, None::<&str>),
                (env_vars::XDG_RUNTIME_DIR, None::<&str>),
            ],
            || {
                assert!(CryptoDelegate::resolve().is_none());
            },
        );
    }

    #[test]
    fn test_resolve_security_provider_first() {
        temp_env::with_vars(
            [
                (env_vars::BEARDOG_SOCKET, Some("/run/beardog.sock")),
                (
                    env_vars::SECURITY_PROVIDER_SOCKET,
                    Some("/run/security.sock"),
                ),
            ],
            || {
                let d = CryptoDelegate::resolve().unwrap();
                assert_eq!(d.socket_path(), Path::new("/run/security.sock"));
            },
        );
    }

    #[test]
    fn test_resolve_falls_through_to_beardog_alias() {
        temp_env::with_vars(
            [
                (env_vars::SECURITY_PROVIDER_SOCKET, None::<&str>),
                (env_vars::BEARDOG_SOCKET, Some("/run/beardog.sock")),
            ],
            || {
                let d = CryptoDelegate::resolve().unwrap();
                assert_eq!(d.socket_path(), Path::new("/run/beardog.sock"));
            },
        );
    }
}
