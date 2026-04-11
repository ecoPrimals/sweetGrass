// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP integration test with a mock `BearDog` security provider.
//!
//! Spins up a fake `BearDog` UDS that responds to the 3 BTSP JSON-RPC
//! methods (`btsp.session.create`, `btsp.session.verify`, `btsp.negotiate`),
//! then exercises `perform_server_handshake_with` end-to-end using DI
//! (no `set_var` needed — safe under `forbid(unsafe_code)`).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

#[cfg(unix)]
mod btsp_tests {
    use sweet_grass_service::btsp::protocol::{
        ChallengeResponse, ClientHello, HandshakeComplete, ServerHello, read_message, write_message,
    };
    use sweet_grass_service::btsp::server::perform_server_handshake_with;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    /// Spin up a mock `BearDog` UDS that speaks newline-delimited JSON-RPC.
    fn start_mock_beardog(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
        let path = socket_path.to_path_buf();
        tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind mock beardog");
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let request: serde_json::Value =
                            serde_json::from_str(&line).unwrap_or_default();
                        let method = request
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        let id = request
                            .get("id")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!(1));

                        let result = match method {
                            "btsp.session.create" => serde_json::json!({
                                "session_id": "mock-session-001",
                                "server_ephemeral_pub": "c2VydmVyLXB1Yg==",
                                "challenge": "Y2hhbGxlbmdlLWRhdGE=",
                            }),
                            "btsp.session.verify" => serde_json::json!({
                                "verified": true,
                            }),
                            "btsp.negotiate" => serde_json::json!({
                                "cipher": "AES-256-GCM",
                            }),
                            _ => serde_json::json!(null),
                        };

                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": result,
                            "id": id,
                        });
                        let mut resp_str = serde_json::to_string(&response).unwrap();
                        resp_str.push('\n');
                        let _ = writer.write_all(resp_str.as_bytes()).await;
                        let _ = writer.flush().await;
                    }
                });
            }
        })
    }

    /// Spin up a mock `BearDog` that returns `"verified": false`.
    fn start_rejecting_beardog(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
        let path = socket_path.to_path_buf();
        tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind mock");
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (reader, mut writer) = stream.into_split();
                    let mut lines = BufReader::new(reader).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let request: serde_json::Value =
                            serde_json::from_str(&line).unwrap_or_default();
                        let method = request
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("");
                        let id = request
                            .get("id")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!(1));

                        let result = match method {
                            "btsp.session.create" => serde_json::json!({
                                "session_id": "reject-session",
                                "server_ephemeral_pub": "c2VydmVyLXB1Yg==",
                                "challenge": "Y2hhbGxlbmdlLWRhdGE=",
                            }),
                            "btsp.session.verify" => serde_json::json!({
                                "verified": false,
                            }),
                            _ => serde_json::json!(null),
                        };

                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "result": result,
                            "id": id,
                        });
                        let mut resp_str = serde_json::to_string(&response).unwrap();
                        resp_str.push('\n');
                        let _ = writer.write_all(resp_str.as_bytes()).await;
                        let _ = writer.flush().await;
                    }
                });
            }
        })
    }

    /// Full end-to-end BTSP handshake over a `DuplexStream`.
    ///
    /// Server side runs `perform_server_handshake_with` on one end;
    /// client side drives the protocol (send `ClientHello`, read
    /// `ServerHello`, send `ChallengeResponse`, read `HandshakeComplete`).
    #[tokio::test]
    async fn handshake_with_mock_beardog() {
        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("security.sock");

        let mock_handle = start_mock_beardog(&security_sock);
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let (mut client, mut server) = tokio::io::duplex(8192);

        let sec_path = security_sock.clone();
        let server_handle =
            tokio::spawn(
                async move { perform_server_handshake_with(&mut server, &sec_path).await },
            );

        let client_hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
        };
        write_message(&mut client, &client_hello)
            .await
            .expect("write ClientHello");

        let _server_hello: ServerHello = read_message(&mut client).await.expect("read ServerHello");

        let challenge_resp = ChallengeResponse {
            response: "aG1hYy1yZXNwb25zZQ==".to_string(),
            preferred_cipher: "AES-256-GCM".to_string(),
        };
        write_message(&mut client, &challenge_resp)
            .await
            .expect("write ChallengeResponse");

        let complete: HandshakeComplete = read_message(&mut client)
            .await
            .expect("read HandshakeComplete");

        assert_eq!(complete.cipher, "AES-256-GCM");
        assert_eq!(complete.session_id, "mock-session-001");

        let server_result = server_handle.await.expect("server task");
        let server_complete = server_result.expect("server handshake");
        assert_eq!(server_complete.cipher, "AES-256-GCM");
        assert_eq!(server_complete.session_id, "mock-session-001");

        mock_handle.abort();
    }

    /// Handshake fails when `BearDog` rejects the challenge response.
    #[tokio::test]
    async fn handshake_fails_when_beardog_rejects_verify() {
        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("security-reject.sock");

        let mock_handle = start_rejecting_beardog(&security_sock);
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let (mut client, mut server) = tokio::io::duplex(8192);

        let sec_path = security_sock.clone();
        let server_handle =
            tokio::spawn(
                async move { perform_server_handshake_with(&mut server, &sec_path).await },
            );

        let client_hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
        };
        write_message(&mut client, &client_hello)
            .await
            .expect("write ClientHello");

        let _server_hello: ServerHello = read_message(&mut client).await.expect("read ServerHello");

        let challenge_resp = ChallengeResponse {
            response: "YmFkLXJlc3BvbnNl".to_string(),
            preferred_cipher: "AES-256-GCM".to_string(),
        };
        write_message(&mut client, &challenge_resp)
            .await
            .expect("write ChallengeResponse");

        let server_result = server_handle.await.expect("server task");
        assert!(
            server_result.is_err(),
            "should fail when verify returns false"
        );
        let err = server_result.unwrap_err().to_string();
        assert!(
            err.contains("family_verification"),
            "expected verification error, got: {err}"
        );

        mock_handle.abort();
    }

    /// Handshake fails immediately when `BearDog` socket doesn't exist.
    #[tokio::test]
    async fn handshake_fails_when_beardog_unreachable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nonexistent = dir.path().join("does-not-exist.sock");

        let (mut client, mut server) = tokio::io::duplex(8192);

        let path = nonexistent.clone();
        let server_handle =
            tokio::spawn(async move { perform_server_handshake_with(&mut server, &path).await });

        let client_hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
        };
        write_message(&mut client, &client_hello)
            .await
            .expect("write ClientHello");

        let server_result = server_handle.await.expect("server task");
        assert!(
            server_result.is_err(),
            "should fail when BearDog is unreachable"
        );
        let err = server_result.unwrap_err().to_string();
        assert!(
            err.contains("unreachable") || err.contains("Unavailable"),
            "expected unreachable error, got: {err}"
        );
    }

    /// Verify that `HandshakeComplete` fields round-trip through serde.
    #[test]
    fn handshake_complete_serde_roundtrip() {
        let complete = HandshakeComplete {
            cipher: "AES-256-GCM".to_string(),
            session_id: "test-session".to_string(),
        };
        let json = serde_json::to_string(&complete).expect("serialize");
        let back: HandshakeComplete = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.cipher, "AES-256-GCM");
        assert_eq!(back.session_id, "test-session");
    }
}
