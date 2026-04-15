// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the production `PostgreSQL` embedded migration path.
//!
//! Uses `PostgresStore::run_migrations()` (the same code path as production)
//! rather than `sqlx::migrate!`, ensuring tests validate the actual schema.

#![expect(clippy::expect_used, reason = "test file: expect is standard in tests")]

use sweet_grass_integration::testing::test_db_url;
use sweet_grass_store_postgres::{PostgresConfig, PostgresStore};

async fn connect_store() -> PostgresStore {
    let url = test_db_url();
    let config = PostgresConfig::new(&url)
        .max_connections(5)
        .min_connections(1);
    let store = PostgresStore::connect(&config)
        .await
        .expect("Failed to connect to test database");
    store
        .run_migrations()
        .await
        .expect("Failed to run production migrations");
    store
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_production_migrations_apply() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sweetgrass_migrations")
        .fetch_one(store.pool())
        .await
        .expect("Failed to query migrations table");

    assert!(
        result.0 > 0,
        "Production migrations should have been applied"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_production_migrations_idempotent() {
    let store = connect_store().await;

    store
        .run_migrations()
        .await
        .expect("Second run_migrations should be idempotent");

    store
        .run_migrations()
        .await
        .expect("Third run_migrations should also be idempotent");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_braids_table_schema() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_name = 'braids'
         AND column_name IN ('braid_id', 'data_hash', 'mime_type', 'size',
                             'attributed_to', 'generated_at_time', 'metadata')",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query schema");

    assert!(
        result.0 >= 7,
        "Braids table should have all required columns (found {}, expected >=7)",
        result.0
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_activities_table_exists() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public' AND table_name = 'activities'",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query schema");

    assert_eq!(result.0, 1, "activities table should exist after migration");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_indexes_created() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_indexes
         WHERE tablename = 'braids'
         AND indexname LIKE 'idx_%'",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query indexes");

    assert!(result.0 > 0, "Braids table should have indexes");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_foreign_key_constraints() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.table_constraints
         WHERE constraint_type = 'FOREIGN KEY'
         AND table_name IN ('braid_activities', 'braid_tags')",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query constraints");

    assert!(
        result.0 > 0,
        "braid_activities and braid_tags should have FK constraints"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_jsonb_columns_exist() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_name = 'braids'
         AND data_type = 'jsonb'",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query JSONB columns");

    assert!(
        result.0 >= 3,
        "Braids should have >=3 JSONB columns (metadata, ecop, was_derived_from)"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_migration_check_reports_up_to_date() {
    let store = connect_store().await;

    let up_to_date = store
        .check_migrations()
        .await
        .expect("check_migrations should succeed");

    assert!(up_to_date, "Migrations should be up to date after run");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_utf8_support() {
    let store = connect_store().await;

    let result: (String,) = sqlx::query_as(
        "SELECT pg_encoding_to_char(encoding) FROM pg_database WHERE datname = current_database()",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query encoding");

    assert_eq!(result.0, "UTF8", "Database should use UTF-8 encoding");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_required_extensions() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_available_extensions
         WHERE name IN ('uuid-ossp', 'pgcrypto')",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query extensions");

    assert!(result.0 > 0, "Should have UUID extension available");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_braid_tags_table_exists() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public' AND table_name = 'braid_tags'",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query schema");

    assert_eq!(result.0, 1, "braid_tags table should exist after migration");
}

#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_updated_at_trigger_exists() {
    let store = connect_store().await;

    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.triggers
         WHERE trigger_name = 'update_braids_updated_at'",
    )
    .fetch_one(store.pool())
    .await
    .expect("Failed to query triggers");

    assert_eq!(result.0, 1, "update_braids_updated_at trigger should exist");
}
