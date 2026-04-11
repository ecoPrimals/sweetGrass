// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `PostgreSQL` `BraidStore` implementation.

mod row_mapping;

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument};

use sweet_grass_core::{
    Braid, BraidId, ContentHash,
    activity::{Activity, ActivityId},
    agent::Did,
};
use sweet_grass_store::{
    BraidStore, DEFAULT_QUERY_LIMIT, QueryFilter, QueryOrder, QueryResult, StoreError,
};

use crate::{PostgresConfig, PostgresError, Result, migrations};
#[cfg(test)]
use row_mapping::i64_to_u64;
use row_mapping::{i64_to_usize, row_to_activity, row_to_braid, u64_to_i64};

/// Pre-validated filter parameters, ready for binding without error handling.
struct ValidatedFilter<'a> {
    filter: &'a QueryFilter,
    created_after_i64: Option<i64>,
    created_before_i64: Option<i64>,
}

impl<'a> ValidatedFilter<'a> {
    fn new(filter: &'a QueryFilter) -> sweet_grass_store::Result<Self> {
        let created_after_i64 = filter.created_after.map(u64_to_i64).transpose()?;
        let created_before_i64 = filter.created_before.map(u64_to_i64).transpose()?;
        Ok(Self {
            filter,
            created_after_i64,
            created_before_i64,
        })
    }

    fn where_clause(&self) -> String {
        let mut conditions = vec!["1=1".to_string()];
        let mut n = 0u32;
        let mut next = || {
            n += 1;
            n
        };
        if self.filter.data_hash.is_some() {
            conditions.push(format!("data_hash = ${}", next()));
        }
        if self.filter.attributed_to.is_some() {
            conditions.push(format!("attributed_to = ${}", next()));
        }
        if self.filter.created_after.is_some() {
            conditions.push(format!("generated_at_time >= ${}", next()));
        }
        if self.filter.created_before.is_some() {
            conditions.push(format!("generated_at_time <= ${}", next()));
        }
        if self.filter.mime_type.is_some() {
            conditions.push(format!("mime_type = ${}", next()));
        }
        if self.filter.tag.is_some() {
            conditions.push(format!(
                "braid_id IN (SELECT braid_id FROM braid_tags WHERE tag = ${})",
                next()
            ));
        }
        if self.filter.braid_type.is_some() {
            conditions.push(format!("braid_type = ${}", next()));
        }
        conditions.join(" AND ")
    }
}

macro_rules! bind_filter {
    ($query:expr, $vf:expr) => {{
        let mut q = $query;
        if let Some(hash) = &$vf.filter.data_hash {
            q = q.bind(hash.as_str());
        }
        if let Some(agent) = &$vf.filter.attributed_to {
            q = q.bind(agent.as_str());
        }
        if let Some(ts) = $vf.created_after_i64 {
            q = q.bind(ts);
        }
        if let Some(ts) = $vf.created_before_i64 {
            q = q.bind(ts);
        }
        if let Some(mime) = &$vf.filter.mime_type {
            q = q.bind(mime.as_str());
        }
        if let Some(tag) = &$vf.filter.tag {
            q = q.bind(tag.as_str());
        }
        if let Some(braid_type) = &$vf.filter.braid_type {
            q = q.bind(format!("{braid_type:?}"));
        }
        q
    }};
}

