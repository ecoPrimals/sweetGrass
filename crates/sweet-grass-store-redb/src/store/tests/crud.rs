// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
async fn test_get_nonexistent() {
    let (store, _temp) = create_test_store();

    let result = store
        .get(&BraidId::from_string("nonexistent".to_string()))
        .await
        .expect("get");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let (store, _temp) = create_test_store();

    store
        .delete(&BraidId::from_string("nonexistent".to_string()))
        .await
        .expect("delete");
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
    let db_path = temp.path().join("db.redb");
    let store = RedbStore::open_path(&db_path).expect("open");

    let braid = create_test_braid("sha256:path_test");
    store.put(&braid).await.expect("put");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
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
async fn test_exists() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:exists_test");

    assert!(!store.exists(&braid.id).await.expect("exists"));

    store.put(&braid).await.expect("put");

    assert!(store.exists(&braid.id).await.expect("exists"));
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
async fn test_open_nested_parent_dirs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let nested = temp.path().join("a").join("b").join("c").join("db.redb");
    let store = RedbStore::open_path(&nested).expect("open nested");

    let braid = create_test_braid("sha256:nested_path_test");
    store.put(&braid).await.expect("put");
    assert!(store.exists(&braid.id).await.expect("exists"));
}
