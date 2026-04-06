// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_query_empty() {
    let (store, _temp) = create_test_store();

    let filter = QueryFilter::new();
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
    assert_eq!(result.total_count, 0);
    assert!(!result.has_more);
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
    braid2.mime_type = "application/json".into();

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");

    let filter = QueryFilter::new().with_mime_type("application/json");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert_eq!(&*result.braids[0].mime_type, "application/json");
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
async fn test_query_by_type() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:type_test");
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_type(braid.braid_type.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(!result.braids.is_empty());
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
async fn test_query_with_tag_filter() {
    let (store, _temp) = create_test_store();

    let mut braid_with_tag = create_test_braid("sha256:tagged");
    braid_with_tag.metadata.tags = vec!["important".into(), "review".into()];
    store.put(&braid_with_tag).await.expect("put");

    let braid_no_tag = create_test_braid("sha256:untagged");
    store.put(&braid_no_tag).await.expect("put");

    let filter = QueryFilter::new().with_tag("important");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert!(
        result.braids[0]
            .metadata
            .tags
            .iter()
            .any(|t| t.as_ref() == "important")
    );
}

#[tokio::test]
async fn test_query_combined_filters() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:combined");
    braid.metadata.tags.push("test".into());
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
async fn test_query_with_hash_filter_no_match() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:hash_filter");
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_hash("sha256:nonexistent");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
    assert_eq!(result.total_count, 0);
}

#[tokio::test]
async fn test_query_with_hash_filter_match() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:hash_match_test");
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_hash(braid.data_hash.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].data_hash, braid.data_hash);
}

#[tokio::test]
async fn test_query_has_more_pagination() {
    let (store, _temp) = create_test_store();

    for i in 0..5 {
        let braid = create_test_braid(&format!("sha256:pagination{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(2).with_offset(0);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 2);
    assert_eq!(result.total_count, 5);
    assert!(result.has_more);
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
async fn test_count_with_filter() {
    let (store, _temp) = create_test_store();

    let braid1 = create_test_braid("sha256:count_filter1");
    let mut braid2 = create_test_braid("sha256:count_filter2");
    braid2.mime_type = "application/json".into();

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");

    let mime_filter = QueryFilter::new().with_mime_type("application/json");
    assert_eq!(store.count(&mime_filter).await.expect("count"), 1);

    let agent_filter = QueryFilter::new().with_agent(braid1.was_attributed_to.clone());
    assert_eq!(store.count(&agent_filter).await.expect("count"), 2);
}
