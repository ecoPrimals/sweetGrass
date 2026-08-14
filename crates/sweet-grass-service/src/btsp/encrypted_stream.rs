// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP Phase 3 encrypted frame loop — AEAD decrypt/process/encrypt cycle.
//!
//! Shared by UDS and TCP connection handlers after successful cipher negotiation.

use tracing::warn;

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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use crate::btsp::phase3::SessionKeys;
    use crate::btsp::transport::NegotiateOutcome;
    use crate::btsp::transport::try_phase3_negotiate;
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
            write_frame(&mut sw, &encrypted)
                .await
                .expect("server write");
        });

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 42
        });
        let plaintext = serde_json::to_vec(&request).unwrap();
        let encrypted = client_keys.encrypt(&plaintext).unwrap();
        write_frame(&mut cw, &encrypted)
            .await
            .expect("client write");

        let resp_frame = read_frame(&mut cr).await.expect("client read");
        let decrypted = client_keys.decrypt(&resp_frame).expect("client decrypt");
        let response: serde_json::Value =
            serde_json::from_slice(&decrypted).expect("parse response");

        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["alive"], true);

        server_handle.await.unwrap();
    }

    /// Proves encrypt → `write_frame` → `read_frame` → decrypt roundtrip
    /// works for the BTSP Phase 3 wire format.
    #[tokio::test]
    async fn encrypted_frame_wire_roundtrip() {
        let (server_keys, client_keys) = test_keys();

        let (mut left, mut right): (DuplexStream, DuplexStream) = tokio::io::duplex(8192);

        let plaintext = b"hello encrypted btsp frame";
        let encrypted = client_keys.encrypt(plaintext).unwrap();
        write_frame(&mut left, &encrypted)
            .await
            .expect("write frame");

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
        let decrypted = client_keys
            .decrypt(&response_frame)
            .expect("decrypt response");
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

            let outcome = try_phase3_negotiate(&neg_req, Some(&handshake_key), &mut sw, false)
                .await
                .expect("negotiate");
            let NegotiateOutcome::Encrypted(session_keys) = outcome else {
                panic!("expected Encrypted, got NotNegotiate or NullCipher");
            };

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

        let client_keys = SessionKeys::derive(&handshake_key, &client_nonce, &server_nonce, false)
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
        let decrypted = client_keys.decrypt(&resp_frame).expect("decrypt response");
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
