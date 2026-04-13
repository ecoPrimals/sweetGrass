// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Composition health handlers per `wateringHole/COMPOSITION_HEALTH_STANDARD.md`.
//!
//! Each tier probes upstream capability sockets to report subsystem health:
//! - **tower**: `BearDog` (security) + `Songbird` (discovery)
//! - **node**: tower + `ToadStool` (compute)
//! - **nest**: tower + `NestGate` (storage)
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

/// Probe a capability socket with `health.liveness`.
///
/// Returns `"ok"` if the socket responds, `"degraded"` on timeout/error,
/// or `"unavailable"` if the socket doesn't exist.
async fn probe_capability(domain: &str) -> &'static str {
    let socket = discover_capability_socket(domain);
    if !socket.exists() {
        return "unavailable";
    }

    let result = tokio::time::timeout(PROBE_TIMEOUT, async {
        let stream = UnixStream::connect(&socket).await?;
        let (reader, mut writer) = stream.into_split();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.liveness",
            "params": {},
            "id": 1,
        });
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        writer.write_all(line.as_bytes()).await?;
        writer.flush().await?;

        let mut buf = BufReader::new(reader);
        let mut response = String::new();
        buf.read_line(&mut response).await?;

        let parsed: serde_json::Value = serde_json::from_str(&response)?;
        if parsed.get("result").is_some() {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        } else {
            Err("no result in response".into())
        }
    })
    .await;

    match result {
        Ok(Ok(())) => "ok",
        _ => "degraded",
    }
}

/// Discover a capability socket following ecosystem conventions.
///
/// Resolution: `{BIOMEOS_SOCKET_DIR}/{domain}.sock` → `{XDG_RUNTIME_DIR}/biomeos/{domain}.sock`
/// → `/tmp/biomeos/{domain}.sock`.
fn discover_capability_socket(domain: &str) -> PathBuf {
    let sock_name = format!("{domain}.sock");

    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(&sock_name);
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg).join("biomeos").join(&sock_name);
    }

    PathBuf::from("/tmp/biomeos").join(&sock_name)
}

/// `composition.tower_health` — `BearDog` + `Songbird`.
pub(super) async fn handle_tower_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let beardog = probe_capability("security").await;
    let songbird = probe_capability("discovery").await;

    let healthy = beardog == "ok" && songbird == "ok";
    debug!(beardog, songbird, healthy, "composition.tower_health");

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "tower",
        "subsystems": {
            "security": beardog,
            "discovery": songbird,
        },
    }))
}

/// `composition.node_health` — Tower + `ToadStool`.
pub(super) async fn handle_node_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let beardog = probe_capability("security").await;
    let songbird = probe_capability("discovery").await;
    let toadstool = probe_capability("compute").await;

    let healthy = beardog == "ok" && songbird == "ok" && toadstool == "ok";
    debug!(
        beardog,
        songbird, toadstool, healthy, "composition.node_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "node",
        "subsystems": {
            "security": beardog,
            "discovery": songbird,
            "compute": toadstool,
        },
    }))
}

/// `composition.nest_health` — Tower + `NestGate`.
pub(super) async fn handle_nest_health(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let beardog = probe_capability("security").await;
    let songbird = probe_capability("discovery").await;
    let nestgate = probe_capability("storage").await;

    let healthy = beardog == "ok" && songbird == "ok" && nestgate == "ok";
    debug!(
        beardog,
        songbird, nestgate, healthy, "composition.nest_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "nest",
        "subsystems": {
            "security": beardog,
            "discovery": songbird,
            "storage": nestgate,
        },
    }))
}

/// `composition.nucleus_health` — All subsystems including provenance trio.
pub(super) async fn handle_nucleus_health(
    state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let beardog = probe_capability("security").await;
    let songbird = probe_capability("discovery").await;
    let toadstool = probe_capability("compute").await;
    let nestgate = probe_capability("storage").await;
    let rhizocrypt = probe_capability("provenance").await;
    let loamspine = probe_capability("ledger").await;

    let self_healthy = state
        .store
        .count(&sweet_grass_store::QueryFilter::default())
        .await
        .is_ok();
    let sweetgrass = if self_healthy { "ok" } else { "degraded" };

    let healthy = beardog == "ok"
        && songbird == "ok"
        && toadstool == "ok"
        && nestgate == "ok"
        && rhizocrypt == "ok"
        && loamspine == "ok"
        && self_healthy;

    debug!(
        beardog,
        songbird,
        toadstool,
        nestgate,
        rhizocrypt,
        loamspine,
        sweetgrass,
        healthy,
        "composition.nucleus_health"
    );

    to_value(&json!({
        "healthy": healthy,
        "deploy_graph": "nucleus",
        "subsystems": {
            "security": beardog,
            "discovery": songbird,
            "compute": toadstool,
            "storage": nestgate,
            "provenance": rhizocrypt,
            "ledger": loamspine,
            "attribution": sweetgrass,
        },
    }))
}
