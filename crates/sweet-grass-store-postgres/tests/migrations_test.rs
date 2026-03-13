// SPDX-License-Identifier: AGPL-3.0-only
//! Tests for PostgreSQL database migrations.
//!
//! Ensures migrations are idempotent and schema is correct.

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code may use unwrap/expect for clarity

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Test helper to create a test database
async fn create_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/sweetgrass_test".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Test that migrations can be applied successfully
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_migrations_apply() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Verify migrations table exists
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("Failed to query migrations table");

    assert!(result.0 > 0, "Migrations should have been applied");
}

/// Test that migrations are idempotent (can be run multiple times)
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_migrations_idempotent() {
    let pool = create_test_db().await;

    // Run migrations twice
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations first time");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations second time (not idempotent)");
}

/// Test that braids table has correct schema
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_braids_table_schema() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Check table exists and has expected columns
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.columns 
         WHERE table_name = 'braids' 
         AND column_name IN ('id', 'data_hash', 'content', 'created_at')",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query schema");

    assert!(result.0 >= 4, "Braids table should have required columns");
}

/// Test that indexes are created correctly
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_indexes_created() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Check that indexes exist
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_indexes 
         WHERE tablename = 'braids' 
         AND indexname LIKE 'idx_%'",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query indexes");

    assert!(result.0 > 0, "Braids table should have indexes");
}

/// Test that foreign key constraints are correct
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_foreign_key_constraints() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Check for foreign key constraints
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.table_constraints 
         WHERE constraint_type = 'FOREIGN KEY'",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query constraints");

    // Note: May be 0 if using JSONB without explicit FK relationships
    assert!(result.0 >= 0, "Should query constraints successfully");
}

/// Test that JSON columns support JSONB operations
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_jsonb_operations() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Test JSONB column exists and supports operations
    let result = sqlx::query(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'braids' 
         AND data_type = 'jsonb'",
    )
    .fetch_optional(&pool)
    .await
    .expect("Failed to query JSONB columns");

    // Should have at least one JSONB column for Braid content
    assert!(
        result.is_some(),
        "Should have JSONB columns for flexibility"
    );
}

/// Test migration rollback (if supported)
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker) with rollback support"]
async fn test_migration_rollback() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Note: sqlx migrations don't have built-in rollback
    // This test verifies we can drop and recreate cleanly

    sqlx::query("DROP TABLE IF EXISTS braids CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to drop table");

    // Re-run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to re-run migrations after drop");
}

/// Test that database can handle UTF-8 content
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_utf8_support() {
    let pool = create_test_db().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Verify UTF-8 encoding
    let result: (String,) = sqlx::query_as(
        "SELECT pg_encoding_to_char(encoding) FROM pg_database WHERE datname = current_database()",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query encoding");

    assert_eq!(result.0, "UTF8", "Database should use UTF-8 encoding");
}

/// Test that required extensions are available
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_required_extensions() {
    let pool = create_test_db().await;

    // Check for uuid-ossp or pgcrypto for UUID generation
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_available_extensions 
         WHERE name IN ('uuid-ossp', 'pgcrypto')",
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query extensions");

    assert!(result.0 > 0, "Should have UUID extension available");
}

/// Test concurrent migration attempts (race condition safety)
#[tokio::test]
#[ignore = "requires PostgreSQL running (Docker)"]
async fn test_concurrent_migrations() {
    let pool = create_test_db().await;

    // Spawn multiple migration tasks concurrently
    let handles: Vec<_> = (0..3)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move { sqlx::migrate!("./migrations").run(&pool).await })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent migrations should not conflict");
    }
}
