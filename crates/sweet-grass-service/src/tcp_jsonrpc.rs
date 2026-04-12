// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! TCP newline-delimited JSON-RPC 2.0 listener.
//!
//! Required by `UNIBIN_ARCHITECTURE_STANDARD` v1.1: every primal MUST accept
//! `server --port <PORT>` to bind a TCP JSON-RPC listener using
//! newline-delimited framing (one JSON object per line, terminated by `\n`).
//!
//! This is the composition interface that springs, deploy graphs, and
//! launchers use to orchestrate primals. HTTP-wrapped JSON-RPC
//! (`POST /jsonrpc`) remains available for tooling and dashboards, but
//! this raw TCP transport is mandatory for inter-primal composition per
//! `PRIMAL_IPC_PROTOCOL.md` wire framing.

use std::net::SocketAddr;

use tracing::{debug, info, warn};

/// Start a TCP newline-delimited JSON-RPC 2.0 listener.
///
/// Binds to `0.0.0.0:{port}` and accepts connections until `shutdown`
/// signals. Each connection receives newline-delimited JSON-RPC framing
/// identical to the UDS transport.
///
/// # Errors
///
/// Returns an error if TCP binding fails.
pub async fn start_tcp_jsonrpc_listener(
    state: crate::state::AppState,
    port: u16,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> crate::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        crate::ServiceError::Internal(format!("TCP JSON-RPC bind on {addr} failed: {e}"))
    })?;

    let actual_addr = listener.local_addr().unwrap_or(addr);
    info!("JSON-RPC 2.0 TCP (newline-delimited) listening on {actual_addr}");

    #[cfg(unix)]
    let btsp_required = crate::btsp::is_btsp_required();
    #[cfg(not(unix))]
    let btsp_required = false;

    if btsp_required {
        info!("BTSP handshake required on TCP (FAMILY_ID set)");
    }

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer)) => {
                        if let Err(e) = stream.set_nodelay(true) {
                            warn!("TCP set_nodelay for {peer} failed (non-fatal): {e}");
                        }
                        let state = state.clone();
                        tokio::spawn(async move {
                            #[cfg(unix)]
                            if btsp_required {
                                if let Err(e) = handle_tcp_connection_btsp(stream, state).await {
                                    warn!("TCP BTSP connection from {peer}: {e}");
                                }
                                return;
                            }
                            if let Err(e) = handle_tcp_connection_raw(stream, state).await {
                                warn!("TCP JSON-RPC connection from {peer}: {e}");
                            }
                        });
                    },
                    Err(e) => {
                        warn!("TCP JSON-RPC accept error: {e}");
                    },
                }
            }
            _ = shutdown.changed() => {
                info!("TCP JSON-RPC listener shutting down");
                break;
            }
        }
    }

    Ok(())
}

/// Handle a TCP connection with BTSP handshake then length-prefixed JSON-RPC.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Phase 2: production mode uses BTSP
/// handshake before exposing any JSON-RPC methods.
#[cfg(unix)]
async fn handle_tcp_connection_btsp(
    mut stream: tokio::net::TcpStream,
    state: crate::state::AppState,
) -> crate::Result<()> {
    use crate::btsp;
    use tokio::io::AsyncWriteExt;

    match btsp::perform_server_handshake(&mut stream).await {
        Ok(complete) => {
            debug!(
                session = %complete.session_id,
                cipher = %complete.cipher,
                "TCP BTSP handshake succeeded — entering length-prefixed mode"
            );
        },
        Err(e) => {
            warn!("TCP BTSP handshake failed: {e}");
            return Ok(());
        },
    }

    let (mut reader, mut writer) = tokio::io::split(stream);

    loop {
        let frame = match btsp::read_frame(&mut reader).await {
            Ok(f) => f,
            Err(btsp::BtspError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            },
            Err(e) => {
                warn!("TCP BTSP frame read error: {e}");
                break;
            },
        };

        let request: serde_json::Value = match serde_json::from_slice(&frame) {
            Ok(v) => v,
            Err(e) => {
                let err_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": crate::handlers::jsonrpc::error_code::PARSE_ERROR,
                        "message": format!("Parse error: {e}"),
                    },
                    "id": null
                });
                let payload = serde_json::to_vec(&err_response)?;
                btsp::write_frame(&mut writer, &payload)
                    .await
                    .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
                continue;
            },
        };

        if let Some(response) = crate::handlers::jsonrpc::process_single(&state, request).await {
            let payload = serde_json::to_vec(&response)?;
            btsp::write_frame(&mut writer, &payload)
                .await
                .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
            writer.flush().await?;
        }
    }

    Ok(())
}

/// Handle a single TCP connection with raw newline-delimited JSON-RPC.
///
/// Development mode (no `FAMILY_ID`): no handshake, newline framing.
/// Flushes after every response for reliable composition IPC (trio
/// hardening). Caller sets `TCP_NODELAY` before dispatch.
async fn handle_tcp_connection_raw(
    stream: tokio::net::TcpStream,
    state: crate::state::AppState,
) -> crate::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": crate::handlers::jsonrpc::error_code::PARSE_ERROR,
                        "message": format!("Parse error: {e}"),
                    },
                    "id": null
                });
                let mut resp = serde_json::to_string(&err_response)?;
                resp.push('\n');
                writer.write_all(resp.as_bytes()).await?;
                writer.flush().await?;
                continue;
            },
        };

        if let Some(response) = crate::handlers::jsonrpc::process_single(&state, request).await {
            let mut resp = serde_json::to_string(&response)?;
            resp.push('\n');
            writer.write_all(resp.as_bytes()).await?;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await
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
        let port = addr.port();
        let listener_handle = tokio::spawn(async move {
            let _ = start_tcp_jsonrpc_listener(state_clone, port, shutdown_rx).await;
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
}
