// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP server-side handshake.
//!
//! Delegates all crypto to `BearDog` via JSON-RPC (`btsp.session.create`,
//! `btsp.session.verify`, `btsp.negotiate`) per the
//! `BTSP_PROTOCOL_STANDARD.md` §Phase 2 pattern.

use base64::Engine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};

use super::protocol::{
    BtspError, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError, ServerHello,
    read_jsonline, read_message, write_jsonline, write_message,
};

/// Default socket name for the ecosystem security provider.
///
/// This is the ecosystem convention per `BTSP_PROTOCOL_STANDARD.md` §Phase 2:
/// the security provider socket lives in the biomeos socket directory alongside
/// other primal sockets.  Production deployments override via
/// `SECURITY_PROVIDER_SOCKET`.
const DEFAULT_SECURITY_SOCKET: &str = "security.sock";

/// Resolved socket path for the security provider.
///
/// Per the capability-based discovery principle, this resolves at runtime
/// through environment-configurable paths — no hardcoded primal names.
///
/// Resolution order:
/// 1. `SECURITY_PROVIDER_SOCKET` — explicit override for any crypto provider
/// 2. `BIOMEOS_SOCKET_DIR/security.sock` — capability-domain symlink
/// 3. `$XDG_RUNTIME_DIR/biomeos/security.sock`
/// 4. `$TMPDIR/security.sock`
fn resolve_security_socket() -> std::path::PathBuf {
    use sweet_grass_core::primal_names::{env_vars, paths};

    if let Ok(path) = std::env::var(env_vars::SECURITY_PROVIDER_SOCKET) {
        return std::path::PathBuf::from(path);
    }

    if let Ok(dir) = std::env::var(env_vars::BIOMEOS_SOCKET_DIR) {
        return std::path::PathBuf::from(dir).join(DEFAULT_SECURITY_SOCKET);
    }

    if let Ok(xdg) = std::env::var(env_vars::XDG_RUNTIME_DIR) {
        return std::path::PathBuf::from(xdg)
            .join(paths::BIOMEOS_DIR)
            .join(DEFAULT_SECURITY_SOCKET);
    }

    std::env::temp_dir().join(DEFAULT_SECURITY_SOCKET)
}

/// Read the family seed from the environment and base64-encode it for
/// the `btsp.session.create` RPC.
///
/// Resolution order:
/// 1. `FAMILY_SEED` — canonical seed variable set by primalSpring guidestone
/// 2. `BEARDOG_FAMILY_SEED` — explicit BearDog-scoped alias
///
/// The env value is typically a hex string (64 ASCII chars = 32 seed
/// bytes).  `BearDog` expects the raw UTF-8 bytes base64-encoded in the
/// `family_seed` JSON-RPC param.
///
/// # Errors
///
/// Returns [`BtspError::CryptoProviderUnavailable`] when neither variable
/// is set.
fn resolve_family_seed() -> Result<String, BtspError> {
    use sweet_grass_core::primal_names::env_vars;

    let raw = std::env::var(env_vars::FAMILY_SEED)
        .or_else(|_| std::env::var(env_vars::BEARDOG_FAMILY_SEED))
        .map_err(|_| {
            BtspError::CryptoProviderUnavailable(
                "FAMILY_SEED not set (checked FAMILY_SEED and BEARDOG_FAMILY_SEED)".to_string(),
            )
        })?;

    Ok(base64::engine::general_purpose::STANDARD.encode(raw.as_bytes()))
}

/// Call a security-provider JSON-RPC method at an explicit socket path.
///
/// Capability-based: this function targets whichever primal provides the
/// `crypto.*` capability domain, discovered via `SECURITY_PROVIDER_SOCKET`
/// or `{BIOMEOS_SOCKET_DIR}/security.sock`.
async fn call_security_provider_at(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, BtspError> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let stream = tokio::net::UnixStream::connect(socket_path)
        .await
        .map_err(|e| {
            BtspError::CryptoProviderUnavailable(format!("{}: {e}", socket_path.display()))
        })?;

    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let mut req_str = serde_json::to_string(&request)?;
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await?;

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines
        .next_line()
        .await?
        .ok_or_else(|| BtspError::CryptoProviderUnavailable("empty response".to_string()))?;

    let response: serde_json::Value = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
        return Err(BtspError::CryptoProviderUnavailable(
            error
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown error")
                .to_string(),
        ));
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| BtspError::CryptoProviderUnavailable("no result field".to_string()))
}

