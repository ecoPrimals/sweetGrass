// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unix domain socket transport for biomeOS IPC.
//!
//! Provides XDG-compliant socket path resolution and a newline-delimited
//! JSON-RPC 2.0 listener over Unix sockets, as required by the
//! `UNIVERSAL_IPC_STANDARD_V3` for primal coordination.
//!
//! ## Socket Resolution Order
//!
//! 1. `SWEETGRASS_SOCKET` — explicit override
//! 2. `BIOMEOS_SOCKET_DIR` + `/sweetgrass-{family_id}.sock`
//! 3. `$XDG_RUNTIME_DIR/biomeos/sweetgrass-{family_id}.sock`
//! 4. `$TMPDIR/biomeos-{user}/sweetgrass-{family_id}.sock`
//! 5. `$TMPDIR/biomeos/sweetgrass-{family_id}.sock`
//!
//! All tiers resolve into a `biomeos/` subdirectory — never directly into
//! `/tmp/`. This supports `ProtectSystem=strict` systemd hardening.

use std::path::PathBuf;

use tracing::{debug, info, warn};

use sweet_grass_core::identity;
use sweet_grass_core::primal_names::env_vars;

#[path = "uds/lifecycle.rs"]
mod lifecycle;
pub use lifecycle::{
    cleanup_capability_symlink, cleanup_pid_file, cleanup_socket_at, create_capability_symlink,
    pid_path, write_pid_file,
};

/// BTSP Phase 1 configuration error: `FAMILY_ID` and `BIOMEOS_INSECURE=1`
/// are mutually exclusive.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Security Model, a primal that claims family
/// membership MUST authenticate via BTSP handshake. Setting `BIOMEOS_INSECURE`
/// while a family is configured is contradictory and the primal MUST refuse
/// to start.
#[derive(Debug, thiserror::Error)]
#[error(
    "BTSP guard violation: FAMILY_ID=\"{family_id}\" and BIOMEOS_INSECURE=1 \
     are mutually exclusive — cannot claim a family and skip authentication \
     (BTSP_PROTOCOL_STANDARD §Phase 1)"
)]
pub struct BtspGuardViolation {
    family_id: String,
}

/// Default primal name when `SelfKnowledge` is unavailable.
const DEFAULT_PRIMAL_NAME: &str = identity::PRIMAL_NAME;

/// Injected socket resolution configuration.
///
/// Follows the airSpring / biomeOS `_with_config` DI pattern so tests
/// can resolve socket paths without mutating process environment.
#[derive(Debug, Clone, Default)]
pub struct SocketConfig {
    /// Explicit socket path override (like `SWEETGRASS_SOCKET` env var).
    pub explicit_socket: Option<String>,
    /// biomeOS socket directory (like `BIOMEOS_SOCKET_DIR` env var).
    pub biomeos_socket_dir: Option<String>,
    /// biomeOS family ID (like `BIOMEOS_FAMILY_ID` env var).
    pub family_id: Option<String>,
    /// XDG runtime directory (like `XDG_RUNTIME_DIR` env var).
    pub xdg_runtime_dir: Option<String>,
    /// System user (like `USER` env var).
    pub user: Option<String>,
    /// Override primal name (otherwise uses default).
    pub primal_name: Option<String>,
}

/// Resolve the effective `FAMILY_ID` from the environment.
///
/// Resolution order per `BTSP_PROTOCOL_STANDARD` §Phase 1:
/// 1. `SWEETGRASS_FAMILY_ID` (primal-specific override)
/// 2. `BIOMEOS_FAMILY_ID` (ecosystem-wide)
/// 3. `FAMILY_ID` (generic)
///
/// Empty strings and `"default"` are treated as absent.
#[must_use]
pub fn resolve_family_id_from_env() -> Option<String> {
    std::env::var(env_vars::SWEETGRASS_FAMILY_ID)
        .or_else(|_| std::env::var(env_vars::BIOMEOS_FAMILY_ID))
        .or_else(|_| std::env::var(env_vars::FAMILY_ID))
        .ok()
        .filter(|s| !s.is_empty() && s != "default")
}

