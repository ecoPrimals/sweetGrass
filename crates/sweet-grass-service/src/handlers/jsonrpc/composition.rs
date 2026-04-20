// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Composition health handlers per `wateringHole/COMPOSITION_HEALTH_STANDARD.md`.
//!
//! Each tier probes upstream capability sockets to report subsystem health:
//! - **tower**: security + discovery
//! - **node**: tower + compute
//! - **nest**: tower + storage
//! - **nucleus**: all subsystems including provenance trio

use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;
use sweet_grass_store::BraidStore;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;

use sweet_grass_core::primal_names::{env_vars, paths};

use crate::state::AppState;

use super::{DispatchResult, to_value};

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Resolve the `biomeOS` socket directory from environment, following
/// the ecosystem standard fallback chain.
///
/// 1. `BIOMEOS_SOCKET_DIR`
/// 2. `{XDG_RUNTIME_DIR}/biomeos`
/// 3. `{TMPDIR}/biomeos`  (platform-agnostic temp fallback)
/// 4. `/tmp/biomeos`      (last resort — POSIX only)
fn resolve_socket_dir(reader: &impl Fn(&str) -> Option<String>) -> PathBuf {
    if let Some(dir) = reader(env_vars::BIOMEOS_SOCKET_DIR) {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = reader(env_vars::XDG_RUNTIME_DIR) {
        return PathBuf::from(xdg).join(paths::BIOMEOS_DIR);
    }
    if let Some(tmpdir) = reader("TMPDIR") {
        return PathBuf::from(tmpdir).join(paths::BIOMEOS_DIR);
    }
    PathBuf::from(paths::DEFAULT_SOCKET_DIR)
}

/// Probe a capability socket with `health.liveness`.
///
/// Returns `"ok"` if the socket responds, `"degraded"` on timeout/error,
/// or `"unavailable"` if the socket doesn't exist.
async fn probe_capability(domain: &str) -> &'static str {
    probe_capability_with_reader(domain, &|key| std::env::var(key).ok()).await
}

/// DI-friendly probe for testing without real env vars.
async fn probe_capability_with_reader(
    domain: &str,
    reader: &(impl Fn(&str) -> Option<String> + Sync),
) -> &'static str {
    let socket = discover_capability_socket_with_reader(domain, reader);
    if !socket.exists() {
        return "unavailable";
    }

    let result = tokio::time::timeout(PROBE_TIMEOUT, try_liveness_probe(&socket)).await;

    match result {
        Ok(Ok(())) => "ok",
        _ => "degraded",
    }
}

/// Attempt a single `health.liveness` JSON-RPC call over UDS.
async fn try_liveness_probe(socket: &std::path::Path) -> std::io::Result<()> {
    let stream = UnixStream::connect(socket).await?;
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 1,
    });
    let mut line = serde_json::to_string(&request)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    line.push('\n');
    writer.write_all(line.as_bytes()).await?;
    writer.flush().await?;

    let mut buf = BufReader::new(reader);
    let mut response = String::new();
    buf.read_line(&mut response).await?;

    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if parsed.get("result").is_some() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "no result in liveness response",
        ))
    }
}

/// Discover a capability socket following ecosystem conventions.
///
/// Uses the provided reader for env var lookup (DI-friendly).
///
/// Resolution: `{BIOMEOS_SOCKET_DIR}/{domain}.sock` → `{XDG_RUNTIME_DIR}/biomeos/{domain}.sock`
/// → `{TMPDIR}/biomeos/{domain}.sock` → `/tmp/biomeos/{domain}.sock`.
fn discover_capability_socket_with_reader(
    domain: &str,
    reader: &impl Fn(&str) -> Option<String>,
) -> PathBuf {
    let sock_name = format!("{domain}.sock");
    resolve_socket_dir(reader).join(&sock_name)
}

/// `composition.tower_health` — security + discovery.
pub(super) async fn handle_tower_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let security = probe_capability("security").await;
    let discovery = probe_capability("discovery").await;

    let healthy = security == "ok" && discovery == "ok";
    debug!(security, discovery, healthy, "composition.tower_health");

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "tower",
        "subsystems": {
            "security": security,
            "discovery": discovery,
        },
    }))
}

/// `composition.node_health` — tower + compute.
pub(super) async fn handle_node_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let security = probe_capability("security").await;
    let discovery = probe_capability("discovery").await;
    let compute = probe_capability("compute").await;

    let healthy = security == "ok" && discovery == "ok" && compute == "ok";
    debug!(
        security,
        discovery, compute, healthy, "composition.node_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "node",
        "subsystems": {
            "security": security,
            "discovery": discovery,
            "compute": compute,
        },
    }))
}

