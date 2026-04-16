// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query, filter, ordering, pagination, and relationship tests.

use std::sync::Arc;

use sweet_grass_core::Braid;
use sweet_grass_core::activity::{Activity, ActivityType};
use sweet_grass_core::agent::{AgentAssociation, AgentRole, Did};
use sweet_grass_core::braid::BraidMetadata;
use sweet_grass_core::entity::EntityReference;
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

use super::{make_braid, setup};

#[tokio::test]
async fn query_default_filter_returns_all() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:q01", "did:key:z6MkA", "text/plain", 100);
    let b2 = make_braid("sha256:q02", "did:key:z6MkA", "text/plain", 200);

    store.put(&b1).await.expect("put b1");
    store.put(&b2).await.expect("put b2");

    let result = store
        .query(&QueryFilter::default(), QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 2);
    assert_eq!(result.braids.len(), 2);
    handle.abort();
}

#[tokio::test]
async fn query_by_agent_filter() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:af01", "did:key:z6MkAgentA", "text/plain", 100);
    let b2 = make_braid("sha256:af02", "did:key:z6MkAgentB", "text/plain", 200);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");

    let filter = QueryFilter::new().with_agent(Did::new("did:key:z6MkAgentA"));
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 1);
    assert_eq!(
        result.braids[0].was_attributed_to,
        Did::new("did:key:z6MkAgentA")
    );
    handle.abort();
}

#[tokio::test]
async fn query_by_mime_filter() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:mf01", "did:key:z6MkA", "text/plain", 100);
    let b2 = make_braid("sha256:mf02", "did:key:z6MkA", "application/json", 200);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");

    let filter = QueryFilter::new().with_mime_type("application/json");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 1);
    assert_eq!(&*result.braids[0].mime_type, "application/json");
    handle.abort();
}

#[tokio::test]
async fn query_by_tag_filter() {
    let (_mock, store, handle) = setup().await;
    let metadata = BraidMetadata {
        tags: vec![Arc::from("important"), Arc::from("test")],
        ..Default::default()
    };
    let b1 = Braid::builder()
        .data_hash("sha256:tf01")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkA"))
        .metadata(metadata)
        .build()
        .expect("braid with tags");
    let b2 = make_braid("sha256:tf02", "did:key:z6MkA", "text/plain", 200);

    store.put(&b1).await.expect("put b1");
    store.put(&b2).await.expect("put b2");

    let filter = QueryFilter::new().with_tag("important");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 1);
    handle.abort();
}

#[tokio::test]
async fn query_ordering_largest_first() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:ol01", "did:key:z6MkA", "text/plain", 50);
    let b2 = make_braid("sha256:ol02", "did:key:z6MkA", "text/plain", 500);
    let b3 = make_braid("sha256:ol03", "did:key:z6MkA", "text/plain", 200);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&b3).await.expect("put");

    let result = store
        .query(&QueryFilter::default(), QueryOrder::LargestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids[0].size, 500);
    assert_eq!(result.braids[1].size, 200);
    assert_eq!(result.braids[2].size, 50);
    handle.abort();
}

#[tokio::test]
async fn query_ordering_smallest_first() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:os01", "did:key:z6MkA", "text/plain", 500);
    let b2 = make_braid("sha256:os02", "did:key:z6MkA", "text/plain", 50);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");

    let result = store
        .query(&QueryFilter::default(), QueryOrder::SmallestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids[0].size, 50);
    assert_eq!(result.braids[1].size, 500);
    handle.abort();
}

#[tokio::test]
async fn query_ordering_oldest_first() {
    let (_mock, store, handle) = setup().await;
    let mut b1 = make_braid("sha256:oo01", "did:key:z6MkA", "text/plain", 100);
    b1.generated_at_time = 300;
    let mut b2 = make_braid("sha256:oo02", "did:key:z6MkA", "text/plain", 100);
    b2.generated_at_time = 100;

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");

    let result = store
        .query(&QueryFilter::default(), QueryOrder::OldestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids[0].generated_at_time, 100);
    assert_eq!(result.braids[1].generated_at_time, 300);
    handle.abort();
}

#[tokio::test]
async fn query_pagination_with_limit_offset() {
    let (_mock, store, handle) = setup().await;
    for i in 0..5 {
        let b = make_braid(
            &format!("sha256:pg{i:02}"),
            "did:key:z6MkA",
            "text/plain",
            u64::try_from(i * 100).unwrap_or(0),
        );
        store.put(&b).await.expect("put");
    }

    let filter = QueryFilter::new().with_limit(2).with_offset(1);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 5);
    assert_eq!(result.braids.len(), 2);
    assert!(result.has_more);
    handle.abort();
}

#[tokio::test]
async fn count_with_agent_filter() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:cf01", "did:key:z6MkAgentA", "text/plain", 100);
    let b2 = make_braid("sha256:cf02", "did:key:z6MkAgentB", "text/plain", 200);
    let b3 = make_braid("sha256:cf03", "did:key:z6MkAgentA", "text/plain", 300);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&b3).await.expect("put");

    let filter = QueryFilter::new().with_agent(Did::new("did:key:z6MkAgentA"));
    let count = store.count(&filter).await.expect("count");
    assert_eq!(count, 2);
    handle.abort();
}