/// Validate the BTSP insecure guard by reading environment variables.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Security Model: if `FAMILY_ID` is set
/// (non-empty, not `"default"`) AND `BIOMEOS_INSECURE=1`, the primal MUST
/// refuse to start. Delegates to [`validate_insecure_guard_with`] for
/// DI-testable logic.
///
/// # Errors
///
/// Returns [`BtspGuardViolation`] when the conflicting configuration is
/// detected.
pub fn validate_insecure_guard() -> Result<(), BtspGuardViolation> {
    let family_id = resolve_family_id_from_env();
    let insecure = std::env::var(env_vars::BIOMEOS_INSECURE)
        .map(|v| v == "1")
        .unwrap_or(false);

    validate_insecure_guard_with(family_id.as_deref(), insecure)
}

/// DI-friendly BTSP insecure guard validation (no env var reads).
///
/// # Errors
///
/// Returns [`BtspGuardViolation`] when `family_id` is `Some` and
/// `biomeos_insecure` is `true`.
pub fn validate_insecure_guard_with(
    family_id: Option<&str>,
    biomeos_insecure: bool,
) -> Result<(), BtspGuardViolation> {
    if let Some(fid) = family_id
        && biomeos_insecure
    {
        return Err(BtspGuardViolation {
            family_id: fid.to_owned(),
        });
    }
    Ok(())
}

/// Resolve the Unix domain socket path using XDG-compliant resolution.
///
/// The primal name is derived from `SelfKnowledge` when available (e.g. via
/// `state.self_knowledge`). When `primal_name` is `None`, falls back to
/// `PRIMAL_NAME` env var or `"sweetgrass"`.
///
/// Family ID resolution follows the BTSP standard chain:
/// `SWEETGRASS_FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `FAMILY_ID`.
///
/// Delegates to [`resolve_socket_path_with`] after reading env vars.
#[must_use]
pub fn resolve_socket_path(primal_name: Option<&str>) -> PathBuf {
    let config = SocketConfig {
        explicit_socket: std::env::var(env_vars::SWEETGRASS_SOCKET).ok(),
        biomeos_socket_dir: std::env::var(env_vars::BIOMEOS_SOCKET_DIR).ok(),
        family_id: resolve_family_id_from_env(),
        xdg_runtime_dir: std::env::var(env_vars::XDG_RUNTIME_DIR).ok(),
        user: std::env::var(env_vars::USER).ok(),
        primal_name: primal_name
            .map(String::from)
            .or_else(|| std::env::var(env_vars::PRIMAL_NAME).ok()),
    };
    resolve_socket_path_with(&config)
}

/// Resolve socket path with injected configuration (no env var reads).
///
/// DI-friendly variant for tests and embedded contexts. Follows the
/// airSpring `_with` pattern adopted ecosystem-wide per biomeOS V239.
#[must_use]
pub fn resolve_socket_path_with(config: &SocketConfig) -> PathBuf {
    let name = config.primal_name.as_deref().unwrap_or(DEFAULT_PRIMAL_NAME);
    let family_id = config.family_id.as_deref().unwrap_or("");
    let sock_name = if family_id.is_empty() {
        format!("{name}.sock")
    } else {
        format!("{name}-{family_id}.sock")
    };

    if let Some(ref path) = config.explicit_socket {
        return PathBuf::from(path);
    }

    if let Some(ref dir) = config.biomeos_socket_dir {
        return PathBuf::from(dir).join(&sock_name);
    }

    if let Some(ref xdg) = config.xdg_runtime_dir {
        return PathBuf::from(xdg)
            .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
            .join(&sock_name);
    }

    if let Some(ref user) = config.user {
        return std::env::temp_dir()
            .join(format!("biomeos-{user}"))
            .join(&sock_name);
    }

    std::env::temp_dir()
        .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
        .join(&sock_name)
}

