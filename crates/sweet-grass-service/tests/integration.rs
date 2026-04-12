// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! End-to-end integration tests for `SweetGrass`.
//!
//! These tests verify the complete attribution pipeline from
//! data creation through querying and export.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::clone_on_ref_ptr,
    reason = "test file: expect/unwrap are standard in tests"
)]

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;
use sweet_grass_compression::{CompressionEngine, Session, SessionOutcome, SessionVertex};
use sweet_grass_core::{
    activity::ActivityType,
    agent::Did,
    braid::BraidMetadata,
    braid::ContentHash,
    dehydration::{DehydrationSummary, SessionOperation, Witness},
    entity::EntityReference,
    test_fixtures::TEST_SOURCE_PRIMAL,
};
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_service::{AppState, create_router};
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
    assert_eq!(&*result.braids[0].mime_type, "application/json");
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
    assert!(graph.entities.contains_key(derived.data_hash.as_str()));
    assert!(graph.entities.contains_key(source.data_hash.as_str()));
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
        title: Some("Test Entity".into()),
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
        title: Some("A".repeat(1000).into()),
        description: Some("B".repeat(5000).into()),
        tags: (0..100).map(|i| Arc::from(format!("tag-{i}"))).collect(),
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

// =============================================
// Witness chain (JSON-RPC store round-trip)
// =============================================

fn witness_chain_test_server() -> TestServer {
    let state = AppState::new_memory(Did::new("did:key:z6MkWitnessChain"));
    TestServer::new(create_router(state))
}

fn jsonrpc_envelope(method: &str, params: &serde_json::Value, id: u64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    })
}

async fn post_jsonrpc(
    server: &TestServer,
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> serde_json::Value {
    server
        .post("/jsonrpc")
        .json(&jsonrpc_envelope(method, &params, id))
        .await
        .json()
}

fn witness_audit_dehydration_summary() -> DehydrationSummary {
    let alice = Did::new("did:key:z6MkWitnessAlice");
    let bob = Did::new("did:key:z6MkWitnessBob");
    let w_unsigned = Witness::unsigned();
    let w_signed = Witness::from_ed25519(&alice, b"primalSpring-audit-sig");
    let w_hash = Witness {
        agent: bob.clone(),
        kind: "hash".to_string(),
        evidence: "sha256:checkpoint-observation".to_string(),
        witnessed_at: 9_001,
        encoding: sweet_grass_core::dehydration::WITNESS_ENCODING_HEX.to_string(),
        algorithm: None,
        tier: Some("gateway".to_string()),
        context: Some("audit:witness_chain".to_string()),
    };
    DehydrationSummary {
        source_primal: TEST_SOURCE_PRIMAL.to_string(),
        session_id: "witness-chain-dehydrate-001".to_string(),
        merkle_root: ContentHash::new("sha256:witnesschain_merkle_root"),
        vertex_count: 11,
        branch_count: 4,
        agents: vec![alice.clone(), bob],
        witnesses: vec![w_unsigned, w_signed, w_hash],
        operations: vec![SessionOperation {
            op_type: "create".to_string(),
            content_hash: ContentHash::new("sha256:witnesschain_op_artifact"),
            agent: alice,
            timestamp: 500_000,
            description: Some("witness chain op".to_string()),
        }],
        session_start: 100_000,
        dehydrated_at: 300_000,
        frontier: vec![ContentHash::new("sha256:witnesschain_frontier")],
        niche: Some("witness_audit".to_string()),
        compression_ratio: Some(0.61),
    }
}

#[tokio::test]
async fn test_witness_chain_store_roundtrip() {
    let server = witness_chain_test_server();
    let mut next_id = 1_u64;

    // 1–4: create braid via JSON-RPC, retrieve, verify default unsigned witness preserved.
    let create_body: serde_json::Value = post_jsonrpc(
        &server,
        "braid.create",
        json!({
            "data_hash": "sha256:witnesschain_roundtrip_001",
            "mime_type": "application/json",
            "size": 2048
        }),
        next_id,
    )
    .await;
    next_id += 1;
    assert!(
        create_body["error"].is_null(),
        "braid.create: {create_body}"
    );
    let created = &create_body["result"];
    let braid_id = created["@id"].as_str().expect("braid @id");
    let witness_from_create = created["witness"].clone();
    assert_eq!(witness_from_create["kind"], "marker");
    assert_eq!(witness_from_create["encoding"], "none");
    assert_eq!(witness_from_create["tier"], "open");
    assert!(witness_from_create["evidence"].as_str().unwrap().is_empty());

    let get_body: serde_json::Value =
        post_jsonrpc(&server, "braid.get", json!({ "id": braid_id }), next_id).await;
    next_id += 1;
    assert!(get_body["error"].is_null(), "braid.get: {get_body}");
    assert_eq!(get_body["result"]["witness"], witness_from_create);

    // 5–7: dehydration summary with multiple witnesses; record and verify on stored braid.
    let summary = witness_audit_dehydration_summary();
    let dehydration_params = serde_json::to_value(&summary).expect("serialize DehydrationSummary");
    let dehydrate_body: serde_json::Value = post_jsonrpc(
        &server,
        "contribution.record_dehydration",
        dehydration_params,
        next_id,
    )
    .await;
    next_id += 1;
    assert!(
        dehydrate_body["error"].is_null(),
        "contribution.record_dehydration: {dehydrate_body}"
    );
    let dehydrate_result = &dehydrate_body["result"];
    assert_eq!(dehydrate_result["session_id"], summary.session_id);
    assert_eq!(
        dehydrate_result["vertex_count"],
        serde_json::json!(summary.vertex_count)
    );
    assert_eq!(
        dehydrate_result["merkle_root"],
        summary.merkle_root.as_str()
    );
    let braid_ids = dehydrate_result["braid_ids"]
        .as_array()
        .expect("braid_ids array");
    assert_eq!(braid_ids.len(), 1);
    let dehydration_braid_id = braid_ids[0].as_str().expect("braid id string");

    let fetched: serde_json::Value = post_jsonrpc(
        &server,
        "braid.get",
        json!({ "id": dehydration_braid_id }),
        next_id,
    )
    .await;
    assert!(
        fetched["error"].is_null(),
        "braid.get dehydration: {fetched}"
    );
    let d = &fetched["result"];
    let expected_witnesses = serde_json::to_value(&summary.witnesses).expect("witnesses json");
    assert_eq!(d["ecop"]["witnesses"], expected_witnesses);
    assert_eq!(d["ecop"]["source_primal"], summary.source_primal);
    assert_eq!(d["ecop"]["rhizo_session"], summary.session_id);
    assert_eq!(
        d["ecop"]["niche"].as_str(),
        summary.niche.as_deref(),
        "niche preserved on ecop"
    );
    let compression = &d["ecop"]["compression"];
    assert_eq!(compression["vertex_count"], summary.vertex_count);
    assert_eq!(compression["branch_count"], summary.branch_count);
    assert_eq!(compression["ratio"], summary.compression_ratio.unwrap());
}
