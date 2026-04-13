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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::debug;

use crate::state::AppState;

use super::{DispatchResult, to_value};

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Default fallback socket directory when no env overrides exist.
const DEFAULT_SOCKET_DIR: &str = "/tmp/biomeos";

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
/// → `/tmp/biomeos/{domain}.sock`.
fn discover_capability_socket_with_reader(
    domain: &str,
    reader: &impl Fn(&str) -> Option<String>,
) -> PathBuf {
    let sock_name = format!("{domain}.sock");

    if let Some(dir) = reader("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(&sock_name);
    }

    if let Some(xdg) = reader("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg).join("biomeos").join(&sock_name);
    }

    PathBuf::from(DEFAULT_SOCKET_DIR).join(&sock_name)
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
