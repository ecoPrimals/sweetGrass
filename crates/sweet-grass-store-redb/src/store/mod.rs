// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! redb `BraidStore` implementation.

use redb::{Database, ReadableTable};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};

use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
use sweet_grass_store::{
    BraidStore, DEFAULT_QUERY_LIMIT, QueryFilter, QueryOrder, QueryResult, StoreError,
};

use crate::RedbConfig;
use crate::error::RedbError;
use crate::tables::{ACTIVITIES, BRAIDS, BY_AGENT, BY_HASH, BY_TAG, BY_TIME};

/// redb storage backend.
pub struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
    /// Open or create a redb database with configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RedbError::Open`] if the database path is invalid or the file
    /// cannot be created, or a transaction error if table initialization fails.
    #[instrument(skip_all, fields(path = %config.path))]
    pub fn open(config: &RedbConfig) -> Result<Self, RedbError> {
        debug!("Opening redb database at {}", config.path);

        let path = Path::new(&config.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RedbError::Open(format!("Failed to create database directory: {e}"))
            })?;
        }
        let db = Database::create(path).map_err(|e| RedbError::Open(e.to_string()))?;

        let write_txn = db
            .begin_write()
            .map_err(|e| RedbError::Transaction(e.to_string()))?;
        {
            let _ = write_txn.open_table(BRAIDS).map_err(RedbError::from)?;
            let _ = write_txn.open_table(BY_HASH).map_err(RedbError::from)?;
            let _ = write_txn.open_table(BY_AGENT).map_err(RedbError::from)?;
            let _ = write_txn.open_table(BY_TIME).map_err(RedbError::from)?;
            let _ = write_txn.open_table(BY_TAG).map_err(RedbError::from)?;
            let _ = write_txn.open_table(ACTIVITIES).map_err(RedbError::from)?;
        }
        write_txn.commit().map_err(RedbError::from)?;

        debug!("redb database opened successfully");
        Ok(Self { db: Arc::new(db) })
    }

    /// Open with a simple path.
    ///
    /// # Errors
    ///
    /// Returns [`RedbError`] if the database cannot be opened at the given path.
    pub fn open_path(path: impl AsRef<Path>) -> Result<Self, RedbError> {
        Self::open(&RedbConfig::new(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }

    /// Flush all pending writes to disk.
    ///
    /// redb commits are durable by default; this is a no-op for API compatibility.
    ///
    /// # Errors
    ///
    /// Currently infallible; returns `Ok(())` always.
    pub fn flush(&self) -> Result<(), RedbError> {
        Ok(())
    }

    fn serialize_braid(braid: &Braid) -> Result<Vec<u8>, RedbError> {
        serde_json::to_vec(braid).map_err(RedbError::from)
    }

    fn deserialize_braid(bytes: &[u8]) -> Result<Braid, RedbError> {
        serde_json::from_slice(bytes).map_err(RedbError::from)
    }

    fn update_indexes(write_txn: &redb::WriteTransaction, braid: &Braid) -> Result<(), RedbError> {
        let braid_id = braid.id.as_str();
        let braid_id_bytes = braid_id.as_bytes();

        let mut by_hash = write_txn.open_table(BY_HASH).map_err(RedbError::from)?;
        by_hash
            .insert(braid.data_hash.as_str().as_bytes(), braid_id_bytes)
            .map_err(|e| RedbError::Write(e.to_string()))?;

        let mut by_agent = write_txn.open_table(BY_AGENT).map_err(RedbError::from)?;
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid_id);
        by_agent
            .insert(agent_key.as_bytes(), braid_id_bytes)
            .map_err(|e| RedbError::Write(e.to_string()))?;

        let mut by_time = write_txn.open_table(BY_TIME).map_err(RedbError::from)?;
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid_id);
        by_time
            .insert(time_key.as_bytes(), braid_id_bytes)
            .map_err(|e| RedbError::Write(e.to_string()))?;

        let mut by_tag = write_txn.open_table(BY_TAG).map_err(RedbError::from)?;
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{braid_id}");
            by_tag
                .insert(tag_key.as_bytes(), braid_id_bytes)
                .map_err(|e| RedbError::Write(e.to_string()))?;
        }

        Ok(())
    }

    fn remove_indexes(write_txn: &redb::WriteTransaction, braid: &Braid) -> Result<(), RedbError> {
        let braid_id = braid.id.as_str();

        let mut by_hash = write_txn.open_table(BY_HASH).map_err(RedbError::from)?;
        by_hash
            .remove(braid.data_hash.as_str().as_bytes())
            .map_err(|e| RedbError::Delete(e.to_string()))?;

        let mut by_agent = write_txn.open_table(BY_AGENT).map_err(RedbError::from)?;
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid_id);
        by_agent
            .remove(agent_key.as_bytes())
            .map_err(|e| RedbError::Delete(e.to_string()))?;

        let mut by_time = write_txn.open_table(BY_TIME).map_err(RedbError::from)?;
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid_id);
        by_time
            .remove(time_key.as_bytes())
            .map_err(|e| RedbError::Delete(e.to_string()))?;

        let mut by_tag = write_txn.open_table(BY_TAG).map_err(RedbError::from)?;
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{braid_id}");
            by_tag
                .remove(tag_key.as_bytes())
                .map_err(|e| RedbError::Delete(e.to_string()))?;
        }

        Ok(())
    }
}

