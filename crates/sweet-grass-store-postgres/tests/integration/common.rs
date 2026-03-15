// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Common test utilities for `PostgreSQL` integration tests.
//!
//! Note: Some helpers may be unused during incremental test refactoring.
//! They are kept for future test modules being migrated.

#![expect(
    dead_code,
    reason = "test helpers used by multiple integration test files"
)]

use sweet_grass_core::{
    activity::{Activity, ActivityType},
    agent::{AgentAssociation, AgentRole, Did},
    braid::BraidMetadata,
    Braid,
};
use sweet_grass_integration::testing::postgres_test_url_for_port;
use sweet_grass_store_postgres::{PostgresConfig, PostgresStore};
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;

/// Helper to spin up a `PostgreSQL` container and return a connected store.
///
/// This uses `testcontainers` to start a real `PostgreSQL` instance with:
/// - Default postgres user/password
/// - Temporary storage (cleaned up after tests)
/// - Dynamic port allocation (no conflicts)
pub async fn setup_postgres() -> (ContainerAsync<Postgres>, PostgresStore) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get PostgreSQL port");

    let connection_string = postgres_test_url_for_port(host_port);

    let config = PostgresConfig::new(&connection_string)
        .max_connections(5)
        .min_connections(1);

    let store = PostgresStore::connect(&config)
        .await
        .expect("Failed to connect to PostgreSQL");

    store
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    (container, store)
}

/// Create a minimal test braid with the given hash suffix.
///
/// Useful for basic CRUD and relationship testing.
pub fn create_test_braid(hash_suffix: &str) -> Braid {
    Braid::builder()
        .data_hash(format!("sha256:{hash_suffix}"))
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTestAgent"))
        .build()
        .expect("Failed to create test braid")
}

/// Create a test braid with full metadata and tags.
///
/// Useful for query filtering and metadata testing.
pub fn create_braid_with_metadata(hash_suffix: &str, tags: Vec<&str>) -> Braid {
    let metadata = BraidMetadata {
        title: Some(format!("Test Braid {hash_suffix}")),
        description: Some("A test braid for integration testing".to_string()),
        tags: tags.into_iter().map(String::from).collect(),
        ..Default::default()
    };

    Braid::builder()
        .data_hash(format!("sha256:{hash_suffix}"))
        .mime_type("application/json")
        .size(256)
        .attributed_to(Did::new("did:key:z6MkTestAgent"))
        .metadata(metadata)
        .build()
        .expect("Failed to create test braid")
}

/// Create a minimal test activity.
///
/// Useful for activity storage and relationship testing.
pub fn create_test_activity() -> Activity {
    Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkTestAgent"),
            AgentRole::Creator,
        ))
        .compute_units(1.5)
        .build()
}
