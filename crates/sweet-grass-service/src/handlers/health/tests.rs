// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test file: unwrap/expect are standard in tests"
)]

use super::*;
use axum::extract::State;
use std::sync::Arc;
use sweet_grass_core::agent::Did;
use sweet_grass_store::{BraidStore, MemoryStore};

use crate::backend::{BraidBackend, CountFailingStore};

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
    let store = Arc::new(BraidBackend::CountFailing(CountFailingStore(Arc::new(
        MemoryStore::new(),
    ))));
    let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

    let result = health(State(state)).await;
    match &result {
        Err(e) => assert_eq!(e, &StatusCode::SERVICE_UNAVAILABLE),
        Ok(_) => panic!("expected SERVICE_UNAVAILABLE when store fails"),
    }
}

#[tokio::test]
async fn test_health_detailed_returns_service_unavailable_when_store_fails() {
    let store = Arc::new(BraidBackend::CountFailing(CountFailingStore(Arc::new(
        MemoryStore::new(),
    ))));
    let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

    let result = health_detailed(State(state)).await;
    match &result {
        Err(e) => assert_eq!(e, &StatusCode::SERVICE_UNAVAILABLE),
        Ok(_) => panic!("expected SERVICE_UNAVAILABLE when store fails"),
    }
}

#[tokio::test]
async fn test_readiness_unavailable_when_store_fails() {
    let store = Arc::new(BraidBackend::CountFailing(CountFailingStore(Arc::new(
        MemoryStore::new(),
    ))));
    let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));

    let status = readiness(State(state)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn test_integrations_discovery_unknown_via_reader() {
    let integrations = check_integrations_with_reader(|_| None);
    let discovery = integrations
        .discovery
        .as_ref()
        .expect("should have discovery");
    assert!(!discovery.connected);
    assert!(discovery.address.is_none());
    assert!(discovery.error.is_none());
}

#[test]
fn test_integrations_discovery_configured_via_reader() {
    let integrations = check_integrations_with_reader(|key| {
        (key == "DISCOVERY_ADDRESS").then(|| "localhost:9999".to_string())
    });
    let discovery = integrations
        .discovery
        .as_ref()
        .expect("should have discovery");
    assert!(!discovery.connected);
    assert_eq!(discovery.address.as_deref(), Some("localhost:9999"));
    assert!(discovery.error.is_some());
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
