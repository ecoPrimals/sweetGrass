// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Health check handler.
//!
//! Provides comprehensive health monitoring including:
//! - Basic liveness/readiness probes (compatible with orchestrators)
//! - Detailed component status for debugging
//! - Integration status for connected primals

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sweet_grass_store::{BraidStore, QueryFilter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use sweet_grass_core::identity;

use crate::state::AppState;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Service status: "healthy", "degraded", or "unhealthy".
    pub status: &'static str,

    /// Service version.
    pub version: &'static str,

    /// Service name.
    pub service: &'static str,

    /// Uptime in seconds (if tracked).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,

    /// Store status.
    pub store: StoreStatus,

    /// Integration status for connected primals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrations: Option<IntegrationStatus>,
}

/// Store status.
#[derive(Serialize)]
pub struct StoreStatus {
    /// Whether the store is available.
    pub available: bool,

    /// Number of Braids in the store.
    pub braid_count: usize,

    /// Store backend type.
    pub backend: &'static str,
}

/// Integration status for connected capabilities.
///
/// Uses capability names instead of primal names for zero-knowledge architecture.
/// Primals are discovered at runtime based on what they can do, not who they are.
#[derive(Clone, Serialize, Default)]
pub struct IntegrationStatus {
    /// Signing capability status (identity, signatures).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing: Option<PrimalStatus>,

    /// Session events capability status (activity tracking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_events: Option<PrimalStatus>,

    /// Anchoring capability status (permanent storage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchoring: Option<PrimalStatus>,

    /// Discovery capability status (mesh discovery).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery: Option<PrimalStatus>,

    /// Compute capability status (task execution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute: Option<PrimalStatus>,
}

/// Status of a connected primal.
#[derive(Serialize, Clone)]
pub struct PrimalStatus {
    /// Whether the primal is connected.
    pub connected: bool,

    /// Primal address if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Last successful health check time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<String>,

    /// Error message if unhealthy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PrimalStatus {
    /// Create a connected status.
    #[must_use]
    pub fn connected(address: Option<String>) -> Self {
        Self {
            connected: true,
            address,
            last_seen: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        }
    }

    /// Create a disconnected status with error.
    #[must_use]
    pub fn disconnected(error: impl Into<String>) -> Self {
        Self {
            connected: false,
            address: None,
            last_seen: None,
            error: Some(error.into()),
        }
    }

    /// Create an unknown status (not configured).
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            connected: false,
            address: None,
            last_seen: None,
            error: None,
        }
    }
}

/// Determine overall status from component statuses.
fn determine_status(
    store_available: bool,
    integrations: Option<&IntegrationStatus>,
) -> &'static str {
    if !store_available {
        return "unhealthy";
    }

    if let Some(int) = integrations {
        let capabilities = [
            &int.signing,
            &int.session_events,
            &int.anchoring,
            &int.discovery,
            &int.compute,
        ];
        let has_failures = capabilities.iter().any(|p| {
            p.as_ref()
                .is_some_and(|s| !s.connected && s.error.is_some())
        });

        if has_failures {
            return "degraded";
        }
    }

    "healthy"
}

/// Health check endpoint.
///
/// Returns comprehensive health status including store and integration status.
///
/// # Errors
///
/// Returns `SERVICE_UNAVAILABLE` if the store count query fails.
pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let braid_count = state
        .store
        .count(&QueryFilter::default())
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let store = StoreStatus {
        available: true,
        braid_count,
        backend: state.store_backend,
    };

    // Calculate uptime if self-knowledge is available
    let uptime_secs = state
        .self_knowledge
        .as_ref()
        .map(|sk| sk.uptime().as_secs());

    // For now, integrations are not tracked - would need state extension
    let integrations: Option<IntegrationStatus> = None;
    let status = determine_status(store.available, integrations.as_ref());

    Ok(Json(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION"),
        service: identity::PRIMAL_NAME,
        uptime_secs,
        store,
        integrations,
    }))
}

/// Detailed health check with integration status.
///
/// More expensive check that verifies all connected services.
///
/// # Errors
///
/// Returns `SERVICE_UNAVAILABLE` if the store count query fails.
pub async fn health_detailed(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let braid_count = state
        .store
        .count(&QueryFilter::default())
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let store = StoreStatus {
        available: true,
        braid_count,
        backend: state.store_backend,
    };

    // Calculate uptime if self-knowledge is available
    let uptime_secs = state
        .self_knowledge
        .as_ref()
        .map(|sk| sk.uptime().as_secs());

    let integrations = check_integrations(&state).await;
    let status = determine_status(store.available, Some(&integrations));

    Ok(Json(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION"),
        service: identity::PRIMAL_NAME,
        uptime_secs,
        store,
        integrations: Some(integrations),
    }))
}

/// Check all capability endpoints using snapshotted state.
///
/// Uses capability-based discovery to check integration status.
/// No primal names are hardcoded — only capabilities.
async fn check_integrations(state: &AppState) -> IntegrationStatus {
    let dir = &state.socket_dir;
    let (signing, anchoring, discovery, compute) = tokio::join!(
        probe_integration("security", dir),
        probe_integration("provenance", dir),
        probe_integration("discovery", dir),
        probe_integration("compute", dir),
    );

    IntegrationStatus {
        signing: Some(signing),
        session_events: None,
        anchoring: Some(anchoring),
        discovery: Some(discovery),
        compute: Some(compute),
    }
}

async fn probe_integration(domain: &str, socket_dir: &std::path::Path) -> PrimalStatus {
    let socket = socket_dir.join(format!("{domain}.sock"));
    let address = socket.to_string_lossy().into_owned();

    if !socket.exists() {
        return PrimalStatus {
            connected: false,
            address: Some(address.clone()),
            last_seen: None,
            error: Some(format!("socket not found: {address}")),
        };
    }

    match tokio::time::timeout(PROBE_TIMEOUT, try_liveness_probe(&socket)).await {
        Ok(Ok(())) => PrimalStatus::connected(Some(address)),
        Ok(Err(e)) => PrimalStatus {
            connected: false,
            address: Some(address),
            last_seen: None,
            error: Some(e.to_string()),
        },
        Err(_) => PrimalStatus {
            connected: false,
            address: Some(address),
            last_seen: None,
            error: Some("liveness probe timed out".to_string()),
        },
    }
}

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

/// Liveness probe.
pub async fn liveness() -> StatusCode {
    StatusCode::OK
}

/// Readiness probe.
pub async fn readiness(State(state): State<AppState>) -> StatusCode {
    match state.store.count(&QueryFilter::default()).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests;
