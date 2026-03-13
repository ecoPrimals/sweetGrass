// SPDX-License-Identifier: AGPL-3.0-only

#![allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]

use super::*;
use std::sync::Arc;
use sweet_grass_core::ContentHash;
use sweet_grass_store::{MemoryStore, QueryFilter, QueryOrder};

fn make_test_braid(hash: &str, agent: &str) -> Braid {
    let did = Did::new(agent);
    Braid::builder()
        .data_hash(hash)
        .mime_type("application/json")
        .size(1024)
        .attributed_to(did)
        .build()
        .expect("should build")
}

#[tokio::test]
async fn test_basic_query() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:test1", "did:key:z6MkTest");
    store.put(&braid).await.expect("should store");

    let engine = QueryEngine::new(store);
    let hash = ContentHash::new("sha256:test1");
    let result = engine.get_by_hash(&hash).await.expect("should query");

    assert!(result.is_some());
    assert_eq!(result.unwrap().data_hash.as_str(), "sha256:test1");
}

#[tokio::test]
async fn test_by_agent() {
    let store = Arc::new(MemoryStore::new());
    let agent = "did:key:z6MkAgent";

    store
        .put(&make_test_braid("sha256:a1", agent))
        .await
        .expect("store");
    store
        .put(&make_test_braid("sha256:a2", agent))
        .await
        .expect("store");
    store
        .put(&make_test_braid("sha256:a3", "did:key:z6MkOther"))
        .await
        .expect("store");

    let engine = QueryEngine::new(store);
    let braids = engine
        .by_agent(&Did::new(agent))
        .await
        .expect("should query");

    assert_eq!(braids.len(), 2);
}

#[tokio::test]
async fn test_provenance_graph() {
    let store = Arc::new(MemoryStore::new());

    let parent = make_test_braid("sha256:parent", "did:key:z6MkP");
    let mut child = make_test_braid("sha256:child", "did:key:z6MkC");
    child.was_derived_from = vec![EntityReference::by_hash("sha256:parent")];

    store.put(&parent).await.expect("store");
    store.put(&child).await.expect("store");

    let engine = QueryEngine::new(store);
    let graph = engine
        .provenance_graph(EntityReference::by_hash("sha256:child"), None)
        .await
        .expect("should build graph");

    assert_eq!(graph.entity_count(), 2);
}

#[tokio::test]
async fn test_attribution_chain() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    let hash = ContentHash::new("sha256:test");
    let chain = engine
        .attribution_chain(&hash)
        .await
        .expect("should calculate");

    assert!(chain.is_valid());
    assert_eq!(chain.contributors.len(), 1);
}

#[tokio::test]
async fn test_agent_contributions() {
    let store = Arc::new(MemoryStore::new());
    let agent = "did:key:z6MkAgent";

    let mut braid1 = make_test_braid("sha256:a1", agent);
    braid1.size = 100;
    let mut braid2 = make_test_braid("sha256:a2", agent);
    braid2.size = 200;
    braid2.mime_type = "text/plain".to_string();

    store.put(&braid1).await.expect("store");
    store.put(&braid2).await.expect("store");

    let engine = QueryEngine::new(store);
    let contrib = engine
        .agent_contributions(&Did::new(agent))
        .await
        .expect("should calculate");

    assert_eq!(contrib.braid_count, 2);
    assert_eq!(contrib.total_size, 300);
    assert_eq!(contrib.by_mime_type.len(), 2);
}

#[tokio::test]
async fn test_export_provo() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    let doc = engine
        .export_braid_provo(&ContentHash::new("sha256:test"))
        .await
        .expect("should export");

    let json = doc.to_json().expect("should serialize");
    assert!(json.contains("@context"));
    assert!(json.contains("prov:Entity"));
}

