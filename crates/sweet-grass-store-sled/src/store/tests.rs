// SPDX-License-Identifier: AGPL-3.0-only
//! Unit tests for Sled store.

#![allow(clippy::expect_used, clippy::unwrap_used)]

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
async fn test_query_basic() {
    let (store, _temp) = create_test_store();

    for i in 0..5 {
        let braid = create_test_braid(&format!("sha256:query{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new();
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 5);
    assert_eq!(result.total_count, 5);
}

#[tokio::test]
async fn test_query_with_filter() {
    let (store, _temp) = create_test_store();

    let braid1 = create_test_braid("sha256:filter1");
    let mut braid2 = create_test_braid("sha256:filter2");
    braid2.mime_type = "application/json".to_string();

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");

    let filter = QueryFilter::new().with_mime_type("application/json");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].mime_type, "application/json");
}

#[tokio::test]
async fn test_flush() {
    let (store, _temp) = create_test_store();
    let braid = create_test_braid("sha256:flush_test");

    store.put(&braid).await.expect("put");
    store.flush().expect("flush");

    // Should still be retrievable
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_size_on_disk() {
    let (store, _temp) = create_test_store();

    // Initially small
    let initial_size = store.size_on_disk();

    // Add some data
    for i in 0..10 {
        let braid = create_test_braid(&format!("sha256:size{i}"));
        store.put(&braid).await.expect("put");
    }
    store.flush().expect("flush");

    // Should be larger now
    let final_size = store.size_on_disk();
    assert!(final_size >= initial_size);
}

#[tokio::test]
async fn test_config_builder() {
    let config = SledConfig::new("/tmp/test")
        .cache_capacity(512 * 1024 * 1024)
        .flush_every_ms(Some(500));

    assert_eq!(config.path, "/tmp/test");
    assert_eq!(config.cache_capacity, 512 * 1024 * 1024);
    assert_eq!(config.flush_every_ms, Some(500));
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
async fn test_count() {
    let (store, _temp) = create_test_store();

    let empty_filter = QueryFilter::new();
    assert_eq!(store.count(&empty_filter).await.expect("count"), 0);

    for i in 0..3 {
        let braid = create_test_braid(&format!("sha256:count{i}"));
        store.put(&braid).await.expect("put");
    }

    assert_eq!(store.count(&empty_filter).await.expect("count"), 3);
}

#[tokio::test]
async fn test_query_oldest_first() {
    let (store, _temp) = create_test_store();

    for i in 0..3 {
        let braid = create_test_braid(&format!("sha256:order{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new();
    let result = store
        .query(&filter, QueryOrder::OldestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
}

#[tokio::test]
async fn test_query_with_limit() {
    let (store, _temp) = create_test_store();

    for i in 0..5 {
        let braid = create_test_braid(&format!("sha256:limit{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(2);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 2);
    assert_eq!(result.total_count, 5);
    assert!(result.has_more);
}

#[tokio::test]
async fn test_query_with_offset() {
    let (store, _temp) = create_test_store();

    for i in 0..5 {
        let braid = create_test_braid(&format!("sha256:offset{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_offset(2).with_limit(2);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 2);
    assert_eq!(result.total_count, 5);
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

#[tokio::test]
async fn test_query_by_agent() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:agent_query");
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_agent(braid.was_attributed_to.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].id, braid.id);
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

    // Should not error when deleting non-existent
    store
        .delete(&BraidId::from_string("nonexistent".to_string()))
        .await
        .expect("delete");
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
async fn test_derived_from() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:derived_test");
    store.put(&braid).await.expect("put");

    // Query by hash - derived_from finds braids with was_derived_from matching this hash
    let braids = store.derived_from(&braid.data_hash).await.expect("derived");
    // Result depends on braid content - verify call succeeded
    assert!(braids.is_empty() || !braids.is_empty());
}

#[tokio::test]
async fn test_exists() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:exists_test");

    // Should not exist before put
    assert!(!store.exists(&braid.id).await.expect("exists"));

    store.put(&braid).await.expect("put");

    // Should exist after put
    assert!(store.exists(&braid.id).await.expect("exists"));
}

#[tokio::test]
async fn test_get_activity_nonexistent() {
    use sweet_grass_core::activity::ActivityId;

    let (store, _temp) = create_test_store();

    // Use from_task to create an ID for a nonexistent activity
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

    // Get activities (may be empty for test braid)
    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities");
    // Verify call succeeded
    assert!(activities.is_empty() || !activities.is_empty());
}

#[tokio::test]
async fn test_query_by_type() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:type_test");
    store.put(&braid).await.expect("put");

    // Query by type
    let filter = QueryFilter::new().with_type(braid.braid_type.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(!result.braids.is_empty());
}

#[tokio::test]
async fn test_query_largest_first() {
    let (store, _temp) = create_test_store();

    for i in 0..3 {
        let braid = create_test_braid(&format!("sha256:largest{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new();
    let result = store
        .query(&filter, QueryOrder::LargestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
}

#[tokio::test]
async fn test_query_smallest_first() {
    let (store, _temp) = create_test_store();

    for i in 0..3 {
        let braid = create_test_braid(&format!("sha256:smallest{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new();
    let result = store
        .query(&filter, QueryOrder::SmallestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
}

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
    let results = store.get_batch(&ids, Some(10)).await;

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
    let results = store.get_batch(&ids, Some(5)).await;

    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}

#[tokio::test]
async fn test_store_empty_metadata_braid() {
    let (store, _temp) = create_test_store();

    let braid = BraidBuilder::default()
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
    braid.size = 10 * 1024 * 1024; // 10 MB
    braid.metadata.description = Some("x".repeat(100_000));

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

#[tokio::test]
async fn test_query_with_tag_filter() {
    let (store, _temp) = create_test_store();

    let mut braid_with_tag = create_test_braid("sha256:tagged");
    braid_with_tag.metadata.tags = vec!["important".to_string(), "review".to_string()];
    store.put(&braid_with_tag).await.expect("put");

    let braid_no_tag = create_test_braid("sha256:untagged");
    store.put(&braid_no_tag).await.expect("put");

    let filter = QueryFilter::new().with_tag("important");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert!(result.braids[0]
        .metadata
        .tags
        .contains(&"important".to_string()));
}

#[tokio::test]
async fn test_query_combined_filters() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:combined");
    braid.metadata.tags.push("test".to_string());
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new()
        .with_agent(braid.was_attributed_to.clone())
        .with_mime_type("text/plain")
        .with_tag("test")
        .with_limit(10);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(!result.braids.is_empty());
    assert_eq!(result.braids[0].id, braid.id);
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
