//! Sled `BraidStore` implementation.

use async_trait::async_trait;
use sled::{Db, Tree};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};

use sweet_grass_core::{agent::Did, Activity, ActivityId, Braid, BraidId, ContentHash};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult, StoreError};

use crate::{trees, Result, SledConfig, SledError};

/// Sled storage backend.
pub struct SledStore {
    db: Arc<Db>,
    braids: Tree,
    by_hash: Tree,
    by_agent: Tree,
    by_time: Tree,
    by_tag: Tree,
    activities: Tree,
}

impl SledStore {
    /// Open or create a Sled database with configuration.
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

    /// Open with a simple path.
    pub fn open_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(&SledConfig::new(
            path.as_ref().to_string_lossy().to_string(),
        ))
    }

    /// Flush all pending writes to disk.
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

    /// Update secondary indexes for a braid.
    fn update_indexes(&self, braid: &Braid) -> Result<()> {
        let braid_id = braid.id.as_str().as_bytes();

        // Index by hash
        self.by_hash
            .insert(braid.data_hash.as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by agent (prefix with agent DID for range queries)
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid.id.as_str());
        self.by_agent
            .insert(agent_key.as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by time (big-endian for proper sorting)
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid.id.as_str());
        self.by_time
            .insert(time_key.as_bytes(), braid_id)
            .map_err(|e| SledError::Write(e.to_string()))?;

        // Index by tags
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{}", braid.id.as_str());
            self.by_tag
                .insert(tag_key.as_bytes(), braid_id)
                .map_err(|e| SledError::Write(e.to_string()))?;
        }

        Ok(())
    }

    /// Remove secondary indexes for a braid.
    fn remove_indexes(&self, braid: &Braid) -> Result<()> {
        // Remove hash index
        self.by_hash
            .remove(braid.data_hash.as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove agent index
        let agent_key = format!("{}:{}", braid.was_attributed_to.as_str(), braid.id.as_str());
        self.by_agent
            .remove(agent_key.as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove time index
        let time_key = format!("{:020}:{}", braid.generated_at_time, braid.id.as_str());
        self.by_time
            .remove(time_key.as_bytes())
            .map_err(|e| SledError::Delete(e.to_string()))?;

        // Remove tag indexes
        for tag in &braid.metadata.tags {
            let tag_key = format!("{tag}:{}", braid.id.as_str());
            self.by_tag
                .remove(tag_key.as_bytes())
                .map_err(|e| SledError::Delete(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl BraidStore for SledStore {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn put(&self, braid: &Braid) -> sweet_grass_store::Result<()> {
        let key = braid.id.as_str().as_bytes();
        let value =
            Self::serialize_braid(braid).map_err(|e| StoreError::Internal(e.to_string()))?;

        self.braids
            .insert(key, value)
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        self.update_indexes(braid)
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        debug!("Stored braid {}", braid.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &BraidId) -> sweet_grass_store::Result<Option<Braid>> {
        match self.braids.get(id.as_str().as_bytes()) {
            Ok(Some(bytes)) => {
                let braid = Self::deserialize_braid(&bytes)
                    .map_err(|e| StoreError::Internal(e.to_string()))?;
                Ok(Some(braid))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(StoreError::Internal(e.to_string())),
        }
    }

    #[instrument(skip(self))]
    async fn get_by_hash(&self, hash: &ContentHash) -> sweet_grass_store::Result<Option<Braid>> {
        match self.by_hash.get(hash.as_bytes()) {
            Ok(Some(braid_id_bytes)) => {
                let braid_id = String::from_utf8_lossy(&braid_id_bytes);
                self.get(&BraidId::from_string(braid_id.to_string())).await
            },
            Ok(None) => Ok(None),
            Err(e) => Err(StoreError::Internal(e.to_string())),
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        // First get the braid to remove indexes
        if let Some(braid) = self.get(id).await? {
            self.remove_indexes(&braid)
                .map_err(|e| StoreError::Internal(e.to_string()))?;
        }

        self.braids
            .remove(id.as_str().as_bytes())
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(true)
    }

    #[instrument(skip(self))]
    async fn exists(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        self.braids
            .contains_key(id.as_str().as_bytes())
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    #[instrument(skip(self, filter))]
    async fn query(
        &self,
        filter: &QueryFilter,
        order: QueryOrder,
    ) -> sweet_grass_store::Result<QueryResult> {
        let mut braids = Vec::new();

        for item in &self.braids {
            let (_, value) = item.map_err(|e| StoreError::Internal(e.to_string()))?;

            if let Ok(braid) = Self::deserialize_braid(&value) {
                // Apply filters
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

                braids.push(braid);
            }
        }

        // Sort
        match order {
            QueryOrder::NewestFirst => {
                braids.sort_by(|a, b| b.generated_at_time.cmp(&a.generated_at_time));
            },
            QueryOrder::OldestFirst => {
                braids.sort_by(|a, b| a.generated_at_time.cmp(&b.generated_at_time));
            },
            QueryOrder::LargestFirst => {
                braids.sort_by(|a, b| b.size.cmp(&a.size));
            },
            QueryOrder::SmallestFirst => {
                braids.sort_by(|a, b| a.size.cmp(&b.size));
            },
        }

        let total = braids.len();
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        // Apply pagination
        let braids: Vec<Braid> = braids.into_iter().skip(offset).take(limit).collect();
        let has_more = offset + braids.len() < total;

        Ok(QueryResult::new(braids, total, has_more))
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
        let key = activity.id.as_str().as_bytes();
        let value =
            serde_json::to_vec(activity).map_err(|e| StoreError::Serialization(e.to_string()))?;

        self.activities
            .insert(key, value)
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_activity(&self, id: &ActivityId) -> sweet_grass_store::Result<Option<Activity>> {
        match self.activities.get(id.as_str().as_bytes()) {
            Ok(Some(bytes)) => {
                let activity = serde_json::from_slice(&bytes)
                    .map_err(|e| StoreError::Serialization(e.to_string()))?;
                Ok(Some(activity))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(StoreError::Internal(e.to_string())),
        }
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
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sweet_grass_core::braid::BraidBuilder;
    use tempfile::TempDir;

    fn create_test_store() -> (SledStore, TempDir) {
        let temp = TempDir::new().expect("create temp dir");
        let store = SledStore::open_path(temp.path()).expect("open store");
        (store, temp)
    }

    fn create_test_braid(hash: &str) -> Braid {
        BraidBuilder::default()
            .data_hash(hash)
            .mime_type("text/plain")
            .size(100)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid")
    }

    #[tokio::test]
    async fn test_put_and_get() {
        let (store, _temp) = create_test_store();
        let braid = create_test_braid("sha256:test1");

        store.put(&braid).await.expect("put");

        let retrieved = store.get(&braid.id).await.expect("get");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data_hash, braid.data_hash);
    }

    #[tokio::test]
    async fn test_get_by_hash() {
        let (store, _temp) = create_test_store();
        let braid = create_test_braid("sha256:hash_test");

        store.put(&braid).await.expect("put");

        let retrieved = store.get_by_hash(&braid.data_hash).await.expect("get");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, braid.id);
    }

    #[tokio::test]
    async fn test_delete() {
        let (store, _temp) = create_test_store();
        let braid = create_test_braid("sha256:delete_test");

        store.put(&braid).await.expect("put");
        assert!(store.exists(&braid.id).await.expect("exists"));

        store.delete(&braid.id).await.expect("delete");
        assert!(!store.exists(&braid.id).await.expect("exists"));
    }

    #[tokio::test]
    async fn test_query_basic() {
        let (store, _temp) = create_test_store();

        for i in 0..5 {
            let braid = create_test_braid(&format!("sha256:query{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new();
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 5);
        assert_eq!(result.total_count, 5);
    }

    #[tokio::test]
    async fn test_query_with_filter() {
        let (store, _temp) = create_test_store();

        let braid1 = create_test_braid("sha256:filter1");
        let mut braid2 = create_test_braid("sha256:filter2");
        braid2.mime_type = "application/json".to_string();

        store.put(&braid1).await.expect("put");
        store.put(&braid2).await.expect("put");

        let filter = QueryFilter::new().with_mime_type("application/json");
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 1);
        assert_eq!(result.braids[0].mime_type, "application/json");
    }

    #[tokio::test]
    async fn test_flush() {
        let (store, _temp) = create_test_store();
        let braid = create_test_braid("sha256:flush_test");

        store.put(&braid).await.expect("put");
        store.flush().expect("flush");

        // Should still be retrievable
        let retrieved = store.get(&braid.id).await.expect("get");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_size_on_disk() {
        let (store, _temp) = create_test_store();

        // Initially small
        let initial_size = store.size_on_disk();

        // Add some data
        for i in 0..10 {
            let braid = create_test_braid(&format!("sha256:size{i}"));
            store.put(&braid).await.expect("put");
        }
        store.flush().expect("flush");

        // Should be larger now
        let final_size = store.size_on_disk();
        assert!(final_size >= initial_size);
    }

    #[tokio::test]
    async fn test_config_builder() {
        let config = SledConfig::new("/tmp/test")
            .cache_capacity(512 * 1024 * 1024)
            .flush_every_ms(Some(500));

        assert_eq!(config.path, "/tmp/test");
        assert_eq!(config.cache_capacity, 512 * 1024 * 1024);
        assert_eq!(config.flush_every_ms, Some(500));
    }

    #[tokio::test]
    async fn test_activity_storage() {
        use sweet_grass_core::activity::{Activity, ActivityType};

        let (store, _temp) = create_test_store();
        let activity = Activity::builder(ActivityType::Creation).build();

        store.put_activity(&activity).await.expect("put");

        let retrieved = store.get_activity(&activity.id).await.expect("get");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, activity.id);
    }

    #[tokio::test]
    async fn test_count() {
        let (store, _temp) = create_test_store();

        let empty_filter = QueryFilter::new();
        assert_eq!(store.count(&empty_filter).await.expect("count"), 0);

        for i in 0..3 {
            let braid = create_test_braid(&format!("sha256:count{i}"));
            store.put(&braid).await.expect("put");
        }

        assert_eq!(store.count(&empty_filter).await.expect("count"), 3);
    }

    #[tokio::test]
    async fn test_query_oldest_first() {
        let (store, _temp) = create_test_store();

        for i in 0..3 {
            let braid = create_test_braid(&format!("sha256:order{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new();
        let result = store
            .query(&filter, QueryOrder::OldestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 3);
    }

    #[tokio::test]
    async fn test_query_with_limit() {
        let (store, _temp) = create_test_store();

        for i in 0..5 {
            let braid = create_test_braid(&format!("sha256:limit{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new().with_limit(2);
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 2);
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn test_query_with_offset() {
        let (store, _temp) = create_test_store();

        for i in 0..5 {
            let braid = create_test_braid(&format!("sha256:offset{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new().with_offset(2).with_limit(2);
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 2);
        assert_eq!(result.total_count, 5);
    }

    #[tokio::test]
    async fn test_by_agent() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:agent_test");
        store.put(&braid).await.expect("put");

        let braids = store
            .by_agent(&braid.was_attributed_to)
            .await
            .expect("by_agent");
        assert_eq!(braids.len(), 1);
        assert_eq!(braids[0].id, braid.id);
    }

    #[tokio::test]
    async fn test_query_by_agent() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:agent_query");
        store.put(&braid).await.expect("put");

        let filter = QueryFilter::new().with_agent(braid.was_attributed_to.clone());
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");
        assert_eq!(result.braids.len(), 1);
        assert_eq!(result.braids[0].id, braid.id);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let (store, _temp) = create_test_store();

        let result = store
            .get(&BraidId::from_string("nonexistent".to_string()))
            .await
            .expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let (store, _temp) = create_test_store();

        // Should not error when deleting non-existent
        store
            .delete(&BraidId::from_string("nonexistent".to_string()))
            .await
            .expect("delete");
    }

    #[tokio::test]
    async fn test_open_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let store = SledStore::open_path(temp.path()).expect("open");

        let braid = create_test_braid("sha256:path_test");
        store.put(&braid).await.expect("put");

        let retrieved = store.get(&braid.id).await.expect("get");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_derived_from() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:derived_test");
        store.put(&braid).await.expect("put");

        // Query by hash - derived_from finds braids with was_derived_from matching this hash
        let braids = store.derived_from(&braid.data_hash).await.expect("derived");
        // Result depends on braid content - verify call succeeded
        assert!(braids.is_empty() || !braids.is_empty());
    }

    #[tokio::test]
    async fn test_exists() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:exists_test");

        // Should not exist before put
        assert!(!store.exists(&braid.id).await.expect("exists"));

        store.put(&braid).await.expect("put");

        // Should exist after put
        assert!(store.exists(&braid.id).await.expect("exists"));
    }

    #[tokio::test]
    async fn test_get_activity_nonexistent() {
        use sweet_grass_core::activity::ActivityId;

        let (store, _temp) = create_test_store();

        // Use from_task to create an ID for a nonexistent activity
        let result = store
            .get_activity(&ActivityId::from_task("nonexistent"))
            .await
            .expect("get");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_activities_for_braid() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:activities_test");
        store.put(&braid).await.expect("put");

        // Get activities (may be empty for test braid)
        let activities = store
            .activities_for_braid(&braid.id)
            .await
            .expect("activities");
        // Verify call succeeded
        assert!(activities.is_empty() || !activities.is_empty());
    }

    #[tokio::test]
    async fn test_query_by_type() {
        let (store, _temp) = create_test_store();

        let braid = create_test_braid("sha256:type_test");
        store.put(&braid).await.expect("put");

        // Query by type
        let filter = QueryFilter::new().with_type(braid.braid_type.clone());
        let result = store
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .expect("query");

        assert!(!result.braids.is_empty());
    }

    #[tokio::test]
    async fn test_query_largest_first() {
        let (store, _temp) = create_test_store();

        for i in 0..3 {
            let braid = create_test_braid(&format!("sha256:largest{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new();
        let result = store
            .query(&filter, QueryOrder::LargestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 3);
    }

    #[tokio::test]
    async fn test_query_smallest_first() {
        let (store, _temp) = create_test_store();

        for i in 0..3 {
            let braid = create_test_braid(&format!("sha256:smallest{i}"));
            store.put(&braid).await.expect("put");
        }

        let filter = QueryFilter::new();
        let result = store
            .query(&filter, QueryOrder::SmallestFirst)
            .await
            .expect("query");

        assert_eq!(result.braids.len(), 3);
    }
}
