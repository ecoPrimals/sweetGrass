// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for first-byte protocol auto-detection on UDS.
//!
//! Verifies that when BTSP is required (`FAMILY_ID` set), plain JSON-RPC
//! `health.check` probes succeed via first-byte `{` auto-detect,
//! matching the `BearDog`/`Squirrel` ecosystem pattern.
//!
//! These tests call `handle_uds_with_autodetect` directly to avoid
//! env-var pollution that would affect parallel tests.

use sweet_grass_core::agent::Did;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Plain JSON-RPC health probe succeeds via first-byte auto-detect.
#[tokio::test]
async fn test_uds_autodetect_json_health_probe() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("autodetect-health.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkAutoDetect"));

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
        .expect("timeout waiting for response")
        .unwrap()
        .expect("response");

    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

/// Multiple sequential JSON-RPC requests work over a single
/// auto-detected plain connection.
#[tokio::test]
async fn test_uds_autodetect_sequential_requests() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("autodetect-seq.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkAutoDetectSeq"));

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

    let methods = [
        (1_i64, "health.check"),
        (2_i64, "health.liveness"),
        (3_i64, "health.readiness"),
        (4_i64, "capabilities.list"),
    ];

    for (id, method) in methods {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": id
        });
        let mut req_str = serde_json::to_string(&request).unwrap();
        req_str.push('\n');
        writer.write_all(req_str.as_bytes()).await.unwrap();
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
        assert_eq!(response["id"], id);
        assert!(
            response["result"].is_object(),
            "{method} should return a result object"
        );
    }

    listener_handle.abort();
}

/// Concurrent auto-detected clients all get correct responses.
#[tokio::test]
async fn test_uds_autodetect_concurrent_clients() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("autodetect-concurrent.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkAutoDetectConc"));

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");
    let state_clone = state.clone();

    let listener_handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let s = state_clone.clone();
            tokio::spawn(async move {
                crate::uds::handle_uds_with_autodetect(stream, s).await;
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut handles = Vec::new();

    for client_id in [0_i64, 1, 2, 3] {
        let path = sock_path.clone();
        handles.push(tokio::spawn(async move {
            let stream = tokio::net::UnixStream::connect(&path)
                .await
                .expect("connect");
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();

            for req_id in [0_i64, 1, 2] {
                let id = client_id * 100 + req_id;
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "health.liveness",
                    "params": {},
                    "id": id
                });
                let mut req_str = serde_json::to_string(&request).unwrap();
                req_str.push('\n');
                writer.write_all(req_str.as_bytes()).await.unwrap();
                writer.flush().await.unwrap();

                let response_line =
                    tokio::time::timeout(std::time::Duration::from_secs(2), lines.next_line())
                        .await
                        .expect("timeout")
                        .unwrap()
                        .expect("response");

                let response: serde_json::Value =
                    serde_json::from_str(&response_line).expect("parse");
                assert_eq!(response["jsonrpc"], "2.0");
                assert_eq!(response["id"], id);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("client task should succeed");
    }

    listener_handle.abort();
}
