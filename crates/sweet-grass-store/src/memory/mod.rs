// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! In-memory Braid store implementation.
//!
//! This is the default store for development and testing.
//! For production, use the `PostgreSQL` or `Oxigraph` backends.
//!
//! # Architecture
//!
//! The `MemoryStore` is organized into submodules:
//! - `indexes`: Secondary index management for efficient queries
//! - `filter`: Query filtering and sorting logic

mod filter;
mod indexes;

use async_trait::async_trait;
use indexmap::IndexMap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};

use sweet_grass_core::{
    Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did, braid::Timestamp,
};

use crate::Result;
use crate::error::StoreError;
use crate::traits::{BraidStore, IndexStore, QueryFilter, QueryOrder, QueryResult};

use indexes::Indexes;

/// In-memory Braid store.
///
/// Thread-safe storage for Braids and Activities.
/// Uses `IndexMap` to maintain insertion order for consistent queries.
pub struct MemoryStore {
    /// Primary Braid storage by ID.
    braids: RwLock<IndexMap<BraidId, Braid>>,

    /// Secondary indexes for efficient queries.
    indexes: Indexes,

    /// Activity storage.
    activities: RwLock<IndexMap<String, Activity>>,

    /// Index: Braid ID → Activity IDs.
    braid_activities: RwLock<HashMap<String, HashSet<String>>>,
}

impl MemoryStore {
    /// Create a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            braids: RwLock::new(IndexMap::new()),
            indexes: Indexes::new(),
            activities: RwLock::new(IndexMap::new()),
            braid_activities: RwLock::new(HashMap::new()),
        }
    }

    /// Get the number of stored Braids.
    pub fn len(&self) -> usize {
        self.braids.read().len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all data from the store.
    pub fn clear(&self) {
        self.braids.write().clear();
        self.indexes.clear();
        self.activities.write().clear();
        self.braid_activities.write().clear();
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BraidStore for MemoryStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        let id = braid.id.clone();

        if self.braids.read().contains_key(&id) {
            return Err(StoreError::Duplicate(id.as_str().to_string()));
        }

        self.indexes.add(braid);
        self.braids.write().insert(id, braid.clone());

        Ok(())
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        Ok(self.braids.read().get(id).cloned())
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        Ok(self
            .indexes
            .get_by_hash(hash.as_str())
            .and_then(|id| self.braids.read().get(&id).cloned()))
    }

    async fn get_all_by_hash(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        let ids = self.indexes.get_all_by_hash(hash.as_str());
        let braids = self.braids.read();
        Ok(ids
            .iter()
            .filter_map(|id| braids.get(id).cloned())
            .collect())
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        let braid = self.braids.write().shift_remove(id);
        Ok(braid.is_some_and(|b| {
            self.indexes.remove(&b);
            true
        }))
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        Ok(self.braids.read().contains_key(id))
    }

    async fn query(&self, query_filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        let mut matching: Vec<Braid> = {
            let braids = self.braids.read();
            braids
                .values()
                .filter(|b| filter::matches(b, query_filter))
                .cloned()
                .collect()
        };

        let total_count = matching.len();
        filter::sort(&mut matching, &order);
        let (result, has_more) = filter::paginate(matching, query_filter);

        Ok(QueryResult::new(result, total_count, has_more))
    }

    async fn count(&self, query_filter: &QueryFilter) -> Result<usize> {
        Ok(self
            .braids
            .read()
            .values()
            .filter(|b| filter::matches(b, query_filter))
            .count())
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        let ids = self.indexes.get_by_agent(agent.as_str());
        let braids = self.braids.read();
        Ok(ids
            .iter()
            .filter_map(|id| braids.get(id).cloned())
            .collect())
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        let ids = self.indexes.get_by_derivation(hash.as_str());
        let braids = self.braids.read();
        Ok(ids
            .iter()
            .filter_map(|id| braids.get(id).cloned())
            .collect())
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        let id = activity.id.as_str().to_string();
        self.activities.write().insert(id, activity.clone());
        Ok(())
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        Ok(self.activities.read().get(id.as_str()).cloned())
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        let ids = self
            .braid_activities
            .read()
            .get(braid_id.as_str())
            .cloned()
            .unwrap_or_default();

        let activities = self.activities.read();
        Ok(ids
            .iter()
            .filter_map(|id| activities.get(id).cloned())
            .collect())
    }
}

#[async_trait]
impl IndexStore for MemoryStore {
    async fn index_braid(&self, braid: &Braid) -> Result<()> {
        self.indexes.add(braid);
        Ok(())
    }

    async fn unindex_braid(&self, id: &BraidId) -> Result<()> {
        let braid = self.braids.read().get(id).cloned();
        if let Some(b) = braid {
            self.indexes.remove(&b);
        }
        Ok(())
    }

    async fn by_tag(&self, tag: &str) -> Result<Vec<BraidId>> {
        Ok(self.indexes.get_by_tag(tag).into_iter().collect())
    }

    async fn by_mime_type(&self, mime: &str) -> Result<Vec<BraidId>> {
        Ok(self.indexes.get_by_mime(mime).into_iter().collect())
    }

    async fn by_time_range(&self, start: Timestamp, end: Timestamp) -> Result<Vec<BraidId>> {
        Ok(self
            .braids
            .read()
            .values()
            .filter(|b| b.generated_at_time >= start && b.generated_at_time <= end)
            .map(|b| b.id.clone())
            .collect())
    }

    async fn rebuild(&self) -> Result<()> {
        self.indexes.clear();

        let braids: Vec<Braid> = self.braids.read().values().cloned().collect();
        for braid in &braids {
            self.indexes.add(braid);
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests;