impl BraidStore for RedbStore {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn put(&self, braid: &Braid) -> sweet_grass_store::Result<()> {
        let db = Arc::clone(&self.db);
        let braid = braid.clone();

        tokio::task::spawn_blocking(move || {
            let value = Self::serialize_braid(&braid).map_err(StoreError::from)?;
            let write_txn = db
                .begin_write()
                .map_err(|e| StoreError::from(RedbError::Transaction(e.to_string())))?;
            {
                let mut braids = write_txn
                    .open_table(BRAIDS)
                    .map_err(|e| StoreError::from(RedbError::from(e)))?;
                braids
                    .insert(braid.id.as_str().as_bytes(), value.as_slice())
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
                Self::update_indexes(&write_txn, &braid).map_err(StoreError::from)?;
            }
            write_txn
                .commit()
                .map_err(|e| StoreError::from(RedbError::Transaction(e.to_string())))?;
            debug!("Stored braid {}", braid.id);
            Ok(())
        })
        .await
        .map_err(StoreError::from)?
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &BraidId) -> sweet_grass_store::Result<Option<Braid>> {
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            let braids = read_txn
                .open_table(BRAIDS)
                .map_err(|e| StoreError::from(RedbError::from(e)))?;
            match braids.get(id.as_bytes()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let braid = Self::deserialize_braid(bytes).map_err(StoreError::from)?;
                    Ok(Some(braid))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(StoreError::Internal(e.to_string())),
            }
        })
        .await
        .map_err(StoreError::from)?
    }

    #[instrument(skip(self))]
    async fn get_by_hash(&self, hash: &ContentHash) -> sweet_grass_store::Result<Option<Braid>> {
        let db = Arc::clone(&self.db);
        let hash = hash.clone();

        let braid_id_opt = tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            let by_hash = read_txn
                .open_table(BY_HASH)
                .map_err(|e| StoreError::from(RedbError::from(e)))?;
            match by_hash.get(hash.as_str().as_bytes()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let braid_id = String::from_utf8_lossy(bytes).to_string();
                    Ok(Some(BraidId::from_string(braid_id)))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(StoreError::Internal(e.to_string())),
            }
        })
        .await
        .map_err(StoreError::from)??;

        match braid_id_opt {
            Some(braid_id) => self.get(&braid_id).await,
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        let braid_opt = self.get(id).await?;

        if let Some(braid) = braid_opt {
            let db = Arc::clone(&self.db);
            let id = id.as_str().to_string();

            tokio::task::spawn_blocking(move || {
                let write_txn = db
                    .begin_write()
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
                {
                    Self::remove_indexes(&write_txn, &braid).map_err(StoreError::from)?;
                    let mut braids = write_txn
                        .open_table(BRAIDS)
                        .map_err(|e| StoreError::from(RedbError::from(e)))?;
                    braids
                        .remove(id.as_bytes())
                        .map_err(|e| StoreError::Internal(e.to_string()))?;
                }
                write_txn
                    .commit()
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
                Ok(true)
            })
            .await
            .map_err(StoreError::from)?
        } else {
            Ok(true)
        }
    }

    #[instrument(skip(self))]
    async fn exists(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            let braids = read_txn
                .open_table(BRAIDS)
                .map_err(|e| StoreError::from(RedbError::from(e)))?;
            braids
                .get(id.as_bytes())
                .map(|opt| opt.is_some())
                .map_err(|e| StoreError::Internal(e.to_string()))
        })
        .await
        .map_err(StoreError::from)?
    }

    #[instrument(skip(self, filter))]
    async fn query(
        &self,
        filter: &QueryFilter,
        order: QueryOrder,
    ) -> sweet_grass_store::Result<QueryResult> {
        let db = Arc::clone(&self.db);
        let filter = filter.clone();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            let braids_table = read_txn
                .open_table(BRAIDS)
                .map_err(|e| StoreError::from(RedbError::from(e)))?;
            let mut braids = Vec::new();

            for item in braids_table
                .iter()
                .map_err(|e| StoreError::Internal(e.to_string()))?
            {
                let (_, guard) = item.map_err(|e| StoreError::Internal(e.to_string()))?;
                let bytes = guard.value();
                if let Ok(braid) = Self::deserialize_braid(bytes) {
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

            match order {
                QueryOrder::OldestFirst => {
                    braids.sort_by(|a, b| a.generated_at_time.cmp(&b.generated_at_time));
                },
                QueryOrder::LargestFirst => braids.sort_by(|a, b| b.size.cmp(&a.size)),
                QueryOrder::SmallestFirst => braids.sort_by(|a, b| a.size.cmp(&b.size)),
                QueryOrder::NewestFirst | _ => {
                    braids.sort_by(|a, b| b.generated_at_time.cmp(&a.generated_at_time));
                },
            }

            let total = braids.len();
            let offset = filter.offset.unwrap_or(0);
            let limit = filter.limit.unwrap_or(DEFAULT_QUERY_LIMIT);

            let braids: Vec<Braid> = braids.into_iter().skip(offset).take(limit).collect();
            let has_more = offset + braids.len() < total;

            Ok(QueryResult::new(braids, total, has_more))
        })
        .await
        .map_err(StoreError::from)?
    }

    #[instrument(skip(self))]
    async fn count(&self, filter: &QueryFilter) -> sweet_grass_store::Result<usize> {
        let result = self.query(filter, QueryOrder::NewestFirst).await?;
        Ok(result.total_count)
    }

    #[instrument(skip(self))]
    async fn by_agent(&self, agent: &Did) -> sweet_grass_store::Result<Vec<Braid>> {
        let filter = QueryFilter::new().with_agent(agent.clone());
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
        let db = Arc::clone(&self.db);
        let activity = activity.clone();

        tokio::task::spawn_blocking(move || {
            let value =
                serde_json::to_vec(&activity).map_err(|e| StoreError::from(RedbError::from(e)))?;
            let write_txn = db
                .begin_write()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            {
                let mut activities = write_txn
                    .open_table(ACTIVITIES)
                    .map_err(|e| StoreError::from(RedbError::from(e)))?;
                activities
                    .insert(activity.id.as_str().as_bytes(), value.as_slice())
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
            }
            write_txn
                .commit()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(StoreError::from)?
    }

    #[instrument(skip(self))]
    async fn get_activity(&self, id: &ActivityId) -> sweet_grass_store::Result<Option<Activity>> {
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| StoreError::Internal(e.to_string()))?;
            let activities = read_txn
                .open_table(ACTIVITIES)
                .map_err(|e| StoreError::from(RedbError::from(e)))?;
            match activities.get(id.as_bytes()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let activity = serde_json::from_slice(bytes)
                        .map_err(|e| StoreError::from(RedbError::from(e)))?;
                    Ok(Some(activity))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(StoreError::Internal(e.to_string())),
            }
        })
        .await
        .map_err(StoreError::from)?
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
