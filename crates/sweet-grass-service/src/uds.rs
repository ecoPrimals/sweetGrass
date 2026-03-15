// SPDX-License-Identifier: AGPL-3.0-only
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
//! 5. `$TMPDIR/sweetgrass-{family_id}.sock`

use std::path::PathBuf;

use tracing::{debug, info, warn};

use sweet_grass_core::identity;

/// Default primal name when `SelfKnowledge` is unavailable.
const DEFAULT_PRIMAL_NAME: &str = identity::PRIMAL_NAME;

/// Resolve the Unix domain socket path using XDG-compliant resolution.
///
/// The primal name is derived from `SelfKnowledge` when available (e.g. via
/// `state.self_knowledge`). When `primal_name` is `None`, falls back to
/// `PRIMAL_NAME` env var or `"sweetgrass"`.
///
/// Follows the same resolution order as other ecoPrimals primals
/// (ludoSpring, bearDog, etc.) for biomeOS coordination.
#[must_use]
pub fn resolve_socket_path(primal_name: Option<&str>) -> PathBuf {
    let env_name = std::env::var("PRIMAL_NAME").ok();
    let name = primal_name
        .or(env_name.as_deref())
        .unwrap_or(DEFAULT_PRIMAL_NAME);
    let family_id = std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_default();
    let sock_name = if family_id.is_empty() {
        format!("{name}.sock")
    } else {
        format!("{name}-{family_id}.sock")
    };

    // 1. Explicit override
    if let Ok(path) = std::env::var("SWEETGRASS_SOCKET") {
        debug!(path, "Using explicit SWEETGRASS_SOCKET");
        return PathBuf::from(path);
    }

    // 2. BIOMEOS_SOCKET_DIR
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(&dir).join(&sock_name);
        debug!(?path, "Using BIOMEOS_SOCKET_DIR");
        return path;
    }

    // 3. XDG_RUNTIME_DIR/biomeos/
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = PathBuf::from(&xdg).join("biomeos");
        let path = dir.join(&sock_name);
        debug!(?path, "Using XDG_RUNTIME_DIR/biomeos");
        return path;
    }

    // 4. $TMPDIR/biomeos-{user}/ (platform-agnostic)
    if let Ok(user) = std::env::var("USER") {
        let path = std::env::temp_dir()
            .join(format!("biomeos-{user}"))
            .join(&sock_name);
        debug!(?path, "Using TMPDIR/biomeos-USER");
        return path;
    }

    // 5. $TMPDIR fallback (platform-agnostic)
    let path = std::env::temp_dir().join(&sock_name);
    debug!(?path, "Using TMPDIR fallback");
    path
}

/// Start the Unix domain socket JSON-RPC listener.
///
/// Accepts newline-delimited JSON-RPC 2.0 requests and routes them through
/// the same dispatch table as the HTTP endpoint.
///
/// # Errors
///
/// Returns an error if socket binding fails.
pub async fn start_uds_listener(
    state: crate::state::AppState,
) -> std::result::Result<(), crate::ServiceError> {
    let primal_name = state.self_knowledge.as_ref().map(|sk| sk.name.as_str());
    let path = resolve_socket_path(primal_name);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::ServiceError::Internal(format!("mkdir failed: {e}")))?;
        }
    }

    // Remove stale socket
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| crate::ServiceError::Internal(format!("remove stale socket: {e}")))?;
    }

    let listener = tokio::net::UnixListener::bind(&path)
        .map_err(|e| crate::ServiceError::Internal(format!("UDS bind failed: {e}")))?;
    info!("JSON-RPC 2.0 UDS listening on {}", path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_uds_connection(stream, state).await {
                        warn!("UDS connection error: {e}");
                    }
                });
            },
            Err(e) => {
                warn!("UDS accept error: {e}");
            },
        }
    }
}

/// Handle a single UDS connection with newline-delimited JSON-RPC.
async fn handle_uds_connection(
    stream: tokio::net::UnixStream,
    state: crate::state::AppState,
) -> std::result::Result<(), crate::ServiceError> {
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
                    "error": {"code": crate::handlers::jsonrpc::error_code::PARSE_ERROR, "message": format!("Parse error: {e}")},
                    "id": null
                });
                let mut resp = serde_json::to_string(&err_response)?;
                resp.push('\n');
                writer.write_all(resp.as_bytes()).await?;
                continue;
            },
        };

        let response = crate::handlers::jsonrpc::handle_jsonrpc(
            axum::extract::State(state.clone()),
            axum::Json(request),
        )
        .await;

        let mut resp = serde_json::to_string(&response.0)?;
        resp.push('\n');
        writer.write_all(resp.as_bytes()).await?;
    }

    Ok(())
}