/// Start the Unix domain socket JSON-RPC listener with coordinated shutdown.
///
/// Accepts newline-delimited JSON-RPC 2.0 requests and routes them through
/// the same dispatch table as the HTTP endpoint.
///
/// # Errors
///
/// Returns an error if socket binding fails.
pub async fn start_uds_listener(
    state: crate::state::AppState,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> std::result::Result<(), crate::ServiceError> {
    let primal_name = state.self_knowledge.as_ref().map(|sk| sk.name.as_str());
    let path = resolve_socket_path(primal_name);
    start_uds_listener_at(state, &path, shutdown).await
}

/// Start the Unix domain socket JSON-RPC listener at an explicit path.
///
/// DI-friendly variant: tests pass a path directly instead of going
/// through env-based resolution. Accepts connections until `shutdown` signals.
///
/// # Errors
///
/// Returns an error if socket binding fails.
pub async fn start_uds_listener_at(
    state: crate::state::AppState,
    path: &std::path::Path,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> std::result::Result<(), crate::ServiceError> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| crate::ServiceError::Internal(format!("mkdir failed: {e}")))?;
    }

    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| crate::ServiceError::Internal(format!("remove stale socket: {e}")))?;
    }
    let stale_pid = lifecycle::pid_path(path);
    if stale_pid.exists() {
        let _ = std::fs::remove_file(&stale_pid);
    }

    let listener = tokio::net::UnixListener::bind(path)
        .map_err(|e| crate::ServiceError::Internal(format!("UDS bind failed: {e}")))?;
    info!("JSON-RPC 2.0 UDS listening on {}", path.display());

    lifecycle::write_pid_file(path);
    create_capability_symlink(path);

    if state.btsp_required {
        info!("BTSP handshake required on UDS (FAMILY_ID set)");
    }

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            handle_uds_with_autodetect(stream, state).await;
                        });
                    },
                    Err(e) => {
                        warn!("UDS accept error: {e}");
                    },
                }
            }
            _ = shutdown.changed() => {
                info!("UDS listener shutting down");
                break;
            }
        }
    }

    Ok(())
}

/// riboCipher-aware protocol detection for UDS connections.
///
/// Checks for riboCipher signal prefix bytes (`0xEC`/`0xED`/`0xEE`) first,
/// then falls back to legacy peek logic for unsignalled connections (WARN).
///
/// ## riboCipher Tier 1 (clear signal) routing
///
/// | Protocol type | Route |
/// |---------------|-------|
/// | 0x00 (Probe)  | Lightweight health response |
/// | 0x01 (NDJSON) | Raw JSON-RPC handler |
/// | 0x02 (BTSP binary) | Length-prefixed BTSP handshake |
/// | 0x03 (BTSP JSON-line) | JSON-line BTSP handshake |
/// | other | Reject with `-32002` |
pub(crate) async fn handle_uds_with_autodetect(
    mut stream: tokio::net::UnixStream,
    state: crate::state::AppState,
) {
    use crate::peek::{DetectedProtocol, detect_protocol};

    let protocol = match detect_protocol(&mut stream).await {
        Ok(p) => p,
        Err(e) => {
            warn!("UDS: protocol detection failed: {e}");
            let _ = write_jsonrpc_error(
                &mut stream,
                serde_json::Value::Null,
                crate::handlers::jsonrpc::error_code::PARSE_ERROR,
                format!("Protocol detection failed: {e}"),
            )
            .await;
            return;
        },
    };

    match protocol {
        DetectedProtocol::RiboCipherClear {
            protocol_type: pt,
        }
        | DetectedProtocol::RiboCipherMito {
            protocol_type: pt,
        } => {
            handle_ribocipher_clear_uds(stream, state, pt).await;
        }
        DetectedProtocol::Rejected { first_byte } => {
            warn!(
                first_byte,
                "UDS: rejected unsignalled connection (riboCipher signal required)"
            );
            let _ = write_jsonrpc_error(
                &mut stream,
                serde_json::Value::Null,
                -32002,
                "riboCipher signal required. Send [0xEC/0xED, protocol_type] prefix. \
                 See RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD.md.",
            )
            .await;
        }
    }
}

