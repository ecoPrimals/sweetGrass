// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! UDS roundtrip, protocol, and concurrent-load tests.

use std::sync::Arc;

use crate::backend::BraidBackend;
use sweet_grass_core::SelfKnowledge;
use sweet_grass_core::agent::Did;
use sweet_grass_core::primal_names::env_vars;
use sweet_grass_store::MemoryStore;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::super::*;

#[tokio::test]
async fn test_uds_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("test-sweetgrass.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
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

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_parse_error_returns_jsonrpc_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("parse-error-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(b"{ invalid json }\n")
        .await
        .expect("write");
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(
        response["error"]["code"],
        crate::handlers::jsonrpc::error_code::PARSE_ERROR
    );

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_empty_lines_skipped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("empty-lines-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(b"\n").await.expect("write");
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 2
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_concurrent_clients() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("concurrent-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkConcurrent"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let num_clients = 8;
    let requests_per_client = 5;
    let mut handles = Vec::with_capacity(num_clients);

    for client_id in 0..num_clients {
        let path = sock_path.clone();
        handles.push(tokio::spawn(async move {
            let stream = tokio::net::UnixStream::connect(&path)
                .await
                .expect("connect");
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();

            for req_id in 0..requests_per_client {
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

                let response_line = lines.next_line().await.unwrap().expect("response");
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

#[tokio::test]
async fn test_uds_notification_no_response_then_request_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("notification-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkNotify"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
    });
    let mut note_str = serde_json::to_string(&notification).unwrap();
    note_str.push('\n');
    writer.write_all(note_str.as_bytes()).await.unwrap();

    let follow_up = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 77
    });
    let mut follow_str = serde_json::to_string(&follow_up).unwrap();
    follow_str.push('\n');
    writer.write_all(follow_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines
        .next_line()
        .await
        .unwrap()
        .expect("response after notification");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 77);
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_sequential_methods_on_one_connection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sequential-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkSequential"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let steps = [
        (1_i64, "health.check", "status"),
        (2_i64, "health.liveness", "alive"),
        (3_i64, "capabilities.list", "capabilities"),
    ];

    for (id, method, key) in steps {
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

        let response_line = lines.next_line().await.unwrap().expect("response line");
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], id);
        assert!(
            response["result"].get(key).is_some(),
            "result should contain {key}: {response}"
        );
    }

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_listener_graceful_shutdown() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("shutdown-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkShutdown"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle =
        tokio::spawn(
            async move { start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await },
        );

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    shutdown_tx.send(true).expect("signal shutdown");

    let finished = tokio::time::timeout(std::time::Duration::from_secs(3), listener_handle)
        .await
        .expect("listener should exit within timeout");

    let join_result = finished.expect("listener task join");
    assert!(
        join_result.is_ok(),
        "listener should exit cleanly: {join_result:?}"
    );
}

#[test]
fn test_start_uds_listener_resolves_path_from_self_knowledge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let biome_dir = dir.path().to_string_lossy().into_owned();
    let primal_name = "uds-from-self-knowledge";
    let sock_path = dir.path().join(format!("{primal_name}.sock"));

    let sk = SelfKnowledge {
        name: primal_name.to_string(),
        ..Default::default()
    };
    let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
    let state = crate::state::AppState::with_self_knowledge(
        store,
        Did::new("did:key:z6MkUdsSelfKnowledge"),
        sk,
        "memory",
    );

    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_SOCKET_DIR, Some(biome_dir.as_str())),
        ],
        || {
            assert_eq!(
                resolve_socket_path(Some(primal_name)),
                sock_path,
                "resolve_socket_path must match start_uds_listener bind path"
            );

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");

            rt.block_on(async {
                let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                let state_clone = state.clone();
                let listener_handle =
                    tokio::spawn(async move { start_uds_listener(state_clone, shutdown_rx).await });

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
                let response_line = lines.next_line().await.unwrap().expect("response");
                let response: serde_json::Value =
                    serde_json::from_str(&response_line).expect("parse response");
                assert_eq!(response["result"]["status"], "healthy");

                shutdown_tx.send(true).expect("shutdown");
                let _ = tokio::time::timeout(std::time::Duration::from_secs(3), listener_handle)
                    .await
                    .expect("timeout")
                    .expect("join");
            });
        },
    );
}

#[tokio::test]
async fn test_uds_stale_socket_file_removed_before_bind() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("stale-socket-test.sock");

    std::fs::write(&sock_path, b"not-a-socket").expect("stale file");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkStaleSock"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 42
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");
    assert_eq!(response["id"], 42);
    assert_eq!(response["result"]["alive"], true);

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_listener_creates_parent_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("nested/deep/run.sock");

    assert!(!sock_path.parent().expect("parent").exists());

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkMkdir"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(sock_path.exists());

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 0
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

