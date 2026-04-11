// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

use super::*;

#[test]
fn test_query_filter_new() {
    let filter = QueryFilter::new();
    assert!(filter.data_hash.is_none());
    assert!(filter.attributed_to.is_none());
    assert!(filter.braid_type.is_none());
    assert!(filter.limit.is_none());
}

#[test]
fn test_query_filter_with_hash() {
    let filter = QueryFilter::new().with_hash("sha256:abc123");
    assert!(filter.data_hash.is_some());
    assert_eq!(filter.data_hash.unwrap().as_str(), "sha256:abc123");
}

#[test]
fn test_query_filter_with_agent() {
    let did = Did::new("did:key:z6MkTest");
    let filter = QueryFilter::new().with_agent(did.clone());
    assert!(filter.attributed_to.is_some());
    assert_eq!(filter.attributed_to.unwrap(), did);
}

#[test]
fn test_query_filter_with_type() {
    let filter = QueryFilter::new().with_type(BraidType::Entity);
    assert!(filter.braid_type.is_some());
    assert_eq!(filter.braid_type.unwrap(), BraidType::Entity);
}

#[test]
fn test_query_filter_with_time_range() {
    let filter = QueryFilter::new().with_time_range(1000, 2000);
    assert_eq!(filter.created_after, Some(1000));
    assert_eq!(filter.created_before, Some(2000));
}

#[test]
fn test_query_filter_with_mime_type() {
    let filter = QueryFilter::new().with_mime_type("application/json");
    assert_eq!(filter.mime_type, Some("application/json".to_string()));
}

#[test]
fn test_query_filter_with_limit() {
    let filter = QueryFilter::new().with_limit(50);
    assert_eq!(filter.limit, Some(50));
}

#[test]
fn test_query_filter_with_offset() {
    let filter = QueryFilter::new().with_offset(100);
    assert_eq!(filter.offset, Some(100));
}

#[test]
fn test_query_filter_chained() {
    let filter = QueryFilter::new()
        .with_hash("sha256:test")
        .with_type(BraidType::Activity)
        .with_limit(10)
        .with_offset(5);

    assert!(filter.data_hash.is_some());
    assert_eq!(filter.braid_type, Some(BraidType::Activity));
    assert_eq!(filter.limit, Some(10));
    assert_eq!(filter.offset, Some(5));
}

#[test]
fn test_query_order_default() {
    let order = QueryOrder::default();
    assert!(matches!(order, QueryOrder::NewestFirst));
}

#[test]
fn test_query_order_variants() {
    // Ensure all variants can be constructed and formatted
    assert!(!format!("{:?}", QueryOrder::NewestFirst).is_empty());
    assert!(!format!("{:?}", QueryOrder::OldestFirst).is_empty());
    assert!(!format!("{:?}", QueryOrder::LargestFirst).is_empty());
    assert!(!format!("{:?}", QueryOrder::SmallestFirst).is_empty());
}

#[test]
fn test_query_result_new() {
    let result = QueryResult::new(vec![], 100, true);
    assert!(result.braids.is_empty());
    assert_eq!(result.total_count, 100);
    assert!(result.has_more);
}

#[test]
fn test_query_result_empty() {
    let result = QueryResult::empty();
    assert!(result.braids.is_empty());
    assert_eq!(result.total_count, 0);
    assert!(!result.has_more);
}

#[test]
fn test_query_filter_serialization() {
    let filter = QueryFilter::new().with_hash("sha256:test").with_limit(10);

    let json = serde_json::to_string(&filter).expect("serialize");
    let parsed: QueryFilter = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.data_hash, filter.data_hash);
    assert_eq!(parsed.limit, filter.limit);
}

#[test]
fn test_query_order_serialization() {
    let order = QueryOrder::OldestFirst;
    let json = serde_json::to_string(&order).expect("serialize");
    let parsed: QueryOrder = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, QueryOrder::OldestFirst));
}

mod proptests {
    use proptest::prelude::*;

    use super::*;

    fn arb_query_filter() -> impl Strategy<Value = QueryFilter> {
        (
            proptest::option::of("[a-z0-9]{8,64}"),
            proptest::option::of("did:key:z6Mk[a-zA-Z0-9]{30}"),
            proptest::option::of("[a-z/]{3,20}"),
            proptest::option::of("[a-z]{2,10}"),
            proptest::option::of(0usize..10_000),
            proptest::option::of(0usize..10_000),
        )
            .prop_map(|(hash, agent, mime, tag, limit, offset)| {
                let mut f = QueryFilter::new();
                if let Some(h) = hash {
                    f = f.with_hash(format!("sha256:{h}"));
                }
                if let Some(a) = agent {
                    f = f.with_agent(sweet_grass_core::agent::Did::new(a));
                }
                if let Some(m) = mime {
                    f.mime_type = Some(m);
                }
                if let Some(t) = tag {
                    f.tag = Some(t);
                }
                if let Some(l) = limit {
                    f = f.with_limit(l);
                }
                if let Some(o) = offset {
                    f = f.with_offset(o);
                }
                f
            })
    }

    proptest! {
        #[test]
        fn query_filter_serialization_roundtrip(filter in arb_query_filter()) {
            let json = serde_json::to_string(&filter).expect("serialize");
            let parsed: QueryFilter = serde_json::from_str(&json).expect("deserialize");
            prop_assert_eq!(parsed.data_hash, filter.data_hash);
            prop_assert_eq!(parsed.limit, filter.limit);
            prop_assert_eq!(parsed.offset, filter.offset);
            prop_assert_eq!(parsed.tag, filter.tag);
            prop_assert_eq!(parsed.mime_type, filter.mime_type);
        }

        #[test]
        fn query_filter_limit_respects_default(
            limit in proptest::option::of(0usize..100_000)
        ) {
            let filter = QueryFilter {
                limit,
                ..Default::default()
            };
            let effective = filter.limit.unwrap_or(DEFAULT_QUERY_LIMIT);
            prop_assert!(effective <= 100_000);
            if limit.is_none() {
                prop_assert_eq!(effective, DEFAULT_QUERY_LIMIT);
            }
        }

        #[test]
        fn query_result_has_more_consistency(
            total in 0usize..1000,
            limit in 1usize..100,
            offset in 0usize..500,
        ) {
            let result = QueryResult {
                braids: Vec::new(),
                total_count: total,
                has_more: offset + limit < total,
            };
            if result.has_more {
                prop_assert!(result.total_count > offset + limit);
            }
        }
    }
}
