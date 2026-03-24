// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Edge-case, corruption, batch, and concurrency tests for Sled store.

use super::*;
use tempfile::TempDir;

// --- Batch operations ---

#[tokio::test]
async fn test_put_batch() {
    let (store, _temp) = create_test_store();

    let braids: Vec<Braid> = (0..5)
        .map(|i| create_test_braid(&format!("sha256:batch{i}")))
        .collect();

    let (succeeded, errors) = store.put_batch(&braids, Some(5)).await;
    assert_eq!(succeeded, 5);
    assert!(errors.is_empty());

    for braid in &braids {
        let retrieved = store.get(&braid.id).await.expect("get");
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_get_batch() {
    let (store, _temp) = create_test_store();

    let braids: Vec<Braid> = (0..4)
        .map(|i| create_test_braid(&format!("sha256:getbatch{i}")))
        .collect();

    for braid in &braids {
        store.put(braid).await.expect("put");
    }

    let ids: Vec<BraidId> = braids.iter().map(|b| b.id.clone()).collect();
    let (results, errors) = store.get_batch(&ids, Some(10)).await;

    assert!(errors.is_empty());
    assert_eq!(results.len(), 4);
    let expected_hashes: std::collections::HashSet<_> =
        (0..4).map(|i| format!("sha256:getbatch{i}")).collect();
    let found_hashes: std::collections::HashSet<_> = results
        .iter()
        .filter_map(|opt| opt.as_ref())
        .map(|b| b.data_hash.as_str().to_string())
        .collect();
    assert_eq!(found_hashes, expected_hashes);
}

#[tokio::test]
async fn test_get_batch_mixed_existent_nonexistent() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:batch_mixed");
    store.put(&braid).await.expect("put");

    let ids = vec![
        braid.id.clone(),
        BraidId::from_string("nonexistent-1".to_string()),
        braid.id.clone(),
    ];
    let (results, errors) = store.get_batch(&ids, Some(5)).await;

    assert!(errors.is_empty());
    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}

// --- Edge cases ---

#[tokio::test]
async fn test_store_empty_metadata_braid() {
    let (store, _temp) = create_test_store();

    let braid = sweet_grass_core::braid::BraidBuilder::default()
        .data_hash("sha256:empty_meta")
        .mime_type("text/plain")
        .size(0)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .metadata(sweet_grass_core::braid::BraidMetadata::default())
        .build()
        .expect("build braid");

    store.put(&braid).await.expect("put");
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().metadata.tags.is_empty());
}

#[tokio::test]
async fn test_store_large_braid() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:large");
    braid.size = 10 * 1024 * 1024;
    braid.metadata.description = Some("x".repeat(100_000).into());

    store.put(&braid).await.expect("put");
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().size, 10 * 1024 * 1024);
}

#[tokio::test]
async fn test_concurrent_put_and_get() {
    let (store, _temp) = create_test_store();
    let store = std::sync::Arc::new(store);

    let mut handles = vec![];
    for i in 0..10 {
        let s = std::sync::Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let braid = create_test_braid(&format!("sha256:conc_{i}"));
            s.put(&braid).await.expect("put");
            let retrieved = s.get(&braid.id).await.expect("get");
            assert!(retrieved.is_some());
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("join");
    }
}

// --- Config tests ---

#[tokio::test]
async fn test_config_builder() {
    let config = crate::SledConfig::new("/tmp/test")
        .cache_capacity(512 * 1024 * 1024)
        .flush_every_ms(Some(500));

    assert_eq!(config.path, "/tmp/test");
    assert_eq!(config.cache_capacity, 512 * 1024 * 1024);
    assert_eq!(config.flush_every_ms, Some(500));
}

#[tokio::test]
async fn test_sled_config_default() {
    let config = crate::SledConfig::default();
    assert_eq!(config.path, crate::DEFAULT_DB_PATH);
    assert_eq!(config.cache_capacity, crate::DEFAULT_CACHE_CAPACITY);
    assert_eq!(config.flush_every_ms, Some(crate::DEFAULT_FLUSH_EVERY_MS));
    assert!(!config.use_compression);
}

#[test]
fn test_sled_config_clone_debug() {
    let config = crate::SledConfig::new("/tmp/clone_test");
    let debug = format!("{config:?}");
    assert!(debug.contains("SledConfig"));
    assert!(debug.contains("clone_test"));
}

