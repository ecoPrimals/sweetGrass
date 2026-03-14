// SPDX-License-Identifier: AGPL-3.0-only
//! redb `BraidStore` implementation.

use async_trait::async_trait;
use redb::{Database, ReadableTable};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};

use sweet_grass_core::{agent::Did, Activity, ActivityId, Braid, BraidId, ContentHash};
use sweet_grass_store::{
    BraidStore, QueryFilter, QueryOrder, QueryResult, StoreError, DEFAULT_QUERY_LIMIT,
};

use crate::tables::{ACTIVITIES, BRAIDS, BY_AGENT, BY_HASH, BY_TAG, BY_TIME};
use crate::RedbConfig;

fn map_storage_error(e: impl std::fmt::Display) -> StoreError {
    StoreError::Internal(format!("storage error: {e}"))
}

fn map_serde_error(e: serde_json::Error) -> StoreError {
    StoreError::Serialization(e.to_string())
}

/// redb storage backend.
pub struct RedbStore {
    db: Arc<Database>,
}

impl RedbStore {
    /// Open or create a redb database with configuration.
    #[instrument(skip_all, fields(path = ?config.path))]
    pub fn open(config: &RedbConfig) -> Result<Self, StoreError> {
        debug!("Opening redb database at {:?}", config.path);

        let db = Database::create(&config.path).map_err(map_storage_error)?;

        // Create tables on first open (redb creates them implicitly)
        let write_txn = db.begin_write().map_err(map_storage_error)?;
        {
            let _ = write_txn.open_table(BRAIDS).map_err(map_storage_error)?;
            let _ = write_txn.open_table(BY_HASH).map_err(map_storage_error)?;
            let _ = write_txn.open_table(BY_AGENT).map_err(map_storage_error)?;
            let _ = write_txn.open_table(BY_TIME).map_err(map_storage_error)?;
            let _ = write_txn.open_table(BY_TAG).map_err(map_storage_error)?;
            let _ = write_txn.open_table(ACTIVITIES).map_err(map_storage_error)?;
        }
        write_txn.commit().map_err(map_storage_error)?;

        debug!("redb database opened successfully");
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Open with a simple path.
    pub fn open_path(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::open(&RedbConfig::new(path.as_ref().to_path_buf()))
    }

    /// Flush all pending writes to disk.
    ///
    /// redb commits are durable by default; this is a no-op for API compatibility.
    pub fn flush(&self) -> Result<(), StoreError> {
        Ok(())
    }

    fn serialize_braid(braid: &Braid) -> Result<Vec<u8>, StoreError> {
        serde_json::to_vec(braid).map_err(map_serde_error)
    }

    fn deserialize_braid(bytes: &[u8]) -> Result<Braid, StoreError> {
        serde_json::from_slice(bytes).map_err(map_serde_error)
    }

    fn update_indexes(
        write_txn: &redb::WriteTransaction,
        braid: &Braid,
    ) -> Result<(), StoreError> {
        let braid_id = braid.id.as_str();
        let braid_id_bytes = braid_id.as_bytes();

        let mut by_hash = write_txn.open_table(BY_HASH).map_err(map_storage_error)?;
        by_hash
            .insert(braid.data_hash.as_str(), braid_id_bytes)
            .map_err(map_storage_error)?;

        let mut by_agent = write_txn.open_table(BY_AGENT).map_err(map_storage_error)?;
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid_id);
        by_agent
            .insert(agent_key.as_str(), braid_id_bytes)
            .map_err(map_storage_error)?;

        let mut by_time = write_txn.open_table(BY_TIME).map_err(map_storage_error)?;
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid_id);
        by_time
            .insert(time_key.as_str(), braid_id_bytes)
            .map_err(map_storage_error)?;

        let mut by_tag = write_txn.open_table(BY_TAG).map_err(map_storage_error)?;
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{braid_id}");
            by_tag
                .insert(tag_key.as_str(), braid_id_bytes)
                .map_err(map_storage_error)?;
        }

        Ok(())
    }

    fn remove_indexes(
        write_txn: &redb::WriteTransaction,
        braid: &Braid,
    ) -> Result<(), StoreError> {
        let braid_id = braid.id.as_str();

        let mut by_hash = write_txn.open_table(BY_HASH).map_err(map_storage_error)?;
        by_hash
            .remove(braid.data_hash.as_str())
            .map_err(map_storage_error)?;

        let mut by_agent = write_txn.open_table(BY_AGENT).map_err(map_storage_error)?;
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid_id);
        by_agent.remove(agent_key.as_str()).map_err(map_storage_error)?;

        let mut by_time = write_txn.open_table(BY_TIME).map_err(map_storage_error)?;
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid_id);
        by_time.remove(time_key.as_str()).map_err(map_storage_error)?;

        let mut by_tag = write_txn.open_table(BY_TAG).map_err(map_storage_error)?;
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{braid_id}");
            by_tag.remove(tag_key.as_str()).map_err(map_storage_error)?;
        }

        Ok(())
    }
}

#[async_trait]
impl BraidStore for RedbStore {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn put(&self, braid: &Braid) -> sweet_grass_store::Result<()> {
        let db = Arc::clone(&self.db);
        let braid = braid.clone();

