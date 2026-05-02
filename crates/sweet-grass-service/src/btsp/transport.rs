// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP Phase 3 transport — negotiate handler and frame loops.
//!
//! These functions are transport-agnostic (generic over `AsyncRead` /
//! `AsyncWrite`) and shared by both UDS and TCP connection handlers.

use tracing::{debug, info, warn};

/// Attempt Phase 3 `btsp.negotiate` on the first post-handshake request.
///
/// Shared by UDS and TCP handlers.
///
/// If the request is `btsp.negotiate` and the handshake key is available,
/// derives `SessionKeys`, responds, and returns `Some(keys)`.  If it's not
/// a negotiate request or if encrypted framing cannot be established, returns
/// `None` so the caller falls through to plaintext mode.
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
) -> std::result::Result<Option<super::phase3::SessionKeys>, crate::ServiceError> {
    use base64::Engine;
    use super::phase3::{
        NegotiateParams, NegotiateResult, Phase3Cipher, SessionKeys, generate_server_nonce,
        select_cipher,
    };

    let method = request
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if method != "btsp.negotiate" {
        return Ok(None);
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
        return Ok(None);
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
        return Ok(None);
    };

    if selected == Phase3Cipher::Null {
        debug!("BTSP Phase 3: client did not offer supported cipher — null fallback");
        let result = NegotiateResult {
            cipher: Phase3Cipher::Null.wire_name().to_owned(),
            server_nonce: String::new(),
        };
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": request_id});
        write_negotiate_response(writer, &resp, use_jsonline).await?;
        return Ok(None);
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

    Ok(Some(keys))
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

/// Encrypted BTSP frame loop — reads length-prefixed encrypted frames,
/// decrypts, processes JSON-RPC, encrypts response, writes.
///
/// # Errors
///
/// Returns [`crate::ServiceError`] on I/O or serialization failure.
pub async fn run_encrypted_frame_loop<R, W>(
    reader: &mut R,
    writer: &mut W,
    state: &crate::state::AppState,
    session_keys: &super::phase3::SessionKeys,
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
                warn!("BTSP encrypted frame read error: {e}");
                break;
            },
        };

        let plaintext = match session_keys.decrypt(&frame) {
            Ok(p) => p,
            Err(e) => {
                warn!("BTSP decrypt error: {e}");
                break;
            },
        };

        let request: serde_json::Value = match serde_json::from_slice(&plaintext) {
            Ok(v) => v,
            Err(e) => {
                let err_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": crate::handlers::jsonrpc::error_code::PARSE_ERROR, "message": format!("Parse error: {e}")},
                    "id": null
                });
                let payload = serde_json::to_vec(&err_response)?;
                let encrypted = session_keys
                    .encrypt(&payload)
                    .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
                super::write_frame(writer, &encrypted)
                    .await
                    .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
                continue;
            },
        };

        if let Some(response) = crate::handlers::jsonrpc::process_single(state, request).await {
            let payload = serde_json::to_vec(&response)?;
            let encrypted = session_keys
                .encrypt(&payload)
                .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
            super::write_frame(writer, &encrypted)
                .await
                .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
            writer.flush().await?;
        }
    }

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