/// Route a riboCipher Tier 1 (clear signal) connection on UDS.
async fn handle_ribocipher_clear_uds(
    mut stream: tokio::net::UnixStream,
    state: crate::state::AppState,
    pt: u8,
) {
    use crate::peek::protocol_type;

    match pt {
        protocol_type::PROBE => {
            debug!("UDS riboCipher: probe (0x00)");
            let resp = serde_json::json!({
                "jsonrpc": "2.0",
                "result": { "status": "healthy" },
                "id": null,
            });
            let _ = write_jsonrpc_response(&mut stream, &resp).await;
        }
        protocol_type::NDJSON_JSONRPC => {
            debug!("UDS riboCipher: NDJSON JSON-RPC (0x01)");
            if let Err(e) = handle_uds_connection_raw(stream, state).await {
                warn!("UDS raw connection error (riboCipher): {e}");
            }
        }
        protocol_type::BTSP_BINARY => {
            debug!("UDS riboCipher: BTSP binary (0x02)");
            if let Err(e) = handle_uds_connection_btsp(stream, state).await {
                warn!("UDS BTSP connection error (riboCipher): {e}");
            }
        }
        protocol_type::BTSP_JSONLINE => {
            debug!("UDS riboCipher: BTSP JSON-line (0x03) — reading ClientHello");
            match read_jsonline_client_hello(&mut stream).await {
                Ok(hello) => {
                    handle_uds_connection_btsp_jsonline(stream, state, hello).await;
                }
                Err(e) => {
                    warn!("UDS riboCipher BTSP JSON-line: failed to read ClientHello: {e}");
                }
            }
        }
        unknown => {
            warn!(
                protocol_type = unknown,
                "UDS riboCipher: unsupported protocol type"
            );
            let _ = write_jsonrpc_error(
                &mut stream,
                serde_json::Value::Null,
                -32002,
                format!("Unsupported riboCipher protocol type: 0x{unknown:02X}"),
            )
            .await;
        }
    }
}

/// Read a JSON-line `ClientHello` from the stream (for riboCipher BTSP JSON-line).
async fn read_jsonline_client_hello<S: tokio::io::AsyncRead + Unpin>(
    stream: &mut S,
) -> std::io::Result<crate::btsp::protocol::ClientHello> {
    use tokio::io::AsyncBufReadExt;

    let mut reader = tokio::io::BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    serde_json::from_str(&line)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Write a JSON-RPC response object and a trailing newline.
async fn write_jsonrpc_response(
    stream: &mut tokio::net::UnixStream,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut buf = serde_json::to_string(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    buf.push('\n');
    stream.write_all(buf.as_bytes()).await?;
    stream.flush().await
}

/// Handle a UDS connection with BTSP handshake then length-prefixed JSON-RPC.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Phase 2–3: when `FAMILY_ID` is set, every
/// incoming connection runs the 4-step handshake.  After the handshake, the
/// first frame is inspected for a Phase 3 `btsp.negotiate` request.  If the
/// client negotiates ChaCha20-Poly1305, subsequent frames use encrypted
/// AEAD framing; otherwise plaintext length-prefixed JSON-RPC continues.
///
/// Generic over stream type for riboCipher protocol routing.
async fn handle_uds_connection_btsp(
    mut stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send,
    state: crate::state::AppState,
) -> std::result::Result<(), crate::ServiceError> {
    use crate::btsp;
    use tokio::io::AsyncWriteExt;

    let outcome =
        match btsp::perform_server_handshake_with(&mut stream, &state.security_socket_path).await {
            Ok(o) => o,
            Err(e) => {
                warn!("UDS BTSP handshake failed: {e}");
                return Ok(());
            },
        };

    debug!(
        session = %outcome.complete.session_id,
        cipher = %outcome.complete.cipher,
        has_phase3_key = outcome.handshake_key.is_some(),
        "UDS BTSP handshake succeeded — entering length-prefixed mode"
    );

    let (mut reader, mut writer) = tokio::io::split(stream);

    let first_frame = match btsp::read_frame(&mut reader).await {
        Ok(f) => f,
        Err(btsp::BtspError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Ok(());
        },
        Err(e) => {
            warn!("UDS BTSP frame read error: {e}");
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
            crate::btsp::transport::run_encrypted_frame_loop(
                &mut reader,
                &mut writer,
                &state,
                &session_keys,
            )
            .await?;
            return Ok(());
        },
        crate::btsp::transport::NegotiateOutcome::NullCipher => {},
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
        },
    }

    crate::btsp::transport::run_plaintext_frame_loop(&mut reader, &mut writer, &state).await
}

/// Handle a UDS connection with JSON-line BTSP handshake.
///
/// After the 4-step JSON-line handshake, reads one newline-delimited JSON-RPC
/// line.  If it is a Phase 3 `btsp.negotiate`, responds and switches to
/// encrypted length-prefixed framing.  Otherwise processes it as a regular
/// JSON-RPC request and enters the plaintext newline-delimited loop.
async fn handle_uds_connection_btsp_jsonline(
    mut stream: tokio::net::UnixStream,
    state: crate::state::AppState,
    client_hello: crate::btsp::ClientHello,
) {
    let outcome = match crate::btsp::perform_server_handshake_jsonline_with(
        &mut stream,
        client_hello,
        &state.security_socket_path,
    )
    .await
    {
        Ok(o) => o,
        Err(e) => {
            warn!("UDS BTSP JSON-line handshake failed: {e}");
            return;
        },
    };

    debug!(
        session = %outcome.complete.session_id,
        cipher = %outcome.complete.cipher,
        has_phase3_key = outcome.handshake_key.is_some(),
        "UDS BTSP JSON-line handshake succeeded"
    );

    if let Err(e) = handle_post_jsonline_handshake(stream, state, outcome.handshake_key).await {
        warn!("UDS JSON-RPC error (post BTSP JSON-line handshake): {e}");
    }
}

/// Post-JSON-line handshake: read first line, check for Phase 3, route.
async fn handle_post_jsonline_handshake(
    stream: tokio::net::UnixStream,
    state: crate::state::AppState,
    handshake_key: Option<[u8; 32]>,
) -> std::result::Result<(), crate::ServiceError> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);

    let mut first_line = String::new();
    match buf_reader.read_line(&mut first_line).await {
        Ok(0) => return Ok(()),
        Ok(_) => {},
        Err(e) => {
            warn!("UDS BTSP JSON-line: failed to read first post-handshake line: {e}");
            return Ok(());
        },
    }

    let first_request: serde_json::Value = match serde_json::from_str(first_line.trim()) {
        Ok(v) => v,
        Err(e) => {
            let _ = write_jsonrpc_error(
                &mut writer,
                serde_json::Value::Null,
                crate::handlers::jsonrpc::error_code::PARSE_ERROR,
                format!("Parse error: {e}"),
            )
            .await;
            return Ok(());
        },
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
        },
        crate::btsp::transport::NegotiateOutcome::NullCipher => {},
        crate::btsp::transport::NegotiateOutcome::NotNegotiate => {
            if let Some(response) =
                crate::handlers::jsonrpc::process_single(&state, first_request).await
            {
                let mut resp_str = serde_json::to_string(&response)?;
                resp_str.push('\n');
                writer.write_all(resp_str.as_bytes()).await?;
                writer.flush().await?;
            }
        },
    }

    let stream = buf_reader
        .into_inner()
        .reunite(writer)
        .map_err(|e| crate::ServiceError::Internal(format!("reunite: {e}")))?;
    handle_uds_connection_raw(stream, state).await
}

