//! Provenance graph traversal.
//!
//! This module provides tools for building and traversing
//! provenance graphs rooted at a specific entity.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sweet_grass_core::{entity::EntityReference, Activity, Braid, ContentHash};
use sweet_grass_store::BraidStore;

use crate::Result;

/// A provenance graph rooted at a specific entity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceGraph {
    /// Root entity reference.
    pub root: EntityReference,

    /// All entities (Braids) in the graph.
    pub entities: HashMap<String, Braid>,

    /// All activities in the graph.
    pub activities: HashMap<String, Activity>,

    /// Edges: child hash -> parent hashes.
    pub derivation_edges: HashMap<String, Vec<String>>,

    /// Edges: entity hash -> activity ID (`was_generated_by`).
    pub generation_edges: HashMap<String, String>,

    /// Maximum depth reached.
    pub depth: u32,

    /// Whether the graph was truncated due to depth limit.
    pub truncated: bool,
}

impl ProvenanceGraph {
    /// Get the root Braid if available.
    #[must_use]
    pub fn root_braid(&self) -> Option<&Braid> {
        self.root.content_hash().and_then(|h| self.entities.get(h))
    }

    /// Get all entity hashes.
    #[must_use]
    pub fn entity_hashes(&self) -> Vec<&String> {
        self.entities.keys().collect()
    }

    /// Get all activity IDs.
    #[must_use]
    pub fn activity_ids(&self) -> Vec<&String> {
        self.activities.keys().collect()
    }

    /// Get the number of entities.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the number of activities.
    #[must_use]
    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }

    /// Get parents of an entity (what it was derived from).
    #[must_use]
    pub fn parents(&self, hash: &str) -> Vec<&Braid> {
        self.derivation_edges
            .get(hash)
            .map(|parents| {
                parents
                    .iter()
                    .filter_map(|h| self.entities.get(h))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get children of an entity (what was derived from it).
    #[must_use]
    pub fn children(&self, hash: &str) -> Vec<&Braid> {
        self.entities
            .values()
            .filter(|b| {
                b.was_derived_from
                    .iter()
                    .any(|e| e.content_hash() == Some(&hash.to_string()))
            })
            .collect()
    }

    /// Get the generating activity for an entity.
    #[must_use]
    pub fn generating_activity(&self, hash: &str) -> Option<&Activity> {
        self.generation_edges
            .get(hash)
            .and_then(|id| self.activities.get(id))
    }

    /// Check if the graph contains an entity.
    #[must_use]
    pub fn contains_entity(&self, hash: &str) -> bool {
        self.entities.contains_key(hash)
    }
}

/// Builder for provenance graphs.
pub struct ProvenanceGraphBuilder {
    max_depth: u32,
    include_activities: bool,
}

impl ProvenanceGraphBuilder {
    /// Create a new builder with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_depth: 10,
            include_activities: true,
        }
    }

    /// Set the maximum traversal depth.
    #[must_use]
    pub fn max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set whether to include activities.
    #[must_use]
    pub fn include_activities(mut self, include: bool) -> Self {
        self.include_activities = include;
        self
    }

    /// Build a provenance graph from a root entity.
    pub async fn build(
        &self,
        root: EntityReference,
        store: &Arc<dyn BraidStore>,
    ) -> Result<ProvenanceGraph> {
        let mut graph = ProvenanceGraph {
            root: root.clone(),
            entities: HashMap::new(),
            activities: HashMap::new(),
            derivation_edges: HashMap::new(),
            generation_edges: HashMap::new(),
            depth: 0,
            truncated: false,
        };

        let mut visited = HashSet::new();

        if let Some(hash) = root.content_hash() {
            self.traverse(store, hash, 0, &mut graph, &mut visited)
                .await?;
        }

        Ok(graph)
    }

    fn traverse<'a>(
        &'a self,
        store: &'a Arc<dyn BraidStore>,
        hash: &'a ContentHash,
        depth: u32,
        graph: &'a mut ProvenanceGraph,
        visited: &'a mut HashSet<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Check depth limit
            if depth > self.max_depth {
                graph.truncated = true;
                return Ok(());
            }

            // Check for cycles
            if visited.contains(hash) {
                return Ok(());
            }
            visited.insert(hash.clone());

            // Update max depth
            if depth > graph.depth {
                graph.depth = depth;
            }

            // Get the Braid
            let Some(braid) = store.get_by_hash(hash).await? else {
                return Ok(()); // Entity not in store
            };

            // Add to graph
            graph.entities.insert(hash.clone(), braid.clone());

            // Add activity if present
            if self.include_activities {
                if let Some(activity) = &braid.was_generated_by {
                    let activity_id = activity.id.as_str().to_string();
                    graph
                        .activities
                        .insert(activity_id.clone(), activity.clone());
                    graph.generation_edges.insert(hash.clone(), activity_id);
                }
            }

            // Collect derivation edges
            let parent_hashes: Vec<String> = braid
                .was_derived_from
                .iter()
                .filter_map(|e| e.content_hash().cloned())
                .collect();

            if !parent_hashes.is_empty() {
                graph
                    .derivation_edges
                    .insert(hash.clone(), parent_hashes.clone());

                // Recursively traverse parents
                for parent_hash in parent_hashes {
                    self.traverse(store, &parent_hash, depth + 1, graph, visited)
                        .await?;
                }
            }

            Ok(())
        })
    }
}

impl Default for ProvenanceGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::similar_names
)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;
    use sweet_grass_store::MemoryStore;

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

        // Create a chain: child -> parent -> grandparent
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

        // Check derivation edges
        let parents = graph.parents("sha256:child");
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].data_hash, "sha256:parent");
    }

    #[tokio::test]
    async fn test_depth_limit() {
        let store = Arc::new(MemoryStore::new());

        // Create a longer chain
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
        assert!(graph.entity_count() <= 7); // root + 5 levels max
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

        // Build graph from child1 - won't include child2 since we traverse upward
        let builder = ProvenanceGraphBuilder::new();
        let graph = builder
            .build(
                EntityReference::by_hash("sha256:c1"),
                &(store as Arc<dyn BraidStore>),
            )
            .await
            .expect("should build");

        // Children of parent within this graph
        let children = graph.children("sha256:parent");
        assert_eq!(children.len(), 1); // Only child1 is in the graph
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
        assert_eq!(root.unwrap().data_hash, "sha256:root");
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
        // Activities depend on the braid content
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

        // Activities should not be included
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

        // Activity lookup - may or may not exist depending on braid content
        let activity = graph.generating_activity("sha256:gen_test");
        let _ = activity; // Just verify it doesn't panic
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
        // Just verify default() works
        let _ = builder;
    }

    #[tokio::test]
    async fn test_missing_root() {
        let store = Arc::new(MemoryStore::new());
        // Don't store anything

        let builder = ProvenanceGraphBuilder::new();
        let graph = builder
            .build(
                EntityReference::by_hash("sha256:missing"),
                &(store as Arc<dyn BraidStore>),
            )
            .await
            .expect("should build");

        // Graph should be empty since root wasn't found
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
        };

        let json = serde_json::to_string(&graph).expect("serialize");
        let parsed: ProvenanceGraph = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.depth, graph.depth);
        assert_eq!(parsed.truncated, graph.truncated);
    }
}
