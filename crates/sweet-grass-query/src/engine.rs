//! Query Engine implementation.
//!
//! Provides a high-level interface for querying Braids and
//! computing attribution chains.

use std::sync::Arc;
use sweet_grass_core::{agent::Did, entity::EntityReference, Braid, BraidId, ContentHash};
use sweet_grass_factory::{AttributionCalculator, AttributionChain};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult};

use crate::error::QueryError;
use crate::provo::{JsonLdDocument, ProvoExport};
use crate::traversal::{ProvenanceGraph, ProvenanceGraphBuilder};
use crate::Result;

// Re-export futures for parallel operations
use futures;

/// Query Engine for `SweetGrass`.
///
/// Provides a unified interface for:
/// - Basic Braid queries
/// - Provenance graph traversal
/// - Attribution chain calculation
/// - PROV-O export
pub struct QueryEngine {
    store: Arc<dyn BraidStore>,
    max_depth: u32,
}

impl QueryEngine {
    /// Create a new query engine.
    #[must_use]
    pub fn new(store: Arc<dyn BraidStore>) -> Self {
        Self {
            store,
            max_depth: 10,
        }
    }

    /// Set the maximum traversal depth.
    #[must_use]
    pub fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    // === Basic Queries ===

    /// Get a Braid by ID.
    pub async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        Ok(self.store.get(id).await?)
    }

    /// Get a Braid by content hash.
    pub async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        Ok(self.store.get_by_hash(hash).await?)
    }

    /// Query Braids with a filter.
    pub async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        Ok(self.store.query(filter, order).await?)
    }

    /// Get all Braids attributed to an agent.
    pub async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        Ok(self.store.by_agent(agent).await?)
    }

    /// Get all Braids derived from an entity.
    pub async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        Ok(self.store.derived_from(hash).await?)
    }

    /// Check if a Braid exists.
    pub async fn exists(&self, id: &BraidId) -> Result<bool> {
        Ok(self.store.exists(id).await?)
    }

    // === Provenance Queries ===

    /// Build a provenance graph from a root entity.
    pub async fn provenance_graph(
        &self,
        root: EntityReference,
        depth: Option<u32>,
    ) -> Result<ProvenanceGraph> {
        let builder = ProvenanceGraphBuilder::new()
            .max_depth(depth.unwrap_or(self.max_depth))
            .include_activities(true);

        builder.build(root, &self.store).await
    }

    /// Get ancestors of an entity (what it was derived from) in parallel.
    ///
    /// This method uses concurrent queries to fetch multiple ancestors simultaneously,
    /// improving performance for deep provenance chains.
    pub async fn ancestors_parallel(
        &self,
        hash: &ContentHash,
        depth: Option<u32>,
    ) -> Result<Vec<Braid>> {
        use futures::future::try_join_all;
        
        let max_depth = depth.unwrap_or(self.max_depth);
        let mut all_braids = Vec::new();
        let mut current_hashes = vec![hash.clone()];
        
        for _ in 0..max_depth {
            if current_hashes.is_empty() {
                break;
            }
            
            // Spawn concurrent queries for all current level hashes
            let store = Arc::clone(&self.store);
            let mut handles = Vec::new();
            
            for hash in current_hashes.drain(..) {
                let store = Arc::clone(&store);
                handles.push(tokio::spawn(async move {
                    store.get_by_hash(&hash).await
                }));
            }
            
            // Collect results
            let results = try_join_all(handles)
                .await
                .map_err(|e| QueryError::Internal(format!("Task join error: {e}")))?;
            
            let mut next_hashes = Vec::new();
            for result in results {
                if let Ok(Some(braid)) = result {
                    // Extract parent hashes for next level
                    for deriv in &braid.was_derived_from {
                        if let EntityReference::ByHash { data_hash, .. } = deriv {
                            next_hashes.push(data_hash.clone());
                        }
                    }
                    all_braids.push(braid);
                }
            }
            
            current_hashes = next_hashes;
        }
        
        Ok(all_braids)
    }

    /// Get ancestors of an entity (what it was derived from).
    pub async fn ancestors(&self, hash: &ContentHash, depth: Option<u32>) -> Result<Vec<Braid>> {
        let graph = self
            .provenance_graph(EntityReference::by_hash(hash), depth)
            .await?;

        // Return all entities except the root
        Ok(graph
            .entities
            .into_values()
            .filter(|b| &b.data_hash != hash)
            .collect())
    }

    /// Get descendants of an entity (what was derived from it).
    pub async fn descendants(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        // For descendants, we need to search forward, not backward
        // This is done by querying for Braids that have this hash as a derivation source
        self.derived_from(hash).await
    }

    // === Attribution Queries ===

    /// Calculate the attribution chain for an entity.
    pub async fn attribution_chain(&self, hash: &ContentHash) -> Result<AttributionChain> {
        let braid = self
            .get_by_hash(hash)
            .await?
            .ok_or_else(|| QueryError::NotFound(hash.clone()))?;

        let calculator = AttributionCalculator::new();

        // For now, calculate without derivation chain traversal
        // (async closures are complex; we'd need a synchronous cache or async-trait)
        Ok(calculator.calculate_single(&braid))
    }

    /// Calculate attribution for an entity including its derivation chain.
    ///
    /// This is more expensive as it traverses the full provenance graph.
    pub async fn full_attribution_chain(
        &self,
        hash: &ContentHash,
        depth: Option<u32>,
    ) -> Result<AttributionChain> {
        // First, build the provenance graph
        let graph = self
            .provenance_graph(EntityReference::by_hash(hash), depth)
            .await?;

        // Get the root braid
        let braid = graph
            .root_braid()
            .ok_or_else(|| QueryError::NotFound(hash.clone()))?;

        // Build a synchronous resolver from the graph
        let resolver = |h: &ContentHash| graph.entities.get(h).cloned();

        let calculator = AttributionCalculator::new();
        Ok(calculator.calculate_with_derivations(braid, resolver))
    }

    /// Get contributions summary for an agent.
    pub async fn agent_contributions(&self, agent: &Did) -> Result<AgentContributions> {
        let braids = self.by_agent(agent).await?;

        let total_size: u64 = braids.iter().map(|b| b.size).sum();
        let total_count = braids.len();

        // Count by MIME type
        let mut by_mime_type = std::collections::HashMap::new();
        for braid in &braids {
            *by_mime_type
                .entry(braid.mime_type.clone())
                .or_insert(0usize) += 1;
        }

        Ok(AgentContributions {
            agent: agent.clone(),
            braid_count: total_count,
            total_size,
            by_mime_type,
        })
    }

    // === Export ===

    /// Export a Braid as PROV-O JSON-LD.
    pub async fn export_braid_provo(&self, hash: &ContentHash) -> Result<JsonLdDocument> {
        let braid = self
            .get_by_hash(hash)
            .await?
            .ok_or_else(|| QueryError::NotFound(hash.clone()))?;

        let exporter = ProvoExport::new();
        exporter.export_braid(&braid)
    }

    /// Export a provenance graph as PROV-O JSON-LD.
    pub async fn export_graph_provo(
        &self,
        root: EntityReference,
        depth: Option<u32>,
    ) -> Result<JsonLdDocument> {
        let graph = self.provenance_graph(root, depth).await?;
        let exporter = ProvoExport::new();
        exporter.export_graph(&graph)
    }
}

/// Summary of an agent's contributions.
#[derive(Clone, Debug)]
pub struct AgentContributions {
    /// The agent.
    pub agent: Did,

    /// Total number of Braids.
    pub braid_count: usize,

    /// Total size of all Braids.
    pub total_size: u64,

    /// Breakdown by MIME type.
    pub by_mime_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sweet_grass_store::MemoryStore;

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
        let result = engine
            .get_by_hash(&"sha256:test1".to_string())
            .await
            .expect("should query");

        assert!(result.is_some());
        assert_eq!(result.unwrap().data_hash, "sha256:test1");
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
        let chain = engine
            .attribution_chain(&"sha256:test".to_string())
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
            .export_braid_provo(&"sha256:test".to_string())
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
            .get_by_hash(&"sha256:nonexistent".to_string())
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
            .derived_from(&"sha256:parent-df".to_string())
            .await
            .expect("should query");
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].data_hash, "sha256:child-df");
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
            .attribution_chain(&"sha256:nonexistent".to_string())
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
}
