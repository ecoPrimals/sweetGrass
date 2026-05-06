// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! TCP newline-delimited JSON-RPC 2.0 listener.
//!
//! Required by `UNIBIN_ARCHITECTURE_STANDARD` v1.1: every primal MUST accept
//! `server --port <host:port>` to bind a TCP JSON-RPC listener using
//! newline-delimited framing (one JSON object per line, terminated by `\n`).
//! A bare port number (e.g. `9850`) binds `127.0.0.1:9850` (localhost-only
//! per PG-55 security hardening); use `0.0.0.0:9850` for all-interfaces.
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
/// Binds to the given `addr` and accepts connections until `shutdown`
/// signals. Each connection receives newline-delimited JSON-RPC framing
/// identical to the UDS transport.
///
/// # Errors
///
/// Returns an error if TCP binding fails.
pub async fn start_tcp_jsonrpc_listener(
    state: crate::state::AppState,
    addr: SocketAddr,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> crate::Result<()> {
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
                                handle_tcp_with_autodetect(stream, peer, state).await;
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

/// First-line protocol auto-detection for BTSP-required TCP connections.
///
/// Reads the first line to determine the protocol:
/// - First byte not `{` → length-prefixed BTSP handshake
/// - `{"protocol":"btsp",...}` → JSON-line BTSP (primalSpring-compatible)
/// - `{"jsonrpc":"2.0",...}` → raw JSON-RPC (health probes, biomeOS, springs)
///
/// Aligns with Phase 45b wire-format guidance, `BearDog` (PG-35) / `Squirrel` (PG-30).
#[cfg(unix)]
async fn handle_tcp_with_autodetect(
    mut stream: tokio::net::TcpStream,
    peer: SocketAddr,
    state: crate::state::AppState,
) {
    use crate::peek::{DetectedProtocol, detect_protocol};

    let protocol = match detect_protocol(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            warn!("TCP from {peer}: protocol detection failed: {e}");
            return;
        },
    };

    match protocol {
        DetectedProtocol::LengthPrefixedBtsp(byte) => {
            let peeked = crate::peek::PeekedStream::new(byte, stream);
            if let Err(e) = handle_tcp_connection_btsp(peeked, state).await {
                warn!("TCP BTSP connection from {peer}: {e}");
            }
        },
        DetectedProtocol::JsonLineBtsp(client_hello) => {
            debug!("TCP from {peer}: first-line auto-detect → JSON-line BTSP");
            match crate::btsp::perform_server_handshake_jsonline(&mut stream, client_hello).await {
                Ok(outcome) => {
                    debug!(
                        session = %outcome.complete.session_id,
                        cipher = %outcome.complete.cipher,
                        has_phase3_key = outcome.handshake_key.is_some(),
                        "TCP BTSP JSON-line handshake from {peer} succeeded"
                    );
                    if let Err(e) =
                        handle_tcp_post_jsonline(stream, state, outcome.handshake_key).await
                    {
                        warn!("TCP JSON-RPC from {peer} (post BTSP JSON-line): {e}");
                    }
                },
                Err(e) => {
                    warn!("TCP BTSP JSON-line handshake from {peer} failed: {e}");
                },
            }
        },
        DetectedProtocol::JsonRpc(first_request) => {
            debug!("TCP from {peer}: first-line auto-detect → raw JSON-RPC");
            if let Err(e) = handle_tcp_connection_raw_with_first(stream, state, first_request).await
            {
                warn!("TCP raw connection from {peer} (auto-detected): {e}");
            }
        },
        DetectedProtocol::Unknown(obj) => {
            warn!("TCP from {peer}: unrecognized first-line protocol: {obj}");
        },
    }
}

/// Handle a TCP connection with BTSP handshake then length-prefixed JSON-RPC.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Phase 2–3: production mode uses BTSP
/// handshake before exposing any JSON-RPC methods.  After the handshake,
/// checks for Phase 3 `btsp.negotiate` to upgrade to encrypted framing.
///
/// Generic over stream type to support [`PeekedStream`](crate::peek::PeekedStream)
/// wrapping for first-byte auto-detection.
#[cfg(unix)]
async fn handle_tcp_connection_btsp(
    mut stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send,
    state: crate::state::AppState,
) -> crate::Result<()> {
    use crate::btsp;
    use tokio::io::AsyncWriteExt;

    let outcome = match btsp::perform_server_handshake(&mut stream).await {
        Ok(o) => o,
        Err(e) => {
            warn!("TCP BTSP handshake failed: {e}");
            return Ok(());
        },
    };

    debug!(
        session = %outcome.complete.session_id,
        cipher = %outcome.complete.cipher,
        has_phase3_key = outcome.handshake_key.is_some(),
        "TCP BTSP handshake succeeded — entering length-prefixed mode"
    );

    let (mut reader, mut writer) = tokio::io::split(stream);

    let first_frame = match btsp::read_frame(&mut reader).await {
        Ok(f) => f,
        Err(btsp::BtspError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Ok(());
        },
        Err(e) => {
            warn!("TCP BTSP frame read error: {e}");
            return Ok(());
        },
    };

    let first_request: serde_json::Value = match serde_json::from_slice(&first_frame) {
        Ok(v) => v,
        Err(e) => {
            let err_response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": crate::handlers::jsonrpc::error_code::PARSE_ERROR, "message": format!("Parse error: {e}")},
                "id": null
            });
            let payload = serde_json::to_vec(&err_response)?;
            btsp::write_frame(&mut writer, &payload)
                .await
                .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
            return Ok(());
        },
    };

    match crate::btsp::transport::try_phase3_negotiate(
        &first_request,
        outcome.handshake_key.as_ref(),
        &mut writer,
        false,
    )
    .await?
    {
        crate::btsp::transport::NegotiateOutcome::Encrypted(session_keys) => {
            crate::btsp::transport::run_encrypted_frame_loop(&mut reader, &mut writer, &state, &session_keys)
                .await?;
            return Ok(());
        }
        crate::btsp::transport::NegotiateOutcome::NullCipher => {}
        crate::btsp::transport::NegotiateOutcome::NotNegotiate => {
            if let Some(response) =
                crate::handlers::jsonrpc::process_single(&state, first_request).await
            {
                let payload = serde_json::to_vec(&response)?;
                btsp::write_frame(&mut writer, &payload)
                    .await
                    .map_err(|e| crate::ServiceError::Internal(e.to_string()))?;
                writer.flush().await?;
            }
        }
    }

    crate::btsp::transport::run_plaintext_frame_loop(&mut reader, &mut writer, &state).await
}