/// `PostgreSQL` storage backend.
#[derive(Clone, Debug)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    /// Connect to `PostgreSQL` with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresError::Connection`] if the database is unreachable or
    /// the connection pool cannot be established within the configured timeout.
    #[instrument(skip_all)]
    pub async fn connect(config: &PostgresConfig) -> Result<Self> {
        debug!("Connecting to PostgreSQL");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect(&config.database_url)
            .await
            .map_err(|e| PostgresError::Connection(e.to_string()))?;

        debug!("Connected to PostgreSQL");
        Ok(Self { pool })
    }

    /// Connect with a simple URL.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresError::Connection`] if the database is unreachable.
    pub async fn connect_url(url: &str) -> Result<Self> {
        Self::connect(&PostgresConfig::new(url)).await
    }

    /// Run database migrations.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresError`] if any migration fails to apply.
    pub async fn run_migrations(&self) -> Result<()> {
        migrations::run_migrations(&self.pool).await
    }

    /// Check if migrations are up to date.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresError`] if the migration status cannot be queried.
    pub async fn check_migrations(&self) -> Result<bool> {
        migrations::check_migrations(&self.pool).await
    }

    /// Get the underlying connection pool.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check database health.
    ///
    /// # Errors
    ///
    /// Returns [`PostgresError`] if the health-check query fails.
    pub async fn health(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| true)
            .map_err(PostgresError::from)
    }

    /// Insert tags for a braid.
    async fn insert_tags(&self, braid_id: &str, tags: &[Arc<str>]) -> Result<()> {
        for tag in tags {
            sqlx::query(
                "INSERT INTO braid_tags (braid_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(braid_id)
            .bind(tag.as_ref())
            .execute(&self.pool)
            .await
            .map_err(PostgresError::from)?;
        }
        Ok(())
    }

    /// Delete tags for a braid.
    async fn delete_tags(&self, braid_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM braid_tags WHERE braid_id = $1")
            .bind(braid_id)
            .execute(&self.pool)
            .await
            .map_err(PostgresError::from)?;
        Ok(())
    }

    /// Link a braid to an activity.
    async fn link_braid_activity(&self, braid_id: &str, activity_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO braid_activities (braid_id, activity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(braid_id)
        .bind(activity_id)
            .execute(&self.pool)
            .await
            .map_err(PostgresError::from)?;
        Ok(())
    }
}

#[async_trait]
impl BraidStore for PostgresStore {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn put(&self, braid: &Braid) -> sweet_grass_store::Result<()> {
        let braid_id = braid.id.as_str();
        let metadata_json = serde_json::to_value(&braid.metadata)?;
        let ecop_json = serde_json::to_value(&braid.ecop)?;
        let derived_from_json = serde_json::to_value(&braid.was_derived_from)?;
        let generated_by_json = braid
            .was_generated_by
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?;
        let signature_json = serde_json::to_value(&braid.witness)?;
        let braid_type = format!("{:?}", braid.braid_type);

        sqlx::query(
            r"
            INSERT INTO braids (
                braid_id, data_hash, mime_type, size, attributed_to,
                generated_at_time, braid_type, metadata, ecop,
                was_derived_from, was_generated_by, signature
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (braid_id) DO UPDATE SET
                data_hash = EXCLUDED.data_hash,
                mime_type = EXCLUDED.mime_type,
                size = EXCLUDED.size,
                attributed_to = EXCLUDED.attributed_to,
                generated_at_time = EXCLUDED.generated_at_time,
                braid_type = EXCLUDED.braid_type,
                metadata = EXCLUDED.metadata,
                ecop = EXCLUDED.ecop,
                was_derived_from = EXCLUDED.was_derived_from,
                was_generated_by = EXCLUDED.was_generated_by,
                signature = EXCLUDED.signature
            ",
        )
        .bind(braid_id)
        .bind(braid.data_hash.as_str())
        .bind(&*braid.mime_type)
        .bind(u64_to_i64(braid.size)?)
        .bind(braid.was_attributed_to.as_str())
        .bind(u64_to_i64(braid.generated_at_time)?)
        .bind(&braid_type)
        .bind(&metadata_json)
        .bind(&ecop_json)
        .bind(&derived_from_json)
        .bind(&generated_by_json)
        .bind(&signature_json)
        .execute(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        // Update tags
        self.delete_tags(braid_id)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;
        self.insert_tags(braid_id, &braid.metadata.tags)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        // Link to generating activity if present
        if let Some(activity) = &braid.was_generated_by {
            self.link_braid_activity(braid_id, activity.id.as_str())
                .await
                .map_err(|e| StoreError::Internal(e.to_string()))?;
        }

        debug!("Stored braid {}", braid_id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get(&self, id: &BraidId) -> sweet_grass_store::Result<Option<Braid>> {
        let row = sqlx::query(
            r"
            SELECT braid_id, data_hash, mime_type, size, attributed_to,
                   generated_at_time, braid_type, metadata, ecop,
                   was_derived_from, was_generated_by, signature
            FROM braids WHERE braid_id = $1
            ",
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(row_to_braid(&row)?)),
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn get_by_hash(&self, hash: &ContentHash) -> sweet_grass_store::Result<Option<Braid>> {
        let row = sqlx::query(
            r"
            SELECT braid_id, data_hash, mime_type, size, attributed_to,
                   generated_at_time, braid_type, metadata, ecop,
                   was_derived_from, was_generated_by, signature
            FROM braids WHERE data_hash = $1
            ",
        )
        .bind(hash.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(row_to_braid(&row)?)),
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        let result = sqlx::query("DELETE FROM braids WHERE braid_id = $1")
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    #[instrument(skip(self))]
    async fn exists(&self, id: &BraidId) -> sweet_grass_store::Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM braids WHERE braid_id = $1)")
                .bind(id.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(exists)
    }

    #[instrument(skip(self, filter))]
    async fn query(
        &self,
        filter: &QueryFilter,
        order: QueryOrder,
    ) -> sweet_grass_store::Result<QueryResult> {
        let vf = ValidatedFilter::new(filter)?;
        let where_clause = vf.where_clause();

        let order_clause = match order {
            QueryOrder::OldestFirst => "generated_at_time ASC",
            QueryOrder::LargestFirst => "size DESC",
            QueryOrder::SmallestFirst => "size ASC",
            QueryOrder::NewestFirst | _ => "generated_at_time DESC",
        };

        let limit = filter.limit.unwrap_or(DEFAULT_QUERY_LIMIT);
        let offset = filter.offset.unwrap_or(0);

        let query = format!(
            r"
            SELECT braid_id, data_hash, mime_type, size, attributed_to,
                   generated_at_time, braid_type, metadata, ecop,
                   was_derived_from, was_generated_by, signature
            FROM braids
            WHERE {where_clause}
            ORDER BY {order_clause}
            LIMIT {limit} OFFSET {offset}
            ",
        );

        let q = bind_filter!(sqlx::query(&query), &vf);
        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let braids: Vec<Braid> = rows
            .iter()
            .filter_map(|row| row_to_braid(row).ok())
            .collect();

        let count_query = format!("SELECT COUNT(*) FROM braids WHERE {where_clause}");
        let count_q = bind_filter!(sqlx::query_scalar::<_, i64>(&count_query), &vf);
        let total: i64 = count_q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let total_usize = i64_to_usize(total);
        let has_more = (offset + braids.len()) < total_usize;
        Ok(QueryResult::new(braids, total_usize, has_more))
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
        use sweet_grass_core::entity::EntityReference;

        let entity_ref = EntityReference::by_hash(hash.as_str());
        let filter_json = serde_json::to_value(&[entity_ref])?;

        let rows = sqlx::query(
            r"
            SELECT braid_id, data_hash, mime_type, size, attributed_to,
                   generated_at_time, braid_type, metadata, ecop,
                   was_derived_from, was_generated_by, signature
            FROM braids
            WHERE was_derived_from @> $1::jsonb
            ",
        )
        .bind(filter_json)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        let braids: Vec<Braid> = rows
            .iter()
            .filter_map(|row| row_to_braid(row).ok())
            .collect();

        Ok(braids)
    }

    #[instrument(skip(self, activity))]
    async fn put_activity(&self, activity: &Activity) -> sweet_grass_store::Result<()> {
        let _activity_json = serde_json::to_value(activity)?;

        sqlx::query(
            r"
            INSERT INTO activities (activity_id, activity_type, started_at_time, 
                                   ended_at_time, used_entities, was_associated_with,
                                   metadata, ecop)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (activity_id) DO UPDATE SET
                activity_type = EXCLUDED.activity_type,
                started_at_time = EXCLUDED.started_at_time,
                ended_at_time = EXCLUDED.ended_at_time,
                used_entities = EXCLUDED.used_entities,
                was_associated_with = EXCLUDED.was_associated_with,
                metadata = EXCLUDED.metadata,
                ecop = EXCLUDED.ecop
            ",
        )
        .bind(activity.id.as_str())
        .bind(format!("{:?}", activity.activity_type))
        .bind(u64_to_i64(activity.started_at_time)?)
        .bind(activity.ended_at_time.map(u64_to_i64).transpose()?)
        .bind(serde_json::to_value(&activity.used)?)
        .bind(serde_json::to_value(&activity.was_associated_with)?)
        .bind(serde_json::to_value(&activity.metadata)?)
        .bind(serde_json::to_value(&activity.ecop)?)
        .execute(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_activity(&self, id: &ActivityId) -> sweet_grass_store::Result<Option<Activity>> {
        let row = sqlx::query(
            r"
            SELECT activity_id, activity_type, started_at_time, ended_at_time,
                   used_entities, was_associated_with, metadata, ecop
            FROM activities WHERE activity_id = $1
            ",
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(row_to_activity(&row)?)),
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn activities_for_braid(
        &self,
        braid_id: &BraidId,
    ) -> sweet_grass_store::Result<Vec<Activity>> {
        let rows = sqlx::query(
            r"
            SELECT a.activity_id, a.activity_type, a.started_at_time, a.ended_at_time,
                   a.used_entities, a.was_associated_with, a.metadata, a.ecop
            FROM activities a
            INNER JOIN braid_activities ba ON a.activity_id = ba.activity_id
            WHERE ba.braid_id = $1
            ORDER BY a.started_at_time DESC
            ",
        )
        .bind(braid_id.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?;

        let activities: Vec<Activity> = rows
            .iter()
            .filter_map(|row| row_to_activity(row).ok())
            .collect();

        Ok(activities)
    }
}

#[cfg(test)]
mod tests;
