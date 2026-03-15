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
mod tests {
    use super::*;
    use sweet_grass_core::activity::ActivityType;

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
    async fn test_put_and_get() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:test1", "did:key:z6MkTest1");

        store.put(&braid).await.expect("should store");

        let retrieved = store.get(&braid.id).await.expect("should get");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data_hash.as_str(), "sha256:test1");
    }

    #[tokio::test]
    async fn test_get_by_hash() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:hashtest", "did:key:z6MkTest");

        store.put(&braid).await.expect("should store");

        let hash = ContentHash::new("sha256:hashtest");
        let retrieved = store.get_by_hash(&hash).await.expect("should get");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_duplicate_rejection() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:dup", "did:key:z6MkTest");

        store.put(&braid).await.expect("first put should succeed");
        let result = store.put(&braid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:delete", "did:key:z6MkTest");

        store.put(&braid).await.expect("should store");
        assert!(store.exists(&braid.id).await.expect("should check"));

        let deleted = store.delete(&braid.id).await.expect("should delete");
        assert!(deleted);
        assert!(!store.exists(&braid.id).await.expect("should check"));
    }

    #[tokio::test]
    async fn test_by_agent() {
        let store = MemoryStore::new();
        let agent = "did:key:z6MkAgent1";

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

        let braids = store
            .by_agent(&Did::new(agent))
            .await
            .expect("should query");
        assert_eq!(braids.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_filter() {
        let store = MemoryStore::new();
        let agent = "did:key:z6MkQueryAgent";

        store
            .put(&make_test_braid("sha256:q1", agent))
            .await
            .expect("store");
        store
            .put(&make_test_braid("sha256:q2", agent))
            .await
            .expect("store");
        store
            .put(&make_test_braid("sha256:q3", "did:key:z6MkOther"))
            .await
            .expect("store");

        let query = QueryFilter::new().with_agent(Did::new(agent));
        let result = store
            .query(&query, QueryOrder::NewestFirst)
            .await
            .expect("should query");

        assert_eq!(result.total_count, 2);
        assert_eq!(result.braids.len(), 2);
    }

    #[tokio::test]
    async fn test_query_pagination() {
        let store = MemoryStore::new();

        for i in 0..10 {
            store
                .put(&make_test_braid(
                    &format!("sha256:page{i}"),
                    "did:key:z6MkPager",
                ))
                .await
                .expect("store");
        }

        let query = QueryFilter::new().with_limit(3).with_offset(2);
        let result = store
            .query(&query, QueryOrder::NewestFirst)
            .await
            .expect("should query");

        assert_eq!(result.total_count, 10);
        assert_eq!(result.braids.len(), 3);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn test_activity_storage() {
        let store = MemoryStore::new();
        let activity = Activity::builder(ActivityType::Computation)
            .compute_units(1.5)
            .build();

        store
            .put_activity(&activity)
            .await
            .expect("should store activity");

        let retrieved = store.get_activity(&activity.id).await.expect("should get");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_clear() {
        let store = MemoryStore::new();

        store
            .put(&make_test_braid("sha256:clear1", "did:key:z6MkClear"))
            .await
            .expect("store");

        assert!(!store.is_empty());
        store.clear();
        assert!(store.is_empty());
    }

    #[tokio::test]
    async fn test_index_rebuild() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:rebuild", "did:key:z6MkRebuild");

        store.put(&braid).await.expect("store");

        // Clear indexes manually
        store.indexes.clear();

        // Verify lookup fails
        let hash = ContentHash::new("sha256:rebuild");
        let by_hash = store.get_by_hash(&hash).await;
        assert!(by_hash.expect("should work").is_none());

        // Rebuild indexes
        store.rebuild().await.expect("should rebuild");

        // Verify lookup works again
        let by_hash = store.get_by_hash(&hash).await;
        assert!(by_hash.expect("should work").is_some());
    }

    #[tokio::test]
    async fn test_by_tag() {
        let store = MemoryStore::new();
        let mut braid = make_test_braid("sha256:tagged", "did:key:z6MkTest");
        braid.metadata.tags.push("test-tag".to_string());

        store.put(&braid).await.expect("store");

        let ids = store.by_tag("test-tag").await.expect("should query");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].as_str(), braid.id.as_str());
    }

    #[tokio::test]
    async fn test_by_mime_type() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:json", "did:key:z6MkTest");

        store.put(&braid).await.expect("store");

        let ids = store
            .by_mime_type("application/json")
            .await
            .expect("should query");
        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_put_batch() {
        let store = MemoryStore::new();
        let braids: Vec<Braid> = (0..5)
            .map(|i| make_test_braid(&format!("sha256:batch{i}"), "did:key:z6MkBatch"))
            .collect();

        let (succeeded, errors) = store.put_batch(&braids, Some(5)).await;
        assert_eq!(succeeded, 5);
        assert!(errors.is_empty());

        for braid in &braids {
            assert!(store.exists(&braid.id).await.expect("check"));
        }
    }

    #[tokio::test]
    async fn test_put_batch_with_duplicate() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:dup", "did:key:z6MkBatch");
        store.put(&braid).await.expect("first put");

        let braids = vec![braid.clone(), braid];
        let (succeeded, errors) = store.put_batch(&braids, Some(5)).await;
        assert!(
            succeeded <= 1,
            "at most one insert can succeed for duplicate"
        );
        assert!(!errors.is_empty(), "duplicate should produce errors");
        assert_eq!(succeeded + errors.len(), 2);
    }

    #[tokio::test]
    async fn test_get_batch() {
        let store = MemoryStore::new();
        let braids: Vec<Braid> = (0..4)
            .map(|i| make_test_braid(&format!("sha256:getbatch{i}"), "did:key:z6MkGetBatch"))
            .collect();

        for braid in &braids {
            store.put(braid).await.expect("store");
        }

        let ids: Vec<BraidId> = braids.iter().map(|b| b.id.clone()).collect();
        let results = store.get_batch(&ids, Some(10)).await;

        assert_eq!(results.len(), 4);
        for (i, opt) in results.iter().enumerate() {
            assert!(opt.is_some());
            assert_eq!(
                opt.as_ref().unwrap().data_hash.as_str(),
                &format!("sha256:getbatch{i}")
            );
        }
    }

    #[tokio::test]
    async fn test_get_batch_mixed_existent_nonexistent() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:batch_mixed", "did:key:z6MkTest");
        store.put(&braid).await.expect("store");

        let nonexistent_id = BraidId::from_hash(&ContentHash::new("sha256:nonexistent"));
        let ids = vec![braid.id.clone(), nonexistent_id.clone()];
        let results = store.get_batch(&ids, Some(5)).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_some());
        assert_eq!(results[0].as_ref().unwrap().id, braid.id);
        assert!(results[1].is_none());
    }

    #[tokio::test]
    async fn test_get_batch_empty() {
        let store = MemoryStore::new();
        let results = store.get_batch(&[], Some(5)).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_query_empty_store() {
        let store = MemoryStore::new();
        let query = QueryFilter::new().with_limit(10);
        let result = store
            .query(&query, QueryOrder::NewestFirst)
            .await
            .expect("should query");

        assert_eq!(result.total_count, 0);
        assert!(result.braids.is_empty());
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_query_large_offset() {
        let store = MemoryStore::new();
        store
            .put(&make_test_braid("sha256:one", "did:key:z6MkTest"))
            .await
            .expect("store");

        let query = QueryFilter::new().with_offset(100).with_limit(10);
        let result = store
            .query(&query, QueryOrder::NewestFirst)
            .await
            .expect("should query");

        assert_eq!(result.total_count, 1);
        assert!(result.braids.is_empty());
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let store = MemoryStore::new();
        let id = BraidId::from_hash(&ContentHash::new("sha256:nonexistent"));
        let deleted = store.delete(&id).await.expect("should not error");
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_activities_for_braid_empty() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:no_activities", "did:key:z6MkTest");
        store.put(&braid).await.expect("store");

        let activities = store
            .activities_for_braid(&braid.id)
            .await
            .expect("should query");
        assert!(activities.is_empty());
    }

    #[tokio::test]
    async fn test_derived_from() {
        use sweet_grass_core::entity::EntityReference;

        let store = MemoryStore::new();
        let mut braid = make_test_braid("sha256:derived", "did:key:z6MkTest");
        braid
            .was_derived_from
            .push(EntityReference::by_hash("sha256:source"));

        store.put(&braid).await.expect("store");

        let hash = ContentHash::new("sha256:source");
        let derived = store.derived_from(&hash).await.expect("should query");
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].id, braid.id);
    }

    #[tokio::test]
    async fn test_derived_from_empty() {
        let store = MemoryStore::new();
        let hash = ContentHash::new("sha256:nonexistent");
        let derived = store.derived_from(&hash).await.expect("should query");
        assert!(derived.is_empty());
    }

    #[tokio::test]
    async fn test_get_by_hash_not_found() {
        let store = MemoryStore::new();
        let hash = ContentHash::new("sha256:nonexistent");
        let result = store.get_by_hash(&hash).await.expect("should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let store = MemoryStore::new();
        let id = BraidId::from_hash(&ContentHash::new("sha256:nonexistent"));
        let result = store.get(&id).await.expect("should not error");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_index_and_unindex_braid() {
        let store = MemoryStore::new();
        let braid = make_test_braid("sha256:index_test", "did:key:z6MkTest");
        store.put(&braid).await.expect("store");

        store.unindex_braid(&braid.id).await.expect("unindex");
        let by_hash = store
            .get_by_hash(&ContentHash::new("sha256:index_test"))
            .await;
        assert!(by_hash.expect("ok").is_none());

        store.index_braid(&braid).await.expect("reindex");
        let by_hash = store
            .get_by_hash(&ContentHash::new("sha256:index_test"))
            .await;
        assert!(by_hash.expect("ok").is_some());
    }

    #[tokio::test]
    async fn test_by_time_range() {
        let store = MemoryStore::new();
        let mut braid = make_test_braid("sha256:time_range", "did:key:z6MkTest");
        braid.generated_at_time = 500;

        store.put(&braid).await.expect("store");

        let ids = store.by_time_range(400, 600).await.expect("should query");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], braid.id);

        let empty = store.by_time_range(1000, 2000).await.expect("should query");
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_count_with_filter() {
        let store = MemoryStore::new();
        let agent = "did:key:z6MkCountAgent";
        store
            .put(&make_test_braid("sha256:c1", agent))
            .await
            .expect("store");
        store
            .put(&make_test_braid("sha256:c2", agent))
            .await
            .expect("store");
        store
            .put(&make_test_braid("sha256:c3", "did:key:z6MkOther"))
            .await
            .expect("store");

        let filter = QueryFilter::new().with_agent(Did::new(agent));
        let count = store.count(&filter).await.expect("should count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_len_and_is_empty() {
        let store = MemoryStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store
            .put(&make_test_braid("sha256:len", "did:key:z6MkTest"))
            .await
            .expect("store");
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn test_default_store() {
        let store = MemoryStore::default();
        assert!(store.is_empty());
    }
}