/// `composition.nest_health` — tower + storage.
pub(super) async fn handle_nest_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let security = probe_capability("security").await;
    let discovery = probe_capability("discovery").await;
    let storage = probe_capability("storage").await;

    let healthy = security == "ok" && discovery == "ok" && storage == "ok";
    debug!(
        security,
        discovery, storage, healthy, "composition.nest_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "nest",
        "subsystems": {
            "security": security,
            "discovery": discovery,
            "storage": storage,
        },
    }))
}

/// `composition.nucleus_health` — all subsystems including provenance trio.
pub(super) async fn handle_nucleus_health(
    state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let security = probe_capability("security").await;
    let discovery = probe_capability("discovery").await;
    let compute = probe_capability("compute").await;
    let storage = probe_capability("storage").await;
    let provenance = probe_capability("provenance").await;
    let ledger = probe_capability("ledger").await;

    let self_healthy = state
        .store
        .count(&sweet_grass_store::QueryFilter::default())
        .await
        .is_ok();
    let attribution = if self_healthy { "ok" } else { "degraded" };

    let healthy = security == "ok"
        && discovery == "ok"
        && compute == "ok"
        && storage == "ok"
        && provenance == "ok"
        && ledger == "ok"
        && self_healthy;

    debug!(
        security,
        discovery,
        compute,
        storage,
        provenance,
        ledger,
        attribution,
        healthy,
        "composition.nucleus_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "nucleus",
        "subsystems": {
            "security": security,
            "discovery": discovery,
            "compute": compute,
            "storage": storage,
            "provenance": provenance,
            "ledger": ledger,
            "attribution": attribution,
        },
    }))
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code: expect is standard in tests")]
mod tests {
    use super::*;

