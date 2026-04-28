// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! PG-52 domain method verification over UDS.
//!
//! Validates that `braid.create`, `braid.query`, and `provenance.graph`
//! return well-formed JSON-RPC responses over UDS, covering both raw
//! and auto-detected connection paths. Also exercises the composition
//! single-shot pattern (one connection per method call).

use sweet_grass_core::agent::Did;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::super::*;

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

#[tokio::test]
async fn test_uds_braid_create_tower_signed() {
    use std::os::unix::net::UnixListener as StdUnixListener;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("braid-signed-test.sock");
    let beardog_sock = dir.path().join("beardog-mock.sock");

    let std_listener = StdUnixListener::bind(&beardog_sock).expect("bind mock beardog");
    std_listener.set_nonblocking(true).unwrap();
    let mock_listener = tokio::net::UnixListener::from_std(std_listener).unwrap();

    let mock_handle = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = mock_listener.accept().await else {
                break;
            };
            let (reader, mut writer) = stream.into_split();
            let mut lines = tokio::io::BufReader::new(reader).lines();

            if let Ok(Some(line)) = lines.next_line().await {
                if let Ok(req) = serde_json::from_str::<serde_json::Value>(&line) {
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "result": {
                            "signature": "dG93ZXItc2lnLWJ5dGVz",
                            "algorithm": "ed25519",
                            "public_key": "dG93ZXItcHViLWtleQ=="
                        }
                    });
                    let mut resp = serde_json::to_string(&response).unwrap();
                    resp.push('\n');
                    let _ = writer.write_all(resp.as_bytes()).await;
                }
            }
        }
    });

    let crypto = crate::crypto_delegate::CryptoDelegate::with_socket(beardog_sock);
    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkSignedBraid"))
        .with_crypto(crypto);

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
            "data_hash": "sha256:aabbccdd01020304050607080910111213141516171819202122232425262728",
            "mime_type": "application/json",
            "size": 512
        },
        "id": 42
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        lines.next_line(),
    )
    .await
    .expect("braid.create signed should respond within 10s")
    .unwrap()
    .expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 42);
    assert!(response["result"].is_object(), "should return result");

    let witness = &response["result"]["witness"];
    assert_eq!(
        witness["kind"], "signature",
        "witness should be a signature: {witness}"
    );
    assert_eq!(
        witness["algorithm"], "ed25519",
        "algorithm should be ed25519: {witness}"
    );
    assert_eq!(
        witness["tier"], "tower",
        "tier should be tower (delegated): {witness}"
    );
    assert!(
        witness["evidence"].is_string() && !witness["evidence"].as_str().unwrap().is_empty(),
        "evidence should be non-empty base64: {witness}"
    );
    assert!(
        witness["agent"]
            .as_str()
            .is_some_and(|a| a.starts_with("did:key:z6Mk")),
        "agent should be a did:key from BearDog public key: {witness}"
    );

    mock_handle.abort();
    listener_handle.abort();
}
