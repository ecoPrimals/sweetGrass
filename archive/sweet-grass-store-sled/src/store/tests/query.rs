// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query, filter, pagination, and ordering tests for Sled store.

use std::sync::Arc;

use super::*;

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
async fn test_query_largest_first_orders_by_descending_size() {
    let (store, _temp) = create_test_store();

    let mut small = create_test_braid("sha256:sz_order_small");
    small.size = 10;
    let mut mid = create_test_braid("sha256:sz_order_mid");
    mid.size = 500;
    let mut large = create_test_braid("sha256:sz_order_large");
    large.size = 2_000;

    store.put(&small).await.expect("put");
    store.put(&mid).await.expect("put");
    store.put(&large).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::LargestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
    assert_eq!(result.braids[0].size, 2_000);
    assert_eq!(result.braids[1].size, 500);
    assert_eq!(result.braids[2].size, 10);
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
async fn test_query_smallest_first_orders_by_ascending_size() {
    let (store, _temp) = create_test_store();

    let mut small = create_test_braid("sha256:sz_asc_small");
    small.size = 10;
    let mut mid = create_test_braid("sha256:sz_asc_mid");
    mid.size = 500;
    let mut large = create_test_braid("sha256:sz_asc_large");
    large.size = 2_000;

    store.put(&large).await.expect("put");
    store.put(&small).await.expect("put");
    store.put(&mid).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::SmallestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
    assert_eq!(result.braids[0].size, 10);
    assert_eq!(result.braids[1].size, 500);
    assert_eq!(result.braids[2].size, 2_000);
}

#[tokio::test]
async fn test_query_mime_type_excludes_non_matching() {
    let (store, _temp) = create_test_store();

    let mut json_braid = create_test_braid("sha256:mime_json");
    json_braid.mime_type = "application/json".into();
    store.put(&json_braid).await.expect("put");

    let filter = QueryFilter::new().with_mime_type("text/plain");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
}

#[tokio::test]
async fn test_query_tag_excludes_non_matching() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:tag_other");
    braid.metadata.tags = vec!["alpha".into()];
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_tag("beta");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
}

#[tokio::test]
async fn test_query_braid_type_entity_excludes_activity_variant() {
    let (store, _temp) = create_test_store();

    let entity_braid = create_test_braid("sha256:only_entity");
    let mut activity_braid = create_test_braid("sha256:only_activity");
    activity_braid.braid_type = sweet_grass_core::braid::BraidType::Activity;

    store.put(&entity_braid).await.expect("put");
    store.put(&activity_braid).await.expect("put");

    let filter = QueryFilter::new().with_type(sweet_grass_core::braid::BraidType::Entity);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].id, entity_braid.id);
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
async fn test_query_has_more_false_on_last_page() {
    let (store, _temp) = create_test_store();

    for i in 0..5 {
        let braid = create_test_braid(&format!("sha256:lastpage{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(3).with_offset(3);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 2);
    assert_eq!(result.total_count, 5);
    assert!(!result.has_more);
}

#[tokio::test]
async fn test_query_offset_beyond_end() {
    let (store, _temp) = create_test_store();

    for i in 0..3 {
        let braid = create_test_braid(&format!("sha256:beyond{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(10).with_offset(100);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
    assert_eq!(result.total_count, 3);
    assert!(!result.has_more);
}

#[tokio::test]
async fn test_query_agent_filter_excludes_other_agent() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:agent_excl");
    store.put(&braid).await.expect("put");

    let other = Did::new("did:key:z6MkOtherAgent");
    let filter = QueryFilter::new().with_agent(other);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert!(result.braids.is_empty());
}

#[tokio::test]
async fn test_query_oldest_first_time_ordering() {
    let (store, _temp) = create_test_store();

    let mut b1 = create_test_braid("sha256:oldest_order1");
    b1.generated_at_time = 300;
    let mut b2 = create_test_braid("sha256:oldest_order2");
    b2.generated_at_time = 100;
    let mut b3 = create_test_braid("sha256:oldest_order3");
    b3.generated_at_time = 200;

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&b3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::OldestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
    assert_eq!(result.braids[0].generated_at_time, 100);
    assert_eq!(result.braids[1].generated_at_time, 200);
    assert_eq!(result.braids[2].generated_at_time, 300);
}

#[tokio::test]
async fn test_query_newest_first_time_ordering() {
    let (store, _temp) = create_test_store();

    let mut b1 = create_test_braid("sha256:newest_order1");
    b1.generated_at_time = 300;
    let mut b2 = create_test_braid("sha256:newest_order2");
    b2.generated_at_time = 100;
    let mut b3 = create_test_braid("sha256:newest_order3");
    b3.generated_at_time = 200;

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&b3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
    assert_eq!(result.braids[0].generated_at_time, 300);
    assert_eq!(result.braids[1].generated_at_time, 200);
    assert_eq!(result.braids[2].generated_at_time, 100);
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
async fn test_query_filter_braid_type_excludes_mismatch() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:type_filter");
    store.put(&braid).await.expect("put");

    let filter = QueryFilter::new().with_type(sweet_grass_core::braid::BraidType::Collection {
        member_count: 5,
        summary_type: sweet_grass_core::braid::SummaryType::Temporal { start: 0, end: 100 },
    });
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert!(result.braids.is_empty());
}

#[tokio::test]
async fn test_query_filter_time_range() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:time_range");
    braid.generated_at_time = 500;
    store.put(&braid).await.expect("put");

    let in_range = QueryFilter::new().with_time_range(100, 900);
    let result = store
        .query(&in_range, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);

    let too_early = QueryFilter {
        created_after: Some(600),
        ..QueryFilter::new()
    };
    let result = store
        .query(&too_early, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert!(result.braids.is_empty());

    let too_late = QueryFilter {
        created_before: Some(400),
        ..QueryFilter::new()
    };
    let result = store
        .query(&too_late, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert!(result.braids.is_empty());

    let only_after_ok = QueryFilter {
        created_after: Some(100),
        ..QueryFilter::new()
    };
    let result = store
        .query(&only_after_ok, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);

    let only_before_ok = QueryFilter {
        created_before: Some(900),
        ..QueryFilter::new()
    };
    let result = store
        .query(&only_before_ok, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);
}

#[tokio::test]
async fn test_query_filter_source_primal() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:source_primal");
    braid.ecop.source_primal = Some(Arc::from("sweetGrass"));
    store.put(&braid).await.expect("put");

    let matching = QueryFilter {
        source_primal: Some("sweetGrass".to_string()),
        ..QueryFilter::new()
    };
    let result = store
        .query(&matching, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);

    let not_matching = QueryFilter {
        source_primal: Some("otherPrimal".to_string()),
        ..QueryFilter::new()
    };
    let result = store
        .query(&not_matching, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert!(result.braids.is_empty());
}

#[tokio::test]
async fn test_query_filter_niche() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:niche_filter");
    braid.ecop.niche = Some(Arc::from("chemistry"));
    store.put(&braid).await.expect("put");

    let matching = QueryFilter {
        niche: Some("chemistry".to_string()),
        ..QueryFilter::new()
    };
    let result = store
        .query(&matching, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);

    let not_matching = QueryFilter {
        niche: Some("biology".to_string()),
        ..QueryFilter::new()
    };
    let result = store
        .query(&not_matching, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert!(result.braids.is_empty());
}