/// Intermediate state after `btsp.session.create` succeeds.
///
/// `session_token` is `BearDog`'s opaque handle; `session_id` is resolved
/// later from the `btsp.session.verify` response.
struct SessionContext {
    session_token: String,
    server_pub: String,
    challenge: String,
}

/// Extract a required string field from a JSON-RPC result.
fn extract_str(value: &serde_json::Value, field: &str) -> Result<String, BtspError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .ok_or_else(|| BtspError::CryptoProviderUnavailable(format!("missing {field}")))
}

/// Step 1–2: Read `ClientHello` and create a session via `BearDog`.
async fn receive_hello_and_create_session<S>(
    stream: &mut S,
    security_socket: &std::path::Path,
) -> Result<(ClientHello, SessionContext), BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    let client_hello: ClientHello = read_message(stream).await.map_err(|e| {
        debug!("BTSP: failed to read ClientHello: {e}");
        e
    })?;

    if client_hello.version != 1 {
        let err = HandshakeError {
            error: "handshake_failed".to_owned(),
            reason: format!("unsupported version: {}", client_hello.version),
        };
        let _ = write_message(stream, &err).await;
        return Err(BtspError::HandshakeFailed { reason: err.reason });
    }

    debug!(client_pub = %client_hello.client_ephemeral_pub, "BTSP: received ClientHello");

    let family_seed = resolve_family_seed()?;

    let session = call_security_provider_at(
        security_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed": family_seed,
        }),
    )
    .await?;

    let ctx = SessionContext {
        session_token: extract_str(&session, "session_token")?,
        server_pub: extract_str(&session, "server_ephemeral_pub")?,
        challenge: extract_str(&session, "challenge")?,
    };

    Ok((client_hello, ctx))
}

/// Steps 3–5: Exchange challenge, verify response via `BearDog`.
///
/// Returns `(ChallengeResponse, session_id)` — the `session_id` comes from
/// `BearDog`'s verify response (falling back to `session_token` if absent).
async fn exchange_challenge<S>(
    stream: &mut S,
    client_hello: &ClientHello,
    ctx: &SessionContext,
    security_socket: &std::path::Path,
) -> Result<(ChallengeResponse, String), BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    let server_hello = ServerHello {
        version: 1,
        server_ephemeral_pub: ctx.server_pub.clone(),
        challenge: ctx.challenge.clone(),
    };
    write_message(stream, &server_hello).await?;
    debug!("BTSP: sent ServerHello with challenge");

    let challenge_response: ChallengeResponse = read_message(stream).await?;
    debug!("BTSP: received ChallengeResponse");

    let verify_result = call_security_provider_at(
        security_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_token": ctx.session_token,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "response": challenge_response.response,
            "preferred_cipher": challenge_response.preferred_cipher,
        }),
    )
    .await?;

    let verified = verify_result
        .get("verified")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if !verified {
        let err = HandshakeError {
            error: "handshake_failed".to_owned(),
            reason: "family_verification".to_owned(),
        };
        write_message(stream, &err).await?;
        warn!("BTSP: handshake failed — family verification");
        return Err(BtspError::HandshakeFailed { reason: err.reason });
    }

    let session_id = verify_result
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&ctx.session_token)
        .to_owned();

    Ok((challenge_response, session_id))
}

/// Run the server-side BTSP handshake on an accepted connection.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Handshake Sequence:
/// 1. Read `ClientHello` (length-prefixed)
/// 2. Call `BearDog` `btsp.session.create` to get server ephemeral key + challenge
/// 3. Send `ServerHello`
/// 4. Read `ChallengeResponse`
/// 5. Call `BearDog` `btsp.session.verify` to validate
/// 6. Send `HandshakeComplete` or `HandshakeError`
///
/// On success, the stream is ready for length-prefixed JSON-RPC frames.
/// On failure, `HandshakeError` is sent and the connection is dropped.
///
/// # Errors
///
/// Returns [`BtspError`] on I/O, protocol, or verification failure.
pub async fn perform_server_handshake<S>(stream: &mut S) -> Result<HandshakeComplete, BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    perform_server_handshake_with(stream, &resolve_security_socket()).await
}

