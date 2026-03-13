// SPDX-License-Identifier: AGPL-3.0-only
//! End-to-end integration tests for `SweetGrass`.
//!
//! These tests verify the complete attribution pipeline from
//! data creation through querying and export.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::clone_on_ref_ptr
)] // Test code may use unwrap/expect for clarity

use std::sync::Arc;

use sweet_grass_compression::{CompressionEngine, Session, SessionOutcome, SessionVertex};
use sweet_grass_core::{
    activity::ActivityType, agent::Did, braid::BraidMetadata, entity::EntityReference,
};
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_store::{BraidStore, MemoryStore, QueryFilter, QueryOrder};

/// Helper to create a test environment.
fn setup() -> (
    Arc<dyn BraidStore>,
    Arc<BraidFactory>,
    Arc<QueryEngine>,
    Arc<CompressionEngine>,
) {
    let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
    let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkTest")));
    let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
    let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));
    (store, factory, query, compression)
}

// =============================================
// Basic CRUD Tests
// =============================================

#[tokio::test]
async fn test_create_and_retrieve_braid() {
    let (store, factory, _, _) = setup();

    // Create a Braid
    let braid = factory
        .from_data(b"Hello, World!", "text/plain", None)
        .expect("should create braid");

    // Store it
    store.put(&braid).await.expect("should store");

    // Retrieve by ID
    let retrieved = store.get(&braid.id).await.expect("should query");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, braid.id);
    assert_eq!(retrieved.data_hash, braid.data_hash);
}

#[tokio::test]
async fn test_retrieve_by_hash() {
    let (store, factory, _, _) = setup();

    let braid = factory
        .from_data(b"Test data", "application/octet-stream", None)
        .expect("should create");

    store.put(&braid).await.expect("should store");

    let retrieved = store
        .get_by_hash(&braid.data_hash)
        .await
        .expect("should query");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);
}

#[tokio::test]
async fn test_delete_braid() {
    let (store, factory, _, _) = setup();

    let braid = factory
        .from_data(b"To delete", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    // Verify exists
    assert!(store.get(&braid.id).await.expect("query").is_some());

    // Delete
    store.delete(&braid.id).await.expect("delete");

    // Verify gone
    assert!(store.get(&braid.id).await.expect("query").is_none());
}

// =============================================
// Query Tests
// =============================================

#[tokio::test]
async fn test_query_by_agent() {
    let (store, _, _, _) = setup();

    let alice = Did::new("did:key:z6MkAlice");
    let bob = Did::new("did:key:z6MkBob");

    let alice_factory = BraidFactory::new(alice.clone());
    let bob_factory = BraidFactory::new(bob.clone());

    // Alice creates 2 Braids
    let alice1 = alice_factory
        .from_data(b"Alice 1", "text/plain", None)
        .expect("create");
    let alice2 = alice_factory
        .from_data(b"Alice 2", "text/plain", None)
        .expect("create");

    // Bob creates 1 Braid
    let bob1 = bob_factory
        .from_data(b"Bob 1", "text/plain", None)
        .expect("create");

    store.put(&alice1).await.expect("store");
    store.put(&alice2).await.expect("store");
    store.put(&bob1).await.expect("store");

    // Query Alice's Braids
    let alice_braids = store.by_agent(&alice).await.expect("query");
    assert_eq!(alice_braids.len(), 2);

    // Query Bob's Braids
    let bob_braids = store.by_agent(&bob).await.expect("query");
    assert_eq!(bob_braids.len(), 1);
}

#[tokio::test]
async fn test_query_with_filter() {
    let (store, factory, _, _) = setup();

    // Create Braids with different MIME types
    let json_braid = factory
        .from_data(b"{}", "application/json", None)
        .expect("create");
    let text_braid = factory
        .from_data(b"text", "text/plain", None)
        .expect("create");
    let csv_braid = factory
        .from_data(b"a,b,c", "text/csv", None)
        .expect("create");

    store.put(&json_braid).await.expect("store");
    store.put(&text_braid).await.expect("store");
    store.put(&csv_braid).await.expect("store");

    // Query JSON only
    let filter = QueryFilter::new().with_mime_type("application/json");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].mime_type, "application/json");
}

