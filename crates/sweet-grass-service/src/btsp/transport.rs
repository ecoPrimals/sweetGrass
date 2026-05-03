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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use crate::btsp::phase3::SessionKeys;
    use crate::btsp::{read_frame, write_frame};
    use sweet_grass_core::agent::Did;
    use tokio::io::DuplexStream;

    fn test_keys() -> (SessionKeys, SessionKeys) {
        let hk = [0xABu8; 32];
        let cn = [1u8; 32];
        let sn = [2u8; 32];
        let server = SessionKeys::derive(&hk, &cn, &sn, true).unwrap();
        let client = SessionKeys::derive(&hk, &cn, &sn, false).unwrap();
        (server, client)
    }

    /// Single encrypted request-response over a `DuplexStream`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn encrypted_frame_manual_step() {
        let (server_keys, client_keys) = test_keys();
        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkManual"));

        let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);
        let (mut sr, mut sw) = tokio::io::split(server_stream);
        let (mut cr, mut cw) = tokio::io::split(client_stream);

        let server_handle = tokio::spawn(async move {
            let frame = read_frame(&mut sr).await.expect("server read frame");
            let plaintext = server_keys.decrypt(&frame).expect("server decrypt");
            let request: serde_json::Value =
                serde_json::from_slice(&plaintext).expect("server parse");

            let resp = crate::handlers::jsonrpc::process_single(&state, request)
                .await
                .expect("should produce response");

            let payload = serde_json::to_vec(&resp).unwrap();
            let encrypted = server_keys.encrypt(&payload).unwrap();
            write_frame(&mut sw, &encrypted).await.expect("server write");
        });

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 42
        });
        let plaintext = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&plaintext).unwrap();
        write_frame(&mut cw, &encrypted).await.expect("client write");

        let resp_frame = read_frame(&mut cr).await.expect("client read");
        let decrypted = client_keys.decrypt(&resp_frame).expect("client decrypt");
        let response: serde_json::Value =
            serde_json::from_slice(&decrypted).expect("parse response");

        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["alive"], true);

        server_handle.await.unwrap();
    }

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

    /// Proves encrypt → write_frame → read_frame → decrypt roundtrip
    /// works for the BTSP Phase 3 wire format.
    #[tokio::test]
    async fn encrypted_frame_wire_roundtrip() {
        let (server_keys, client_keys) = test_keys();

        let (mut left, mut right): (DuplexStream, DuplexStream) = tokio::io::duplex(8192);

        let plaintext = b"hello encrypted btsp frame";
        let encrypted = client_keys.encrypt(plaintext).unwrap();
        write_frame(&mut left, &encrypted).await.expect("write frame");

        let frame = read_frame(&mut right).await.expect("read frame");
        let decrypted = server_keys.decrypt(&frame).expect("decrypt frame");
        assert_eq!(&decrypted, plaintext);
    }

    /// Proves the encrypted frame loop correctly decrypts client frames
    /// and returns encrypted responses — the critical transport switch
    /// that primalSpring's Phase 3 interop depends on.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn encrypted_frame_loop_roundtrip() {
        let (server_keys, client_keys) = test_keys();
        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkPhase3Test"));

        let (client_stream, server_stream): (DuplexStream, DuplexStream) =
            tokio::io::duplex(64 * 1024);

        let server_handle = tokio::spawn(async move {
            let (mut sr, mut sw) = tokio::io::split(server_stream);
            run_encrypted_frame_loop(&mut sr, &mut sw, &state, &server_keys)
                .await
                .expect("encrypted loop");
        });

        let (mut client_reader, mut client_writer) = tokio::io::split(client_stream);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 42
        });
        let plaintext = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&plaintext).unwrap();
        write_frame(&mut client_writer, &encrypted)
            .await
            .expect("write encrypted frame");

        let response_frame = read_frame(&mut client_reader).await.expect("read response");
        let decrypted = client_keys.decrypt(&response_frame).expect("decrypt response");
        let response: serde_json::Value =
            serde_json::from_slice(&decrypted).expect("parse response JSON");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["alive"], true);

        drop(client_writer);
        drop(client_reader);
        server_handle.await.unwrap();
    }

    /// Multiple sequential encrypted requests on the same connection.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn encrypted_frame_loop_sequential_requests() {
        let (server_keys, client_keys) = test_keys();
        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkPhase3Seq"));

        let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);

        let server_handle = tokio::spawn(async move {
            let (mut sr, mut sw) = tokio::io::split(server_stream);
            run_encrypted_frame_loop(&mut sr, &mut sw, &state, &server_keys)
                .await
                .expect("encrypted loop");
        });

        let (mut cr, mut cw) = tokio::io::split(client_stream);

        for req_id in 1..=5 {
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "health.liveness",
                "params": {},
                "id": req_id
            });
            let plaintext = serde_json::to_vec(&request).unwrap();
            let encrypted = client_keys.encrypt(&plaintext).unwrap();
            write_frame(&mut cw, &encrypted).await.unwrap();

            let resp_frame = read_frame(&mut cr).await.unwrap();
            let decrypted = client_keys.decrypt(&resp_frame).unwrap();
            let resp: serde_json::Value = serde_json::from_slice(&decrypted).unwrap();

            assert_eq!(resp["id"], req_id);
            assert_eq!(resp["result"]["alive"], true);
        }

        drop(cw);
        drop(cr);
        server_handle.await.unwrap();
    }

    /// Tampered ciphertext causes the server to break the connection.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn encrypted_frame_loop_rejects_tampered_frame() {
        let (server_keys, client_keys) = test_keys();
        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkPhase3Tamper"));

        let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);

        let server_handle = tokio::spawn(async move {
            let (mut sr, mut sw) = tokio::io::split(server_stream);
            run_encrypted_frame_loop(&mut sr, &mut sw, &state, &server_keys)
                .await
                .expect("encrypted loop");
        });

        let request = serde_json::json!({
            "jsonrpc": "2.0", "method": "health.liveness", "params": {}, "id": 1
        });
        let plaintext = serde_json::to_vec(&request).unwrap();
        let mut encrypted = client_keys.encrypt(&plaintext).unwrap();
        if let Some(byte) = encrypted.last_mut() {
            *byte ^= 0xFF;
        }

        {
            let (_, mut cw) = tokio::io::split(client_stream);
            write_frame(&mut cw, &encrypted).await.unwrap();
        }

        server_handle.await.unwrap();
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

        let result = try_phase3_negotiate(
            &request,
            Some(&handshake_key),
            &mut server_write,
            true,
        )
        .await
        .expect("negotiate should not error");

        assert!(result.is_some(), "should return session keys");

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

    /// Non-negotiate request returns `None` (pass-through to caller).
    #[tokio::test]
    async fn negotiate_returns_none_for_non_negotiate_method() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 1
        });

        let (_, mut writer) = tokio::io::duplex(4096);

        let result = try_phase3_negotiate(
            &request,
            Some(&[0u8; 32]),
            &mut writer,
            false,
        )
        .await
        .expect("should not error");

        assert!(result.is_none());
    }

    /// Missing handshake key returns null cipher and `None`.
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

        let result = try_phase3_negotiate(
            &request,
            None,
            &mut server_write,
            true,
        )
        .await
        .expect("should not error");

        assert!(result.is_none());

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

    /// Full negotiate → encrypted roundtrip simulating the exact
    /// primalSpring client wire protocol (length-prefixed negotiate,
    /// then length-prefixed encrypted frames).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn full_negotiate_then_encrypted_roundtrip() {
        use base64::Engine;

        let handshake_key = [0x99u8; 32];
        let client_nonce = [5u8; 32];
        let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);

        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkFullNeg"));

        let (client_stream, server_stream) = tokio::io::duplex(64 * 1024);

        let neg_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "btsp.negotiate",
            "params": {
                "session_id": "full-test",
                "ciphers": ["chacha20-poly1305"],
                "client_nonce": client_nonce_b64,
            },
            "id": 1
        });

        let server_handle = tokio::spawn(async move {
            let (mut sr, mut sw) = tokio::io::split(server_stream);
            let neg_frame = read_frame(&mut sr).await.expect("read negotiate");
            let neg_req: serde_json::Value =
                serde_json::from_slice(&neg_frame).expect("parse negotiate");

            let session_keys = try_phase3_negotiate(
                &neg_req,
                Some(&handshake_key),
                &mut sw,
                false,
            )
            .await
            .expect("negotiate")
            .expect("should produce keys");

            run_encrypted_frame_loop(&mut sr, &mut sw, &state, &session_keys)
                .await
                .expect("encrypted loop");
        });

        let (mut cr, mut cw) = tokio::io::split(client_stream);

        let neg_bytes = serde_json::to_vec(&neg_request).unwrap();
        write_frame(&mut cw, &neg_bytes).await.unwrap();

        let neg_resp_frame = read_frame(&mut cr).await.expect("read negotiate response");
        let neg_resp: serde_json::Value =
            serde_json::from_slice(&neg_resp_frame).expect("parse negotiate response");

        assert_eq!(neg_resp["result"]["cipher"], "chacha20-poly1305");
        let server_nonce_b64 = neg_resp["result"]["server_nonce"].as_str().unwrap();
        let server_nonce = base64::engine::general_purpose::STANDARD
            .decode(server_nonce_b64)
            .unwrap();

        let client_keys =
            SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false)
                .expect("client key derivation");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 100
        });
        let plaintext = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&plaintext).unwrap();
        write_frame(&mut cw, &encrypted).await.unwrap();

        let resp_frame = read_frame(&mut cr).await.expect("read encrypted response");
        let decrypted = client_keys
            .decrypt(&resp_frame)
            .expect("decrypt response");
        let response: serde_json::Value =
            serde_json::from_slice(&decrypted).expect("parse decrypted response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 100);
        assert_eq!(response["result"]["alive"], true);

        drop(cw);
        drop(cr);
        server_handle.await.unwrap();
    }
}
