// SPDX-License-Identifier: AGPL-3.0-only
//! `PostgreSQL` `BraidStore` implementation.

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{debug, instrument};

use sweet_grass_core::{
    activity::{Activity, ActivityId, ActivityType},
    agent::Did,
    Braid, BraidId, ContentHash,
};
use sweet_grass_store::{
    BraidStore, QueryFilter, QueryOrder, QueryResult, StoreError, DEFAULT_QUERY_LIMIT,
};

use crate::{migrations, PostgresConfig, PostgresError, Result};

// ============================================================================
// Safe integer conversion helpers for PostgreSQL storage
// ============================================================================
// PostgreSQL doesn't have unsigned 64-bit integers, so we store u64 as i64.
// These helpers ensure safe conversion with proper error handling.

/// Convert `u64` to `i64` for `PostgreSQL` storage.
/// Returns an error if the value would overflow.
fn u64_to_i64(value: u64) -> std::result::Result<i64, StoreError> {
    i64::try_from(value)
        .map_err(|_| StoreError::Internal(format!("Value {value} exceeds maximum storable size")))
}

/// Convert `i64` from `PostgreSQL` to `u64`.
/// Negative values are clamped to 0 (shouldn't happen with valid data).
fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value.max(0)).unwrap_or(0)
}

/// Convert `i64` from `PostgreSQL` to `usize` for counts/offsets.
/// Truncation on 32-bit targets is acceptable; PG row counts fit in usize.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "Clamp to 0 then cast; truncation on 32-bit is acceptable for PG counts/offsets"
)]
fn i64_to_usize(value: i64) -> usize {
    value.max(0) as usize
}

