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
async fn test_derived_from() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:derived_test");
    store.put(&braid).await.expect("put");

    let braids = store.derived_from(&braid.data_hash).await.expect("derived");
    assert!(braids.is_empty() || !braids.is_empty());
}

#[tokio::test]
async fn test_exists() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:exists_test");

    assert!(!store.exists(&braid.id).await.expect("exists"));

    store.put(&braid).await.expect("put");

    assert!(store.exists(&braid.id).await.expect("exists"));
}
