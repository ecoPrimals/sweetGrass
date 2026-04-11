// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Sled `BraidStore` implementation.

use async_trait::async_trait;
use sled::{Db, Tree};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};

use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
use sweet_grass_store::{
    BraidStore, DEFAULT_QUERY_LIMIT, QueryFilter, QueryOrder, QueryResult, StoreError,
};

use crate::{Result, SledConfig, SledError, trees};

/// Sled storage backend.
///
/// **Deprecated**: Use `RedbStore` from `sweet-grass-store-redb` instead.
/// sled is unmaintained upstream. This backend will be removed in a future release.
#[deprecated(
    since = "0.7.26",
    note = "sled is unmaintained; migrate to redb backend"
)]
pub struct SledStore {
    db: Arc<Db>,
    braids: Tree,
    by_hash: Tree,
    by_agent: Tree,
    by_time: Tree,
    by_tag: Tree,
    activities: Tree,
}

#[expect(
    deprecated,
    reason = "impl block for deprecated SledStore — migration period"
)]
impl SledStore {
    /// Open or create a Sled database with configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SledError::Open`] if the database directory is invalid, or
    /// [`SledError::Tree`] if required trees cannot be opened.
    #[instrument(skip_all, fields(path = %config.path))]
    pub fn open(config: &SledConfig) -> Result<Self> {
        debug!("Opening Sled database at {}", config.path);

        let db = sled::Config::new()
            .path(&config.path)
            .cache_capacity(config.cache_capacity)
            .use_compression(config.use_compression)
            .flush_every_ms(config.flush_every_ms)
            .open()
            .map_err(|e| SledError::Open(e.to_string()))?;

        let braids = db
            .open_tree(trees::BRAIDS)
            .map_err(|e| SledError::Tree(e.to_string()))?;
        let by_hash = db
            .open_tree(trees::BY_HASH)
            .map_err(|e| SledError::Tree(e.to_string()))?;
        let by_agent = db
            .open_tree(trees::BY_AGENT)
            .map_err(|e| SledError::Tree(e.to_string()))?;
        let by_time = db
            .open_tree(trees::BY_TIME)
            .map_err(|e| SledError::Tree(e.to_string()))?;
        let by_tag = db
            .open_tree(trees::BY_TAG)
            .map_err(|e| SledError::Tree(e.to_string()))?;
        let activities = db
            .open_tree(trees::ACTIVITIES)
            .map_err(|e| SledError::Tree(e.to_string()))?;

        debug!("Sled database opened successfully");
        Ok(Self {
            db: Arc::new(db),
            braids,
            by_hash,
            by_agent,
            by_time,
            by_tag,
            activities,
        })
    }

    /// Cheap handles to braid + secondary index trees (`sled::Tree` clones share inner state).
    fn braid_index_trees(&self) -> (Tree, Tree, Tree, Tree, Tree) {
        (
            self.braids.clone(),
            self.by_hash.clone(),
            self.by_agent.clone(),
            self.by_time.clone(),
            self.by_tag.clone(),
        )
    }

