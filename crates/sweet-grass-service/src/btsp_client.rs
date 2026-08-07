// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP client-side handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set on the security provider,
//! plain JSON-RPC is rejected. This module implements the consumer-side
//! 4-step BTSP handshake so sweetGrass can authenticate before delegating
//! `crypto.sign` requests.
//!
//! The challenge response uses LOCAL HMAC-SHA256 with the family seed —
//! this avoids the chicken-and-egg of needing bearDog to compute HMAC for
//! the handshake that authenticates us TO bearDog.
//!
//! ## Wire Format (NDJSON — newline-delimited)
//!
//! ```text
//! 1. Send  ClientHello       { protocol: "btsp", version: 1, client_ephemeral_pub }
//! 2. Read  ServerHello       { version, server_ephemeral_pub, challenge, session_id }
//! 3. Send  ChallengeResponse { response, preferred_cipher }
//! 4. Read  HandshakeComplete { cipher, session_id }
//! ```
//!
//! Reference: `songBird/crates/songbird-crypto-provider/src/btsp_client.rs`

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::debug;

use sweet_grass_core::primal_names::env_vars;

type HmacSha256 = Hmac<Sha256>;

const BTSP_VERSION: u8 = 1;
const PREFERRED_CIPHER: &str = "chacha20_poly1305";

#[derive(Debug, Serialize)]
struct ClientHello {
    protocol: &'static str,
    version: u8,
    client_ephemeral_pub: String,
}

#[derive(Debug, Deserialize)]
struct ServerHello {
    version: u8,
    server_ephemeral_pub: String,
    challenge: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    response: String,
    preferred_cipher: &'static str,
}

