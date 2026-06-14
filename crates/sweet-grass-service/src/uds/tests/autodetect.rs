// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for riboCipher signal detection on UDS.
//!
//! Wave 113: unsignalled connections are **rejected** with `-32002`.
//! All clients must send `[0xEC, protocol_type]` prefix.
//!
//! These tests call `handle_uds_with_autodetect` directly to avoid
//! env-var pollution that would affect parallel tests.

use sweet_grass_core::agent::Did;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// ==================== riboCipher signal tests ====================

/// riboCipher clear signal (0xEC 0x01) routes to NDJSON JSON-RPC handler.
#[tokio::test]
async fn test_uds_ribocipher_clear_jsonrpc() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("ribocipher-jsonrpc.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkRiboCipher"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            crate::uds::handle_uds_with_autodetect(stream, state_clone).await;
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let mut payload = vec![0xEC, 0x01];
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    payload.extend_from_slice(req_str.as_bytes());

    writer.write_all(&payload).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(std::time::Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout waiting for riboCipher response")
        .unwrap()
        .expect("response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

/// riboCipher clear signal probe (0xEC 0x00) returns lightweight health.
#[tokio::test]
async fn test_uds_ribocipher_clear_probe() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("ribocipher-probe.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkRiboProbe"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            crate::uds::handle_uds_with_autodetect(stream, state_clone).await;
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(&[0xEC, 0x00]).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(std::time::Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout waiting for probe response")
        .unwrap()
        .expect("response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

/// Sequential JSON-RPC requests over riboCipher-signalled connection.
#[tokio::test]
async fn test_uds_ribocipher_sequential_requests() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("ribocipher-seq.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkRiboSeq"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            crate::uds::handle_uds_with_autodetect(stream, state_clone).await;
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let mut signal = vec![0xEC, 0x01];
    let methods = [
        (1_i64, "health.check"),
        (2_i64, "health.liveness"),
        (3_i64, "health.readiness"),
        (4_i64, "capabilities.list"),
    ];

    for (i, (id, method)) in methods.iter().enumerate() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": id
        });
        let mut req_str = serde_json::to_string(&request).unwrap();
        req_str.push('\n');

        if i == 0 {
            signal.extend_from_slice(req_str.as_bytes());
            writer.write_all(&signal).await.unwrap();
        } else {
            writer.write_all(req_str.as_bytes()).await.unwrap();
        }
        writer.flush().await.unwrap();

        let response_line =
            tokio::time::timeout(std::time::Duration::from_secs(2), lines.next_line())
                .await
                .expect("timeout waiting for response")
                .unwrap()
                .expect("response");

        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], *id);
        assert!(
            response["result"].is_object(),
            "{method} should return a result object"
        );
    }

    listener_handle.abort();
}

/// `braid.create` succeeds via riboCipher-signalled connection.
#[tokio::test]
async fn test_uds_ribocipher_braid_create() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("ribocipher-braid-create.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkRiboBraid"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            crate::uds::handle_uds_with_autodetect(stream, state_clone).await;
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let mut payload = vec![0xEC, 0x01];
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "braid.create",
        "params": {
            "data_hash": "sha256:ribocipher0102030405060708091011121314151617181920212223242526",
            "mime_type": "application/json",
            "size": 256
        },
        "id": 1
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    payload.extend_from_slice(req_str.as_bytes());

    writer.write_all(&payload).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(std::time::Duration::from_secs(5), lines.next_line())
        .await
        .expect("braid.create via riboCipher should respond within 5s")
        .unwrap()
        .expect("response");

    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["result"].is_object(),
        "braid.create via riboCipher should return result: {response}"
    );
    assert!(
        response["result"]["@id"].is_string(),
        "braid.create result should contain @id (JSON-LD): {response}"
    );

    listener_handle.abort();
}

// ==================== Wave 113: Unsignalled rejection tests ====================

/// Unsignalled JSON-RPC is rejected with -32002 (Wave 113).
#[tokio::test]
async fn test_uds_unsignalled_json_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("unsignalled-reject.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkUnsignalled"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            crate::uds::handle_uds_with_autodetect(stream, state_clone).await;
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(std::time::Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout waiting for rejection response")
        .unwrap()
        .expect("response");

    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(
        response["error"]["code"], -32002,
        "unsignalled connection should be rejected with -32002"
    );
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("riboCipher"),
        "error message should mention riboCipher"
    );

    listener_handle.abort();
}