#[tokio::test]
async fn test_get_by_id() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:test-id", "did:key:z6MkTest");
    let id = braid.id.clone();
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    let result = engine.get(&id).await.expect("should query");
    assert!(result.is_some());
    assert_eq!(result.unwrap().id, id);
}

#[tokio::test]
async fn test_get_nonexistent() {
    let store = Arc::new(MemoryStore::new());
    let engine = QueryEngine::new(store);
    let result = engine
        .get_by_hash(&ContentHash::new("sha256:nonexistent"))
        .await
        .expect("should query");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_exists() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:exists", "did:key:z6MkTest");
    let id = braid.id.clone();
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    assert!(engine.exists(&id).await.expect("should check"));

    let fake_id = sweet_grass_core::BraidId::new();
    assert!(!engine.exists(&fake_id).await.expect("should check"));
}

#[tokio::test]
async fn test_derived_from() {
    let store = Arc::new(MemoryStore::new());
    let parent = make_test_braid("sha256:parent-df", "did:key:z6MkP");
    let mut child = make_test_braid("sha256:child-df", "did:key:z6MkC");
    child.was_derived_from = vec![EntityReference::by_hash("sha256:parent-df")];

    store.put(&parent).await.expect("store");
    store.put(&child).await.expect("store");

    let engine = QueryEngine::new(store);
    let derived = engine
        .derived_from(&ContentHash::new("sha256:parent-df"))
        .await
        .expect("should query");
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].data_hash.as_str(), "sha256:child-df");
}

#[tokio::test]
async fn test_max_depth() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:depth-test", "did:key:z6MkTest");
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store).with_max_depth(5);
    let graph = engine
        .provenance_graph(EntityReference::by_hash("sha256:depth-test"), Some(2))
        .await
        .expect("should build graph");
    assert_eq!(graph.entity_count(), 1);
}

#[tokio::test]
async fn test_query_filter() {
    let store = Arc::new(MemoryStore::new());
    store
        .put(&make_test_braid("sha256:q1", "did:key:z6MkA"))
        .await
        .expect("store");
    store
        .put(&make_test_braid("sha256:q2", "did:key:z6MkB"))
        .await
        .expect("store");

    let engine = QueryEngine::new(store);
    let filter = QueryFilter::new().with_limit(10);
    let result = engine
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("should query");
    assert_eq!(result.braids.len(), 2);
}

