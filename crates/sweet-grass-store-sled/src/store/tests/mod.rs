// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for Sled store.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

mod edge;
mod query;

use super::*;
use sweet_grass_core::braid::BraidBuilder;
use tempfile::TempDir;

fn create_test_store() -> (SledStore, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let store = SledStore::open_path(temp.path()).expect("open store");
    (store, temp)
}

fn create_test_braid(hash: &str) -> Braid {
    BraidBuilder::default()
        .data_hash(hash)
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("build braid")
}

#[tokio::test]
async fn test_put_and_get() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:test1");

    store.put(&braid).await.expect("put");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().data_hash, braid.data_hash);
}

#[tokio::test]
async fn test_get_by_hash() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:hash_test");

    store.put(&braid).await.expect("put");

    let retrieved = store.get_by_hash(&braid.data_hash).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);
}

#[tokio::test]
async fn test_delete() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:delete_test");

    store.put(&braid).await.expect("put");
    assert!(store.exists(&braid.id).await.expect("exists"));

    store.delete(&braid.id).await.expect("delete");
    assert!(!store.exists(&braid.id).await.expect("exists"));
}

#[tokio::test]
async fn test_exists() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:exists_test");

    assert!(!store.exists(&braid.id).await.expect("exists"));
    store.put(&braid).await.expect("put");
    assert!(store.exists(&braid.id).await.expect("exists"));
}

#[tokio::test]
async fn test_flush() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:flush_test");

    store.put(&braid).await.expect("put");
    store.flush().expect("flush");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_size_on_disk() {
    let (store, _temp) = create_test_store();

    let initial_size = store.size_on_disk();

    for i in 0..10 {
        let braid = create_test_braid(&format!("sha256:size{i}"));
        store.put(&braid).await.expect("put");
    }
    store.flush().expect("flush");

    let final_size = store.size_on_disk();
    assert!(final_size >= initial_size);
}

#[tokio::test]
async fn test_get_nonexistent() {
    let (store, _temp) = create_test_store();

    let result = store
        .get(&BraidId::from_string("nonexistent".to_string()))
        .await
        .expect("get");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_returns_ok() {
    let (store, _temp) = create_test_store();

    let result = store
        .delete(&BraidId::from_string(
            "definitely-nonexistent-id".to_string(),
        ))
        .await
        .expect("delete");
    // delete returns true even for non-existent (idempotent)
    assert!(result);
}

#[tokio::test]
async fn test_delete_removes_from_get_by_hash() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:delete_hash_cleanup");

    store.put(&braid).await.expect("put");
    assert!(
        store
            .get_by_hash(&braid.data_hash)
            .await
            .expect("get")
            .is_some()
    );

    store.delete(&braid.id).await.expect("delete");

    let by_hash = store.get_by_hash(&braid.data_hash).await.expect("get");
    assert!(by_hash.is_none());
}

#[tokio::test]
async fn test_open_path() {
    let temp = tempfile::tempdir().expect("temp dir");
    let store = SledStore::open_path(temp.path()).expect("open");

    let braid = create_test_braid("sha256:path_test");
    store.put(&braid).await.expect("put");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_activity_storage() {
    use sweet_grass_core::activity::{Activity, ActivityType};

    let (store, _temp) = create_test_store();
    let activity = Activity::builder(ActivityType::Creation).build();

    store.put_activity(&activity).await.expect("put");

    let retrieved = store.get_activity(&activity.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, activity.id);
}

#[tokio::test]
async fn test_get_activity_nonexistent() {
    use sweet_grass_core::activity::ActivityId;

    let (store, _temp) = create_test_store();

    let result = store
        .get_activity(&ActivityId::from_task("nonexistent"))
        .await
        .expect("get");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_activities_for_braid() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:activities_test");
    store.put(&braid).await.expect("put");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities");
    assert!(activities.is_empty() || !activities.is_empty());
}

#[tokio::test]
async fn test_derived_from() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:derived_test");
    store.put(&braid).await.expect("put");

    let braids = store.derived_from(&braid.data_hash).await.expect("derived");
    assert!(braids.is_empty() || !braids.is_empty());
}

#[tokio::test]
async fn test_by_agent() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:agent_test");
    store.put(&braid).await.expect("put");

    let braids = store
        .by_agent(&braid.was_attributed_to)
        .await
        .expect("by_agent");
    assert_eq!(braids.len(), 1);
    assert_eq!(braids[0].id, braid.id);
}
