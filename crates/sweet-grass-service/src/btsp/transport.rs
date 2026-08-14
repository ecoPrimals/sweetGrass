// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP Phase 3 transport — negotiate handler and frame loops.
//!
//! These functions are transport-agnostic (generic over `AsyncRead` /
//! `AsyncWrite`) and shared by both UDS and TCP connection handlers.

pub use super::encrypted_stream::run_encrypted_frame_loop;

use tracing::{debug, info, warn};

/// Result of attempting Phase 3 cipher negotiation.
pub enum NegotiateOutcome {
    /// Not a `btsp.negotiate` request — caller should dispatch the request normally.
    NotNegotiate,
    /// Negotiate completed with null cipher (or error) — response already sent,
    /// caller should skip dispatching and continue in plaintext mode.
    NullCipher,
    /// Negotiate completed with encrypted cipher — switch to encrypted framing.
    Encrypted(super::phase3::SessionKeys),
}

/// Attempt Phase 3 `btsp.negotiate` on the first post-handshake request.
///
/// Shared by UDS and TCP handlers.
///
/// Returns [`NegotiateOutcome::Encrypted`] with session keys when encrypted
/// framing is established, [`NegotiateOutcome::NullCipher`] when the request
/// was a negotiate but fell back to null cipher (response already sent), or
/// [`NegotiateOutcome::NotNegotiate`] when the request is not a negotiate
/// (caller should dispatch normally).
///
/// When `use_jsonline` is true, the response is written as newline-delimited
/// JSON; otherwise as a length-prefixed frame.
///
/// # Errors
///
/// Returns [`crate::ServiceError`] on I/O or serialization failure.
pub async fn try_phase3_negotiate<W: tokio::io::AsyncWrite + Unpin + Send>(
    request: &serde_json::Value,
    handshake_key: Option<&[u8; 32]>,
    writer: &mut W,
    use_jsonline: bool,
) -> std::result::Result<NegotiateOutcome, crate::ServiceError> {
    use super::phase3::{
        NegotiateParams, NegotiateResult, Phase3Cipher, SessionKeys, generate_server_nonce,
        select_cipher,
    };
    use base64::Engine;

    let method = request
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if method != "btsp.negotiate" {
        return Ok(NegotiateOutcome::NotNegotiate);
    }

    let request_id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let Some(Ok(params)) = request
        .get("params")
        .cloned()
        .map(serde_json::from_value::<NegotiateParams>)
    else {
        let err = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": crate::handlers::jsonrpc::error_code::INVALID_PARAMS, "message": "Invalid btsp.negotiate params"},
            "id": request_id
        });
        write_negotiate_response(writer, &err, use_jsonline).await?;
        return Ok(NegotiateOutcome::NullCipher);
    };

    let selected = select_cipher(&params.ciphers);

    let Some(hk) = handshake_key else {
        debug!("BTSP Phase 3: no handshake key — responding with null cipher");
        let result = NegotiateResult {
            cipher: Phase3Cipher::Null.wire_name().to_owned(),
            server_nonce: String::new(),
        };
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": request_id});
        write_negotiate_response(writer, &resp, use_jsonline).await?;
        return Ok(NegotiateOutcome::NullCipher);
    };

    if selected == Phase3Cipher::Null {
        debug!("BTSP Phase 3: client did not offer supported cipher — null fallback");
        let result = NegotiateResult {
            cipher: Phase3Cipher::Null.wire_name().to_owned(),
            server_nonce: String::new(),
        };
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": request_id});
        write_negotiate_response(writer, &resp, use_jsonline).await?;
        return Ok(NegotiateOutcome::NullCipher);
    }

    let server_nonce = generate_server_nonce()
        .map_err(|e| crate::ServiceError::Internal(format!("nonce gen: {e}")))?;

    let client_nonce = base64::engine::general_purpose::STANDARD
        .decode(&params.client_nonce)
        .map_err(|e| crate::ServiceError::Internal(format!("client_nonce decode: {e}")))?;

    let keys = SessionKeys::derive(hk, &client_nonce, &server_nonce, true)
        .map_err(|e| crate::ServiceError::Internal(format!("HKDF: {e}")))?;

    let server_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(server_nonce);

    let result = NegotiateResult {
        cipher: selected.wire_name().to_owned(),
        server_nonce: server_nonce_b64,
    };
    let resp = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": request_id});
    write_negotiate_response(writer, &resp, use_jsonline).await?;

    info!(
        cipher = selected.wire_name(),
        "BTSP Phase 3: encrypted channel established"
    );

    Ok(NegotiateOutcome::Encrypted(keys))
}

/// Write a Phase 3 negotiate JSON-RPC response in the appropriate framing.
///
/// # Errors
///
/// Returns [`crate::ServiceError`] on I/O or serialization failure.
pub async fn write_negotiate_response<W: tokio::io::AsyncWrite + Unpin + Send>(
    writer: &mut W,
    response: &serde_json::Value,
    use_jsonline: bool,
) -> std::result::Result<(), crate::ServiceError> {
    use tokio::io::AsyncWriteExt;

    if use_jsonline {
        let mut line = serde_json::to_string(response)?;
        line.push('\n');
        writer.write_all(line.as_bytes()).await?;
    } else {
        let payload = serde_json::to_vec(response)?;
        super::write_frame(writer, &payload)
            .await
            .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
    }
    writer.flush().await?;
    Ok(())
}