        tokio::task::spawn_blocking(move || {
            let value = Self::serialize_braid(&braid)?;
            let write_txn = db.begin_write().map_err(map_storage_error)?;
            {
                let mut braids = write_txn.open_table(BRAIDS).map_err(map_storage_error)?;
                braids
                    .insert(braid.id.as_str(), value.as_slice())
                    .map_err(map_storage_error)?;
                Self::update_indexes(&write_txn, &braid)?;
            }
            write_txn.commit().map_err(map_storage_error)?;
            debug!("Stored braid {}", braid.id);
            Ok(())
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &BraidId) -> sweet_grass_store::Result<Option<Braid>> {
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(map_storage_error)?;
            let braids = read_txn.open_table(BRAIDS).map_err(map_storage_error)?;
            match braids.get(id.as_str()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let braid = Self::deserialize_braid(bytes)?;
                    Ok(Some(braid))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(map_storage_error(e)),
            }
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get_by_hash(&self, hash: &ContentHash) -> sweet_grass_store::Result<Option<Braid>> {
        let db = Arc::clone(&self.db);
        let hash = hash.clone();

        let braid_id_opt = tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(map_storage_error)?;
            let by_hash = read_txn.open_table(BY_HASH).map_err(map_storage_error)?;
            match by_hash.get(hash.as_str()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let braid_id = String::from_utf8_lossy(bytes).to_string();
                    Ok(Some(BraidId::from_string(braid_id)))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(map_storage_error(e)),
            }
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))??;

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
            let braid = braid.clone();
            let id = id.as_str().to_string();

            tokio::task::spawn_blocking(move || {
                let write_txn = db.begin_write().map_err(map_storage_error)?;
                {
                    Self::remove_indexes(&write_txn, &braid)?;
                    let mut braids = write_txn.open_table(BRAIDS).map_err(map_storage_error)?;
                    braids.remove(id.as_str()).map_err(map_storage_error)?;
                }
                write_txn.commit().map_err(map_storage_error)?;
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
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(map_storage_error)?;
            let braids = read_txn.open_table(BRAIDS).map_err(map_storage_error)?;
            braids
                .get(id.as_str())
                .map(|opt| opt.is_some())
                .map_err(map_storage_error)
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
        let db = Arc::clone(&self.db);
        let filter = filter.clone();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(map_storage_error)?;
            let braids_table = read_txn.open_table(BRAIDS).map_err(map_storage_error)?;
            let mut braids = Vec::new();

            for item in braids_table.iter().map_err(map_storage_error)? {
                let (_, guard) = item.map_err(map_storage_error)?;
                let bytes = guard.value();
                if let Ok(braid) = Self::deserialize_braid(bytes) {
                    if let Some(hash) = &filter.data_hash {
                        if &braid.data_hash != hash {
                            continue;
                        }
                    }
                    if let Some(agent) = &filter.attributed_to {
                        if &braid.was_attributed_to != agent {
                            continue;
                        }
                    }
                    if let Some(mime) = &filter.mime_type {
                        if &braid.mime_type != mime {
                            continue;
                        }
                    }
                    if let Some(tag) = &filter.tag {
                        if !braid.metadata.tags.contains(tag) {
                            continue;
                        }
                    }
                    if let Some(bt) = &filter.braid_type {
                        if &braid.braid_type != bt {
                            continue;
                        }
                    }
                    if let Some(after) = filter.created_after {
                        if braid.generated_at_time < after {
                            continue;
                        }
                    }
                    if let Some(before) = filter.created_before {
                        if braid.generated_at_time > before {
                            continue;
                        }
                    }
                    braids.push(braid);
                }
            }

            match order {
                QueryOrder::NewestFirst => {
                    braids.sort_by(|a, b| b.generated_at_time.cmp(&a.generated_at_time));
                },
                QueryOrder::OldestFirst => {
                    braids.sort_by(|a, b| a.generated_at_time.cmp(&b.generated_at_time));
                },
                QueryOrder::LargestFirst => braids.sort_by(|a, b| b.size.cmp(&a.size)),
                QueryOrder::SmallestFirst => braids.sort_by(|a, b| a.size.cmp(&b.size)),
            }

            let total = braids.len();
            let offset = filter.offset.unwrap_or(0);
            let limit = filter.limit.unwrap_or(DEFAULT_QUERY_LIMIT);

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
            let value = serde_json::to_vec(&activity).map_err(map_serde_error)?;
            let write_txn = db.begin_write().map_err(map_storage_error)?;
            {
                let mut activities = write_txn.open_table(ACTIVITIES).map_err(map_storage_error)?;
                activities
                    .insert(activity.id.as_str(), value.as_slice())
                    .map_err(map_storage_error)?;
            }
            write_txn.commit().map_err(map_storage_error)?;
            Ok(())
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn get_activity(&self, id: &ActivityId) -> sweet_grass_store::Result<Option<Activity>> {
        let db = Arc::clone(&self.db);
        let id = id.as_str().to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(map_storage_error)?;
            let activities = read_txn.open_table(ACTIVITIES).map_err(map_storage_error)?;
            match activities.get(id.as_str()) {
                Ok(Some(guard)) => {
                    let bytes = guard.value();
                    let activity = serde_json::from_slice(bytes).map_err(map_serde_error)?;
                    Ok(Some(activity))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(map_storage_error(e)),
            }
        })
        .await
        .map_err(|e| StoreError::Internal(format!("Task join error: {e}")))?
    }

    #[instrument(skip(self))]
    async fn activities_for_braid(
        &self,
        _braid_id: &BraidId,
    ) -> sweet_grass_store::Result<Vec<Activity>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests;