/// Post-JSON-line handshake on TCP: check for Phase 3, then route.
#[cfg(unix)]
async fn handle_tcp_post_jsonline(
    stream: tokio::net::TcpStream,
    state: crate::state::AppState,
    handshake_key: Option<[u8; 32]>,
) -> crate::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    let mut first_line = String::new();
    match buf_reader.read_line(&mut first_line).await {
        Ok(0) => return Ok(()),
        Ok(_) => {},
        Err(e) => {
            warn!("TCP BTSP JSON-line: failed to read first post-handshake line: {e}");
            return Ok(());
        },
    }

    let first_request: serde_json::Value = match serde_json::from_str(first_line.trim()) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    match crate::btsp::transport::try_phase3_negotiate(
        &first_request,
        handshake_key.as_ref(),
        &mut writer,
        true,
    )
    .await?
    {
        crate::btsp::transport::NegotiateOutcome::Encrypted(session_keys) => {
            let mut combined = buf_reader
                .into_inner()
                .reunite(writer)
                .map_err(|e| crate::ServiceError::Internal(format!("reunite: {e}")))?;
            let (mut enc_reader, mut enc_writer) = tokio::io::split(&mut combined);
            crate::btsp::transport::run_encrypted_frame_loop(
                &mut enc_reader,
                &mut enc_writer,
                &state,
                &session_keys,
            )
            .await?;
            return Ok(());
        }
        crate::btsp::transport::NegotiateOutcome::NullCipher => {}
        crate::btsp::transport::NegotiateOutcome::NotNegotiate => {
            if let Some(response) =
                crate::handlers::jsonrpc::process_single(&state, first_request).await
            {
                let mut resp_str = serde_json::to_string(&response)?;
                resp_str.push('\n');
                writer.write_all(resp_str.as_bytes()).await?;
                writer.flush().await?;
            }
        }
    }

    let stream = buf_reader
        .into_inner()
        .reunite(writer)
        .map_err(|e| crate::ServiceError::Internal(format!("reunite: {e}")))?;
    handle_tcp_connection_raw(stream, state).await
}

/// Handle a single TCP connection with raw newline-delimited JSON-RPC.
///
/// Development mode (no `FAMILY_ID`): no handshake, newline framing.
/// Also used for auto-detected plain JSON-RPC connections when BTSP is
/// required but the client sent `{` as the first byte (health probes).
///
/// Generic over stream type to support both `TcpStream` and
/// [`PeekedStream`](crate::peek::PeekedStream).
async fn handle_tcp_connection_raw(
    stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send,
    state: crate::state::AppState,
) -> crate::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = tokio::io::split(stream);
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

/// Handle a TCP connection where the first JSON-RPC request has already been
/// consumed by the auto-detect layer.
async fn handle_tcp_connection_raw_with_first(
    stream: tokio::net::TcpStream,
    state: crate::state::AppState,
    first_request: serde_json::Value,
) -> crate::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = tokio::io::split(stream);

    if let Some(response) = crate::handlers::jsonrpc::process_single(&state, first_request).await {
        let mut resp = serde_json::to_string(&response)?;
        resp.push('\n');
        writer.write_all(resp.as_bytes()).await?;
        writer.flush().await?;
    }

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
#[path = "tcp_jsonrpc/tests.rs"]
mod tests;