/// Run the server-side BTSP handshake using an explicit security-provider
/// socket path (DI-friendly for integration tests).
///
/// # Errors
///
/// Returns [`BtspError`] on I/O, protocol, or verification failure.
pub async fn perform_server_handshake_with<S>(
    stream: &mut S,
    security_socket: &std::path::Path,
) -> Result<HandshakeComplete, BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    let (client_hello, ctx) = receive_hello_and_create_session(stream, security_socket).await?;
    let (challenge_response, session_id) =
        exchange_challenge(stream, &client_hello, &ctx, security_socket).await?;

    let negotiate_result = call_security_provider_at(
        security_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_token": ctx.session_token,
            "cipher": challenge_response.preferred_cipher,
        }),
    )
    .await?;

    let cipher = negotiate_result
        .get("cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();

    let complete = HandshakeComplete { cipher, session_id };
    write_message(stream, &complete).await?;

    info!(
        cipher = %complete.cipher,
        session = %complete.session_id,
        "BTSP: handshake complete"
    );

    Ok(complete)
}

/// Run the server-side BTSP handshake using **JSON-line** framing.
///
/// This is the wire format used by primalSpring: newline-delimited JSON
/// messages instead of 4-byte length-prefixed frames. The `ClientHello`
/// has already been parsed from the first line by the auto-detect layer
/// (it starts with `{"protocol":"btsp",...}`).
///
/// After a successful handshake the stream is in **newline-delimited
/// JSON-RPC** mode (same as `handle_*_connection_raw`).
///
/// # Errors
///
/// Returns [`BtspError`] on I/O, protocol, or verification failure.
pub async fn perform_server_handshake_jsonline<S>(
    stream: &mut S,
    client_hello: ClientHello,
) -> Result<HandshakeComplete, BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    perform_server_handshake_jsonline_with(stream, client_hello, &resolve_security_socket()).await
}

