// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! [`BraidStore`](sweet_grass_store::BraidStore) implementation backed by `NestGate` `JSON-RPC`.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
use sweet_grass_store::traits::{QueryFilter, QueryOrder, QueryResult};
use sweet_grass_store::{BraidStore, StoreError};
use tracing::{debug, warn};

use crate::NestGateConfig;
use crate::client::NestGateClient;
use crate::discovery;

type Result<T> = std::result::Result<T, StoreError>;

/// `NestGate`-backed store that delegates persistence to `NestGate` via `JSON-RPC`
/// over UDS, following the ecosystem pattern where storage is `NestGate`'s domain.
///
/// Key schema:
/// - `{prefix}:braid:{id}` — Braid JSON
/// - `{prefix}:activity:{id}` — Activity JSON
/// - `{prefix}:idx:agent:{did}` — JSON array of `BraidId`s for agent index
/// - `{prefix}:idx:derived:{hash}` — JSON array of `BraidId`s for derivation index
pub struct NestGateStore {
    client: Arc<NestGateClient>,
    prefix: String,
}

impl std::fmt::Debug for NestGateStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NestGateStore")
            .field("socket", &self.client.socket_path())
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl NestGateStore {
    /// Create a new `NestGate` store from explicit configuration.
    ///
    /// # Errors
    ///
    /// Reserved for future configuration validation; currently returns [`Ok`] for all inputs.
    pub fn new(config: &NestGateConfig) -> Result<Self> {
        let env_reader = |key: &str| std::env::var(key).ok();
        Self::new_with_reader(config, &env_reader)
    }

    /// Create a new `NestGate` store with an injectable env reader (DI-friendly).
    ///
    /// # Errors
    ///
    /// Reserved for future configuration validation; currently returns [`Ok`] for all inputs.
    pub fn new_with_reader(
        config: &NestGateConfig,
        reader: &impl Fn(&str) -> Option<String>,
    ) -> Result<Self> {
        let socket_path = config.socket_path.as_ref().map_or_else(
            || discovery::discover_socket_with_family(reader, config.family_id.as_deref()),
            std::path::PathBuf::from,
        );

        debug!(
            socket = %socket_path.display(),
            prefix = %config.key_prefix,
            "NestGate store configured"
        );

        let client = Arc::new(NestGateClient::new(socket_path, config.family_id.clone()));

        Ok(Self {
            client,
            prefix: config.key_prefix.clone(),
        })
    }

    fn braid_key(&self, id: &BraidId) -> String {
        format!("{}:braid:{}", self.prefix, id.as_str())
    }

    fn activity_key(&self, id: &ActivityId) -> String {
        format!("{}:activity:{}", self.prefix, id.as_str())
    }

    fn agent_index_key(&self, did: &Did) -> String {
        format!("{}:idx:agent:{}", self.prefix, did.as_str())
    }

    fn derived_index_key(&self, hash: &ContentHash) -> String {
        format!("{}:idx:derived:{}", self.prefix, hash.as_str())
    }

    async fn store_value(&self, key: &str, value: &Value) -> Result<()> {
        let params = self.client.with_family(json!({
            "key": key,
            "value": value,
        }));
        self.client
            .call("storage.store", params)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn retrieve_value(&self, key: &str) -> Result<Option<Value>> {
        let params = self.client.with_family(json!({ "key": key }));
        let result = self
            .client
            .call("storage.retrieve", params)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let value = result.get("value").or_else(|| result.get("data"));
        match value {
            Some(v) if !v.is_null() => Ok(Some(v.clone())),
            _ => Ok(None),
        }
    }

    async fn delete_key(&self, key: &str) -> Result<bool> {
        let params = self.client.with_family(json!({ "key": key }));
        self.client
            .call("storage.delete", params)
            .await
            .map(|_| true)
            .or(Ok(false))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let params = self.client.with_family(json!({ "prefix": prefix }));
        let result = self
            .client
            .call("storage.list", params)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let keys = result
            .get("keys")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(keys)
    }

    async fn append_to_index(&self, key: &str, id: &str) -> Result<()> {
        let existing = self.retrieve_value(key).await?;
        let mut ids: Vec<String> = existing
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        if !ids.iter().any(|existing_id| existing_id == id) {
            ids.push(id.to_string());
            let val =
                serde_json::to_value(&ids).map_err(|e| StoreError::Internal(e.to_string()))?;
            self.store_value(key, &val).await?;
        }
        Ok(())
    }

    async fn remove_from_index(&self, key: &str, id: &str) -> Result<()> {
        let existing = self.retrieve_value(key).await?;
        if let Some(mut ids) = existing.and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        {
            ids.retain(|existing_id| existing_id != id);
            if ids.is_empty() {
                let _ = self.delete_key(key).await;
            } else {
                let val =
                    serde_json::to_value(&ids).map_err(|e| StoreError::Internal(e.to_string()))?;
                self.store_value(key, &val).await?;
            }
        }
        Ok(())
    }

    async fn update_indices_on_put(&self, braid: &Braid) -> Result<()> {
        let braid_id_str = braid.id.as_str();

        let agent_key = self.agent_index_key(&braid.was_attributed_to);
        self.append_to_index(&agent_key, braid_id_str).await?;

        for derived in &braid.was_derived_from {
            if let Some(hash) = derived.content_hash() {
                let key = self.derived_index_key(hash);
                self.append_to_index(&key, braid_id_str).await?;
            }
        }

        Ok(())
    }

    async fn update_indices_on_delete(&self, braid: &Braid) -> Result<()> {
        let braid_id_str = braid.id.as_str();

        let agent_key = self.agent_index_key(&braid.was_attributed_to);
        self.remove_from_index(&agent_key, braid_id_str).await?;

        for derived in &braid.was_derived_from {
            if let Some(hash) = derived.content_hash() {
                let key = self.derived_index_key(hash);
                self.remove_from_index(&key, braid_id_str).await?;
            }
        }

        Ok(())
    }

    fn matches_filter(braid: &Braid, filter: &QueryFilter) -> bool {
        if let Some(ref hash) = filter.data_hash
            && &braid.data_hash != hash
        {
            return false;
        }
        if let Some(ref agent) = filter.attributed_to
            && &braid.was_attributed_to != agent
        {
            return false;
        }
        if let Some(ref braid_type) = filter.braid_type
            && &braid.braid_type != braid_type
        {
            return false;
        }
        if let Some(ref after) = filter.created_after
            && &braid.generated_at_time < after
        {
            return false;
        }
        if let Some(ref before) = filter.created_before
            && &braid.generated_at_time > before
        {
            return false;
        }
        if let Some(ref mime) = filter.mime_type
            && !braid.mime_type.starts_with(mime.as_str())
        {
            return false;
        }
        if let Some(ref tag) = filter.tag
            && !braid
                .metadata
                .tags
                .iter()
                .any(|t| t.as_ref() == tag.as_str())
        {
            return false;
        }
        true
    }
}

#[async_trait]
impl BraidStore for NestGateStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        let key = self.braid_key(&braid.id);
        let value = serde_json::to_value(braid).map_err(|e| StoreError::Internal(e.to_string()))?;
        self.store_value(&key, &value).await?;
        self.update_indices_on_put(braid).await?;
        debug!(braid_id = %braid.id, "Stored braid in NestGate");
        Ok(())
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        let key = self.braid_key(id);
        let value = self.retrieve_value(&key).await?;
        match value {
            Some(v) => {
                let braid: Braid =
                    serde_json::from_value(v).map_err(|e| StoreError::Internal(e.to_string()))?;
                Ok(Some(braid))
            },
            None => Ok(None),
        }
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        let prefix = format!("{}:braid:", self.prefix);
        let keys = self.list_keys(&prefix).await?;

        for key in keys {
            if let Some(value) = self.retrieve_value(&key).await?
                && let Ok(braid) = serde_json::from_value::<Braid>(value)
                && &braid.data_hash == hash
            {
                return Ok(Some(braid));
            }
        }
        Ok(None)
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        if let Some(braid) = self.get(id).await? {
            self.update_indices_on_delete(&braid).await?;
            let key = self.braid_key(id);
            self.delete_key(&key).await?;
            debug!(braid_id = %id, "Deleted braid from NestGate");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        let key = self.braid_key(id);
        let params = self.client.with_family(json!({ "key": key }));
        let result = self
            .client
            .call("storage.exists", params)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;
        Ok(result
            .get("exists")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        let prefix = format!("{}:braid:", self.prefix);
        let keys = self.list_keys(&prefix).await?;

        let mut braids = Vec::new();
        for key in &keys {
            if let Some(value) = self.retrieve_value(key).await? {
                match serde_json::from_value::<Braid>(value) {
                    Ok(braid) if Self::matches_filter(&braid, filter) => {
                        braids.push(braid);
                    },
                    Ok(_) => {},
                    Err(e) => {
                        warn!(key, error = %e, "Skipping corrupt braid in NestGate");
                    },
                }
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
            _ => {},
        }

        let total = braids.len();
        let offset = filter.offset.unwrap_or(0);
        let limit = filter
            .limit
            .unwrap_or(sweet_grass_store::DEFAULT_QUERY_LIMIT);

        let page: Vec<Braid> = braids.into_iter().skip(offset).take(limit).collect();
        let has_more = offset + page.len() < total;

        Ok(QueryResult::new(page, total, has_more))
    }

    async fn count(&self, filter: &QueryFilter) -> Result<usize> {
        let is_default = filter.data_hash.is_none()
            && filter.attributed_to.is_none()
            && filter.braid_type.is_none()
            && filter.created_after.is_none()
            && filter.created_before.is_none()
            && filter.mime_type.is_none()
            && filter.tag.is_none();

        if is_default {
            let prefix = format!("{}:braid:", self.prefix);
            let keys = self.list_keys(&prefix).await?;
            return Ok(keys.len());
        }

        let result = self.query(filter, QueryOrder::NewestFirst).await?;
        Ok(result.total_count)
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        let key = self.agent_index_key(agent);
        let ids: Vec<String> = self
            .retrieve_value(&key)
            .await?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let mut braids = Vec::new();
        for id_str in ids {
            let braid_id = BraidId::from_string(id_str);
            if let Some(braid) = self.get(&braid_id).await? {
                braids.push(braid);
            }
        }
        Ok(braids)
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        let key = self.derived_index_key(hash);
        let ids: Vec<String> = self
            .retrieve_value(&key)
            .await?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let mut braids = Vec::new();
        for id_str in ids {
            let braid_id = BraidId::from_string(id_str);
            if let Some(braid) = self.get(&braid_id).await? {
                braids.push(braid);
            }
        }
        Ok(braids)
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        let key = self.activity_key(&activity.id);
        let value =
            serde_json::to_value(activity).map_err(|e| StoreError::Internal(e.to_string()))?;
        self.store_value(&key, &value).await?;
        debug!(activity_id = %activity.id, "Stored activity in NestGate");
        Ok(())
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        let key = self.activity_key(id);
        let value = self.retrieve_value(&key).await?;
        match value {
            Some(v) => {
                let activity: Activity =
                    serde_json::from_value(v).map_err(|e| StoreError::Internal(e.to_string()))?;
                Ok(Some(activity))
            },
            None => Ok(None),
        }
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        let braid = self.get(braid_id).await?;
        Ok(braid.and_then(|b| b.was_generated_by).into_iter().collect())
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::unwrap_used, reason = "test code")]
mod tests;
