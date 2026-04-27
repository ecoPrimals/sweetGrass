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

// ==================== PG-52 Domain Method Verification ====================

#[tokio::test]
async fn test_uds_braid_create_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("braid-create-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkBraidCreate"));

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
        "method": "braid.create",
        "params": {
            "data_hash": "sha256:deadbeef01020304050607080910111213141516171819202122232425262728",
            "mime_type": "application/json",
            "size": 1024
        },
        "id": 1
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(std::time::Duration::from_secs(5), lines.next_line())
        .await
        .expect("braid.create should respond within 5s")
        .unwrap()
        .expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response["result"].is_object(),
        "braid.create should return a result object, got: {response}"
    );
    assert!(
        response["result"]["@id"].is_string(),
        "braid result should contain @id (JSON-LD): {response}"
    );
    assert_eq!(response["result"]["mime_type"], "application/json");
    assert_eq!(response["result"]["size"], 1024);

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_braid_query_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("braid-query-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkBraidQuery"));

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

    let create_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "braid.create",
        "params": {
            "data_hash": "sha256:querytest010203040506070809101112131415161718192021222324252627",
            "mime_type": "text/plain",
            "size": 42
        },
        "id": 1
    });
    let mut req_str = serde_json::to_string(&create_req).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let create_resp = lines.next_line().await.unwrap().expect("create response");
    let create_val: serde_json::Value = serde_json::from_str(&create_resp).unwrap();
    assert!(
        create_val["result"].is_object(),
        "braid.create should succeed for query setup"
    );

    let query_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "braid.query",
        "params": {
            "filter": {}
        },
        "id": 2
    });
    let mut req_str = serde_json::to_string(&query_req).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let query_resp_line = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        lines.next_line(),
    )
    .await
    .expect("braid.query should respond within 5s")
    .unwrap()
    .expect("query response");
    let query_resp: serde_json::Value = serde_json::from_str(&query_resp_line).unwrap();

    assert_eq!(query_resp["jsonrpc"], "2.0");
    assert_eq!(query_resp["id"], 2);
    assert!(
        query_resp["result"].is_object(),
        "braid.query should return a result object, got: {query_resp}"
    );
    let braids = &query_resp["result"]["braids"];
    assert!(
        braids.is_array(),
        "braid.query result should contain a braids array"
    );
    assert!(
        !braids.as_array().unwrap().is_empty(),
        "query after create should return at least one braid"
    );

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_provenance_graph_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("provenance-graph-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkProvGraph"));

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

    let create_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "braid.create",
        "params": {
            "data_hash": "sha256:provgraph0102030405060708091011121314151617181920212223242526272829",
            "mime_type": "application/octet-stream",
            "size": 256
        },
        "id": 1
    });
    let mut req_str = serde_json::to_string(&create_req).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let create_resp = lines.next_line().await.unwrap().expect("create response");
    let create_val: serde_json::Value = serde_json::from_str(&create_resp).unwrap();
    let braid_id = create_val["result"]["@id"].as_str().expect("braid @id");

    let graph_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "provenance.graph",
        "params": {
            "entity": { "braid_id": braid_id }
        },
        "id": 2
    });
    let mut req_str = serde_json::to_string(&graph_req).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let graph_resp_line = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        lines.next_line(),
    )
    .await
    .expect("provenance.graph should respond within 5s")
    .unwrap()
    .expect("graph response");
    let graph_resp: serde_json::Value = serde_json::from_str(&graph_resp_line).unwrap();

    assert_eq!(graph_resp["jsonrpc"], "2.0");
    assert_eq!(graph_resp["id"], 2);
    assert!(
        graph_resp["result"].is_object(),
        "provenance.graph should return a result object, got: {graph_resp}"
    );

    listener_handle.abort();
}

/// Composition-like single-shot: create + query on separate connections.
///
/// Simulates how shell compositions call sweetGrass: one connection per
/// method call, send request, read response, disconnect.
#[tokio::test]
async fn test_uds_composition_pattern_single_shot() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("composition-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkComposition"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let send_and_receive = |path: std::path::PathBuf,
                            request: serde_json::Value|
     -> std::pin::Pin<Box<dyn std::future::Future<Output = serde_json::Value> + Send>> {
        Box::pin(async move {
            let stream = tokio::net::UnixStream::connect(&path)
                .await
                .expect("connect");
            let (reader, mut writer) = stream.into_split();

            let mut req_str = serde_json::to_string(&request).unwrap();
            req_str.push('\n');
            writer.write_all(req_str.as_bytes()).await.unwrap();
            writer.flush().await.expect("flush");

            let mut lines = BufReader::new(reader).lines();
            let resp_line = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                lines.next_line(),
            )
            .await
            .expect("should respond within 5s")
            .unwrap()
            .expect("response line");

            serde_json::from_str(&resp_line).expect("parse response")
        })
    };

    let liveness = send_and_receive(
        sock_path.clone(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1
        }),
    )
    .await;
    assert_eq!(liveness["result"]["alive"], true);

    let created = send_and_receive(
        sock_path.clone(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "braid.create",
            "params": {
                "data_hash": "sha256:comp01020304050607080910111213141516171819202122232425262728293031",
                "mime_type": "application/json",
                "size": 512
            },
            "id": 2
        }),
    )
    .await;
    assert!(
        created["result"]["@id"].is_string(),
        "braid.create should succeed: {created}"
    );

    let queried = send_and_receive(
        sock_path.clone(),
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "braid.query",
            "params": { "filter": {} },
            "id": 3
        }),
    )
    .await;
    assert!(
        queried["result"]["braids"].is_array(),
        "braid.query should return braids array: {queried}"
    );

    listener_handle.abort();
}
