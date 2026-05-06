// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP integration test with a mock `BearDog` security provider.
//!
//! Spins up a fake `BearDog` UDS that responds to the 3 BTSP JSON-RPC
//! methods (`btsp.session.create`, `btsp.session.verify`, `btsp.session.negotiate`),
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
                                "session_token": "mock-token-001",
                                "server_ephemeral_pub": "c2VydmVyLXB1Yg==",
                                "challenge": "Y2hhbGxlbmdlLWRhdGE=",
                            }),
                            "btsp.session.verify" => serde_json::json!({
                                "verified": true,
                                "session_id": "mock-session-001",
                                "cipher": "AES-256-GCM",
                            }),
                            "btsp.session.negotiate" => serde_json::json!({
                                "accepted": true,
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
                                "session_token": "reject-token",
                                "server_ephemeral_pub": "c2VydmVyLXB1Yg==",
                                "challenge": "Y2hhbGxlbmdlLWRhdGE=",
                            }),
                            "btsp.session.verify" => serde_json::json!({
                                "verified": false,
                                "error": "bad HMAC",
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
    #[test]
    fn handshake_with_mock_beardog() {
        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("security.sock");

        temp_env::with_vars(
            [
                (
                    "FAMILY_SEED",
                    Some("deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567"),
                ),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                rt.block_on(async {
                    let mock_handle = start_mock_beardog(&security_sock);
                    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                    let (mut client, mut server) = tokio::io::duplex(8192);

                    let sec_path = security_sock.clone();
                    let server_handle = tokio::spawn(async move {
                        perform_server_handshake_with(&mut server, &sec_path).await
                    });

                    let client_hello = ClientHello {
                        version: 1,
                        client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
                    };
                    write_message(&mut client, &client_hello)
                        .await
                        .expect("write ClientHello");

                    let _server_hello: ServerHello =
                        read_message(&mut client).await.expect("read ServerHello");

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
                    let server_outcome = server_result.expect("server handshake");
                    assert_eq!(server_outcome.complete.cipher, "AES-256-GCM");
                    assert_eq!(server_outcome.complete.session_id, "mock-session-001");

                    mock_handle.abort();
                });
            },
        );
    }

    /// Handshake fails when `BearDog` rejects the challenge response.
    #[test]
    fn handshake_fails_when_beardog_rejects_verify() {
        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("security-reject.sock");

        temp_env::with_vars(
            [
                (
                    "FAMILY_SEED",
                    Some("deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567"),
                ),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                rt.block_on(async {
                    let mock_handle = start_rejecting_beardog(&security_sock);
                    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

                    let (mut client, mut server) = tokio::io::duplex(8192);

                    let sec_path = security_sock.clone();
                    let server_handle = tokio::spawn(async move {
                        perform_server_handshake_with(&mut server, &sec_path).await
                    });

                    let client_hello = ClientHello {
                        version: 1,
                        client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
                    };
                    write_message(&mut client, &client_hello)
                        .await
                        .expect("write ClientHello");

                    let _server_hello: ServerHello =
                        read_message(&mut client).await.expect("read ServerHello");

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
                });
            },
        );
    }

    /// Handshake fails immediately when `BearDog` socket doesn't exist.
    #[test]
    fn handshake_fails_when_beardog_unreachable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let nonexistent = dir.path().join("does-not-exist.sock");

        temp_env::with_vars(
            [
                (
                    "FAMILY_SEED",
                    Some("deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567"),
                ),
                ("BEARDOG_FAMILY_SEED", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                rt.block_on(async {
                    let (mut client, mut server) = tokio::io::duplex(8192);

                    let path = nonexistent.clone();
                    let server_handle = tokio::spawn(async move {
                        perform_server_handshake_with(&mut server, &path).await
                    });

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
                });
            },
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

    /// Exercises the production `handle_uds_connection_btsp` path: `start_uds_listener_at`
    /// with `FAMILY_ID` set (BTSP required), full client handshake against mock `BearDog`,
    /// then length-prefixed JSON-RPC (`health.check`).
    #[test]
    fn test_uds_btsp_full_handler_roundtrip() {
        use std::time::Duration;

        use sweet_grass_core::agent::Did;
        use sweet_grass_core::primal_names::env_vars;
        use sweet_grass_service::AppState;
        use sweet_grass_service::btsp::protocol::{
            ChallengeResponse, ClientHello, HandshakeComplete, ServerHello, read_message,
            write_message,
        };
        use sweet_grass_service::btsp::{read_frame, write_frame};
        use sweet_grass_service::uds::start_uds_listener_at;

        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("beardog-uds-full.sock");
        let uds_path = dir.path().join("sweetgrass-btsp-full.sock");

        let security_sock_str = security_sock.to_string_lossy().into_owned();

        temp_env::with_vars(
            [
                (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
                (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
                (env_vars::FAMILY_ID, Some("test-family")),
                (
                    env_vars::FAMILY_SEED,
                    Some("deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567"),
                ),
                ("SECURITY_PROVIDER_SOCKET", Some(security_sock_str.as_str())),
                (env_vars::BIOMEOS_INSECURE, None::<&str>),
            ],
            || {
                let state = AppState::new_memory(Did::new("did:key:z6MkBtspUdsFull"));
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");

                rt.block_on(async {
                    let mock_handle = start_mock_beardog(&security_sock);
                    tokio::time::sleep(Duration::from_millis(30)).await;

                    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                    let state_clone = state.clone();
                    let path = uds_path.clone();
                    let listener_handle = tokio::spawn(async move {
                        let _ = start_uds_listener_at(state_clone, &path, shutdown_rx).await;
                    });

                    tokio::time::sleep(Duration::from_millis(80)).await;

                    let mut stream = tokio::net::UnixStream::connect(&uds_path)
                        .await
                        .expect("connect UDS");

                    let client_hello = ClientHello {
                        version: 1,
                        client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
                    };
                    write_message(&mut stream, &client_hello)
                        .await
                        .expect("write ClientHello");

                    let _server_hello: ServerHello =
                        read_message(&mut stream).await.expect("read ServerHello");

                    let challenge_resp = ChallengeResponse {
                        response: "aG1hYy1yZXNwb25zZQ==".to_string(),
                        preferred_cipher: "AES-256-GCM".to_string(),
                    };
                    write_message(&mut stream, &challenge_resp)
                        .await
                        .expect("write ChallengeResponse");

                    let complete: HandshakeComplete = read_message(&mut stream)
                        .await
                        .expect("read HandshakeComplete");

                    assert_eq!(complete.cipher, "AES-256-GCM");
                    assert_eq!(complete.session_id, "mock-session-001");

                    let request = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "health.check",
                        "params": {},
                        "id": 99
                    });
                    let payload = serde_json::to_vec(&request).expect("serialize json-rpc");
                    write_frame(&mut stream, &payload)
                        .await
                        .expect("write_frame json-rpc");

                    let frame = read_frame(&mut stream).await.expect("read_frame response");
                    let response: serde_json::Value =
                        serde_json::from_slice(&frame).expect("parse json-rpc response");

                    assert_eq!(response["jsonrpc"], "2.0");
                    assert_eq!(response["id"], 99);
                    assert_eq!(response["result"]["status"], "healthy");

                    shutdown_tx.send(true).expect("shutdown");
                    tokio::time::timeout(Duration::from_secs(3), listener_handle)
                        .await
                        .expect("listener should exit within timeout")
                        .expect("listener task join");

                    mock_handle.abort();
                });
            },
        );
    }

    /// Exercises the production `handle_tcp_connection_btsp` path: TCP JSON-RPC listener
    /// with BTSP required, handshake, then framed `health.check`.
    #[test]
    fn test_tcp_btsp_full_handler_roundtrip() {
        use std::time::Duration;

        use sweet_grass_core::agent::Did;
        use sweet_grass_core::primal_names::env_vars;
        use sweet_grass_service::AppState;
        use sweet_grass_service::btsp::protocol::{
            ChallengeResponse, ClientHello, HandshakeComplete, ServerHello, read_message,
            write_message,
        };
        use sweet_grass_service::btsp::{read_frame, write_frame};
        use sweet_grass_service::tcp_jsonrpc::start_tcp_jsonrpc_listener;

        let dir = tempfile::tempdir().expect("tempdir");
        let security_sock = dir.path().join("beardog-tcp-full.sock");

        let security_sock_str = security_sock.to_string_lossy().into_owned();

        temp_env::with_vars(
            [
                (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
                (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
                (env_vars::FAMILY_ID, Some("test-family")),
                (
                    env_vars::FAMILY_SEED,
                    Some("deadbeef01234567deadbeef01234567deadbeef01234567deadbeef01234567"),
                ),
                ("SECURITY_PROVIDER_SOCKET", Some(security_sock_str.as_str())),
                (env_vars::BIOMEOS_INSECURE, None::<&str>),
            ],
            || {
                let state = AppState::new_memory(Did::new("did:key:z6MkBtspTcpFull"));
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");

                rt.block_on(async {
                    let mock_handle = start_mock_beardog(&security_sock);
                    tokio::time::sleep(Duration::from_millis(30)).await;

                    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
                        .await
                        .expect("bind probe");
                    let addr = probe.local_addr().expect("local_addr");
                    drop(probe);

                    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                    let state_clone = state.clone();
                    let listener_handle = tokio::spawn(async move {
                        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
                    });

                    tokio::time::sleep(Duration::from_millis(80)).await;
                    let mut stream = tokio::net::TcpStream::connect(addr)
                        .await
                        .expect("connect TCP");

                    let client_hello = ClientHello {
                        version: 1,
                        client_ephemeral_pub: "Y2xpZW50LXB1Yg==".to_string(),
                    };
                    write_message(&mut stream, &client_hello)
                        .await
                        .expect("write ClientHello");

                    let _server_hello: ServerHello =
                        read_message(&mut stream).await.expect("read ServerHello");

                    let challenge_resp = ChallengeResponse {
                        response: "aG1hYy1yZXNwb25zZQ==".to_string(),
                        preferred_cipher: "AES-256-GCM".to_string(),
                    };
                    write_message(&mut stream, &challenge_resp)
                        .await
                        .expect("write ChallengeResponse");

                    let complete: HandshakeComplete = read_message(&mut stream)
                        .await
                        .expect("read HandshakeComplete");

                    assert_eq!(complete.cipher, "AES-256-GCM");
                    assert_eq!(complete.session_id, "mock-session-001");

                    let request = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "health.check",
                        "params": {},
                        "id": 100
                    });
                    let payload = serde_json::to_vec(&request).expect("serialize json-rpc");
                    write_frame(&mut stream, &payload)
                        .await
                        .expect("write_frame json-rpc");

                    let frame = read_frame(&mut stream).await.expect("read_frame response");
                    let response: serde_json::Value =
                        serde_json::from_slice(&frame).expect("parse json-rpc response");

                    assert_eq!(response["jsonrpc"], "2.0");
                    assert_eq!(response["id"], 100);
                    assert_eq!(response["result"]["status"], "healthy");

                    shutdown_tx.send(true).expect("shutdown");
                    tokio::time::timeout(Duration::from_secs(3), listener_handle)
                        .await
                        .expect("listener should exit within timeout")
                        .expect("listener task join");

                    mock_handle.abort();
                });
            },
        );
    }
}
