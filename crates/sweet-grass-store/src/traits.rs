//! Core storage traits for Braid persistence.
//!
//! This module defines the primary interfaces for storing and retrieving
//! Braids and related provenance data.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sweet_grass_core::{
    agent::Did, braid::Timestamp, Activity, ActivityId, Braid, BraidId, BraidType, ContentHash,
};

use crate::Result;

/// Query filter for searching Braids.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Filter by content hash.
    pub data_hash: Option<ContentHash>,

    /// Filter by attributed agent.
    pub attributed_to: Option<Did>,

    /// Filter by Braid type.
    pub braid_type: Option<BraidType>,

    /// Filter by minimum timestamp.
    pub created_after: Option<Timestamp>,

    /// Filter by maximum timestamp.
    pub created_before: Option<Timestamp>,

    /// Filter by MIME type prefix.
    pub mime_type: Option<String>,

    /// Filter by tag.
    pub tag: Option<String>,

    /// Filter by source primal.
    pub source_primal: Option<String>,

    /// Filter by niche.
    pub niche: Option<String>,

    /// Maximum results to return.
    pub limit: Option<usize>,

    /// Offset for pagination.
    pub offset: Option<usize>,
}

impl QueryFilter {
    /// Create a new empty filter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by content hash.
    #[must_use]
    pub fn with_hash(mut self, hash: impl Into<ContentHash>) -> Self {
        self.data_hash = Some(hash.into());
        self
    }

    /// Filter by attributed agent.
    #[must_use]
    pub fn with_agent(mut self, did: Did) -> Self {
        self.attributed_to = Some(did);
        self
    }

    /// Filter by Braid type.
    #[must_use]
    pub fn with_type(mut self, braid_type: BraidType) -> Self {
        self.braid_type = Some(braid_type);
        self
    }

    /// Filter by time range.
    #[must_use]
    pub const fn with_time_range(mut self, after: Timestamp, before: Timestamp) -> Self {
        self.created_after = Some(after);
        self.created_before = Some(before);
        self
    }

    /// Filter by MIME type.
    #[must_use]
    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    /// Filter by tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Limit results.
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination offset.
    #[must_use]
    pub const fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Query ordering options.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum QueryOrder {
    /// Order by creation time, newest first.
    #[default]
    NewestFirst,

    /// Order by creation time, oldest first.
    OldestFirst,

    /// Order by size, largest first.
    LargestFirst,

    /// Order by size, smallest first.
    SmallestFirst,
}

/// Result of a query operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    /// Braids matching the query.
    pub braids: Vec<Braid>,

    /// Total count of matching Braids (before pagination).
    pub total_count: usize,

    /// Whether there are more results.
    pub has_more: bool,
}

impl QueryResult {
    /// Create a new query result.
    #[must_use]
    pub const fn new(braids: Vec<Braid>, total_count: usize, has_more: bool) -> Self {
        Self {
            braids,
            total_count,
            has_more,
        }
    }

    /// Create an empty result.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            braids: Vec::new(),
            total_count: 0,
            has_more: false,
        }
    }
}

/// Core trait for storing and retrieving Braids.
#[async_trait]
pub trait BraidStore: Send + Sync {
    /// Store a new Braid.
    async fn put(&self, braid: &Braid) -> Result<()>;

    /// Store multiple Braids in parallel with bounded concurrency.
    ///
    /// This method provides optimized batch insertion by processing
    /// multiple braids concurrently with controlled parallelism.
    ///
    /// # Arguments
    /// * `braids` - Slice of braids to store
    /// * `concurrency` - Maximum number of concurrent operations (defaults to 10)
    ///
    /// # Returns
    /// A tuple of (success_count, errors) where errors contains any failures
    ///
    /// # Example
    /// ```rust,ignore
    /// let braids = vec![braid1, braid2, braid3];
    /// let (succeeded, errors) = store.put_batch(&braids, Some(10)).await;
    /// println!("Stored {succeeded} braids, {errors} errors", errors = errors.len());
    /// ```
    async fn put_batch(
        &self,
        braids: &[Braid],
        concurrency: Option<usize>,
    ) -> (usize, Vec<crate::StoreError>) {
        use futures::stream::{self, StreamExt};

        let concurrency = concurrency.unwrap_or(10);

        // Collect futures first, then execute in parallel
        let mut futures = Vec::with_capacity(braids.len());
        for braid in braids {
            futures.push(self.put(braid));
        }

        let results: Vec<Result<()>> = stream::iter(futures)
            .buffer_unordered(concurrency)
            .collect()
            .await;

        let mut success_count = 0;
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(()) => success_count += 1,
                Err(e) => errors.push(e),
            }
        }

        (success_count, errors)
    }

    /// Get a Braid by ID.
    async fn get(&self, id: &BraidId) -> Result<Option<Braid>>;

    /// Get multiple Braids by ID in parallel with bounded concurrency.
    ///
    /// This method provides optimized batch retrieval by processing
    /// multiple IDs concurrently with controlled parallelism.
    ///
    /// # Arguments
    /// * `ids` - Slice of braid IDs to retrieve
    /// * `concurrency` - Maximum number of concurrent operations (defaults to 20)
    ///
    /// # Returns
    /// A vector of Option<Braid>, one for each ID (None if not found)
    ///
    /// # Example
    /// ```rust,ignore
    /// let ids = vec![id1, id2, id3];
    /// let braids = store.get_batch(&ids, Some(20)).await;
    /// ```
    async fn get_batch(&self, ids: &[BraidId], concurrency: Option<usize>) -> Vec<Option<Braid>> {
        use futures::stream::{self, StreamExt};

        let concurrency = concurrency.unwrap_or(20);

        // Collect futures first, then execute in parallel
        let mut futures = Vec::with_capacity(ids.len());
        for id in ids {
            futures.push(async move { self.get(id).await.ok().flatten() });
        }

        stream::iter(futures)
            .buffer_unordered(concurrency)
            .collect()
            .await
    }

    /// Get a Braid by content hash.
    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>>;

    /// Delete a Braid.
    async fn delete(&self, id: &BraidId) -> Result<bool>;

    /// Check if a Braid exists.
    async fn exists(&self, id: &BraidId) -> Result<bool>;

    /// Query Braids with a filter.
    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult>;

    /// Count Braids matching a filter.
    async fn count(&self, filter: &QueryFilter) -> Result<usize>;

    /// Get all Braids attributed to an agent.
    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>>;

    /// Get all Braids derived from a given entity.
    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>>;

    /// Store an Activity.
    async fn put_activity(&self, activity: &Activity) -> Result<()>;

    /// Get an Activity by ID.
    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>>;

    /// Get activities associated with a Braid.
    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>>;
}