/// JSON-line handshake with explicit security-provider socket (DI-friendly).
///
/// # Errors
///
/// Returns [`BtspError`] on I/O, protocol, or verification failure.
pub async fn perform_server_handshake_jsonline_with<S>(
    stream: &mut S,
    client_hello: ClientHello,
    security_socket: &std::path::Path,
) -> Result<HandshakeComplete, BtspError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin + Send,
{
    if client_hello.version != 1 {
        let err = HandshakeError {
            error: "handshake_failed".to_owned(),
            reason: format!("unsupported version: {}", client_hello.version),
        };
        let _ = write_jsonline(stream, &err).await;
        return Err(BtspError::HandshakeFailed { reason: err.reason });
    }

    debug!(
        client_pub = %client_hello.client_ephemeral_pub,
        "BTSP JSON-line: received ClientHello"
    );

    let family_seed = resolve_family_seed()?;

    let session = call_security_provider_at(
        security_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed": family_seed,
        }),
    )
    .await?;

    let ctx = SessionContext {
        session_token: extract_str(&session, "session_token")?,
        server_pub: extract_str(&session, "server_ephemeral_pub")?,
        challenge: extract_str(&session, "challenge")?,
    };

    let server_hello = ServerHello {
        version: 1,
        server_ephemeral_pub: ctx.server_pub.clone(),
        challenge: ctx.challenge.clone(),
    };
    write_jsonline(stream, &server_hello).await?;
    debug!("BTSP JSON-line: sent ServerHello with challenge");

    let challenge_response: ChallengeResponse = read_jsonline(stream).await?;
    debug!("BTSP JSON-line: received ChallengeResponse");

    let verify_result = call_security_provider_at(
        security_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_token": ctx.session_token,
            "client_ephemeral_pub": client_hello.client_ephemeral_pub,
            "response": challenge_response.response,
            "preferred_cipher": challenge_response.preferred_cipher,
        }),
    )
    .await?;

    let verified = verify_result
        .get("verified")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if !verified {
        let err = HandshakeError {
            error: "handshake_failed".to_owned(),
            reason: "family_verification".to_owned(),
        };
        write_jsonline(stream, &err).await?;
        warn!("BTSP JSON-line: handshake failed — family verification");
        return Err(BtspError::HandshakeFailed { reason: err.reason });
    }

    let session_id = verify_result
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&ctx.session_token)
        .to_owned();

    let negotiate_result = call_security_provider_at(
        security_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_token": ctx.session_token,
            "cipher": challenge_response.preferred_cipher,
        }),
    )
    .await?;

    let cipher = negotiate_result
        .get("cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();

    let complete = HandshakeComplete { cipher, session_id };
    write_jsonline(stream, &complete).await?;

    info!(
        cipher = %complete.cipher,
        session = %complete.session_id,
        "BTSP JSON-line: handshake complete"
    );

    Ok(complete)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn resolve_security_socket_default() {
        let path = resolve_security_socket();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("security"),
            "should resolve to security.sock: {path_str}"
        );
    }

    #[tokio::test]
    async fn handshake_rejects_bad_version() {
        let hello = ClientHello {
            version: 99,
            client_ephemeral_pub: "dGVzdA==".to_string(),
        };

        let mut buf = Vec::new();
        super::super::protocol::write_message(&mut buf, &hello)
            .await
            .expect("write hello");

        let mut cursor = std::io::Cursor::new(buf);
        let result = perform_server_handshake(&mut cursor).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("unsupported version"),
            "expected version error, got: {err}"
        );
    }

    #[test]
    fn extract_str_missing_field() {
        let value = serde_json::json!({"other": "val"});
        let err = extract_str(&value, "missing").unwrap_err();
        assert!(
            err.to_string().contains("missing"),
            "should mention field name: {err}"
        );
    }

    #[test]
    fn extract_str_success() {
        let value = serde_json::json!({"session_id": "abc123"});
        let result = extract_str(&value, "session_id").expect("should extract");
        assert_eq!(result, "abc123");
    }

    #[test]
    fn extract_str_non_string_field() {
        let value = serde_json::json!({"count": 42});
        let err = extract_str(&value, "count").unwrap_err();
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn resolve_security_socket_explicit_env() {
        temp_env::with_vars(
            [
                ("SECURITY_PROVIDER_SOCKET", Some("/custom/path.sock")),
                ("BIOMEOS_SOCKET_DIR", None::<&str>),
                ("XDG_RUNTIME_DIR", None::<&str>),
            ],
            || {
                assert_eq!(
                    resolve_security_socket(),
                    std::path::PathBuf::from("/custom/path.sock")
                );
            },
        );
    }

    #[test]
    fn resolve_security_socket_biomeos_dir() {
        temp_env::with_vars(
            [
                ("SECURITY_PROVIDER_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_DIR", Some("/run/biomeos")),
                ("XDG_RUNTIME_DIR", None::<&str>),
            ],
            || {
                assert_eq!(
                    resolve_security_socket(),
                    std::path::PathBuf::from("/run/biomeos/security.sock")
                );
            },
        );
    }

    #[test]
    fn resolve_security_socket_xdg_runtime() {
        temp_env::with_vars(
            [
                ("SECURITY_PROVIDER_SOCKET", None::<&str>),
                ("BIOMEOS_SOCKET_DIR", None::<&str>),
                ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
            ],
            || {
                assert_eq!(
                    resolve_security_socket(),
                    std::path::PathBuf::from("/run/user/1000/biomeos/security.sock")
                );
            },
        );
    }

    #[test]
    fn resolve_family_seed_from_primary() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", Some("deadbeef01234567")),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let b64 = resolve_family_seed().expect("should resolve");
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&b64)
                    .expect("valid base64");
                assert_eq!(decoded, b"deadbeef01234567");
            },
        );
    }

    #[test]
    fn resolve_family_seed_fallback_to_beardog() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", None::<&str>),
                ("BEARDOG_FAMILY_SEED", Some("fallback_seed_hex")),
            ],
            || {
                let b64 = resolve_family_seed().expect("should resolve");
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&b64)
                    .expect("valid base64");
                assert_eq!(decoded, b"fallback_seed_hex");
            },
        );
    }

    #[test]
    fn resolve_family_seed_primary_takes_precedence() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", Some("primary")),
                ("BEARDOG_FAMILY_SEED", Some("secondary")),
            ],
            || {
                let b64 = resolve_family_seed().expect("should resolve");
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&b64)
                    .expect("valid base64");
                assert_eq!(decoded, b"primary");
            },
        );
    }

    #[test]
    fn resolve_family_seed_missing_both() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", None::<&str>),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let err = resolve_family_seed().unwrap_err();
                assert!(
                    err.to_string().contains("FAMILY_SEED"),
                    "error should mention variable: {err}"
                );
            },
        );
    }

    #[test]
    fn resolve_family_seed_hex_roundtrip() {
        let hex_seed = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        temp_env::with_vars(
            [
                ("FAMILY_SEED", Some(hex_seed)),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let b64 = resolve_family_seed().expect("should resolve");
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&b64)
                    .expect("valid base64");
                assert_eq!(
                    std::str::from_utf8(&decoded).expect("utf8"),
                    hex_seed,
                    "BearDog receives the raw hex string bytes"
                );
            },
        );
    }
}
