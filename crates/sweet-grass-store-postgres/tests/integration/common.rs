// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Common test utilities for `PostgreSQL` integration tests.
//!
//! Note: Some helpers may be unused during incremental test refactoring.
//! They are kept for future test modules being migrated.

use std::sync::Arc;

use sweet_grass_core::{
    Braid,
    activity::{Activity, ActivityType},
    agent::{AgentAssociation, AgentRole, Did},
    braid::BraidMetadata,
};
use sweet_grass_store_postgres::{PostgresConfig, PostgresStore};

/// Connect to an external `PostgreSQL` instance and return a store.
///
/// Reads `DATABASE_URL` from the environment (set by CI or Docker Compose).
/// Start Postgres externally before running integration tests:
///
/// ```bash
/// docker run --rm -d -p 5432:5432 \
///   -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres \
///   -e POSTGRES_DB=sweetgrass_test --name sg-test-pg postgres:16
/// DATABASE_URL="postgres://postgres:postgres@localhost:5432/sweetgrass_test" \
///   cargo test -p sweet-grass-store-postgres --test integration \
///   --features integration-tests -- --ignored
/// ```
pub async fn setup_postgres() -> PostgresStore {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");

    let config = PostgresConfig::new(&url)
        .max_connections(5)
        .min_connections(1);

    let store = PostgresStore::connect(&config)
        .await
        .expect("Failed to connect to PostgreSQL");

    store
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    store
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
        title: Some(format!("Test Braid {hash_suffix}").into()),
        description: Some("A test braid for integration testing".into()),
        tags: tags.into_iter().map(Arc::from).collect(),
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
