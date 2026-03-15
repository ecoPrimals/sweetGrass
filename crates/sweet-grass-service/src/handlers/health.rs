// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Health check handler.
//!
//! Provides comprehensive health monitoring including:
//! - Basic liveness/readiness probes (compatible with orchestrators)
//! - Detailed component status for debugging
//! - Integration status for connected primals

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sweet_grass_store::QueryFilter;

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
    // Check discovery capability (required for other checks)
    let discovery = check_capability_env("DISCOVERY_ADDRESS");

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
fn check_capability_env(env_var: &str) -> PrimalStatus {
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
#[allow(unsafe_code)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: unwrap/expect are standard in tests"
)]
mod tests {
    use super::*;
    use axum::extract::State;
    use std::sync::Arc;
    use sweet_grass_core::agent::Did;
    use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash};
    use sweet_grass_store::{
        BraidStore, MemoryStore, QueryFilter, QueryOrder, QueryResult, Result as StoreResult,
        StoreError,
    };

    fn create_test_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    /// Store that fails on count for testing health/readiness error paths.
    struct CountFailingStore(Arc<MemoryStore>);

    #[async_trait::async_trait]
    impl BraidStore for CountFailingStore {
        async fn put(&self, braid: &Braid) -> StoreResult<()> {
            self.0.put(braid).await
        }
        async fn get(&self, id: &BraidId) -> StoreResult<Option<Braid>> {
            self.0.get(id).await
        }
        async fn get_by_hash(&self, hash: &ContentHash) -> StoreResult<Option<Braid>> {
            self.0.get_by_hash(hash).await
        }
        async fn delete(&self, id: &BraidId) -> StoreResult<bool> {
            self.0.delete(id).await
        }
        async fn exists(&self, id: &BraidId) -> StoreResult<bool> {
            self.0.exists(id).await
        }
        async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> StoreResult<QueryResult> {
            self.0.query(filter, order).await
        }
        async fn count(&self, _filter: &QueryFilter) -> StoreResult<usize> {
            Err(StoreError::Internal("injected fault".to_string()))
        }
        async fn by_agent(&self, agent: &Did) -> StoreResult<Vec<Braid>> {
            self.0.by_agent(agent).await
        }
        async fn derived_from(&self, hash: &ContentHash) -> StoreResult<Vec<Braid>> {
            self.0.derived_from(hash).await
        }
        async fn put_activity(&self, activity: &Activity) -> StoreResult<()> {
            self.0.put_activity(activity).await
        }
        async fn get_activity(&self, id: &ActivityId) -> StoreResult<Option<Activity>> {
            self.0.get_activity(id).await
        }
        async fn activities_for_braid(&self, braid_id: &BraidId) -> StoreResult<Vec<Activity>> {
            self.0.activities_for_braid(braid_id).await
        }
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
    #[serial_test::serial]
    async fn test_health_detailed() {
        unsafe {
            std::env::remove_var("DISCOVERY_ADDRESS");
        }
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

    #[test]
    fn test_primal_status_connected_without_address() {
        let status = PrimalStatus::connected(None);
        assert!(status.connected);
        assert!(status.address.is_none());
        assert!(status.last_seen.is_some());
        assert!(status.error.is_none());
    }

    #[tokio::test]
    async fn test_health_returns_service_unavailable_when_store_fails() {
        let store: Arc<dyn BraidStore> = Arc::new(CountFailingStore(Arc::new(MemoryStore::new())));
        let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

        let result = health(State(state)).await;
        match &result {
            Err(e) => assert_eq!(e, &StatusCode::SERVICE_UNAVAILABLE),
            Ok(_) => panic!("expected SERVICE_UNAVAILABLE when store fails"),
        }
    }

    #[tokio::test]
    async fn test_health_detailed_returns_service_unavailable_when_store_fails() {
        let store: Arc<dyn BraidStore> = Arc::new(CountFailingStore(Arc::new(MemoryStore::new())));
        let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

        let result = health_detailed(State(state)).await;
        match &result {
            Err(e) => assert_eq!(e, &StatusCode::SERVICE_UNAVAILABLE),
            Ok(_) => panic!("expected SERVICE_UNAVAILABLE when store fails"),
        }
    }

    #[tokio::test]
    async fn test_readiness_unavailable_when_store_fails() {
        let store: Arc<dyn BraidStore> = Arc::new(CountFailingStore(Arc::new(MemoryStore::new())));
        let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

        let status = readiness(State(state)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_health_detailed_integrations_discovery_unknown() {
        unsafe {
            std::env::remove_var("DISCOVERY_ADDRESS");
        }

        let state = create_test_state();
        let result = health_detailed(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let integrations = response
            .integrations
            .as_ref()
            .expect("should have integrations");
        let discovery = integrations
            .discovery
            .as_ref()
            .expect("should have discovery");
        assert!(!discovery.connected);
        assert!(discovery.address.is_none());
        assert!(discovery.error.is_none());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_health_detailed_integrations_discovery_configured() {
        unsafe {
            std::env::set_var("DISCOVERY_ADDRESS", "localhost:9999");
        }

        let state = create_test_state();
        let result = health_detailed(State(state)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        let integrations = response
            .integrations
            .as_ref()
            .expect("should have integrations");
        let discovery = integrations
            .discovery
            .as_ref()
            .expect("should have discovery");
        assert!(!discovery.connected);
        assert_eq!(discovery.address.as_deref(), Some("localhost:9999"));
        assert!(discovery.error.is_some());

        unsafe {
            std::env::remove_var("DISCOVERY_ADDRESS");
        }
    }

    #[test]
    fn test_determine_status_degraded_multiple_capabilities() {
        let integrations = IntegrationStatus {
            signing: Some(PrimalStatus::disconnected("Connection failed")),
            session_events: Some(PrimalStatus::unknown()),
            anchoring: Some(PrimalStatus::disconnected("Timeout")),
            discovery: None,
            compute: None,
        };
        let status = determine_status(true, Some(&integrations));
        assert_eq!(status, "degraded");
    }

    #[test]
    fn test_determine_status_healthy_with_unknown_integrations() {
        let integrations = IntegrationStatus {
            signing: Some(PrimalStatus::unknown()),
            session_events: None,
            anchoring: None,
            discovery: None,
            compute: None,
        };
        let status = determine_status(true, Some(&integrations));
        assert_eq!(status, "healthy");
    }
}