/// Index storage for efficient lookups.
#[async_trait]
pub trait IndexStore: Send + Sync {
    /// Index a Braid for search.
    async fn index_braid(&self, braid: &Braid) -> Result<()>;

    /// Remove a Braid from the index.
    async fn unindex_braid(&self, id: &BraidId) -> Result<()>;

    /// Get Braid IDs by tag.
    async fn by_tag(&self, tag: &str) -> Result<Vec<BraidId>>;

    /// Get Braid IDs by MIME type.
    async fn by_mime_type(&self, mime: &str) -> Result<Vec<BraidId>>;

    /// Get Braid IDs by time range.
    async fn by_time_range(&self, start: Timestamp, end: Timestamp) -> Result<Vec<BraidId>>;

    /// Rebuild indexes.
    async fn rebuild(&self) -> Result<()>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_query_filter_new() {
        let filter = QueryFilter::new();
        assert!(filter.data_hash.is_none());
        assert!(filter.attributed_to.is_none());
        assert!(filter.braid_type.is_none());
        assert!(filter.limit.is_none());
    }

    #[test]
    fn test_query_filter_with_hash() {
        let filter = QueryFilter::new().with_hash("sha256:abc123");
        assert!(filter.data_hash.is_some());
        assert_eq!(filter.data_hash.unwrap().as_str(), "sha256:abc123");
    }

    #[test]
    fn test_query_filter_with_agent() {
        let did = Did::new("did:key:z6MkTest");
        let filter = QueryFilter::new().with_agent(did.clone());
        assert!(filter.attributed_to.is_some());
        assert_eq!(filter.attributed_to.unwrap(), did);
    }

    #[test]
    fn test_query_filter_with_type() {
        let filter = QueryFilter::new().with_type(BraidType::Entity);
        assert!(filter.braid_type.is_some());
        assert_eq!(filter.braid_type.unwrap(), BraidType::Entity);
    }

    #[test]
    fn test_query_filter_with_time_range() {
        let filter = QueryFilter::new().with_time_range(1000, 2000);
        assert_eq!(filter.created_after, Some(1000));
        assert_eq!(filter.created_before, Some(2000));
    }

    #[test]
    fn test_query_filter_with_mime_type() {
        let filter = QueryFilter::new().with_mime_type("application/json");
        assert_eq!(filter.mime_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_query_filter_with_limit() {
        let filter = QueryFilter::new().with_limit(50);
        assert_eq!(filter.limit, Some(50));
    }

    #[test]
    fn test_query_filter_with_offset() {
        let filter = QueryFilter::new().with_offset(100);
        assert_eq!(filter.offset, Some(100));
    }

    #[test]
    fn test_query_filter_chained() {
        let filter = QueryFilter::new()
            .with_hash("sha256:test")
            .with_type(BraidType::Activity)
            .with_limit(10)
            .with_offset(5);

        assert!(filter.data_hash.is_some());
        assert_eq!(filter.braid_type, Some(BraidType::Activity));
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(5));
    }

    #[test]
    fn test_query_order_default() {
        let order = QueryOrder::default();
        assert!(matches!(order, QueryOrder::NewestFirst));
    }

    #[test]
    fn test_query_order_variants() {
        // Ensure all variants can be constructed and formatted
        assert!(!format!("{:?}", QueryOrder::NewestFirst).is_empty());
        assert!(!format!("{:?}", QueryOrder::OldestFirst).is_empty());
        assert!(!format!("{:?}", QueryOrder::LargestFirst).is_empty());
        assert!(!format!("{:?}", QueryOrder::SmallestFirst).is_empty());
    }

    #[test]
    fn test_query_result_new() {
        let result = QueryResult::new(vec![], 100, true);
        assert!(result.braids.is_empty());
        assert_eq!(result.total_count, 100);
        assert!(result.has_more);
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::empty();
        assert!(result.braids.is_empty());
        assert_eq!(result.total_count, 0);
        assert!(!result.has_more);
    }

    #[test]
    fn test_query_filter_serialization() {
        let filter = QueryFilter::new().with_hash("sha256:test").with_limit(10);

        let json = serde_json::to_string(&filter).expect("serialize");
        let parsed: QueryFilter = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.data_hash, filter.data_hash);
        assert_eq!(parsed.limit, filter.limit);
    }

    #[test]
    fn test_query_order_serialization() {
        let order = QueryOrder::OldestFirst;
        let json = serde_json::to_string(&order).expect("serialize");
        let parsed: QueryOrder = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, QueryOrder::OldestFirst));
    }
}
