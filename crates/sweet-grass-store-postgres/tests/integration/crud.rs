// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! CRUD operations tests for `PostgreSQL` backend.
//!
//! Tests basic create, read, update, delete operations and data integrity.

use super::common::{create_test_braid, setup_postgres};
use sweet_grass_store::BraidStore;

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_basic_crud() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("crud001");

    // Create
    store.put(&braid).await.expect("Failed to store braid");

    // Read
    let retrieved = store.get(&braid.id).await.expect("Failed to get braid");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.data_hash, braid.data_hash);
    assert_eq!(retrieved.mime_type, braid.mime_type);

    // Exists
    assert!(
        store
            .exists(&braid.id)
            .await
            .expect("Failed to check exists")
    );

    // Delete
    let deleted = store.delete(&braid.id).await.expect("Failed to delete");
    assert!(deleted);

    // Verify deleted
    assert!(
        !store
            .exists(&braid.id)
            .await
            .expect("Failed to check exists")
    );
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_get_by_hash() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("hash001");
    store.put(&braid).await.expect("Failed to store braid");

    let retrieved = store
        .get_by_hash(&braid.data_hash)
        .await
        .expect("Failed to get by hash");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_upsert_behavior() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("upsert001");

    // First insert
    store.put(&braid).await.expect("put");

    // Update (same ID, different size)
    let mut updated = braid.clone();
    updated.size = 200;

    store.put(&updated).await.expect("upsert");

    // Verify updated
    let retrieved = store.get(&braid.id).await.expect("get").expect("exists");
    assert_eq!(retrieved.size, 200);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_update_existing_braid() {
    let (_container, store) = setup_postgres().await;

    let mut braid = create_test_braid("update001");
    store.put(&braid).await.expect("initial put");

    // Update size
    braid.size = 999;
    store.put(&braid).await.expect("update");

    let retrieved = store.get(&braid.id).await.expect("get").unwrap();
    assert_eq!(retrieved.size, 999);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_delete_nonexistent() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("nonexistent");
    let deleted = store.delete(&braid.id).await.expect("delete");

    // Deleting nonexistent should return false
    assert!(!deleted);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_get_by_hash_nonexistent() {
    let (_container, store) = setup_postgres().await;

    let result = store
        .get_by_hash(&"sha256:nonexistent".into())
        .await
        .expect("query");

    assert!(result.is_none());
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_exists_correctness() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("exists001");

    // Should not exist initially
    assert!(!store.exists(&braid.id).await.expect("exists check"));

    // Put it
    store.put(&braid).await.expect("put");

    // Should exist now
    assert!(store.exists(&braid.id).await.expect("exists check"));

    // Delete it
    store.delete(&braid.id).await.expect("delete");

    // Should not exist again
    assert!(!store.exists(&braid.id).await.expect("exists check"));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_health_check() {
    let (_container, store) = setup_postgres().await;

    // PostgreSQL store doesn't have health_check method in current trait
    // This is a placeholder for when it's added
    // For now, test that we can query
    let count = store
        .count(&sweet_grass_store::QueryFilter::default())
        .await
        .expect("count");

    assert_eq!(count, 0);
}
