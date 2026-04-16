// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Core storage traits for Braid persistence.
//!
//! This module defines the primary interfaces for storing and retrieving
//! Braids and related provenance data.

use std::future::Future;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sweet_grass_core::{
    Activity, ActivityId, Braid, BraidId, BraidType, ContentHash, agent::Did, braid::Timestamp,
};

use crate::Result;

/// Default number of concurrent operations for batch put operations.
pub const DEFAULT_BATCH_CONCURRENCY: usize = 10;

/// Default maximum number of results when no limit is specified.
///
/// Shared across all store backends (memory, sled, postgres) to ensure
/// consistent pagination behavior regardless of storage choice.
pub const DEFAULT_QUERY_LIMIT: usize = 100;

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
#[non_exhaustive]
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
///
/// Uses `#[async_trait]` because `BraidStore` is used as `Arc<dyn BraidStore>`
/// for runtime backend selection (memory, redb, postgres, nestgate, sled).
/// Native `async fn in trait` does not yet support `dyn` dispatch. When Rust
/// stabilizes dyn-compatible async traits, this can migrate.
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
    /// A tuple of (`success_count`, errors) where errors contains any failures
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

        let concurrency = concurrency.unwrap_or(DEFAULT_BATCH_CONCURRENCY);

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
    /// Returns a tuple of (`results`, `errors`) — matching `put_batch` semantics.
    /// Each result corresponds positionally to the input ID: `Some(braid)` if found,
    /// `None` if the ID does not exist. Store failures are collected in `errors`
    /// and the corresponding position gets `None`.
    ///
    /// # Arguments
    /// * `ids` - Slice of braid IDs to retrieve
    /// * `concurrency` - Maximum number of concurrent operations (defaults to 20)
    ///
    /// # Example
    /// ```rust,ignore
    /// let ids = vec![id1, id2, id3];
    /// let (braids, errors) = store.get_batch(&ids, Some(20)).await;
    /// if !errors.is_empty() {
    ///     eprintln!("get_batch had {} store errors", errors.len());
    /// }
    /// ```
    async fn get_batch(
        &self,
        ids: &[BraidId],
        concurrency: Option<usize>,
    ) -> (Vec<Option<Braid>>, Vec<crate::StoreError>) {
        use futures::stream::{self, StreamExt};

        let concurrency = concurrency.unwrap_or(DEFAULT_BATCH_CONCURRENCY);

        let mut futures = Vec::with_capacity(ids.len());
        for id in ids {
            futures.push(self.get(id));
        }

        let results: Vec<Result<Option<Braid>>> =
            stream::iter(futures).buffered(concurrency).collect().await;

        let mut braids = Vec::with_capacity(results.len());
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(braid) => braids.push(braid),
                Err(e) => {
                    errors.push(e);
                    braids.push(None);
                },
            }
        }

        (braids, errors)
    }

    /// Get a Braid by content hash.
    ///
    /// When multiple braids share the same content hash (provenance
    /// convergence), returns one arbitrarily. Use
    /// [`get_all_by_hash`](Self::get_all_by_hash) to retrieve all.
    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>>;

    /// Get all Braids sharing a content hash (content convergence).
    ///
    /// Independent provenance paths may produce identical content;
    /// this method returns every braid at the convergence point.
    /// Backends that do not support 1:many hash indexing fall back
    /// to wrapping the single-result `get_by_hash`.
    async fn get_all_by_hash(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        Ok(self.get_by_hash(hash).await?.into_iter().collect())
    }

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
///
/// Uses native `impl Future + Send` (Rust 2024) instead of `#[async_trait]`
/// because this trait is never used as a trait object (`dyn IndexStore`).
pub trait IndexStore: Send + Sync {
    /// Index a Braid for search.
    fn index_braid(&self, braid: &Braid) -> impl Future<Output = Result<()>> + Send;

    /// Remove a Braid from the index.
    fn unindex_braid(&self, id: &BraidId) -> impl Future<Output = Result<()>> + Send;

    /// Get Braid IDs by tag.
    fn by_tag(&self, tag: &str) -> impl Future<Output = Result<Vec<BraidId>>> + Send;

    /// Get Braid IDs by MIME type.
    fn by_mime_type(&self, mime: &str) -> impl Future<Output = Result<Vec<BraidId>>> + Send;

    /// Get Braid IDs by time range.
    fn by_time_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> impl Future<Output = Result<Vec<BraidId>>> + Send;

    /// Rebuild indexes.
    fn rebuild(&self) -> impl Future<Output = Result<()>> + Send;
}

#[cfg(test)]
mod tests;
