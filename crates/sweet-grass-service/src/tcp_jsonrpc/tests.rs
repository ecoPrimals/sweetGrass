// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use super::*;
use sweet_grass_core::agent::Did;

#[tokio::test]
async fn tcp_jsonrpc_roundtrip() {
    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpTest"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await;
    if stream.is_err() {
        let _ = shutdown_tx.send(true);
        listener_handle.abort();
        return;
    }

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_health_check() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpHealth"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
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
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "healthy");

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_parse_error() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpParse"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(b"{ not valid json }\n")
        .await
        .expect("write");
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(
        response["error"]["code"],
        crate::handlers::jsonrpc::error_code::PARSE_ERROR
    );

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_notification_no_response_then_roundtrip() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpNotify"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {}
    });
    let mut note_str = serde_json::to_string(&notification).unwrap();
    note_str.push('\n');
    writer.write_all(note_str.as_bytes()).await.unwrap();

    let follow_up = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 7
    });
    let mut follow_str = serde_json::to_string(&follow_up).unwrap();
    follow_str.push('\n');
    writer.write_all(follow_str.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 7);
    assert_eq!(response["result"]["status"], "healthy");

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_sequential_requests_same_connection() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpSeq"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let mut lines = BufReader::new(reader).lines();

    let req1 = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let mut s = serde_json::to_string(&req1).unwrap();
    s.push('\n');
    writer.write_all(s.as_bytes()).await.unwrap();
    let first_response = lines.next_line().await.unwrap().expect("response 1");
    let r1: serde_json::Value = serde_json::from_str(&first_response).expect("parse 1");
    assert_eq!(r1["id"], 1);
    assert_eq!(r1["result"]["status"], "healthy");

    let req2 = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 2
    });
    let mut s = serde_json::to_string(&req2).unwrap();
    s.push('\n');
    writer.write_all(s.as_bytes()).await.unwrap();
    let second_response = lines.next_line().await.unwrap().expect("response 2");
    let r2: serde_json::Value = serde_json::from_str(&second_response).expect("parse 2");
    assert_eq!(r2["id"], 2);
    assert_eq!(r2["result"]["alive"], true);

    let req3 = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.readiness",
        "params": {},
        "id": 3
    });
    let mut s = serde_json::to_string(&req3).unwrap();
    s.push('\n');
    writer.write_all(s.as_bytes()).await.unwrap();
    let third_response = lines.next_line().await.unwrap().expect("response 3");
    let r3: serde_json::Value = serde_json::from_str(&third_response).expect("parse 3");
    assert_eq!(r3["id"], 3);
    assert_eq!(r3["result"]["ready"], true);

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_skips_empty_lines() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpEmpty"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(b"\n  \n\t\n").await.unwrap();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 11
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["id"], 11);
    assert_eq!(response["result"]["status"], "healthy");

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_method_not_found() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcp404"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "nonexistent.method",
        "params": {},
        "id": 404
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 404);
    assert_eq!(
        response["error"]["code"],
        crate::handlers::jsonrpc::error_code::METHOD_NOT_FOUND
    );

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}

#[tokio::test]
async fn tcp_jsonrpc_listener_shutdown_exits_gracefully() {
    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpShutdown"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    shutdown_tx.send(true).expect("shutdown signal");

    let join_result = tokio::time::timeout(std::time::Duration::from_secs(2), listener_handle)
        .await
        .expect("listener should exit within timeout");

    let run_result = join_result.expect("listener task join");
    assert!(
        run_result.is_ok(),
        "listener returned error: {run_result:?}"
    );
}

#[tokio::test]
async fn tcp_jsonrpc_invalid_jsonrpc_version() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTcpVer"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let state_clone = state.clone();
    let listener_handle = tokio::spawn(async move {
        let _ = start_tcp_jsonrpc_listener(state_clone, addr, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "1.0",
        "method": "health.check",
        "params": {},
        "id": 88
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 88);
    assert_eq!(
        response["error"]["code"],
        crate::handlers::jsonrpc::error_code::INVALID_REQUEST
    );

    let _ = shutdown_tx.send(true);
    listener_handle.abort();
}