    /// Open with a simple path.
    ///
    /// # Errors
    ///
    /// Returns [`SledError`] if the database cannot be opened at the given path.
    pub fn open_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(&SledConfig::new(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }

    /// Flush all pending writes to disk.
    ///
    /// # Errors
    ///
    /// Returns [`SledError::Write`] if the flush operation fails.
    pub fn flush(&self) -> Result<()> {
        self.db
            .flush()
            .map_err(|e| SledError::Write(e.to_string()))?;
        Ok(())
    }

    /// Get database size in bytes.
    #[must_use]
    pub fn size_on_disk(&self) -> u64 {
        self.db.size_on_disk().unwrap_or(0)
    }

    /// Serialize a Braid to bytes.
    fn serialize_braid(braid: &Braid) -> Result<Vec<u8>> {
        serde_json::to_vec(braid).map_err(SledError::from)
    }

    /// Deserialize a Braid from bytes.
    fn deserialize_braid(bytes: &[u8]) -> Result<Braid> {
        serde_json::from_slice(bytes).map_err(SledError::from)
    }

    /// Static helper for updating indexes in `spawn_blocking` context.
    fn update_indexes_blocking(
        braid: &Braid,
        by_hash: &Tree,
        by_agent: &Tree,
        by_time: &Tree,
        by_tag: &Tree,
    ) -> Result<()> {
        let braid_id = braid.id.as_str().as_bytes();

        // Index by hash
        by_hash
            .insert(braid.data_hash.as_str().as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by agent (prefix with agent DID for range queries)
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid.id.as_str());
        by_agent
            .insert(agent_key.as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by time (big-endian for proper sorting)
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid.id.as_str());
        by_time
            .insert(time_key.as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by tags
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{}", braid.id.as_str());
            by_tag
                .insert(tag_key.as_bytes(), braid_id)
                .map_err(|e| SledError::Write(e.to_string()))?;
        }

        Ok(())
    }

    /// Static helper for removing indexes in `spawn_blocking` context.
    fn remove_indexes_blocking(
        braid: &Braid,
        by_hash: &Tree,
        by_agent: &Tree,
        by_time: &Tree,
        by_tag: &Tree,
    ) -> Result<()> {
        // Remove hash index
        by_hash
            .remove(braid.data_hash.as_str().as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove agent index
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid.id.as_str());
        by_agent
            .remove(agent_key.as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove time index
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid.id.as_str());
        by_time
            .remove(time_key.as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove tag indexes
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{}", braid.id.as_str());
            by_tag
                .remove(tag_key.as_bytes())
                .map_err(|e| SledError::Delete(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait]
#[expect(
    deprecated,
    reason = "BraidStore impl for deprecated SledStore — migration period"
)]
impl BraidStore for SledStore {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn put(&self, braid: &Braid) -> sweet_grass_store::Result<()> {
        let (braids, by_hash, by_agent, by_time, by_tag) = self.braid_index_trees();
        let braid = Clone::clone(braid);

        // Wrap blocking Sled operations in spawn_blocking
        tokio::task::spawn_blocking(move || {
            let key = braid.id.as_str().as_bytes();
            let value =
                Self::serialize_braid(&braid).map_err(|e| StoreError::Internal(e.to_string()))?;

            braids
                .insert(key, value)
                .map_err(|e| StoreError::Internal(e.to_string()))?;

            // Update indexes (blocking operations)
            Self::update_indexes_blocking(&braid, &by_hash, &by_agent, &by_time, &by_tag)
                .map_err(|e| StoreError::Internal(e.to_string()))?;

            debug!("Stored braid {}", braid.id);
            Ok(())
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &BraidId) -> sweet_grass_store::Result<Option<Braid>> {
        let braids = self.braids.clone();
        let id = Clone::clone(id);

        tokio::task::spawn_blocking(move || match braids.get(id.as_str().as_bytes()) {
            Ok(Some(bytes)) => {
                let braid = Self::deserialize_braid(&bytes)
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
                Ok(Some(braid))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(StoreError::Internal(e.to_string())),
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get_by_hash(&self, hash: &ContentHash) -> sweet_grass_store::Result<Option<Braid>> {
        let by_hash = self.by_hash.clone();
        let hash = ContentHash::from(hash);

        let braid_id_opt =
            tokio::task::spawn_blocking(move || match by_hash.get(hash.as_str().as_bytes()) {
                Ok(Some(braid_id_bytes)) => {
                    let braid_id = String::from_utf8_lossy(&braid_id_bytes);
                    Ok(Some(BraidId::from_string(braid_id.into_owned())))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(StoreError::Internal(e.to_string())),
            })
            .await
            .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?;

        match braid_id_opt? {
            Some(braid_id) => self.get(&braid_id).await,
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        // First get the braid to remove indexes
        let braid_opt = self.get(id).await?;

        if let Some(braid) = braid_opt {
            let (braids, by_hash, by_agent, by_time, by_tag) = self.braid_index_trees();
            let id = Clone::clone(id);

            tokio::task::spawn_blocking(move || {
                // Remove indexes
                Self::remove_indexes_blocking(&braid, &by_hash, &by_agent, &by_time, &by_tag)
                    .map_err(|e| StoreError::Internal(e.to_string()))?;

                // Remove braid
                braids
                    .remove(id.as_str().as_bytes())
                    .map_err(|e| StoreError::Internal(e.to_string()))?;

                Ok(true)
            })
            .await
            .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
        } else {
            Ok(true)
        }
    }

    #[instrument(skip(self))]
    async fn exists(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        let braids = self.braids.clone();
        let id = Clone::clone(id);

        tokio::task::spawn_blocking(move || {
            braids
                .contains_key(id.as_str().as_bytes())
                .map_err(|e| StoreError::Internal(e.to_string()))
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self, filter))]
    async fn query(
        &self,
        filter: &QueryFilter,
        order: QueryOrder,
    ) -> sweet_grass_store::Result<QueryResult> {
        let braids_tree = self.braids.clone();
        let filter = Clone::clone(filter);

        tokio::task::spawn_blocking(move || {
            let mut braids = Vec::new();

            for item in &braids_tree {
                let (_, value) = item.map_err(|e| StoreError::Internal(e.to_string()))?;

                if let Ok(braid) = Self::deserialize_braid(&value) {
                    if let Some(hash) = &filter.data_hash
                        && &braid.data_hash != hash
                    {
                        continue;
                    }
                    if let Some(agent) = &filter.attributed_to
                        && &braid.was_attributed_to != agent
                    {
                        continue;
                    }
                    if let Some(mime) = &filter.mime_type
                        && &*braid.mime_type != mime
                    {
                        continue;
                    }
                    if let Some(tag) = &filter.tag
                        && !braid
                            .metadata
                            .tags
                            .iter()
                            .any(|t| t.as_ref() == tag.as_str())
                    {
                        continue;
                    }
                    if let Some(bt) = &filter.braid_type
                        && std::mem::discriminant(&braid.braid_type) != std::mem::discriminant(bt)
                    {
                        continue;
                    }
                    if let Some(after) = filter.created_after
                        && braid.generated_at_time < after
                    {
                        continue;
                    }
                    if let Some(before) = filter.created_before
                        && braid.generated_at_time > before
                    {
                        continue;
                    }
                    if let Some(ref primal) = filter.source_primal
                        && braid.ecop.source_primal.as_deref() != Some(primal)
                    {
                        continue;
                    }
                    if let Some(ref niche) = filter.niche
                        && braid.ecop.niche.as_deref() != Some(niche)
                    {
                        continue;
                    }
                    braids.push(braid);
                }
            }

            // Sort
            match order {
                QueryOrder::OldestFirst => {
                    braids.sort_by(|a, b| a.generated_at_time.cmp(&b.generated_at_time));
                },
                QueryOrder::LargestFirst => {
                    braids.sort_by(|a, b| b.size.cmp(&a.size));
                },
                QueryOrder::SmallestFirst => {
                    braids.sort_by(|a, b| a.size.cmp(&b.size));
                },
                QueryOrder::NewestFirst | _ => {
                    braids.sort_by(|a, b| b.generated_at_time.cmp(&a.generated_at_time));
                },
            }

            let total = braids.len();
            let offset = filter.offset.unwrap_or(0);
            let limit = filter.limit.unwrap_or(DEFAULT_QUERY_LIMIT);

            // Apply pagination
            let braids: Vec<Braid> = braids.into_iter().skip(offset).take(limit).collect();
            let has_more = offset + braids.len() < total;

            Ok(QueryResult::new(braids, total, has_more))
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn count(&self, filter: &QueryFilter) -> sweet_grass_store::Result<usize> {
        let result = self.query(filter, QueryOrder::NewestFirst).await?;
        Ok(result.total_count)
    }

    #[instrument(skip(self))]
    async fn by_agent(&self, agent: &Did) -> sweet_grass_store::Result<Vec<Braid>> {
        let filter = QueryFilter::new().with_agent(Clone::clone(agent));
        let result = self.query(&filter, QueryOrder::NewestFirst).await?;
        Ok(result.braids)
    }

    #[instrument(skip(self))]
    async fn derived_from(&self, hash: &ContentHash) -> sweet_grass_store::Result<Vec<Braid>> {
        let filter = QueryFilter::new();
        let result = self.query(&filter, QueryOrder::NewestFirst).await?;

        let braids = result
            .braids
            .into_iter()
            .filter(|b| {
                b.was_derived_from
                    .iter()
                    .any(|d| d.content_hash().is_some_and(|h| h == hash))
            })
            .collect();

        Ok(braids)
    }

    #[instrument(skip(self, activity))]
    async fn put_activity(&self, activity: &Activity) -> sweet_grass_store::Result<()> {
        let activities = self.activities.clone();
        let activity = Clone::clone(activity);

        tokio::task::spawn_blocking(move || {
            let key = activity.id.as_str().as_bytes();
            let value = serde_json::to_vec(&activity)
                .map_err(|e| StoreError::Serialization(e.to_string()))?;

            activities
                .insert(key, value)
                .map_err(|e| StoreError::Internal(e.to_string()))?;

            Ok(())
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get_activity(&self, id: &ActivityId) -> sweet_grass_store::Result<Option<Activity>> {
        let activities = self.activities.clone();
        let id = Clone::clone(id);

        tokio::task::spawn_blocking(move || match activities.get(id.as_str().as_bytes()) {
            Ok(Some(bytes)) => {
                let activity = serde_json::from_slice(&bytes)
                    .map_err(|e| StoreError::Serialization(e.to_string()))?;
                Ok(Some(activity))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(StoreError::Internal(e.to_string())),
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn activities_for_braid(
        &self,
        braid_id: &BraidId,
    ) -> sweet_grass_store::Result<Vec<Activity>> {
        let activities = self
            .get(braid_id)
            .await?
            .and_then(|b| b.was_generated_by)
            .into_iter()
            .collect();
        Ok(activities)
    }
}

#[cfg(test)]
mod tests;