    #[test]
    fn resolve_socket_dir_biomeos_env() {
        let dir = resolve_socket_dir(&|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some("/run/biomeos".to_string()),
            _ => None,
        });
        assert_eq!(dir, PathBuf::from("/run/biomeos"));
    }

    #[test]
    fn resolve_socket_dir_xdg_runtime() {
        let dir = resolve_socket_dir(&|key| match key {
            "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
            _ => None,
        });
        assert_eq!(dir, PathBuf::from("/run/user/1000/biomeos"));
    }

    #[test]
    fn resolve_socket_dir_tmpdir() {
        let dir = resolve_socket_dir(&|key| match key {
            "TMPDIR" => Some("/custom/tmp".to_string()),
            _ => None,
        });
        assert_eq!(dir, PathBuf::from("/custom/tmp/biomeos"));
    }

    #[test]
    fn resolve_socket_dir_fallback() {
        let dir = resolve_socket_dir(&|_| None);
        assert_eq!(dir, PathBuf::from("/tmp/biomeos"));
    }

    #[test]
    fn resolve_socket_dir_priority_order() {
        let dir = resolve_socket_dir(&|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some("/first".to_string()),
            "XDG_RUNTIME_DIR" => Some("/second".to_string()),
            _ => None,
        });
        assert_eq!(
            dir,
            PathBuf::from("/first"),
            "BIOMEOS_SOCKET_DIR should take priority"
        );
    }

    #[test]
    fn discover_capability_socket_uses_domain() {
        let socket = discover_capability_socket_with_reader("security", &|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some("/run/biomeos".to_string()),
            _ => None,
        });
        assert_eq!(socket, PathBuf::from("/run/biomeos/security.sock"));
    }

    #[test]
    fn discover_capability_socket_fallback() {
        let socket = discover_capability_socket_with_reader("discovery", &|_| None);
        assert_eq!(socket, PathBuf::from("/tmp/biomeos/discovery.sock"));
    }

    #[tokio::test]
    async fn probe_capability_unavailable_when_socket_missing() {
        let result = probe_capability_with_reader("nonexistent", &|_| {
            Some("/tmp/sweetgrass-test-noexist".to_string())
        })
        .await;
        assert_eq!(result, "unavailable");
    }

    #[tokio::test]
    async fn probe_capability_ok_with_live_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("testcap.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");

        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut buf = BufReader::new(reader);
                let mut line = String::new();
                let _ = buf.read_line(&mut line).await;
                let response = r#"{"jsonrpc":"2.0","result":{"status":"alive"},"id":1}"#;
                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                let _ = writer.flush().await;
            }
        });

        let dir_str = dir.path().to_string_lossy().to_string();
        let result = probe_capability_with_reader("testcap", &|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
            _ => None,
        })
        .await;
        assert_eq!(result, "ok");

        server_handle.abort();
    }

    #[tokio::test]
    async fn probe_capability_degraded_on_bad_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("badcap.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");

        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut buf = BufReader::new(reader);
                let mut line = String::new();
                let _ = buf.read_line(&mut line).await;
                let response =
                    r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"boom"},"id":1}"#;
                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                let _ = writer.flush().await;
            }
        });

        let dir_str = dir.path().to_string_lossy().to_string();
        let result = probe_capability_with_reader("badcap", &|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
            _ => None,
        })
        .await;
        assert_eq!(result, "degraded");

        server_handle.abort();
    }

    #[tokio::test]
    async fn try_liveness_probe_returns_error_on_missing_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("noresult.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");

        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut buf = BufReader::new(reader);
                let mut line = String::new();
                let _ = buf.read_line(&mut line).await;
                let response =
                    r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"fail"},"id":1}"#;
                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                let _ = writer.flush().await;
            }
        });

        let result = try_liveness_probe(&socket_path).await;
        assert!(result.is_err());

        server_handle.abort();
    }

    fn test_state() -> AppState {
        use sweet_grass_core::agent::Did;
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    #[tokio::test]
    async fn handle_tower_health_returns_structure() {
        let state = test_state();
        let result = handle_tower_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        assert_eq!(val["deploy_graph"], "tower");
        assert!(val.get("healthy").is_some());
        assert!(val["subsystems"]["security"].is_string());
        assert!(val["subsystems"]["discovery"].is_string());
    }

    #[tokio::test]
    async fn handle_node_health_returns_structure() {
        let state = test_state();
        let result = handle_node_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        assert_eq!(val["deploy_graph"], "node");
        assert!(val.get("healthy").is_some());
        assert!(val["subsystems"]["security"].is_string());
        assert!(val["subsystems"]["discovery"].is_string());
        assert!(val["subsystems"]["compute"].is_string());
    }

    #[tokio::test]
    async fn handle_nest_health_returns_structure() {
        let state = test_state();
        let result = handle_nest_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        assert_eq!(val["deploy_graph"], "nest");
        assert!(val.get("healthy").is_some());
        assert!(val["subsystems"]["security"].is_string());
        assert!(val["subsystems"]["discovery"].is_string());
        assert!(val["subsystems"]["storage"].is_string());
    }

    #[tokio::test]
    async fn handle_nucleus_health_returns_structure() {
        let state = test_state();
        let result = handle_nucleus_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        assert_eq!(val["deploy_graph"], "nucleus");
        assert!(val.get("healthy").is_some());
        assert!(val["subsystems"]["security"].is_string());
        assert!(val["subsystems"]["discovery"].is_string());
        assert!(val["subsystems"]["compute"].is_string());
        assert!(val["subsystems"]["storage"].is_string());
        assert!(val["subsystems"]["provenance"].is_string());
        assert!(val["subsystems"]["ledger"].is_string());
        assert!(val["subsystems"]["attribution"].is_string());
    }

    #[tokio::test]
    async fn handle_nucleus_health_attribution_ok_with_healthy_store() {
        let state = test_state();
        let result = handle_nucleus_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        assert_eq!(val["subsystems"]["attribution"], "ok");
    }

    #[tokio::test]
    async fn handle_tower_health_all_unavailable_means_unhealthy() {
        let state = test_state();
        let result = handle_tower_health(&state, serde_json::json!({})).await;
        let val = result.expect("handler should succeed");
        if val["subsystems"]["security"] == "unavailable"
            || val["subsystems"]["discovery"] == "unavailable"
        {
            assert!(!val["healthy"].as_bool().expect("bool"));
        }
    }

    #[tokio::test]
    async fn probe_capability_degraded_on_socket_close() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("closecap.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                drop(stream);
            }
        });

        let dir_str = dir.path().to_string_lossy().to_string();
        let result = probe_capability_with_reader("closecap", &|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
            _ => None,
        })
        .await;
        assert_eq!(result, "degraded");

        server_handle.abort();
    }

    #[tokio::test]
    async fn probe_capability_degraded_on_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("badjson.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
        let server_handle = tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut buf = BufReader::new(reader);
                let mut line = String::new();
                let _ = buf.read_line(&mut line).await;
                let _ = writer.write_all(b"not valid json\n").await;
                let _ = writer.flush().await;
            }
        });

        let dir_str = dir.path().to_string_lossy().to_string();
        let result = probe_capability_with_reader("badjson", &|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
            _ => None,
        })
        .await;
        assert_eq!(result, "degraded");

        server_handle.abort();
    }
}