#[tokio::test]
async fn test_query_pagination() {
    let (store, factory, _, _) = setup();

    // Create 10 Braids
    for i in 0..10 {
        let braid = factory
            .from_data(format!("Data {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("store");
    }

    // Query first 5
    let filter = QueryFilter::new().with_limit(5);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 5);
    assert_eq!(result.total_count, 10);
    assert!(result.has_more);

    // Query next 5
    let filter = QueryFilter::new().with_limit(5).with_offset(5);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 5);
    assert!(!result.has_more);
}

// =============================================
// Derivation Chain Tests
// =============================================

#[tokio::test]
async fn test_derivation_chain() {
    let (store, _, query, _) = setup();

    let alice = Did::new("did:key:z6MkAlice");
    let bob = Did::new("did:key:z6MkBob");

    let alice_factory = BraidFactory::new(alice.clone());
    let bob_factory = BraidFactory::new(bob.clone());

    // Alice creates source data
    let source = alice_factory
        .from_data(b"Source data", "text/plain", None)
        .expect("create");
    store.put(&source).await.expect("store");

    // Bob derives from it
    let derived = bob_factory
        .derived_from(
            b"Derived data",
            "text/plain",
            vec![EntityReference::by_hash(&source.data_hash)],
            ActivityType::Derivation,
            None,
        )
        .expect("create");
    store.put(&derived).await.expect("store");

    // Query provenance graph
    let graph = query
        .provenance_graph(EntityReference::by_hash(&derived.data_hash), Some(5))
        .await
        .expect("query");

    // Should have both entities
    assert_eq!(graph.entities.len(), 2);
    assert!(graph.entities.contains_key(&derived.data_hash));
    assert!(graph.entities.contains_key(&source.data_hash));
}

#[tokio::test]
async fn test_multi_level_derivation() {
    let (store, _, query, _) = setup();

    let factory = BraidFactory::new(Did::new("did:key:z6MkTest"));

    // Level 0: Original
    let level0 = factory
        .from_data(b"Level 0", "text/plain", None)
        .expect("create");
    store.put(&level0).await.expect("store");

    // Level 1: Derived from level 0
    let level1 = factory
        .derived_from(
            b"Level 1",
            "text/plain",
            vec![EntityReference::by_hash(&level0.data_hash)],
            ActivityType::Transformation,
            None,
        )
        .expect("create");
    store.put(&level1).await.expect("store");

    // Level 2: Derived from level 1
    let level2 = factory
        .derived_from(
            b"Level 2",
            "text/plain",
            vec![EntityReference::by_hash(&level1.data_hash)],
            ActivityType::Transformation,
            None,
        )
        .expect("create");
    store.put(&level2).await.expect("store");

    // Query with full depth
    let graph = query
        .provenance_graph(EntityReference::by_hash(&level2.data_hash), Some(10))
        .await
        .expect("query");

    assert_eq!(graph.entities.len(), 3);
    assert_eq!(graph.depth, 2);
}

// =============================================
// Attribution Tests
// =============================================