/// Remove the socket file on shutdown.
pub fn cleanup_socket() {
    let path = resolve_socket_path(None);
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            warn!("Failed to clean up UDS socket {}: {e}", path.display());
        } else {
            debug!("Cleaned up UDS socket {}", path.display());
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use serial_test::serial;
    use sweet_grass_core::agent::Did;

    fn clear_env() {
        std::env::remove_var("SWEETGRASS_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        std::env::remove_var("XDG_RUNTIME_DIR");
        std::env::remove_var("USER");
        std::env::remove_var("PRIMAL_NAME");
    }

    #[test]
    #[serial]
    fn test_resolve_socket_explicit() {
        clear_env();
        std::env::set_var("SWEETGRASS_SOCKET", "/custom/path.sock");
        assert_eq!(
            resolve_socket_path(None),
            PathBuf::from("/custom/path.sock")
        );
    }

    #[test]
    #[serial]
    fn test_resolve_socket_biomeos_dir() {
        clear_env();
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");
        assert_eq!(
            resolve_socket_path(None),
            PathBuf::from("/run/biomeos/sweetgrass.sock")
        );
    }

    #[test]
    #[serial]
    fn test_resolve_socket_biomeos_dir_with_family() {
        clear_env();
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");
        std::env::set_var("BIOMEOS_FAMILY_ID", "alpha");
        assert_eq!(
            resolve_socket_path(None),
            PathBuf::from("/run/biomeos/sweetgrass-alpha.sock")
        );
    }

    #[test]
    #[serial]
    fn test_resolve_socket_xdg() {
        clear_env();
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        assert_eq!(
            resolve_socket_path(None),
            PathBuf::from("/run/user/1000/biomeos/sweetgrass.sock")
        );
    }

    #[test]
    #[serial]
    fn test_resolve_socket_fallback() {
        clear_env();
        std::env::remove_var("USER");
        let path = resolve_socket_path(None);
        let expected = std::env::temp_dir().join("sweetgrass.sock");
        assert_eq!(path, expected);
    }

    #[test]
    #[serial]
    fn test_resolve_socket_user_fallback() {
        clear_env();
        std::env::set_var("USER", "testuser");
        let path = resolve_socket_path(None);
        let expected = std::env::temp_dir()
            .join("biomeos-testuser")
            .join("sweetgrass.sock");
        assert_eq!(path, expected);
    }

    #[test]
    #[serial]
    fn test_resolve_socket_priority_explicit_overrides_all() {
        clear_env();
        std::env::set_var("SWEETGRASS_SOCKET", "/absolute/custom.sock");
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        std::env::set_var("USER", "testuser");

        let path = resolve_socket_path(None);
        assert_eq!(path, PathBuf::from("/absolute/custom.sock"));
    }

    #[test]
    #[serial]
    fn test_resolve_socket_family_id_in_fallback() {
        clear_env();
        std::env::set_var("BIOMEOS_FAMILY_ID", "beta");
        std::env::remove_var("USER");
        let path = resolve_socket_path(None);
        let expected = std::env::temp_dir().join("sweetgrass-beta.sock");
        assert_eq!(path, expected);
    }

    #[test]
    #[serial]
    fn test_resolve_socket_from_self_knowledge() {
        clear_env();
        std::env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");
        // Primal name from SelfKnowledge overrides default
        let path = resolve_socket_path(Some("sweetgrass-prod"));
        assert_eq!(path, PathBuf::from("/run/biomeos/sweetgrass-prod.sock"));
    }

    #[test]
    #[serial]
    fn test_cleanup_socket_nonexistent() {
        clear_env();
        std::env::set_var("SWEETGRASS_SOCKET", "/nonexistent/path/socket.sock");
        // Should not panic when socket doesn't exist
        cleanup_socket();
    }

    #[test]
    #[serial]
    fn test_cleanup_socket_when_exists() {
        clear_env();
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cleanup-test.sock");
        std::env::set_var("SWEETGRASS_SOCKET", sock_path.to_str().unwrap());
        std::fs::write(&sock_path, "").expect("create socket file");
        assert!(sock_path.exists());
        cleanup_socket();
        // Socket should be removed
        assert!(!sock_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_uds_roundtrip() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("test-sweetgrass.sock");
        std::env::set_var("SWEETGRASS_SOCKET", sock_path.to_str().unwrap());

        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

        let state_clone = state.clone();
        let listener_handle = tokio::spawn(async move {
            let _ = start_uds_listener(state_clone).await;
        });

        // Give the listener time to bind
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
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["status"], "healthy");

        listener_handle.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_uds_parse_error_returns_jsonrpc_error() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("parse-error-test.sock");
        std::env::set_var("SWEETGRASS_SOCKET", sock_path.to_str().unwrap());

        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

        let state_clone = state.clone();
        let listener_handle = tokio::spawn(async move {
            let _ = start_uds_listener(state_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let stream = tokio::net::UnixStream::connect(&sock_path)
            .await
            .expect("connect");
        let (reader, mut writer) = stream.into_split();

        // Send malformed JSON
        writer
            .write_all(b"{ invalid json }\n")
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

        listener_handle.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_uds_empty_lines_skipped() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("empty-lines-test.sock");
        std::env::set_var("SWEETGRASS_SOCKET", sock_path.to_str().unwrap());

        let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

        let state_clone = state.clone();
        let listener_handle = tokio::spawn(async move {
            let _ = start_uds_listener(state_clone).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let stream = tokio::net::UnixStream::connect(&sock_path)
            .await
            .expect("connect");
        let (reader, mut writer) = stream.into_split();

        // Send empty line then valid request
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
        // First line might be empty or we get response - skip empties in handler
        let response_line = lines.next_line().await.unwrap().expect("response");
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());

        listener_handle.abort();
    }
}
