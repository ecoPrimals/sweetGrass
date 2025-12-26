//! Simplified E2E test for SweetGrass pipeline.

use std::sync::Arc;
use sweet_grass_core::{
    agent::Did,
    config::Capability,
    primal_info::SelfKnowledge,
};
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_store::memory::MemoryStore;
use sweet_grass_store::BraidStore;

#[tokio::test]
async fn test_basic_braid_workflow() {
    // Setup
    let self_knowledge = SelfKnowledge {
        name: "sweetgrass-test".to_string(),
        instance_id: "test-001".to_string(),
        capabilities: vec![Capability::Custom("provenance".to_string())],
        tarpc_port: 0,
        rest_port: 8080,
        established_at: std::time::SystemTime::now(),
    };

    let agent = Did::new("did:key:z6MkTestAgent");
    let factory = BraidFactory::from_self_knowledge(agent, &self_knowledge);
    let store = Arc::new(MemoryStore::new());
    let query_engine = QueryEngine::new(Arc::clone(&store) as Arc<dyn BraidStore>);

    // Create a Braid
    let data = b"Test data for provenance tracking";
    let braid = factory
        .from_data(data, "text/plain", None)
        .expect("should create braid");

    // Store it
    store.put(&braid).await.expect("should store braid");

    // Retrieve it
    let retrieved = store
        .get(&braid.id)
        .await
        .expect("should get braid")
        .expect("braid should exist");

    assert_eq!(retrieved.id, braid.id);
    assert_eq!(retrieved.data_hash, braid.data_hash);

    // Query provenance graph
    let braid_ref = sweet_grass_core::entity::EntityReference::by_hash(&braid.data_hash);
    let graph = query_engine
        .provenance_graph(braid_ref, Some(10))
        .await
        .expect("should query provenance");

    assert!(!graph.entities.is_empty(), "should have entities in graph");
}

#[tokio::test]
async fn test_concurrent_braid_creation() {
    // Setup
    let self_knowledge = SelfKnowledge::default();
    let agent = Did::new("did:key:z6MkConcurrentAgent");
    let store = Arc::new(MemoryStore::new());

    // Create multiple Braids concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let factory_clone = BraidFactory::from_self_knowledge(agent.clone(), &self_knowledge);
        let store = Arc::clone(&store);

        let handle = tokio::spawn(async move {
            let data = format!("Concurrent data {i}").into_bytes();
            let braid = factory_clone
                .from_data(&data, "text/plain", None)
                .expect("should create braid");

            store.put(&braid).await.expect("should store braid");
            braid.id
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let braid_ids: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("task should succeed"))
        .collect();

    assert_eq!(braid_ids.len(), 10, "should create 10 braids");

    // Verify all are stored
    for braid_id in &braid_ids {
        let retrieved = store.get(braid_id).await.expect("should get braid");
        assert!(retrieved.is_some(), "braid should be stored");
    }
}

#[tokio::test]
async fn test_provenance_graph_query() {
    // Setup
    let self_knowledge = SelfKnowledge::default();
    let agent = Did::new("did:key:z6MkGraphAgent");
    let factory = BraidFactory::from_self_knowledge(agent, &self_knowledge);
    let store = Arc::new(MemoryStore::new());
    let query_engine = QueryEngine::new(Arc::clone(&store) as Arc<dyn BraidStore>);

    // Create source Braid
    let source = factory
        .from_data(b"Source data", "text/plain", None)
        .expect("should create source");
    store.put(&source).await.expect("should store");

    // Query provenance graph
    let source_ref = sweet_grass_core::entity::EntityReference::by_hash(&source.data_hash);
    let graph = query_engine
        .provenance_graph(source_ref, Some(10))
        .await
        .expect("should query provenance");

    assert!(!graph.entities.is_empty(), "should have entities");
}