#[tokio::test]
async fn test_single_contributor_attribution() {
    let (store, factory, query, _) = setup();

    let braid = factory
        .from_data(b"Single author", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    let chain = query
        .attribution_chain(&braid.data_hash)
        .await
        .expect("query");

    // Single contributor should have 100%
    assert_eq!(chain.contributors.len(), 1);
    assert!((chain.contributors[0].share - 1.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_attribution_normalization() {
    let (store, factory, query, _) = setup();

    let braid = factory
        .from_data(b"Test", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    let chain = query
        .attribution_chain(&braid.data_hash)
        .await
        .expect("query");

    // Shares should sum to 1.0
    let total: f64 = chain.contributors.iter().map(|c| c.share).sum();
    assert!((total - 1.0).abs() < 0.001);
}

// =============================================
// Compression Tests
// =============================================

#[tokio::test]
async fn test_empty_session_discarded() {
    let (_, _, _, compression) = setup();

    let session = Session::new("empty-session");

    let result = compression.compress(&session).expect("compress");

    assert!(!result.has_braids());
    assert_eq!(result.count(), 0);
}

#[tokio::test]
async fn test_rollback_session_discarded() {
    let (_, _, _, compression) = setup();

    let mut session = Session::new("rollback-session");
    session.add_vertex(
        SessionVertex::new("v1", "sha256:abc", "text/plain", Did::new("did:test")).committed(),
    );
    session.finalize(SessionOutcome::Rollback);

    let result = compression.compress(&session).expect("compress");
    assert!(!result.has_braids());
}

#[tokio::test]
async fn test_committed_session_creates_braid() {
    let (store, _, _, compression) = setup();

    let mut session = Session::new("committed-session");
    session.add_vertex(
        SessionVertex::new("v1", "sha256:input", "text/plain", Did::new("did:test"))
            .with_size(100)
            .committed(),
    );
    session.add_vertex(
        SessionVertex::new("v2", "sha256:output", "text/plain", Did::new("did:test"))
            .with_size(200)
            .with_parent("v1")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    let result = compression.compress(&session).expect("compress");

    assert!(result.has_braids());
    assert_eq!(result.count(), 1);

    // Store and verify
    for braid in result.braids() {
        store.put(braid).await.expect("store");
    }

    let count = store.count(&QueryFilter::default()).await.expect("count");
    assert_eq!(count, 1);
}

// =============================================
// PROV-O Export Tests
// =============================================

#[tokio::test]
async fn test_provo_export_has_context() {
    let (store, factory, query, _) = setup();

    let braid = factory
        .from_data(b"Test", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    let provo = query
        .export_braid_provo(&braid.data_hash)
        .await
        .expect("export");

    // Should have @context (a serde_json::Value, not Option)
    assert!(!provo.context.is_null());
}

#[tokio::test]
async fn test_provo_export_has_entity() {
    let (store, factory, query, _) = setup();

    let metadata = BraidMetadata {
        title: Some("Test Entity".to_string()),
        ..Default::default()
    };

    let braid = factory
        .from_data(b"Entity data", "text/plain", Some(metadata))
        .expect("create");
    store.put(&braid).await.expect("store");

    let provo = query
        .export_braid_provo(&braid.data_hash)
        .await
        .expect("export");

    // Should have graph entries
    assert!(!provo.graph.is_empty());
}

// =============================================
// Concurrent Access Tests
// =============================================

#[tokio::test]
async fn test_concurrent_writes() {
    let (store, factory, _, _) = setup();

    let store = Arc::clone(&store);

    // Spawn multiple write tasks
    let mut handles = vec![];
    for i in 0..10 {
        let store = Arc::clone(&store);
        let factory = factory.clone();
        handles.push(tokio::spawn(async move {
            let braid = factory
                .from_data(format!("Concurrent {i}").as_bytes(), "text/plain", None)
                .expect("create");
            store.put(&braid).await.expect("store");
        }));
    }

    // Wait for all
    for handle in handles {
        handle.await.expect("task");
    }

    // Verify all written
    let count = store.count(&QueryFilter::default()).await.expect("count");
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_concurrent_reads() {
    let (store, factory, _, _) = setup();

    // Create a Braid
    let braid = factory
        .from_data(b"Shared", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    // Spawn multiple read tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let store = Arc::clone(&store);
        let braid_id = braid.id.clone();
        handles.push(tokio::spawn(async move {
            let result = store.get(&braid_id).await.expect("query");
            assert!(result.is_some());
        }));
    }

    // Wait for all
    for handle in handles {
        handle.await.expect("task");
    }
}

// =============================================
// Edge Cases
// =============================================

#[tokio::test]
async fn test_empty_data_braid() {
    let (store, factory, _, _) = setup();

    let braid = factory.from_data(b"", "text/plain", None).expect("create");
    store.put(&braid).await.expect("store");

    assert_eq!(braid.size, 0);

    let retrieved = store.get(&braid.id).await.expect("query");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_large_metadata() {
    let (store, factory, _, _) = setup();

    let metadata = BraidMetadata {
        title: Some("A".repeat(1000)),
        description: Some("B".repeat(5000)),
        tags: (0..100).map(|i| format!("tag-{i}")).collect(),
        ..Default::default()
    };

    let braid = factory
        .from_data(b"Data", "text/plain", Some(metadata))
        .expect("create");
    store.put(&braid).await.expect("store");

    let retrieved = store.get(&braid.id).await.expect("query").unwrap();
    assert_eq!(retrieved.metadata.tags.len(), 100);
}

#[tokio::test]
async fn test_special_characters_in_data() {
    let (store, factory, _, _) = setup();

    let special_data = "Hello 🌾 World! こんにちは\n\r\t\0".as_bytes();
    let braid = factory
        .from_data(special_data, "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("store");

    let retrieved = store.get(&braid.id).await.expect("query");
    assert!(retrieved.is_some());
}
