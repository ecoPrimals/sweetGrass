// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::similar_names,
    reason = "test module: expect/unwrap are standard; similar_names for closely related variables"
)]

use std::collections::HashMap;
use std::sync::Arc;

use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::entity::EntityReference;
use sweet_grass_store::{BraidStore, MemoryStore};

use super::{ProvenanceGraph, ProvenanceGraphBuilder};

fn make_test_braid(hash: &str, agent: &str, derived_from: Vec<&str>) -> Braid {
    let did = Did::new(agent);
    let mut braid = Braid::builder()
        .data_hash(hash)
        .mime_type("application/json")
        .size(1024)
        .attributed_to(did)
        .build()
        .expect("should build");

    braid.was_derived_from = derived_from
        .into_iter()
        .map(EntityReference::by_hash)
        .collect();

    braid
}

#[tokio::test]
async fn test_single_entity_graph() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:root", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("should store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:root"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.entity_count(), 1);
    assert_eq!(graph.depth, 0);
    assert!(!graph.truncated);
}

#[tokio::test]
async fn test_derivation_chain() {
    let store = Arc::new(MemoryStore::new());

    let grandparent = make_test_braid("sha256:grandparent", "did:key:z6MkGP", vec![]);
    let parent = make_test_braid("sha256:parent", "did:key:z6MkP", vec!["sha256:grandparent"]);
    let child = make_test_braid("sha256:child", "did:key:z6MkC", vec!["sha256:parent"]);

    store.put(&grandparent).await.expect("store");
    store.put(&parent).await.expect("store");
    store.put(&child).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:child"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.entity_count(), 3);
    assert_eq!(graph.depth, 2);

    let parents = graph.parents("sha256:child");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0].data_hash.as_str(), "sha256:parent");
}

#[tokio::test]
async fn test_depth_limit() {
    let store = Arc::new(MemoryStore::new());

    for i in 0..15 {
        let parent = if i > 0 {
            vec![format!("sha256:e{}", i - 1)]
        } else {
            vec![]
        };
        let braid = make_test_braid(
            &format!("sha256:e{i}"),
            "did:key:z6MkTest",
            parent.iter().map(std::string::String::as_str).collect(),
        );
        store.put(&braid).await.expect("store");
    }

    let builder = ProvenanceGraphBuilder::new().max_depth(5);
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:e14"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert!(graph.truncated);
    assert!(graph.entity_count() <= 7);
}

#[tokio::test]
async fn test_multiple_parents() {
    let store = Arc::new(MemoryStore::new());

    let parent1 = make_test_braid("sha256:p1", "did:key:z6MkP1", vec![]);
    let parent2 = make_test_braid("sha256:p2", "did:key:z6MkP2", vec![]);
    let child = make_test_braid(
        "sha256:child",
        "did:key:z6MkC",
        vec!["sha256:p1", "sha256:p2"],
    );

    store.put(&parent1).await.expect("store");
    store.put(&parent2).await.expect("store");
    store.put(&child).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:child"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.entity_count(), 3);

    let parents = graph.parents("sha256:child");
    assert_eq!(parents.len(), 2);
}

#[tokio::test]
async fn test_children() {
    let store = Arc::new(MemoryStore::new());

    let parent = make_test_braid("sha256:parent", "did:key:z6MkP", vec![]);
    let child1 = make_test_braid("sha256:c1", "did:key:z6MkC1", vec!["sha256:parent"]);
    let child2 = make_test_braid("sha256:c2", "did:key:z6MkC2", vec!["sha256:parent"]);

    store.put(&parent).await.expect("store");
    store.put(&child1).await.expect("store");
    store.put(&child2).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:c1"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let children = graph.children("sha256:parent");
    assert_eq!(children.len(), 1);
}

#[tokio::test]
async fn test_root_braid() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:root", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:root"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let root = graph.root_braid();
    assert!(root.is_some());
    assert_eq!(root.unwrap().data_hash.as_str(), "sha256:root");
}

#[tokio::test]
async fn test_entity_and_activity_accessors() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:test"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let hashes = graph.entity_hashes();
    assert_eq!(hashes.len(), 1);
    assert!(hashes.contains(&&"sha256:test".to_string()));

    let activity_ids = graph.activity_ids();
    let _ = activity_ids;
}

#[tokio::test]
async fn test_contains_entity() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:exists", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:exists"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert!(graph.contains_entity("sha256:exists"));
    assert!(!graph.contains_entity("sha256:not_exists"));
}

#[tokio::test]
async fn test_without_activities() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:no_activity", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new().include_activities(false);
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:no_activity"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.activity_count(), 0);
}

#[tokio::test]
async fn test_generating_activity() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:gen_test", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:gen_test"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let activity = graph.generating_activity("sha256:gen_test");
    let _ = activity;
}

#[tokio::test]
async fn test_parents_empty() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:no_parents", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:no_parents"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let parents = graph.parents("sha256:no_parents");
    assert!(parents.is_empty());
}

#[tokio::test]
async fn test_builder_default() {
    let builder = ProvenanceGraphBuilder::default();
    let _ = builder;
}

#[tokio::test]
async fn test_missing_root() {
    let store = Arc::new(MemoryStore::new());

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:missing"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.entity_count(), 0);
}