#[derive(Debug, Deserialize)]
struct HandshakeComplete {
    /// Negotiated cipher suite.
    pub cipher: String,
    /// Server-assigned session identifier.
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
struct HandshakeError {
    pub error: String,
    pub reason: String,
}

/// Result of a successful client-side BTSP handshake.
#[derive(Debug, Clone)]
pub struct BtspClientSession {
    /// Server-assigned session identifier.
    pub session_id: String,
    /// Negotiated cipher (e.g. `chacha20_poly1305` or `null`).
    pub cipher: String,
}

/// Errors from the BTSP client handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspClientError {
    /// Family seed not available in environment.
    #[error("FAMILY_SEED not available — cannot perform BTSP handshake")]
    NoFamilySeed,
    /// I/O error on the UDS stream during handshake.
    #[error("I/O error during BTSP handshake: {0}")]
    Io(#[from] std::io::Error),
    /// Server explicitly rejected the handshake.
    #[error("server rejected handshake: {0}")]
    Rejected(String),
    /// Malformed or unexpected response from server.
    #[error("malformed server response: {0}")]
    Protocol(String),
    /// HMAC computation failed (invalid key length).
    #[error("HMAC computation failed")]
    Hmac,
}

/// Resolve the raw family seed from environment.
///
/// Checks `FAMILY_SEED` first, then `BEARDOG_FAMILY_SEED` as fallback.
fn resolve_family_seed() -> Option<String> {
    std::env::var(env_vars::FAMILY_SEED)
        .or_else(|_| std::env::var(env_vars::BEARDOG_FAMILY_SEED))
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Check whether BTSP strict mode is expected (bearDog requires handshake).
///
/// Returns `true` if `BEARDOG_UDS_REQUIRE_BTSP=1` or `BTSP_STRICT_MODE=1`.
#[must_use]
pub fn btsp_strict_mode_expected() -> bool {
    std::env::var(env_vars::BEARDOG_UDS_REQUIRE_BTSP)
        .or_else(|_| std::env::var(env_vars::BTSP_STRICT_MODE))
        .is_ok_and(|v| v.trim() == "1")
}

/// Perform the client-side BTSP handshake over any transport stream.
///
/// Authenticates to bearDog using the family seed from environment.
/// After success, the stream is ready for JSON-RPC traffic.
///
/// # Errors
///
/// Returns [`BtspClientError`] if the family seed is unavailable, the server
/// rejects the handshake, or I/O fails.
pub async fn perform_client_handshake<S>(
    stream: &mut S,
) -> Result<BtspClientSession, BtspClientError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let family_seed = resolve_family_seed().ok_or(BtspClientError::NoFamilySeed)?;

    let mut ephemeral_key = [0u8; 32];
    getrandom::fill(&mut ephemeral_key).map_err(|_| BtspClientError::Hmac)?;

    // Step 1: Send ClientHello
    let hello = ClientHello {
        protocol: "btsp",
        version: BTSP_VERSION,
        client_ephemeral_pub: BASE64_STANDARD.encode(ephemeral_key),
    };
    let hello_json = serde_json::to_string(&hello)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ClientHello: {e}")))?;
    stream
        .write_all(hello_json.as_bytes())
        .await
        .map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ClientHello");

    // Step 2: Read ServerHello (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    buf_reader
        .read_line(&mut line)
        .await
        .map_err(BtspClientError::Io)?;

    if line.trim().is_empty() {
        return Err(BtspClientError::Protocol(String::from(
            "empty response from server",
        )));
    }

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(format!(
            "{}: {}",
            err.error, err.reason
        )));
    }

    let server_hello: ServerHello = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse ServerHello: {e}")))?;

    if server_hello.version != BTSP_VERSION {
        return Err(BtspClientError::Protocol(format!(
            "version mismatch: expected {BTSP_VERSION}, got {}",
            server_hello.version
        )));
    }

    debug!(
        session_id = %server_hello.session_id,
        server_pub_len = server_hello.server_ephemeral_pub.len(),
        "BTSP client: received ServerHello"
    );

    // Step 3: Compute HMAC-SHA256(family_seed, challenge) and send response
    let challenge_bytes = BASE64_STANDARD
        .decode(&server_hello.challenge)
        .map_err(|e| BtspClientError::Protocol(format!("decode challenge: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(family_seed.trim().as_bytes())
        .map_err(|_| BtspClientError::Hmac)?;
    mac.update(&challenge_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let response = ChallengeResponse {
        response: BASE64_STANDARD.encode(hmac_result),
        preferred_cipher: PREFERRED_CIPHER,
    };
    let resp_json = serde_json::to_string(&response)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ChallengeResponse: {e}")))?;

    let stream = buf_reader.into_inner();
    stream
        .write_all(resp_json.as_bytes())
        .await
        .map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ChallengeResponse");

    // Step 4: Read HandshakeComplete (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    buf_reader
        .read_line(&mut line)
        .await
        .map_err(BtspClientError::Io)?;

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let complete: HandshakeComplete = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse HandshakeComplete: {e}")))?;

    debug!(
        session_id = %complete.session_id,
        cipher = %complete.cipher,
        "BTSP client: handshake COMPLETE"
    );

    Ok(BtspClientSession {
        session_id: complete.session_id,
        cipher: complete.cipher,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tokio::net::UnixStream;

    #[test]
    fn btsp_strict_mode_default_off() {
        temp_env::with_vars(
            [
                ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
                ("BTSP_STRICT_MODE", None::<&str>),
            ],
            || {
                assert!(!btsp_strict_mode_expected());
            },
        );
    }

    #[test]
    fn btsp_strict_mode_on() {
        temp_env::with_vars([("BEARDOG_UDS_REQUIRE_BTSP", Some("1"))], || {
            assert!(btsp_strict_mode_expected());
        });
    }

    #[test]
    fn resolve_family_seed_from_env() {
        temp_env::with_vars(
            [
                (env_vars::FAMILY_SEED, Some("test-seed-hex")),
                (env_vars::BEARDOG_FAMILY_SEED, None::<&str>),
            ],
            || {
                assert_eq!(resolve_family_seed().unwrap(), "test-seed-hex");
            },
        );
    }

    #[test]
    fn resolve_family_seed_fallback() {
        temp_env::with_vars(
            [
                (env_vars::FAMILY_SEED, None::<&str>),
                (env_vars::BEARDOG_FAMILY_SEED, Some("beardog-seed")),
            ],
            || {
                assert_eq!(resolve_family_seed().unwrap(), "beardog-seed");
            },
        );
    }

    #[test]
    fn resolve_family_seed_empty_returns_none() {
        temp_env::with_vars(
            [
                (env_vars::FAMILY_SEED, Some("")),
                (env_vars::BEARDOG_FAMILY_SEED, Some("  ")),
            ],
            || {
                assert!(resolve_family_seed().is_none());
            },
        );
    }

    #[test]
    fn hmac_computation_produces_32_bytes() {
        let key = b"test-family-seed";
        let challenge = b"random-challenge-data";
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(challenge);
        let result = mac.finalize().into_bytes();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn client_hello_serializes_correctly() {
        let hello = ClientHello {
            protocol: "btsp",
            version: 1,
            client_ephemeral_pub: String::from("AAAA"),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("\"protocol\":\"btsp\""));
        assert!(json.contains("\"version\":1"));
    }

    #[test]
    fn test_full_handshake_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("btsp-test.sock");
        let sock_path_clone = sock_path.clone();

        let std_listener = StdUnixListener::bind(&sock_path).unwrap();

        temp_env::with_vars(
            [
                (env_vars::FAMILY_SEED, Some("my-test-seed")),
                (env_vars::BEARDOG_FAMILY_SEED, None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    std_listener.set_nonblocking(true).unwrap();
                    let listener =
                        tokio::net::UnixListener::from_std(std_listener).unwrap();

                    let server_handle = tokio::spawn(async move {
                        let (stream, _) = listener.accept().await.unwrap();
                        let (reader, mut writer) = stream.into_split();
                        let mut lines = BufReader::new(reader).lines();

                        let hello_line = lines.next_line().await.unwrap().unwrap();
                        let hello: serde_json::Value =
                            serde_json::from_str(&hello_line).unwrap();
                        assert_eq!(hello["protocol"], "btsp");
                        assert_eq!(hello["version"], 1);

                        let challenge =
                            BASE64_STANDARD.encode(b"test-challenge-32bytes-padding!");
                        let server_hello = serde_json::json!({
                            "version": 1,
                            "server_ephemeral_pub": BASE64_STANDARD.encode(b"server-ephemeral-key-32bytes!!!!"),
                            "challenge": challenge,
                            "session_id": "sess-001"
                        });
                        let mut resp = serde_json::to_string(&server_hello).unwrap();
                        resp.push('\n');
                        writer.write_all(resp.as_bytes()).await.unwrap();

                        let cr_line = lines.next_line().await.unwrap().unwrap();
                        let cr: serde_json::Value =
                            serde_json::from_str(&cr_line).unwrap();
                        assert_eq!(cr["preferred_cipher"], "chacha20_poly1305");

                        let response_b64 = cr["response"].as_str().unwrap();
                        let response_bytes =
                            BASE64_STANDARD.decode(response_b64).unwrap();
                        let mut mac =
                            HmacSha256::new_from_slice(b"my-test-seed").unwrap();
                        mac.update(b"test-challenge-32bytes-padding!");
                        mac.verify_slice(&response_bytes).unwrap();

                        let complete = serde_json::json!({
                            "cipher": "chacha20_poly1305",
                            "session_id": "sess-001"
                        });
                        let mut s = serde_json::to_string(&complete).unwrap();
                        s.push('\n');
                        writer.write_all(s.as_bytes()).await.unwrap();
                    });

                    let mut stream =
                        UnixStream::connect(&sock_path_clone).await.unwrap();
                    let session =
                        perform_client_handshake(&mut stream).await.unwrap();
                    assert_eq!(session.session_id, "sess-001");
                    assert_eq!(session.cipher, "chacha20_poly1305");

                    server_handle.await.unwrap();
                });
            },
        );
    }

    #[test]
    fn test_handshake_no_family_seed() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("btsp-noseed.sock");
        let sock_path_clone = sock_path.clone();

        let _listener = StdUnixListener::bind(&sock_path).unwrap();

        temp_env::with_vars(
            [
                (env_vars::FAMILY_SEED, None::<&str>),
                (env_vars::BEARDOG_FAMILY_SEED, None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut stream = UnixStream::connect(&sock_path_clone).await.unwrap();
                    let err = perform_client_handshake(&mut stream).await.unwrap_err();
                    assert!(matches!(err, BtspClientError::NoFamilySeed));
                });
            },
        );
    }

    #[test]
    fn test_handshake_server_rejects() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("btsp-reject.sock");
        let sock_path_clone = sock_path.clone();

        let std_listener = StdUnixListener::bind(&sock_path).unwrap();

        temp_env::with_vars([(env_vars::FAMILY_SEED, Some("any-seed"))], || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                std_listener.set_nonblocking(true).unwrap();
                let listener = tokio::net::UnixListener::from_std(std_listener).unwrap();

                let server_handle = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.unwrap();
                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader).lines();
                    let _ = lines.next_line().await.unwrap();

                    let err = serde_json::json!({
                        "error": "handshake_failed",
                        "reason": "unsupported protocol version"
                    });
                    let mut s = serde_json::to_string(&err).unwrap();
                    s.push('\n');
                    writer.write_all(s.as_bytes()).await.unwrap();
                });

                let mut stream = UnixStream::connect(&sock_path_clone).await.unwrap();
                let err = perform_client_handshake(&mut stream).await.unwrap_err();
                assert!(matches!(err, BtspClientError::Rejected(_)));
                assert!(err.to_string().contains("unsupported protocol version"));

                server_handle.await.unwrap();
            });
        });
    }
}
