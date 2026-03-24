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
use sweet_grass_core::primal_names::env_vars;

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

/// Resolve the Unix domain socket path using XDG-compliant resolution.
///
/// The primal name is derived from `SelfKnowledge` when available (e.g. via
/// `state.self_knowledge`). When `primal_name` is `None`, falls back to
/// `PRIMAL_NAME` env var or `"sweetgrass"`.
///
/// Delegates to [`resolve_socket_path_with`] after reading env vars.
#[must_use]
pub fn resolve_socket_path(primal_name: Option<&str>) -> PathBuf {
    let config = SocketConfig {
        explicit_socket: std::env::var("SWEETGRASS_SOCKET").ok(),
        biomeos_socket_dir: std::env::var(env_vars::BIOMEOS_SOCKET_DIR).ok(),
        family_id: std::env::var(env_vars::BIOMEOS_FAMILY_ID)
            .ok()
            .filter(|s| !s.is_empty()),
        xdg_runtime_dir: std::env::var(env_vars::XDG_RUNTIME_DIR).ok(),
        user: std::env::var("USER").ok(),
        primal_name: primal_name
            .map(String::from)
            .or_else(|| std::env::var("PRIMAL_NAME").ok()),
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
        return PathBuf::from(xdg).join("biomeos").join(&sock_name);
    }

    if let Some(ref user) = config.user {
        return std::env::temp_dir()
            .join(format!("biomeos-{user}"))
            .join(&sock_name);
    }

    std::env::temp_dir().join(&sock_name)
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

    let listener = tokio::net::UnixListener::bind(path)
        .map_err(|e| crate::ServiceError::Internal(format!("UDS bind failed: {e}")))?;
    info!("JSON-RPC 2.0 UDS listening on {}", path.display());

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
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
            _ = shutdown.changed() => {
                info!("UDS listener shutting down");
                break;
            }
        }
    }

    Ok(())
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

        if let Some(response) = crate::handlers::jsonrpc::process_single(&state, request).await {
            let mut resp = serde_json::to_string(&response)?;
            resp.push('\n');
            writer.write_all(resp.as_bytes()).await?;
        }
    }

    Ok(())
}

/// Remove the socket file on shutdown.
pub fn cleanup_socket() {
    let path = resolve_socket_path(None);
    cleanup_socket_at(&path);
}

/// Remove a specific socket file.
pub fn cleanup_socket_at(path: &std::path::Path) {
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
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
    use sweet_grass_core::agent::Did;

    // ==================== DI-based socket resolution tests ====================

    #[test]
    fn di_explicit_socket_override() {
        let config = SocketConfig {
            explicit_socket: Some("/custom/path.sock".to_string()),
            biomeos_socket_dir: Some("/run/biomeos".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/custom/path.sock")
        );
    }

    #[test]
    fn di_biomeos_dir() {
        let config = SocketConfig {
            biomeos_socket_dir: Some("/run/biomeos".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/run/biomeos/sweetgrass.sock")
        );
    }

    #[test]
    fn di_biomeos_dir_with_family() {
        let config = SocketConfig {
            biomeos_socket_dir: Some("/run/biomeos".to_string()),
            family_id: Some("alpha".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/run/biomeos/sweetgrass-alpha.sock")
        );
    }

    #[test]
    fn di_xdg_runtime() {
        let config = SocketConfig {
            xdg_runtime_dir: Some("/run/user/1000".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/run/user/1000/biomeos/sweetgrass.sock")
        );
    }

    #[test]
    fn di_user_fallback() {
        let config = SocketConfig {
            user: Some("testuser".to_string()),
            ..Default::default()
        };
        let expected = std::env::temp_dir()
            .join("biomeos-testuser")
            .join("sweetgrass.sock");
        assert_eq!(resolve_socket_path_with(&config), expected);
    }

    #[test]
    fn di_temp_fallback() {
        let config = SocketConfig::default();
        let expected = std::env::temp_dir().join("sweetgrass.sock");
        assert_eq!(resolve_socket_path_with(&config), expected);
    }

    #[test]
    fn di_custom_primal_name() {
        let config = SocketConfig {
            biomeos_socket_dir: Some("/run/biomeos".to_string()),
            primal_name: Some("sweetgrass-prod".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/run/biomeos/sweetgrass-prod.sock")
        );
    }

    #[test]
    fn di_family_id_in_temp_fallback() {
        let config = SocketConfig {
            family_id: Some("beta".to_string()),
            ..Default::default()
        };
        let expected = std::env::temp_dir().join("sweetgrass-beta.sock");
        assert_eq!(resolve_socket_path_with(&config), expected);
    }

    #[test]
    fn di_priority_explicit_overrides_all() {
        let config = SocketConfig {
            explicit_socket: Some("/absolute/custom.sock".to_string()),
            biomeos_socket_dir: Some("/run/biomeos".to_string()),
            xdg_runtime_dir: Some("/run/user/1000".to_string()),
            user: Some("testuser".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_socket_path_with(&config),
            PathBuf::from("/absolute/custom.sock")
        );
    }

    // ==================== Cleanup tests (use tempdir, no env mutation) ====================

    #[test]
    fn test_cleanup_socket_when_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("cleanup-test.sock");
        std::fs::write(&sock_path, "").expect("create socket file");
        assert!(sock_path.exists());
        cleanup_socket_at(&sock_path);
        assert!(!sock_path.exists());
    }

    #[test]
    fn test_cleanup_socket_nonexistent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("nonexistent.sock");
        cleanup_socket_at(&sock_path);
    }

    // ==================== UDS roundtrip tests (use explicit path, no env mutation) ====================

    #[tokio::test]
    async fn test_uds_roundtrip() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["result"]["status"], "healthy");

        listener_handle.abort();
    }

    #[tokio::test]
    async fn test_uds_parse_error_returns_jsonrpc_error() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
    async fn test_uds_empty_lines_skipped() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());

        listener_handle.abort();
    }
}
