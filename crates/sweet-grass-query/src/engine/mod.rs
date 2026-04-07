// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query Engine implementation.
//!
//! Provides a high-level interface for querying Braids and
//! computing attribution chains.

use std::sync::Arc;
use sweet_grass_core::{Braid, BraidId, ContentHash, agent::Did, entity::EntityReference};
use sweet_grass_factory::{AttributionCalculator, AttributionChain};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult};

use crate::Result;
use crate::error::QueryError;
use crate::provo::{JsonLdDocument, ProvoExport};
use crate::traversal::{ProvenanceGraph, ProvenanceGraphBuilder};

/// Re-export for backward compatibility.
pub use sweet_grass_core::config::DEFAULT_MAX_PROVENANCE_DEPTH as DEFAULT_MAX_DEPTH;

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
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Set the maximum traversal depth.
    #[must_use]
    pub const fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    // === Basic Queries ===

    /// Get a Braid by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        Ok(self.store.get(id).await?)
    }

    /// Get a Braid by content hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        Ok(self.store.get_by_hash(hash).await?)
    }

    /// Query Braids with a filter.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        Ok(self.store.query(filter, order).await?)
    }

    /// Get all Braids attributed to an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        Ok(self.store.by_agent(agent).await?)
    }

    /// Get all Braids derived from an entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        Ok(self.store.derived_from(hash).await?)
    }

    /// Check if a Braid exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn exists(&self, id: &BraidId) -> Result<bool> {
        Ok(self.store.exists(id).await?)
    }

    // === Provenance Queries ===

    /// Build a provenance graph from a root entity.
    ///
    /// # Errors
    ///
    /// Returns an error if traversal fails or the store operation fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if a store operation fails or a spawned task fails to join.
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

            for hash in current_hashes {
                let store = Arc::clone(&store);
                handles.push(tokio::spawn(async move { store.get_by_hash(&hash).await }));
            }

            // Collect results
            let results = try_join_all(handles)
                .await
                .map_err(|e| QueryError::Internal(format!("Task join error: {e}")))?;

            let mut next_hashes = Vec::new();
            for braid in results.into_iter().flatten().flatten() {
                // Extract parent hashes for next level
                for deriv in &braid.was_derived_from {
                    if let EntityReference::ByHash { data_hash, .. } = deriv {
                        next_hashes.push(data_hash.clone());
                    }
                }
                all_braids.push(braid);
            }

            current_hashes = next_hashes;
        }

        Ok(all_braids)
    }

    /// Get ancestors of an entity (what it was derived from).
    ///
    /// # Errors
    ///
    /// Returns an error if provenance graph building fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn descendants(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        // For descendants, we need to search forward, not backward
        // This is done by querying for Braids that have this hash as a derivation source
        self.derived_from(hash).await
    }

    // === Attribution Queries ===

    /// Calculate the attribution chain for an entity, walking its full
    /// derivation graph with decay-weighted role attribution.
    ///
    /// Builds the provenance graph up to `max_depth`, then runs
    /// `calculate_with_derivations` so parent contributors receive
    /// inherited credit through the `was_derived_from` chain.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity is not found or provenance graph building fails.
    pub async fn attribution_chain(&self, hash: &ContentHash) -> Result<AttributionChain> {
        self.full_attribution_chain(hash, None).await
    }

    /// Calculate attribution for an entity including its derivation chain,
    /// with an explicit depth limit override.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity is not found or provenance graph building fails.
    pub async fn full_attribution_chain(
        &self,
        hash: &ContentHash,
        depth: Option<u32>,
    ) -> Result<AttributionChain> {
        let graph = self
            .provenance_graph(EntityReference::by_hash(hash), depth)
            .await?;

        let braid = graph
            .root_braid()
            .ok_or_else(|| QueryError::NotFound(hash.clone()))?;

        let resolver = |h: &ContentHash| graph.entities.get(h.as_str()).cloned();

        let calculator = AttributionCalculator::new();
        Ok(calculator.calculate_with_derivations(braid, resolver))
    }

    /// Get contributions summary for an agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the store operation fails.
    pub async fn agent_contributions(&self, agent: &Did) -> Result<AgentContributions> {
        let braids = self.by_agent(agent).await?;

        let total_size: u64 = braids.iter().map(|b| b.size).sum();
        let total_count = braids.len();

        // Count by MIME type
        let mut by_mime_type = std::collections::HashMap::new();
        for braid in &braids {
            *by_mime_type
                .entry(std::sync::Arc::clone(&braid.mime_type))
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
    ///
    /// # Errors
    ///
    /// Returns an error if the entity is not found or the export operation fails.
    pub async fn export_braid_provo(&self, hash: &ContentHash) -> Result<JsonLdDocument> {
        let braid = self
            .get_by_hash(hash)
            .await?
            .ok_or_else(|| QueryError::NotFound(hash.clone()))?;

        let exporter = ProvoExport::new();
        exporter.export_braid(&braid)
    }

    /// Export a provenance graph as PROV-O JSON-LD.
    ///
    /// # Errors
    ///
    /// Returns an error if provenance graph building or export fails.
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
    ///
    /// Keyed on `Arc<str>` to share allocations with `Braid.mime_type` (O(1) clone).
    pub by_mime_type: std::collections::HashMap<std::sync::Arc<str>, usize>,
}

#[cfg(test)]
mod tests;
