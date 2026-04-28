// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Crypto delegation client for `BearDog` Tower signing via UDS JSON-RPC.
//!
//! Implements `crypto.sign` delegation per `NUCLEUS_TWO_TIER_CRYPTO_MODEL`.
//! `sweetGrass` never touches key material — all signing is delegated to
//! `BearDog` over a Unix Domain Socket using newline-delimited JSON-RPC 2.0.

use std::path::{Path, PathBuf};

use base64::Engine;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::debug;

use sweet_grass_core::primal_names::{env_vars, paths};

/// Errors from crypto delegation to `BearDog`.
#[derive(Debug, Error)]
pub enum CryptoDelegateError {
    /// `BearDog` socket is not reachable.
    #[error("crypto provider unavailable: {0}")]
    Unavailable(String),

    /// `BearDog` returned a JSON-RPC error.
    #[error("crypto provider error: {0}")]
    ProviderError(String),

    /// Response parsing failed.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// I/O error communicating with `BearDog`.
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
    /// `BearDog` Tower Ed25519 public key bytes.
    pub public_key: Vec<u8>,
}

/// UDS JSON-RPC client for `BearDog` `crypto.sign` / `crypto.verify`.
#[derive(Debug, Clone)]
pub struct CryptoDelegate {
    socket_path: PathBuf,
}

impl CryptoDelegate {
    /// Resolve the `BearDog` crypto socket from environment variables.
    ///
    /// Resolution order:
    /// 1. `BEARDOG_SOCKET` — explicit socket path
    /// 2. `SECURITY_PROVIDER_SOCKET` — generic crypto provider
    /// 3. `BIOMEOS_SOCKET_DIR/security.sock` — ecosystem convention
    /// 4. `XDG_RUNTIME_DIR/biomeos/security.sock`
    ///
    /// Returns `None` if no viable socket path can be determined.
    #[must_use]
    pub fn resolve() -> Option<Self> {
        let path = Self::resolve_socket_path()?;
        debug!(socket = %path.display(), "crypto delegate resolved");
        Some(Self { socket_path: path })
    }

    /// Create with an explicit socket path (for testing / DI).
    #[must_use]
    pub const fn with_socket(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// The resolved socket path.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
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
            .call_jsonrpc(
                "crypto.sign",
                serde_json::json!({ "message": message_b64 }),
            )
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

        let signature = b64.decode(sig_b64).map_err(|e| {
            CryptoDelegateError::InvalidResponse(format!("signature base64: {e}"))
        })?;

        let public_key = b64.decode(pub_b64).map_err(|e| {
            CryptoDelegateError::InvalidResponse(format!("public_key base64: {e}"))
        })?;

        Ok(CryptoSignResult {
            signature,
            algorithm,
            public_key,
        })
    }

    fn resolve_socket_path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var(env_vars::BEARDOG_SOCKET) {
            return Some(PathBuf::from(p));
        }

        if let Ok(p) = std::env::var(env_vars::SECURITY_PROVIDER_SOCKET) {
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
        let stream =
            tokio::net::UnixStream::connect(&self.socket_path)
                .await
                .map_err(|e| {
                    CryptoDelegateError::Unavailable(format!(
                        "{}: {e}",
                        self.socket_path.display()
                    ))
                })?;

        let (reader, mut writer) = stream.into_split();

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

    async fn start_mock_beardog(
        listener: StdUnixListener,
    ) -> tokio::task::JoinHandle<()> {
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
        let handle = start_mock_beardog(std_listener).await;

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
    fn test_resolve_beardog_socket_first() {
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
                assert_eq!(d.socket_path(), Path::new("/run/beardog.sock"));
            },
        );
    }

    #[test]
    fn test_resolve_falls_through_to_security_provider() {
        temp_env::with_vars(
            [
                (env_vars::BEARDOG_SOCKET, None::<&str>),
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
}
