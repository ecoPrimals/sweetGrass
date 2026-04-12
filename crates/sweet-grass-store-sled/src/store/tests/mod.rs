// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for Sled store.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test file: expect/unwrap are standard in tests"
)]
#![expect(
    deprecated,
    reason = "testing deprecated SledStore during migration period"
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
    assert!(activities.is_empty());
}

#[tokio::test]
async fn test_activities_for_braid_returns_embedded_activity() {
    use sweet_grass_core::activity::{Activity, ActivityType};

    let (store, _temp) = create_test_store();

    let activity = Activity::builder(ActivityType::Creation).build();
    let braid = BraidBuilder::default()
        .data_hash("sha256:with_activity")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .generated_by(activity.clone())
        .build()
        .expect("build braid");

    store.put(&braid).await.expect("put");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, activity.id);
}

#[tokio::test]
async fn test_derived_from_returns_child_braid() {
    use sweet_grass_core::entity::EntityReference;

    let (store, _temp) = create_test_store();

    let parent = create_test_braid("sha256:parent_derived");
    store.put(&parent).await.expect("put parent");

    let child = BraidBuilder::default()
        .data_hash("sha256:child_derived")
        .mime_type("text/plain")
        .size(50)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .derived_from(EntityReference::by_hash(parent.data_hash.clone()))
        .build()
        .expect("build child");
    store.put(&child).await.expect("put child");

    let unrelated = create_test_braid("sha256:unrelated_derived");
    store.put(&unrelated).await.expect("put unrelated");

    let derived = store
        .derived_from(&parent.data_hash)
        .await
        .expect("derived_from");
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].id, child.id);
}

#[tokio::test]
async fn test_derived_from_empty_when_no_derivations() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:no_derivations");
    store.put(&braid).await.expect("put");

    let derived = store
        .derived_from(&ContentHash::from("sha256:unknown_parent"))
        .await
        .expect("derived_from");
    assert!(derived.is_empty());
}

#[tokio::test]
async fn test_delete_multi_tag_cleans_indexes() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:multi_tag_del");
    braid.metadata.tags = vec!["alpha".into(), "beta".into(), "gamma".into()];
    store.put(&braid).await.expect("put");

    let filter_alpha = QueryFilter::new().with_tag("alpha");
    assert_eq!(
        store
            .query(&filter_alpha, QueryOrder::NewestFirst)
            .await
            .expect("query")
            .braids
            .len(),
        1
    );

    store.delete(&braid.id).await.expect("delete");

    for tag in &["alpha", "beta", "gamma"] {
        let filter = QueryFilter::new().with_tag(*tag);
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");
        assert!(result.braids.is_empty(), "tag {tag} index not cleaned");
    }

    assert!(
        store
            .get_by_hash(&braid.data_hash)
            .await
            .expect("get")
            .is_none()
    );
}

#[tokio::test]
async fn test_get_by_hash_missing_returns_none() {
    let (store, _temp) = create_test_store();

    let result = store
        .get_by_hash(&ContentHash::from("sha256:definitely_missing"))
        .await
        .expect("get_by_hash");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_activities_for_braid_unknown_returns_empty() {
    let (store, _temp) = create_test_store();

    let unknown = BraidId::from_string("nonexistent-braid-id".to_string());
    let activities = store
        .activities_for_braid(&unknown)
        .await
        .expect("activities");
    assert!(activities.is_empty());
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
