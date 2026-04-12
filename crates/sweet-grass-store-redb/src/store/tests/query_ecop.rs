// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
