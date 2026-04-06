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

use tracing::{info, warn};

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

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer)) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_tcp_connection(stream, state).await {
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

/// Handle a single TCP connection with newline-delimited JSON-RPC.
async fn handle_tcp_connection(
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
                continue;
            },
        };

        if let Some(response) = crate::handlers::jsonrpc::process_single(&state, request).await {
            let mut resp = serde_json::to_string(&response)?;
            resp.push('\n');
            writer.write_all(resp.as_bytes()).await?;
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
}
