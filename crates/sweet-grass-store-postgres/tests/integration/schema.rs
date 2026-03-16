// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Migration and schema tests for `PostgreSQL` backend.
//!
//! Tests that migrations run correctly and schema is valid.

use super::common::setup_postgres;

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migrations_idempotent() {
    let (_container, store) = setup_postgres().await;

    // Run migrations twice - should succeed both times
    store.run_migrations().await.expect("first migration");
    store
        .run_migrations()
        .await
        .expect("second migration (idempotent)");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_table_exists_after_migration() {
    let (_container, store) = setup_postgres().await;

    let pool = store.pool();
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables 
         WHERE table_schema = 'public' AND table_name = 'braids'",
    )
    .fetch_one(pool)
    .await
    .expect("query schema");

    assert_eq!(count, 1, "braids table should exist after migration");
}