#[test]
fn test_sled_config_sync_flush() {
    let config = crate::SledConfig::new("/tmp/sync").flush_every_ms(Some(0));
    assert_eq!(config.flush_every_ms, Some(0));
}

#[test]
fn test_sled_config_no_flush() {
    let config = crate::SledConfig::new("/tmp/noflush").flush_every_ms(None);
    assert!(config.flush_every_ms.is_none());
}

#[test]
fn test_sled_constants() {
    assert_eq!(crate::DEFAULT_CACHE_CAPACITY, 1024 * 1024 * 1024);
    assert_eq!(crate::DEFAULT_FLUSH_EVERY_MS, 1000);
    assert_eq!(crate::DEFAULT_DB_PATH, "./sweetgrass_sled");
}

#[test]
fn test_tree_names() {
    assert_eq!(crate::trees::BRAIDS, "braids");
    assert_eq!(crate::trees::BY_HASH, "by_hash");
    assert_eq!(crate::trees::BY_AGENT, "by_agent");
    assert_eq!(crate::trees::BY_TIME, "by_time");
    assert_eq!(crate::trees::BY_TAG, "by_tag");
    assert_eq!(crate::trees::ACTIVITIES, "activities");
}

#[tokio::test]
async fn test_open_path_with_config() {
    let temp = TempDir::new().expect("create temp dir");
    let config = crate::SledConfig::new(temp.path().to_string_lossy().to_string())
        .cache_capacity(64 * 1024)
        .flush_every_ms(Some(2000));

    let store = SledStore::open(&config).expect("open");
    let braid = create_test_braid("sha256:config_test");
    store.put(&braid).await.expect("put");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
}

// --- Corruption / error-path tests ---

#[tokio::test]
async fn test_get_corrupted_braid_returns_error() {
    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("corrupt_db");

    let braid = create_test_braid("sha256:corrupt_test");
    let braid_id = braid.id.clone();

    {
        use crate::trees;
        let store = SledStore::open_path(&db_path).expect("open");
        store.put(&braid).await.expect("put");
        store.flush().expect("flush before corrupt");
        drop(store);

        let db = sled::open(&db_path).expect("open sled");
        let braids = db.open_tree(trees::BRAIDS).expect("open braids tree");
        braids
            .insert(braid_id.as_str().as_bytes(), b"invalid json {{{")
            .expect("insert corrupt");
        db.flush().expect("flush corrupt");
        drop(db);
    }

    let store = SledStore::open_path(&db_path).expect("open");
    let result = store.get(&braid_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_query_skips_corrupted_entries() {
    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("query_corrupt_db");

    let braid1 = create_test_braid("sha256:valid1");
    let braid2 = create_test_braid("sha256:valid2");
    {
        let store = SledStore::open_path(&db_path).expect("open");
        store.put(&braid1).await.expect("put");
        store.put(&braid2).await.expect("put");
    }

    {
        use crate::trees;
        let db = sled::open(&db_path).expect("open sled");
        let braid_tree = db.open_tree(trees::BRAIDS).expect("open braids tree");
        braid_tree
            .insert(braid2.id.as_str().as_bytes(), b"{{{ corrupted")
            .expect("insert corrupt");
        db.flush().expect("flush");
    }

    let store = SledStore::open_path(&db_path).expect("open");
    let result = store
        .query(&QueryFilter::new(), QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].id, braid1.id);
}

#[tokio::test]
async fn test_get_activity_corrupted_returns_error() {
    use crate::trees;
    use sweet_grass_core::activity::{Activity, ActivityType};

    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("activity_corrupt_db");

    let activity = Activity::builder(ActivityType::Creation).build();
    {
        let store = SledStore::open_path(&db_path).expect("open");
        store.put_activity(&activity).await.expect("put");
    }

    {
        let db = sled::open(&db_path).expect("open sled");
        let activities = db.open_tree(trees::ACTIVITIES).expect("open activities");
        activities
            .insert(activity.id.as_str().as_bytes(), b"not valid json")
            .expect("insert");
        db.flush().expect("flush");
    }

    let store = SledStore::open_path(&db_path).expect("open");
    let result = store.get_activity(&activity.id).await;
    assert!(result.is_err());
}