#[tokio::test]
async fn by_agent_returns_agent_braids() {
    let (_mock, store, handle) = setup().await;
    let agent = Did::new("did:key:z6MkByAgent");
    let b1 = make_braid("sha256:ba01", agent.as_str(), "text/plain", 100);
    let b2 = make_braid("sha256:ba02", agent.as_str(), "text/plain", 200);
    let other = make_braid("sha256:ba03", "did:key:z6MkOther", "text/plain", 300);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&other).await.expect("put");

    let result = store.by_agent(&agent).await.expect("by_agent");
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|b| b.was_attributed_to == agent));
    handle.abort();
}

#[tokio::test]
async fn by_agent_empty_for_unknown_agent() {
    let (_mock, store, handle) = setup().await;
    let result = store
        .by_agent(&Did::new("did:key:z6MkNobody"))
        .await
        .expect("by_agent");
    assert!(result.is_empty());
    handle.abort();
}

#[tokio::test]
async fn derived_from_returns_derived_braids() {
    let (_mock, store, handle) = setup().await;
    let source_hash = sweet_grass_core::ContentHash::new("sha256:source01");

    let derived = Braid::builder()
        .data_hash("sha256:derived01")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkA"))
        .derived_from(EntityReference::by_hash("sha256:source01"))
        .build()
        .expect("derived braid");

    let unrelated = make_braid("sha256:unrelated01", "did:key:z6MkA", "text/plain", 200);

    store.put(&derived).await.expect("put derived");
    store.put(&unrelated).await.expect("put unrelated");

    let result = store
        .derived_from(&source_hash)
        .await
        .expect("derived_from");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, derived.id);
    handle.abort();
}

#[tokio::test]
async fn derived_from_empty_for_unknown_hash() {
    let (_mock, store, handle) = setup().await;
    let hash = sweet_grass_core::ContentHash::new("sha256:orphan");
    let result = store.derived_from(&hash).await.expect("derived_from");
    assert!(result.is_empty());
    handle.abort();
}

#[tokio::test]
async fn put_activity_and_get_activity() {
    let (_mock, store, handle) = setup().await;
    let activity = Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkA"),
            AgentRole::Creator,
        ))
        .compute_units(2.0)
        .build();

    store.put_activity(&activity).await.expect("put_activity");
    let retrieved = store
        .get_activity(&activity.id)
        .await
        .expect("get_activity");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, activity.id);
    handle.abort();
}

#[tokio::test]
async fn get_activity_nonexistent_returns_none() {
    let (_mock, store, handle) = setup().await;
    let result = store
        .get_activity(&sweet_grass_core::ActivityId::new())
        .await
        .expect("get_activity");
    assert!(result.is_none());
    handle.abort();
}

#[tokio::test]
async fn activities_for_braid_with_generated_by() {
    let (_mock, store, handle) = setup().await;
    let activity = Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkA"),
            AgentRole::Creator,
        ))
        .build();

    let braid = Braid::builder()
        .data_hash("sha256:actbraid01")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkA"))
        .generated_by(activity.clone())
        .build()
        .expect("braid with activity");

    store.put(&braid).await.expect("put braid");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities_for_braid");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, activity.id);
    handle.abort();
}

#[tokio::test]
async fn activities_for_braid_empty_when_no_activity() {
    let (_mock, store, handle) = setup().await;
    let braid = make_braid("sha256:noact01", "did:key:z6MkA", "text/plain", 100);
    store.put(&braid).await.expect("put");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities_for_braid");
    assert!(activities.is_empty());
    handle.abort();
}

#[tokio::test]
async fn query_with_data_hash_filter() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:dhf01", "did:key:z6MkA", "text/plain", 100);
    let b2 = make_braid("sha256:dhf02", "did:key:z6MkA", "text/plain", 200);

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");

    let filter = QueryFilter::new().with_hash(b1.data_hash.clone());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 1);
    assert_eq!(result.braids[0].data_hash, b1.data_hash);
    handle.abort();
}

#[tokio::test]
async fn query_with_time_range_filter() {
    let (_mock, store, handle) = setup().await;
    let mut b1 = make_braid("sha256:tr01", "did:key:z6MkA", "text/plain", 100);
    b1.generated_at_time = 500;
    let mut b2 = make_braid("sha256:tr02", "did:key:z6MkA", "text/plain", 200);
    b2.generated_at_time = 1500;
    let mut b3 = make_braid("sha256:tr03", "did:key:z6MkA", "text/plain", 300);
    b3.generated_at_time = 2500;

    store.put(&b1).await.expect("put");
    store.put(&b2).await.expect("put");
    store.put(&b3).await.expect("put");

    let filter = QueryFilter::new().with_time_range(1000, 2000);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.total_count, 1);
    assert_eq!(result.braids[0].generated_at_time, 1500);
    handle.abort();
}

#[tokio::test]
async fn query_with_braid_type_filter_match() {
    let (_mock, store, handle) = setup().await;
    let b1 = make_braid("sha256:bt01", "did:key:z6MkA", "text/plain", 100);

    store.put(&b1).await.expect("put");

    let filter = QueryFilter::new().with_type(sweet_grass_core::braid::BraidType::default());
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(
        result.total_count, 1,
        "Atomic braid should match Atomic filter"
    );
    handle.abort();
}