/// `PostgreSQL` storage backend.
#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    /// Connect to `PostgreSQL` with the given configuration.
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
    pub async fn connect_url(url: &str) -> Result<Self> {
        Self::connect(&PostgresConfig::new(url)).await
    }

    /// Run database migrations.
    pub async fn run_migrations(&self) -> Result<()> {
        migrations::run_migrations(&self.pool).await
    }

    /// Check if migrations are up to date.
    pub async fn check_migrations(&self) -> Result<bool> {
        migrations::check_migrations(&self.pool).await
    }

    /// Get the underlying connection pool.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Check database health.
    pub async fn health(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| true)
            .map_err(PostgresError::from)
    }

    /// Insert tags for a braid.
    async fn insert_tags(&self, braid_id: &str, tags: &[String]) -> Result<()> {
        for tag in tags {
            sqlx::query(
                "INSERT INTO braid_tags (braid_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(braid_id)
            .bind(tag)
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
        let signature_json = serde_json::to_value(&braid.signature)?;
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
        .bind(&braid.mime_type)
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
        let mut conditions = vec!["1=1".to_string()];
        let mut params: Vec<String> = vec![];

        if let Some(hash) = &filter.data_hash {
            params.push(hash.as_str().to_string());
            conditions.push(format!("data_hash = ${}", params.len()));
        }

        if let Some(agent) = &filter.attributed_to {
            params.push(agent.as_str().to_string());
            conditions.push(format!("attributed_to = ${}", params.len()));
        }

        if let Some(mime) = &filter.mime_type {
            params.push(mime.clone());
            conditions.push(format!("mime_type = ${}", params.len()));
        }

        if let Some(tag) = &filter.tag {
            params.push(tag.clone());
            conditions.push(format!(
                "braid_id IN (SELECT braid_id FROM braid_tags WHERE tag = ${})",
                params.len()
            ));
        }

        let order_clause = match order {
            QueryOrder::NewestFirst => "generated_at_time DESC",
            QueryOrder::OldestFirst => "generated_at_time ASC",
            QueryOrder::LargestFirst => "size DESC",
            QueryOrder::SmallestFirst => "size ASC",
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
            where_clause = conditions.join(" AND "),
        );

        // Build dynamic query (simplified - in production use sqlx::QueryBuilder)
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let braids: Vec<Braid> = rows
            .iter()
            .filter_map(|row| row_to_braid(row).ok())
            .collect();

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) FROM braids WHERE {}",
            conditions.join(" AND ")
        );
        let total: i64 = sqlx::query_scalar(&count_query)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

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
        let rows = sqlx::query(
            r"
            SELECT braid_id, data_hash, mime_type, size, attributed_to,
                   generated_at_time, braid_type, metadata, ecop,
                   was_derived_from, was_generated_by, signature
            FROM braids
            WHERE was_derived_from @> $1::jsonb
            ",
        )
        .bind(serde_json::json!([{"hash": hash}]))
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

/// Convert a database row to an Activity.
fn row_to_activity(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Activity> {
    use sqlx::Row;
    use sweet_grass_core::activity::{ActivityEcoPrimals, ActivityMetadata, UsedEntity};
    use sweet_grass_core::agent::AgentAssociation;

    let activity_id: String = row
        .try_get("activity_id")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let activity_type_str: String = row
        .try_get("activity_type")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let started_at_time: i64 = row
        .try_get("started_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ended_at_time: Option<i64> = row
        .try_get("ended_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let used_entities: serde_json::Value = row
        .try_get("used_entities")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_associated_with: serde_json::Value = row
        .try_get("was_associated_with")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let metadata: serde_json::Value = row
        .try_get("metadata")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ecop: serde_json::Value = row
        .try_get("ecop")
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    // Parse activity type from string
    let activity_type = parse_activity_type(&activity_type_str);

    // Parse JSON fields
    let used: Vec<UsedEntity> = serde_json::from_value(used_entities).unwrap_or_default();
    let associations: Vec<AgentAssociation> =
        serde_json::from_value(was_associated_with).unwrap_or_default();
    let meta: ActivityMetadata = serde_json::from_value(metadata).unwrap_or_default();
    let ecop_parsed: ActivityEcoPrimals = serde_json::from_value(ecop).unwrap_or_default();

    Ok(Activity {
        id: ActivityId::from_string(activity_id),
        activity_type,
        used,
        was_associated_with: associations,
        started_at_time: i64_to_u64(started_at_time),
        ended_at_time: ended_at_time.map(i64_to_u64),
        metadata: meta,
        ecop: ecop_parsed,
    })
}

/// Parse an activity type from its string representation.
fn parse_activity_type(s: &str) -> ActivityType {
    match s {
        "Creation" => ActivityType::Creation,
        "Import" => ActivityType::Import,
        "Extraction" => ActivityType::Extraction,
        "Generation" => ActivityType::Generation,
        "Transformation" => ActivityType::Transformation,
        "Derivation" => ActivityType::Derivation,
        "Aggregation" => ActivityType::Aggregation,
        "Filtering" => ActivityType::Filtering,
        "Merge" => ActivityType::Merge,
        "Split" => ActivityType::Split,
        "Analysis" => ActivityType::Analysis,
        "Computation" => ActivityType::Computation,
        "Simulation" => ActivityType::Simulation,
        "MachineLearning" => ActivityType::MachineLearning,
        "Inference" => ActivityType::Inference,
        "Experiment" => ActivityType::Experiment,
        "Observation" => ActivityType::Observation,
        "Measurement" => ActivityType::Measurement,
        "Validation" => ActivityType::Validation,
        "Editing" => ActivityType::Editing,
        "Review" => ActivityType::Review,
        "Approval" => ActivityType::Approval,
        "Publication" => ActivityType::Publication,
        "SessionStart" => ActivityType::SessionStart,
        "SessionCommit" => ActivityType::SessionCommit,
        "SessionRollback" => ActivityType::SessionRollback,
        "SliceCheckout" => ActivityType::SliceCheckout,
        "SliceReturn" => ActivityType::SliceReturn,
        "CertificateMint" => ActivityType::CertificateMint,
        "CertificateTransfer" => ActivityType::CertificateTransfer,
        "CertificateLoan" => ActivityType::CertificateLoan,
        "CertificateReturn" => ActivityType::CertificateReturn,
        other => ActivityType::Custom {
            type_uri: other.to_string(),
        },
    }
}

/// Convert a database row to a Braid.
fn row_to_braid(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Braid> {
    use sqlx::Row;

    let braid_id: String = row
        .try_get("braid_id")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let data_hash: String = row
        .try_get("data_hash")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let mime_type: String = row
        .try_get("mime_type")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let size: i64 = row
        .try_get("size")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let attributed_to: String = row
        .try_get("attributed_to")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let generated_at_time: i64 = row
        .try_get("generated_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let metadata: serde_json::Value = row
        .try_get("metadata")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ecop: serde_json::Value = row
        .try_get("ecop")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_derived_from: serde_json::Value = row
        .try_get("was_derived_from")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_generated_by: Option<serde_json::Value> = row
        .try_get("was_generated_by")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let signature: serde_json::Value = row
        .try_get("signature")
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    // Build braid using builder
    let mut builder = Braid::builder()
        .data_hash(data_hash.as_str())
        .mime_type(&mime_type)
        .size(i64_to_u64(size))
        .attributed_to(Did::new(attributed_to));

    // Parse metadata
    if let Ok(meta) = serde_json::from_value(metadata) {
        builder = builder.metadata(meta);
    }

    let mut braid = builder
        .build()
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    // Set remaining fields
    braid.id = BraidId::from_string(braid_id);
    braid.generated_at_time = i64_to_u64(generated_at_time);

    if let Ok(derived) = serde_json::from_value(was_derived_from) {
        braid.was_derived_from = derived;
    }

    if let Some(gen_by) = was_generated_by {
        if let Ok(activity) = serde_json::from_value(gen_by) {
            braid.was_generated_by = Some(activity);
        }
    }

    if let Ok(sig) = serde_json::from_value(signature) {
        braid.signature = sig;
    }

    if let Ok(ecop_parsed) = serde_json::from_value(ecop) {
        braid.ecop = ecop_parsed;
    }

    Ok(braid)
}

#[cfg(test)]
mod tests;