#[test]
fn test_provenance_graph_serialization() {
    let graph = ProvenanceGraph {
        root: EntityReference::by_hash("sha256:test"),
        entities: HashMap::new(),
        activities: HashMap::new(),
        derivation_edges: HashMap::new(),
        generation_edges: HashMap::new(),
        depth: 0,
        truncated: false,
        has_cycles: false,
    };

    let json = serde_json::to_string(&graph).expect("serialize");
    let parsed: ProvenanceGraph = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.depth, graph.depth);
    assert_eq!(parsed.truncated, graph.truncated);
    assert_eq!(parsed.has_cycles, graph.has_cycles);
}

#[tokio::test]
async fn test_children_with_multi_derivation() {
    let store = Arc::new(MemoryStore::new());

    let parent = make_test_braid("sha256:parent_c", "did:key:z6MkP", vec![]);
    let child1 = make_test_braid("sha256:child1_c", "did:key:z6MkC1", vec!["sha256:parent_c"]);
    let child2 = make_test_braid("sha256:child2_c", "did:key:z6MkC2", vec!["sha256:parent_c"]);

    store.put(&parent).await.expect("store");
    store.put(&child1).await.expect("store");
    store.put(&child2).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:child1_c"),
            &(Arc::clone(&store) as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let children = graph.children("sha256:parent_c");
    assert_eq!(children.len(), 1);
}

#[tokio::test]
async fn test_root_braid_accessible() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:root_b", "did:key:z6MkTest", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:root_b"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    let root = graph.root_braid();
    assert!(root.is_some());
    assert_eq!(root.unwrap().data_hash.as_str(), "sha256:root_b");
}

#[tokio::test]
async fn test_root_braid_when_missing() {
    let store = Arc::new(MemoryStore::new());

    let builder = ProvenanceGraphBuilder::new();
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:gone"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert!(graph.root_braid().is_none());
}

#[tokio::test]
async fn test_cycle_detection() {
    let store = Arc::new(MemoryStore::new());

    let braid_a = make_test_braid("sha256:cycle_a", "did:key:z6Mk", vec!["sha256:cycle_b"]);
    let braid_b = make_test_braid("sha256:cycle_b", "did:key:z6Mk", vec!["sha256:cycle_a"]);

    store.put(&braid_a).await.expect("store");
    store.put(&braid_b).await.expect("store");

    let builder = ProvenanceGraphBuilder::new().max_depth(20);
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:cycle_a"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should not infinite loop");

    assert_eq!(graph.entity_count(), 2);
    assert!(!graph.truncated);
}

#[tokio::test]
async fn test_max_depth_zero() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:depth0", "did:key:z6Mk", vec![]);
    store.put(&braid).await.expect("store");

    let builder = ProvenanceGraphBuilder::new().max_depth(0);
    let graph = builder
        .build(
            EntityReference::by_hash("sha256:depth0"),
            &(store as Arc<dyn BraidStore>),
        )
        .await
        .expect("should build");

    assert_eq!(graph.entity_count(), 1);
}

#[tokio::test]
async fn test_children_of_nonexistent_entity() {
    let graph = ProvenanceGraph {
        root: EntityReference::by_hash("sha256:test"),
        entities: HashMap::new(),
        activities: HashMap::new(),
        derivation_edges: HashMap::new(),
        generation_edges: HashMap::new(),
        depth: 0,
        truncated: false,
        has_cycles: false,
    };

    let children = graph.children("sha256:nonexistent");
    assert!(children.is_empty());
}

mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_hash() -> impl Strategy<Value = String> {
        "[a-f0-9]{8}".prop_map(|h| format!("sha256:{h}"))
    }

    proptest! {
        #[test]
        fn graph_entity_count_matches_map(hashes in proptest::collection::hash_set(arb_hash(), 0..20)) {
            let mut graph = ProvenanceGraph {
                root: EntityReference::by_hash("sha256:root"),
                entities: HashMap::new(),
                activities: HashMap::new(),
                derivation_edges: HashMap::new(),
                generation_edges: HashMap::new(),
                depth: 0,
                truncated: false,
                has_cycles: false,
            };

            for h in &hashes {
                let braid = make_test_braid(h, "did:key:z6MkProp", vec![]);
                graph.entities.insert(h.clone(), braid);
            }

            prop_assert_eq!(graph.entity_count(), hashes.len());
            for h in &hashes {
                prop_assert!(graph.contains_entity(h));
            }
        }

        #[test]
        fn graph_serialization_roundtrip(depth in 0u32..100, truncated in proptest::bool::ANY) {
            let graph = ProvenanceGraph {
                root: EntityReference::by_hash("sha256:prop-root"),
                entities: HashMap::new(),
                activities: HashMap::new(),
                derivation_edges: HashMap::new(),
                generation_edges: HashMap::new(),
                depth,
                truncated,
                has_cycles: false,
            };

            let json = serde_json::to_string(&graph).expect("serialize");
            let parsed: ProvenanceGraph = serde_json::from_str(&json).expect("deserialize");

            prop_assert_eq!(parsed.depth, depth);
            prop_assert_eq!(parsed.truncated, truncated);
        }
    }
}