/// Handle a single UDS connection with raw newline-delimited JSON-RPC.
///
/// Development mode (no `FAMILY_ID`): no handshake, newline framing.
/// Also used for auto-detected plain JSON-RPC connections when BTSP is
/// required but the client sent `{` as the first byte (health probes),
/// and as the post-handshake mode for JSON-line BTSP.
///
/// Generic over stream type for riboCipher protocol routing.
async fn handle_uds_connection_raw(
    stream: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send,
    state: crate::state::AppState,
) -> std::result::Result<(), crate::ServiceError> {
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
                    "error": {"code": crate::handlers::jsonrpc::error_code::PARSE_ERROR, "message": format!("Parse error: {e}")},
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

/// Write a JSON-RPC error response directly to a stream.
///
/// Used by the auto-detect path when protocol detection fails or the
/// first line is unrecognized — ensures shell callers always receive
/// a well-formed error instead of an empty/closed connection.
async fn write_jsonrpc_error(
    stream: &mut (impl tokio::io::AsyncWrite + Unpin),
    id: serde_json::Value,
    code: i64,
    message: impl Into<String>,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": code, "message": message.into() },
        "id": id,
    });
    let mut resp = serde_json::to_string(&response).map_err(std::io::Error::other)?;
    resp.push('\n');
    stream.write_all(resp.as_bytes()).await?;
    stream.flush().await
}

/// Remove the socket file and capability symlink on shutdown.
pub fn cleanup_socket() {
    let path = resolve_socket_path(None);
    cleanup_socket_at(&path);
}

#[cfg(test)]
#[path = "uds/tests.rs"]
mod tests;
