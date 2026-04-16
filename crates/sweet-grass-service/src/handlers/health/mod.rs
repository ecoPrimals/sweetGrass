// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Health check handler.
//!
//! Provides comprehensive health monitoring including:
//! - Basic liveness/readiness probes (compatible with orchestrators)
//! - Detailed component status for debugging
//! - Integration status for connected primals

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sweet_grass_store::{BraidStore, QueryFilter};

use sweet_grass_core::identity;

use crate::state::AppState;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Service status: "healthy", "degraded", or "unhealthy".
    pub status: String,

    /// Service version.
    pub version: String,

    /// Service name.
    pub service: String,

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
    pub backend: String,
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
fn determine_status(store_available: bool, integrations: Option<&IntegrationStatus>) -> String {
    if !store_available {
        return "unhealthy".to_string();
    }

    // Check if any capabilities are configured but failing
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
            return "degraded".to_string();
        }
    }

    "healthy".to_string()
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
        backend: state.store_backend.clone(),
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
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: identity::PRIMAL_NAME.to_string(),
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
        backend: state.store_backend.clone(),
    };

    // Calculate uptime if self-knowledge is available
    let uptime_secs = state
        .self_knowledge
        .as_ref()
        .map(|sk| sk.uptime().as_secs());

    // Check integrations if discovery is available
    let integrations = check_integrations();
    let status = determine_status(store.available, Some(&integrations));

    Ok(Json(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: identity::PRIMAL_NAME.to_string(),
        uptime_secs,
        store,
        integrations: Some(integrations),
    }))
}

/// Check all capability endpoints.
///
/// Uses capability-based discovery to check integration status.
/// No primal names are hardcoded - only capabilities.
fn check_integrations() -> IntegrationStatus {
    check_integrations_with_reader(|key| std::env::var(key).ok())
}

/// DI-friendly integration checker. Tests inject a reader
/// instead of mutating process-global environment variables.
fn check_integrations_with_reader(reader: impl Fn(&str) -> Option<String>) -> IntegrationStatus {
    let discovery = check_capability(&reader, "DISCOVERY_ADDRESS");

    IntegrationStatus {
        signing: None,        // Discovered via Capability::Signing
        session_events: None, // Discovered via Capability::SessionEvents
        anchoring: None,      // Discovered via Capability::Anchoring
        discovery: Some(discovery),
        compute: None, // Discovered via Capability::Compute
    }
}

/// Check a capability via a key reader.
///
/// Environment variables follow the pattern: `{CAPABILITY}_ADDRESS`
fn check_capability(reader: &impl Fn(&str) -> Option<String>, env_var: &str) -> PrimalStatus {
    reader(env_var).map_or_else(PrimalStatus::unknown, |addr| PrimalStatus {
        connected: false,
        address: Some(addr),
        last_seen: None,
        error: Some("Not connected (health check only)".to_string()),
    })
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
