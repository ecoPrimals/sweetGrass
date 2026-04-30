// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Provenance graph traversal.
//!
//! This module provides tools for building and traversing
//! provenance graphs rooted at a specific entity.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sweet_grass_core::{
    Activity, Braid, ContentHash, DEFAULT_MAX_PROVENANCE_DEPTH, entity::EntityReference,
};
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

    /// Whether any cycles were detected and skipped during traversal.
    #[serde(default)]
    pub has_cycles: bool,
}

impl ProvenanceGraph {
    /// Get the root Braid if available.
    #[must_use]
    pub fn root_braid(&self) -> Option<&Braid> {
        self.root
            .content_hash()
            .and_then(|h| self.entities.get(h.as_str()))
    }

    /// Get all entity hashes.
    #[must_use]
    pub fn entity_hashes(&self) -> Vec<&str> {
        self.entities.keys().map(String::as_str).collect()
    }

    /// Get all activity IDs.
    #[must_use]
    pub fn activity_ids(&self) -> Vec<&str> {
        self.activities.keys().map(String::as_str).collect()
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
                    .any(|e| e.content_hash().map(ContentHash::as_str) == Some(hash))
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
    pub const fn new() -> Self {
        Self {
            max_depth: DEFAULT_MAX_PROVENANCE_DEPTH,
            include_activities: true,
        }
    }

    /// Set the maximum traversal depth.
    #[must_use]
    pub const fn max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set whether to include activities.
    #[must_use]
    pub const fn include_activities(mut self, include: bool) -> Self {
        self.include_activities = include;
        self
    }

    /// Build a provenance graph from a root entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails during traversal.
    pub async fn build<S: BraidStore>(
        &self,
        root: EntityReference,
        store: &Arc<S>,
    ) -> Result<ProvenanceGraph> {
        let mut graph = ProvenanceGraph {
            root: root.clone(),
            entities: HashMap::new(),
            activities: HashMap::new(),
            derivation_edges: HashMap::new(),
            generation_edges: HashMap::new(),
            depth: 0,
            truncated: false,
            has_cycles: false,
        };

        let mut visited: HashSet<ContentHash> = HashSet::new();

        if let Some(hash) = root.content_hash() {
            self.traverse(store, hash, 0, &mut graph, &mut visited)
                .await?;
        }

        Ok(graph)
    }

    fn traverse<'a, S: BraidStore>(
        &'a self,
        store: &'a Arc<S>,
        hash: &'a ContentHash,
        depth: u32,
        graph: &'a mut ProvenanceGraph,
        visited: &'a mut HashSet<ContentHash>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if depth > self.max_depth {
                graph.truncated = true;
                return Ok(());
            }

            if !visited.insert(hash.clone()) {
                graph.has_cycles = true;
                return Ok(());
            }

            if depth > graph.depth {
                graph.depth = depth;
            }

            let Some(braid) = store.get_by_hash(hash).await? else {
                return Ok(());
            };

            let hash_key = String::from(hash.as_str());

            graph.entities.insert(hash_key.clone(), braid.clone());

            if self.include_activities
                && let Some(activity) = &braid.was_generated_by
            {
                let activity_key = String::from(activity.id.as_str());
                graph
                    .activities
                    .insert(activity_key.clone(), activity.clone());
                graph
                    .generation_edges
                    .insert(hash_key.clone(), activity_key);
            }

            let parent_hashes: Vec<ContentHash> = braid
                .was_derived_from
                .iter()
                .filter_map(|e| e.content_hash().cloned())
                .collect();

            if !parent_hashes.is_empty() {
                let parent_keys: Vec<String> = parent_hashes
                    .iter()
                    .map(|h| String::from(h.as_str()))
                    .collect();
                graph.derivation_edges.insert(hash_key, parent_keys);

                for parent in &parent_hashes {
                    self.traverse(store, parent, depth + 1, graph, visited)
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
mod tests;
