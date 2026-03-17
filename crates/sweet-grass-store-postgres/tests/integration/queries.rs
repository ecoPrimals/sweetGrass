// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query filter, ordering, and count tests for `PostgreSQL` backend.
//!
//! Tests `QueryFilter`, `QueryOrder`, `query()`, `count()`, `by_agent()`, `derived_from()`.

use super::common::{create_braid_with_metadata, create_test_braid, setup_postgres};
use sweet_grass_core::{Braid, entity::EntityReference};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_with_agent_filter() {
    let (_container, store) = setup_postgres().await;

    let agent1 = sweet_grass_core::agent::Did::new("did:key:z6MkAgent1");
    let agent2 = sweet_grass_core::agent::Did::new("did:key:z6MkAgent2");

    let braid1 = Braid::builder()
        .data_hash("sha256:agent1a")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(agent1.clone())
        .build()
        .expect("braid");
    let braid2 = Braid::builder()
        .data_hash("sha256:agent1b")
        .mime_type("text/plain")
        .size(200)
        .attributed_to(agent1.clone())
        .build()
        .expect("braid");
    let braid3 = Braid::builder()
        .data_hash("sha256:agent2a")
        .mime_type("text/plain")
        .size(300)
        .attributed_to(agent2.clone())
        .build()
        .expect("braid");

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let filter = QueryFilter::new().with_agent(agent1.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.total_count, 2);
    assert!(result.braids.iter().all(|b| b.was_attributed_to == agent1));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_with_time_range() {
    let (_container, store) = setup_postgres().await;

    let mut braid1 = create_test_braid("time1");
    braid1.generated_at_time = 500;
    let mut braid2 = create_test_braid("time2");
    braid2.generated_at_time = 1500;
    let mut braid3 = create_test_braid("time3");
    braid3.generated_at_time = 2500;

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let filter = QueryFilter::new().with_time_range(1000, 2000);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.total_count, 1);
    assert_eq!(result.braids[0].generated_at_time, 1500);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_by_mime_type() {
    let (_container, store) = setup_postgres().await;

    let braid1 = create_test_braid("mime1"); // text/plain
    let braid2 = create_braid_with_metadata("mime2", vec![]); // application/json

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");

    let filter = QueryFilter::new().with_mime_type("application/json");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.total_count, 1);
    assert_eq!(&*result.braids[0].mime_type, "application/json");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_by_tag() {
    let (_container, store) = setup_postgres().await;

    let braid1 = create_braid_with_metadata("tag1", vec!["important", "test"]);
    let braid2 = create_braid_with_metadata("tag2", vec!["other"]);
    let braid3 = create_braid_with_metadata("tag3", vec!["important"]);

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let filter = QueryFilter::new().with_tag("important");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.total_count, 2);
    assert!(
        result
            .braids
            .iter()
            .all(|b| b.metadata.tags.contains(&"important".to_string()))
    );
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_pagination_limit_offset() {
    let (_container, store) = setup_postgres().await;

    for i in 0..10 {
        let braid = create_test_braid(&format!("page{i}"));
        store.put(&braid).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(3).with_offset(2);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.total_count, 10);
    assert_eq!(result.braids.len(), 3);
    assert!(result.has_more);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering_newest_first() {
    let (_container, store) = setup_postgres().await;

    let mut braid1 = create_test_braid("ord1");
    braid1.generated_at_time = 100;
    let mut braid2 = create_test_braid("ord2");
    braid2.generated_at_time = 300;
    let mut braid3 = create_test_braid("ord3");
    braid3.generated_at_time = 200;

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::NewestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids[0].generated_at_time, 300);
    assert_eq!(result.braids[1].generated_at_time, 200);
    assert_eq!(result.braids[2].generated_at_time, 100);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering_oldest_first() {
    let (_container, store) = setup_postgres().await;

    let mut braid1 = create_test_braid("old1");
    braid1.generated_at_time = 300;
    let mut braid2 = create_test_braid("old2");
    braid2.generated_at_time = 100;
    let mut braid3 = create_test_braid("old3");
    braid3.generated_at_time = 200;

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::OldestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids[0].generated_at_time, 100);
    assert_eq!(result.braids[1].generated_at_time, 200);
    assert_eq!(result.braids[2].generated_at_time, 300);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering_largest_first() {
    let (_container, store) = setup_postgres().await;

    let mut braid1 = create_test_braid("size1");
    braid1.size = 50;
    let mut braid2 = create_test_braid("size2");
    braid2.size = 500;
    let mut braid3 = create_test_braid("size3");
    braid3.size = 200;

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::LargestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids[0].size, 500);
    assert_eq!(result.braids[1].size, 200);
    assert_eq!(result.braids[2].size, 50);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering_smallest_first() {
    let (_container, store) = setup_postgres().await;

    let mut braid1 = create_test_braid("small1");
    braid1.size = 500;
    let mut braid2 = create_test_braid("small2");
    braid2.size = 50;
    let mut braid3 = create_test_braid("small3");
    braid3.size = 200;

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let result = store
        .query(&QueryFilter::new(), QueryOrder::SmallestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids[0].size, 50);
    assert_eq!(result.braids[1].size, 200);
    assert_eq!(result.braids[2].size, 500);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_count_with_filter() {
    let (_container, store) = setup_postgres().await;

    let agent = sweet_grass_core::agent::Did::new("did:key:z6MkCountAgent");
    let braid1 = Braid::builder()
        .data_hash("sha256:count1")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(agent.clone())
        .build()
        .expect("braid");
    let braid2 = Braid::builder()
        .data_hash("sha256:count2")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(agent.clone())
        .build()
        .expect("braid");
    let braid3 = create_test_braid("count3"); // different agent

    store.put(&braid1).await.expect("put");
    store.put(&braid2).await.expect("put");
    store.put(&braid3).await.expect("put");

    let filter = QueryFilter::new().with_agent(agent);
    let count = store.count(&filter).await.expect("count");
    assert_eq!(count, 2);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_by_agent_multiple_braids() {
    let (_container, store) = setup_postgres().await;

    let agent = sweet_grass_core::agent::Did::new("did:key:z6MkByAgent");
    let braids: Vec<_> = (0..5)
        .map(|i| {
            Braid::builder()
                .data_hash(format!("sha256:byagent{i}"))
                .mime_type("text/plain")
                .size(100)
                .attributed_to(agent.clone())
                .build()
                .expect("braid")
        })
        .collect();

    for b in &braids {
        store.put(b).await.expect("put");
    }

    let result = store.by_agent(&agent).await.expect("by_agent");
    assert_eq!(result.len(), 5);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_derived_from_entity_references() {
    let (_container, store) = setup_postgres().await;

    let source_hash = sweet_grass_core::ContentHash::new("sha256:source");
    let mut braid = create_test_braid("derived");
    braid
        .was_derived_from
        .push(EntityReference::by_hash("sha256:source"));

    store.put(&braid).await.expect("put");

    let derived = store
        .derived_from(&source_hash)
        .await
        .expect("derived_from");
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].id, braid.id);
}
