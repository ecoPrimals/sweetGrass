// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Row-to-domain conversion for `PostgreSQL`.
//!
//! Pure functions that convert `PgRow` values into domain types (`Braid`,
//! `Activity`) and handle the `u64 ↔ i64` impedance mismatch between Rust
//! domain types and `PostgreSQL` storage.

use sqlx::Row;

use sweet_grass_core::{
    Braid, BraidId,
    activity::{
        Activity, ActivityEcoPrimals, ActivityId, ActivityMetadata, ActivityType, UsedEntity,
    },
    agent::{AgentAssociation, Did},
};
use sweet_grass_store::StoreError;

// ============================================================================
// Safe integer conversion helpers for PostgreSQL storage
// ============================================================================
// PostgreSQL doesn't have unsigned 64-bit integers, so we store u64 as i64.

/// Convert `u64` to `i64` for `PostgreSQL` storage.
/// Returns an error if the value would overflow.
pub fn u64_to_i64(value: u64) -> std::result::Result<i64, StoreError> {
    i64::try_from(value)
        .map_err(|_| StoreError::Internal(format!("Value {value} exceeds maximum storable size")))
}

/// Convert `i64` from `PostgreSQL` to `u64`.
/// Negative values are clamped to 0 (shouldn't happen with valid data).
pub fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value.max(0)).unwrap_or(0)
}

/// Convert `i64` from `PostgreSQL` to `usize` for counts/offsets.
/// Truncation on 32-bit targets is acceptable; PG row counts fit in usize.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "Clamp to 0 then cast; truncation on 32-bit is acceptable for PG counts/offsets"
)]
pub fn i64_to_usize(value: i64) -> usize {
    value.max(0) as usize
}

// ============================================================================
// Row-to-domain conversion
// ============================================================================

/// Convert a database row to a `Braid`.
pub fn row_to_braid(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Braid> {
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

    let mut builder = Braid::builder()
        .data_hash(data_hash.as_str())
        .mime_type(&mime_type)
        .size(i64_to_u64(size))
        .attributed_to(Did::new(attributed_to));

    if let Ok(meta) = serde_json::from_value(metadata) {
        builder = builder.metadata(meta);
    }

    let mut braid = builder
        .build()
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    braid.id = BraidId::from_string(braid_id);
    braid.generated_at_time = i64_to_u64(generated_at_time);

    if let Ok(derived) = serde_json::from_value(was_derived_from) {
        braid.was_derived_from = derived;
    }

    if let Some(gen_by) = was_generated_by
        && let Ok(activity) = serde_json::from_value(gen_by)
    {
        braid.was_generated_by = Some(activity);
    }

    if let Ok(sig) = serde_json::from_value(signature) {
        braid.signature = sig;
    }

    if let Ok(ecop_parsed) = serde_json::from_value(ecop) {
        braid.ecop = ecop_parsed;
    }

    Ok(braid)
}

/// Convert a database row to an `Activity`.
pub fn row_to_activity(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Activity> {
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

    let activity_type = parse_activity_type(&activity_type_str);

    let used: Vec<UsedEntity> = serde_json::from_value(used_entities)
        .map_err(|e| StoreError::Internal(format!("activity used_entities: {e}")))?;
    let associations: Vec<AgentAssociation> = serde_json::from_value(was_associated_with)
        .map_err(|e| StoreError::Internal(format!("activity was_associated_with: {e}")))?;
    let meta: ActivityMetadata = serde_json::from_value(metadata)
        .map_err(|e| StoreError::Internal(format!("activity metadata: {e}")))?;
    let ecop_parsed: ActivityEcoPrimals = serde_json::from_value(ecop)
        .map_err(|e| StoreError::Internal(format!("activity ecop: {e}")))?;

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
