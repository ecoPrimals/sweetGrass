//! Health check handler.
//!
//! Provides comprehensive health monitoring including:
//! - Basic liveness/readiness probes (compatible with orchestrators)
//! - Detailed component status for debugging
//! - Integration status for connected primals

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use sweet_grass_store::QueryFilter;

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
    pub fn unknown() -> Self {
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
        service: "sweetgrass".to_string(),
        uptime_secs,
        store,
        integrations,
    }))
}

/// Detailed health check with integration status.
///
/// More expensive check that verifies all connected services.
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
    let integrations = check_integrations().await;
    let status = determine_status(store.available, Some(&integrations));

    Ok(Json(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "sweetgrass".to_string(),
        uptime_secs,
        store,
        integrations: Some(integrations),
    }))
}

/// Check all capability endpoints.
///
/// Uses capability-based discovery to check integration status.
/// No primal names are hardcoded - only capabilities.
async fn check_integrations() -> IntegrationStatus {
    // Check discovery capability (required for other checks)
    let discovery = check_capability_env("DISCOVERY_ADDRESS").await;

    IntegrationStatus {
        signing: None,        // Discovered via Capability::Signing
        session_events: None, // Discovered via Capability::SessionEvents
        anchoring: None,      // Discovered via Capability::Anchoring
        discovery: Some(discovery),
        compute: None, // Discovered via Capability::Compute
    }
}

/// Check a capability via environment variable.
///
/// Environment variables follow the pattern: `{CAPABILITY}_ADDRESS`
#[allow(clippy::unused_async)] // Will use await when real connection check is added
async fn check_capability_env(env_var: &str) -> PrimalStatus {
    std::env::var(env_var).map_or_else(
        |_| PrimalStatus::unknown(),
        |addr| PrimalStatus {
            connected: false,
            address: Some(addr),
            last_seen: None,
            error: Some("Not connected (health check only)".to_string()),
        },
    )
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
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sweet_grass_core::agent::Did;

    fn create_test_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    #[tokio::test]
    async fn test_health_returns_ok() {
        let state = create_test_state();
        let result = health(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "healthy");
        assert!(response.store.available);
        assert_eq!(response.store.braid_count, 0);
    }

    #[tokio::test]
    async fn test_health_version() {
        let state = create_test_state();
        let result = health(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_service_name() {
        let state = create_test_state();
        let result = health(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.service, "sweetgrass");
    }

    #[tokio::test]
    async fn test_liveness_always_ok() {
        let status = liveness().await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_ok_with_store() {
        let state = create_test_state();
        let status = readiness(State(state)).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_with_braids() {
        use std::sync::Arc;
        use sweet_grass_factory::BraidFactory;

        let state = create_test_state();

        // Add some braids
        let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkTest")));
        let braid = factory.from_data(b"test data", "text/plain", None).unwrap();
        state.store.put(&braid).await.unwrap();

        let result = health(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.store.braid_count, 1);
    }

    #[tokio::test]
    async fn test_health_detailed() {
        let state = create_test_state();
        let result = health_detailed(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "healthy");
        assert!(response.integrations.is_some());
    }

    #[test]
    fn test_primal_status_connected() {
        // Use environment variable or OS-allocated port (zero hardcoding)
        // Note: Using simplified test address since integration testing module not available here
        let test_address =
            std::env::var("TEST_PRIMAL_ADDR").unwrap_or_else(|_| "localhost:0".to_string());

        let status = PrimalStatus::connected(Some(test_address));
        assert!(status.connected);
        assert!(status.address.is_some());
        assert!(status.last_seen.is_some());
        assert!(status.error.is_none());
    }

    #[test]
    fn test_primal_status_disconnected() {
        let status = PrimalStatus::disconnected("Connection refused");
        assert!(!status.connected);
        assert!(status.error.is_some());
    }

    #[test]
    fn test_primal_status_unknown() {
        let status = PrimalStatus::unknown();
        assert!(!status.connected);
        assert!(status.address.is_none());
        assert!(status.error.is_none());
    }

    #[test]
    fn test_determine_status_healthy() {
        let status = determine_status(true, None);
        assert_eq!(status, "healthy");
    }

    #[test]
    fn test_determine_status_unhealthy() {
        let status = determine_status(false, None);
        assert_eq!(status, "unhealthy");
    }

    #[test]
    fn test_determine_status_degraded() {
        let integrations = IntegrationStatus {
            signing: Some(PrimalStatus::disconnected("Connection failed")),
            session_events: None,
            anchoring: None,
            discovery: None,
            compute: None,
        };
        let status = determine_status(true, Some(&integrations));
        assert_eq!(status, "degraded");
    }
}