/// Plaintext BTSP frame loop — reads length-prefixed plaintext frames,
/// processes JSON-RPC, writes responses.
///
/// # Errors
///
/// Returns [`crate::ServiceError`] on I/O or serialization failure.
pub async fn run_plaintext_frame_loop<R, W>(
    reader: &mut R,
    writer: &mut W,
    state: &crate::state::AppState,
) -> std::result::Result<(), crate::ServiceError>
where
    R: tokio::io::AsyncRead + Unpin + Send,
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    use tokio::io::AsyncWriteExt;

    loop {
        let frame = match super::read_frame(reader).await {
            Ok(f) => f,
            Err(super::BtspError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            },
            Err(e) => {
                warn!("BTSP frame read error: {e}");
                break;
            },
        };

        let request: serde_json::Value = match serde_json::from_slice(&frame) {
            Ok(v) => v,
            Err(e) => {
                let err_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": crate::handlers::jsonrpc::error_code::PARSE_ERROR, "message": format!("Parse error: {e}")},
                    "id": null
                });
                let payload = serde_json::to_vec(&err_response)?;
                super::write_frame(writer, &payload)
                    .await
                    .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
                continue;
            },
        };

        if let Some(response) = crate::handlers::jsonrpc::process_single(state, request).await {
            let payload = serde_json::to_vec(&response)?;
            super::write_frame(writer, &payload)
                .await
                .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
            writer.flush().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;

    /// Smoke test: `process_single` returns for `health.liveness`.
    #[tokio::test]
    async fn process_single_smoke() {
        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkSmoke"));
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1
        });
        let resp = crate::handlers::jsonrpc::process_single(&state, request)
            .await
            .expect("should return a response");
        assert_eq!(resp.id, serde_json::json!(1));
    }

    /// `try_phase3_negotiate` returns `Some(keys)` for a valid negotiate
    /// request with `chacha20-poly1305` and a present handshake key.
    #[tokio::test]
    async fn negotiate_returns_keys_for_valid_request() {
        use base64::Engine;

        let handshake_key = [0x42u8; 32];
        let client_nonce = [3u8; 32];
        let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": "test-session",
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": client_nonce_b64,
            },
            "id": 1
        });

        let (mut client_read, mut server_write) = tokio::io::duplex(4096);

        let result = try_phase3_negotiate(&request, Some(&handshake_key), &mut server_write, true)
            .await
            .expect("negotiate should not error");

        assert!(
            matches!(result, NegotiateOutcome::Encrypted(_)),
            "should return session keys"
        );

        let mut resp_line = String::new();
        tokio::io::AsyncBufReadExt::read_line(
            &mut tokio::io::BufReader::new(&mut client_read),
            &mut resp_line,
        )
        .await
        .unwrap();

        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["result"]["cipher"], "chacha20-poly1305");
        assert!(!resp["result"]["server_nonce"].as_str().unwrap().is_empty());
    }

    /// Non-negotiate request returns `NotNegotiate` (pass-through to caller).
    #[tokio::test]
    async fn negotiate_returns_not_negotiate_for_non_negotiate_method() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 1
        });

        let (_, mut writer) = tokio::io::duplex(4096);

        let result = try_phase3_negotiate(&request, Some(&[0u8; 32]), &mut writer, false)
            .await
            .expect("should not error");

        assert!(matches!(result, NegotiateOutcome::NotNegotiate));
    }

    /// Missing handshake key returns null cipher and `NullCipher`.
    #[tokio::test]
    async fn negotiate_null_cipher_without_handshake_key() {
        use base64::Engine;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": "test",
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": base64::engine::general_purpose::STANDARD.encode([0u8; 32]),
            },
            "id": 1
        });

        let (mut client_read, mut server_write) = tokio::io::duplex(4096);

        let result = try_phase3_negotiate(&request, None, &mut server_write, true)
            .await
            .expect("should not error");

        assert!(matches!(result, NegotiateOutcome::NullCipher));

        let mut resp_line = String::new();
        tokio::io::AsyncBufReadExt::read_line(
            &mut tokio::io::BufReader::new(&mut client_read),
            &mut resp_line,
        )
        .await
        .unwrap();

        let resp: serde_json::Value = serde_json::from_str(resp_line.trim()).unwrap();
        assert_eq!(resp["result"]["cipher"], "null");
    }

    /// Null-cipher negotiate writes exactly one response — callers must
    /// not dispatch a second `METHOD_NOT_FOUND` for the same request.
    #[tokio::test]
    async fn null_cipher_negotiate_sends_single_response() {
        use base64::Engine;
        use tokio::io::AsyncBufReadExt;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": "test",
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": base64::engine::general_purpose::STANDARD.encode([7u8; 32]),
            },
            "id": 99
        });

        let (mut client_read, mut server_write) = tokio::io::duplex(4096);

        let outcome = try_phase3_negotiate(&request, None, &mut server_write, true)
            .await
            .expect("should not error");

        assert!(matches!(outcome, NegotiateOutcome::NullCipher));

        drop(server_write);

        let mut all_lines = Vec::new();
        let mut buf_reader = tokio::io::BufReader::new(&mut client_read);
        let mut line = String::new();
        while buf_reader.read_line(&mut line).await.unwrap() > 0 {
            all_lines.push(line.clone());
            line.clear();
        }

        assert_eq!(
            all_lines.len(),
            1,
            "null-cipher negotiate must produce exactly one response, got {all_lines:?}"
        );
        let resp: serde_json::Value = serde_json::from_str(all_lines[0].trim()).unwrap();
        assert_eq!(resp["result"]["cipher"], "null");
        assert_eq!(resp["id"], 99);
        assert!(resp.get("error").is_none(), "should not contain error");
    }
}