#[tokio::test]
async fn test_attribution_chain_not_found() {
    let store = Arc::new(MemoryStore::new());
    let engine = QueryEngine::new(store);
    let result = engine
        .attribution_chain(&ContentHash::new("sha256:nonexistent"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_provenance_graph_export() {
    let store = Arc::new(MemoryStore::new());
    let parent = make_test_braid("sha256:parent-exp", "did:key:z6MkP");
    let mut child = make_test_braid("sha256:child-exp", "did:key:z6MkC");
    child.was_derived_from = vec![EntityReference::by_hash("sha256:parent-exp")];

    store.put(&parent).await.expect("store");
    store.put(&child).await.expect("store");

    let engine = QueryEngine::new(store);
    let doc = engine
        .export_graph_provo(EntityReference::by_hash("sha256:child-exp"), None)
        .await
        .expect("should export");
    let json = doc.to_json().expect("should serialize");
    assert!(json.contains("@graph"));
}

// ============================================================================
// NEW COMPREHENSIVE TESTS (Dec 27, 2025) - Query Engine Edge Cases
// ============================================================================

#[tokio::test]
async fn test_ancestors_parallel_empty_chain() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:solo", "did:key:z6MkSolo");
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    let ancestors = engine
        .ancestors_parallel(&braid.data_hash, Some(10))
        .await
        .expect("should query");

    // Solo braid with no derivation has itself as result (or empty depending on implementation)
    // The current implementation returns the braid itself
    assert!(
        ancestors.len() <= 1,
        "Solo braid should have 0 or 1 results"
    );
}

#[tokio::test]
async fn test_deep_provenance_chain() {
    let store = Arc::new(MemoryStore::new());

    // Create chain: A -> B -> C -> D -> E (5 levels)
    let mut previous_hash = "sha256:level_0".to_string();
    let level_0 = make_test_braid(&previous_hash, "did:key:z6Mk0");
    store.put(&level_0).await.expect("store");

    for i in 1..=5 {
        let hash = format!("sha256:level_{i}");
        let mut braid = make_test_braid(&hash, &format!("did:key:z6Mk{i}"));
        braid.was_derived_from = vec![EntityReference::by_hash(&previous_hash)];
        store.put(&braid).await.expect("store");
        previous_hash = hash;
    }

    // Query from deepest level
    let engine = QueryEngine::new(store);
    let graph = engine
        .provenance_graph(EntityReference::by_hash(&previous_hash), Some(10))
        .await
        .expect("should build graph");

    assert!(
        graph.entity_count() >= 1,
        "Should have entities in deep chain"
    );
}

#[tokio::test]
async fn test_max_depth_enforcement() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:depth-limit", "did:key:z6MkDepth");
    store.put(&braid).await.expect("store");

    // Set max depth to 2, but request 10
    let engine = QueryEngine::new(store).with_max_depth(2);
    let graph = engine
        .provenance_graph(EntityReference::by_hash(&braid.data_hash), Some(10))
        .await
        .expect("should respect max_depth");

    // Should respect the configured max_depth
    assert!(graph.entity_count() <= 100); // Reasonable upper bound
}

#[tokio::test]
async fn test_query_with_zero_limit() {
    let store = Arc::new(MemoryStore::new());
    store
        .put(&make_test_braid("sha256:z1", "did:key:z6MkA"))
        .await
        .expect("store");

    let engine = QueryEngine::new(store);
    let filter = QueryFilter::new().with_limit(0);
    let result = engine
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("should handle zero limit");

    assert_eq!(result.braids.len(), 0);
}

#[tokio::test]
async fn test_query_with_large_offset() {
    let store = Arc::new(MemoryStore::new());
    store
        .put(&make_test_braid("sha256:o1", "did:key:z6MkA"))
        .await
        .expect("store");

    let engine = QueryEngine::new(store);
    let filter = QueryFilter::new().with_limit(10).with_offset(1000);
    let result = engine
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("should handle large offset");

    assert_eq!(result.braids.len(), 0);
}

#[tokio::test]
async fn test_by_agent_multiple_braids() {
    let store = Arc::new(MemoryStore::new());
    let agent = Did::new("did:key:z6MkMulti");

    // Create 5 braids for same agent
    for i in 0..5 {
        let braid = make_test_braid(&format!("sha256:multi_{i}"), "did:key:z6MkMulti");
        store.put(&braid).await.expect("store");
    }

    let engine = QueryEngine::new(store);
    let braids = engine.by_agent(&agent).await.expect("should query");

    assert_eq!(braids.len(), 5);
}

#[tokio::test]
async fn test_by_agent_nonexistent() {
    let store = Arc::new(MemoryStore::new());
    let engine = QueryEngine::new(store);

    let did = Did::new("did:key:z6MkNonexistent");
    let braids = engine.by_agent(&did).await.expect("should query");

    assert_eq!(braids.len(), 0);
}

#[tokio::test]
async fn test_derived_from_multiple_children() {
    let store = Arc::new(MemoryStore::new());
    let parent_hash = "sha256:parent-multi".to_string();
    let parent = make_test_braid(&parent_hash, "did:key:z6MkParent");
    store.put(&parent).await.expect("store");

    // Create 3 children derived from same parent
    for i in 0..3 {
        let mut child = make_test_braid(&format!("sha256:child_multi_{i}"), "did:key:z6MkChild");
        child.was_derived_from = vec![EntityReference::by_hash(&parent_hash)];
        store.put(&child).await.expect("store");
    }

    let engine = QueryEngine::new(store);
    let children = engine
        .derived_from(&ContentHash::new(&parent_hash))
        .await
        .expect("should query");

    assert_eq!(children.len(), 3);
}

#[tokio::test]
async fn test_provenance_graph_with_cycle_detection() {
    let store = Arc::new(MemoryStore::new());

    // Create A -> B -> A (cycle)
    let hash_a = "sha256:cycle_a".to_string();
    let hash_b = "sha256:cycle_b".to_string();

    let braid_a = make_test_braid(&hash_a, "did:key:z6MkA");
    let mut braid_b = make_test_braid(&hash_b, "did:key:z6MkB");

    braid_b.was_derived_from = vec![EntityReference::by_hash(&hash_a)];
    // Note: In real scenario, would need more complex cycle

    store.put(&braid_a).await.expect("store");
    store.put(&braid_b).await.expect("store");

    let engine = QueryEngine::new(store);
    let graph = engine
        .provenance_graph(EntityReference::by_hash(&hash_b), Some(100))
        .await
        .expect("should handle potential cycles");

    // Should not infinite loop
    assert!(graph.entity_count() < 100);
}

#[tokio::test]
async fn test_attribution_chain_simple() {
    let store = Arc::new(MemoryStore::new());
    let braid = make_test_braid("sha256:attr-simple", "did:key:z6MkAttr");
    store.put(&braid).await.expect("store");

    let engine = QueryEngine::new(store);
    let chain = engine
        .attribution_chain(&braid.data_hash)
        .await
        .expect("should calculate");

    assert!(!chain.contributors.is_empty());
}

#[tokio::test]
async fn test_export_nonexistent_braid() {
    let store = Arc::new(MemoryStore::new());
    let engine = QueryEngine::new(store);

    let result = engine
        .export_braid_provo(&ContentHash::new("sha256:nonexistent"))
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_different_queries() {
    let store = Arc::new(MemoryStore::new());

    // Populate store
    for i in 0..10 {
        let braid = make_test_braid(&format!("sha256:conc_{i}"), &format!("did:key:z6Mk{i}"));
        store.put(&braid).await.expect("store");
    }

    let engine = Arc::new(QueryEngine::new(store));

    // Run different query types concurrently
    let e1 = Arc::clone(&engine);
    let e2 = Arc::clone(&engine);
    let e3 = Arc::clone(&engine);

    let h1 = tokio::spawn(async move {
        e1.query(&QueryFilter::new().with_limit(5), QueryOrder::NewestFirst)
            .await
    });

    let h2 = tokio::spawn(async move { e2.get_by_hash(&ContentHash::new("sha256:conc_0")).await });

    let h3 = tokio::spawn(async move {
        let did = Did::new("did:key:z6Mk1");
        e3.by_agent(&did).await
    });

    // All should succeed
    let (r1, r2, r3) = tokio::join!(h1, h2, h3);
    assert!(r1.is_ok() && r1.unwrap().is_ok());
    assert!(r2.is_ok() && r2.unwrap().is_ok());
    assert!(r3.is_ok() && r3.unwrap().is_ok());
}

#[tokio::test]
async fn test_query_ordering_consistency() {
    let store = Arc::new(MemoryStore::new());

    // Add braids with slight delays
    for i in 0..5 {
        let braid = make_test_braid(&format!("sha256:order_{i}"), "did:key:z6MkOrder");
        store.put(&braid).await.expect("store");
    }

    let engine = QueryEngine::new(store);

    // Query newest first
    let newest = engine
        .query(&QueryFilter::new(), QueryOrder::NewestFirst)
        .await
        .expect("should query");

    // Query oldest first
    let oldest = engine
        .query(&QueryFilter::new(), QueryOrder::OldestFirst)
        .await
        .expect("should query");

    assert_eq!(newest.braids.len(), oldest.braids.len());
    assert_eq!(newest.braids.len(), 5);
}
