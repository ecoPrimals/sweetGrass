// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid management handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sweet_grass_core::{
    Braid,
    agent::Did,
    braid::{BraidId, BraidMetadata},
    entity::EntityReference,
};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

use crate::{error::ServiceError, state::AppState};

/// Request to create a new Braid from raw data.
#[derive(Debug, Deserialize)]
pub struct CreateBraidRequest {
    /// Raw data (base64 encoded).
    pub data: String,

    /// MIME type.
    pub mime_type: String,

    /// Optional title.
    pub title: Option<String>,

    /// Optional description.
    pub description: Option<String>,

    /// Optional tags.
    pub tags: Option<Vec<String>>,
}

/// Request to create a Braid with full provenance metadata.
#[derive(Debug, Deserialize)]
pub struct CreateProvenanceBraidRequest {
    /// Content hash (existing data).
    pub data_hash: String,

    /// MIME type.
    pub mime_type: String,

    /// Data size in bytes.
    pub size: u64,

    /// Attribution DID (who created it).
    pub was_attributed_to: String,

    /// Derivation (what it was derived from).
    #[serde(default)]
    pub was_derived_from: Vec<String>,

    /// Optional tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Response for Braid creation.
#[derive(Debug, Serialize)]
pub struct CreateBraidResponse {
    /// Created Braid ID.
    pub id: String,

    /// Content hash.
    pub hash: String,
}

/// Query parameters for listing Braids.
#[derive(Debug, Deserialize)]
pub struct ListBraidsQuery {
    /// Filter by agent DID.
    pub agent: Option<String>,

    /// Filter by tag.
    pub tag: Option<String>,

    /// Filter by MIME type.
    pub mime_type: Option<String>,

    /// Limit results.
    pub limit: Option<usize>,

    /// Offset for pagination.
    pub offset: Option<usize>,

    /// Order by (newest, oldest, largest, smallest).
    pub order: Option<String>,
}

/// Paginated list response.
#[derive(Debug, Serialize)]
pub struct ListBraidsResponse {
    /// Braids on this page.
    pub braids: Vec<Braid>,

    /// Total matching Braids.
    pub total: usize,

    /// Whether there are more results.
    pub has_more: bool,
}

/// Get a Braid by ID.
///
/// # Errors
///
/// Returns an error if the store lookup fails or the Braid is not found.
pub async fn get_braid(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Braid>, ServiceError> {
    let braid_id = BraidId::from_string(id);

    let braid = state
        .store
        .get(&braid_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Braid not found: {braid_id}")))?;

    Ok(Json(braid))
}

/// Get a Braid by content hash.
///
/// # Errors
///
/// Returns an error if the store lookup fails or the Braid is not found.
pub async fn get_braid_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<Braid>, ServiceError> {
    let content_hash = sweet_grass_core::ContentHash::new(&hash);
    let braid = state
        .store
        .get_by_hash(&content_hash)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Braid not found for hash: {hash}")))?;

    Ok(Json(braid))
}

/// Create a new Braid from raw data.
///
/// # Errors
///
/// Returns an error if base64 decoding fails, factory creation fails, or storage fails.
pub async fn create_braid(
    State(state): State<AppState>,
    Json(request): Json<CreateBraidRequest>,
) -> Result<(StatusCode, Json<CreateBraidResponse>), ServiceError> {
    // Decode base64 data
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&request.data)
        .map_err(|e| ServiceError::BadRequest(format!("Invalid base64 data: {e}")))?;

    // Build metadata
    let metadata = BraidMetadata {
        title: request.title.map(Arc::from),
        description: request.description.map(Arc::from),
        tags: request
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(Arc::from)
            .collect(),
        ..Default::default()
    };

    // Create the Braid
    let braid = state
        .factory
        .from_data(&data, &request.mime_type, Some(metadata))?;

    // Store it
    state.store.put(&braid).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateBraidResponse {
            id: braid.id.to_string(),
            hash: braid.data_hash.as_str().to_string(),
        }),
    ))
}

/// Create a Braid with full provenance metadata (from existing hash).
///
/// # Errors
///
/// Returns an error if factory creation fails or storage fails.
pub async fn create_provenance_braid(
    State(state): State<AppState>,
    Json(request): Json<CreateProvenanceBraidRequest>,
) -> Result<(StatusCode, Json<CreateBraidResponse>), ServiceError> {
    // Create the agent DID
    let agent_did = Did::new(&request.was_attributed_to);

    // Create derivation references
    let was_derived_from: Vec<EntityReference> = request
        .was_derived_from
        .into_iter()
        .map(EntityReference::by_hash)
        .collect();

    // Create metadata
    let metadata = BraidMetadata {
        tags: request.tags.into_iter().map(Arc::from).collect(),
        ..Default::default()
    };

    // Create the Braid from hash
    let mut braid = state.factory.from_hash(
        request.data_hash.into(),
        request.mime_type,
        request.size,
        Some(metadata),
    )?;

    // Set provenance fields
    braid.was_attributed_to = agent_did;
    braid.was_derived_from = was_derived_from;

    // Store it
    state.store.put(&braid).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateBraidResponse {
            id: braid.id.to_string(),
            hash: braid.data_hash.as_str().to_string(),
        }),
    ))
}

/// List/query Braids.
///
/// # Errors
///
/// Returns an error if the store query fails.
pub async fn list_braids(
    State(state): State<AppState>,
    Query(query): Query<ListBraidsQuery>,
) -> Result<Json<ListBraidsResponse>, ServiceError> {
    // Build filter using builder pattern
    let mut filter = QueryFilter::new();

    if let Some(agent) = query.agent {
        filter = filter.with_agent(Did::new(&agent));
    }

    if let Some(mime_type) = query.mime_type {
        filter = filter.with_mime_type(mime_type);
    }

    if let Some(limit) = query.limit {
        filter = filter.with_limit(limit);
    }

    if let Some(offset) = query.offset {
        filter = filter.with_offset(offset);
    }

    // Handle tag separately since it's a direct field
    if let Some(tag) = query.tag {
        filter.tag = Some(tag);
    }

    // Determine order
    let order = match query.order.as_deref() {
        Some("oldest") => QueryOrder::OldestFirst,
        Some("largest") => QueryOrder::LargestFirst,
        Some("smallest") => QueryOrder::SmallestFirst,
        _ => QueryOrder::NewestFirst,
    };

    // Execute query
    let result = state.store.query(&filter, order).await?;

    Ok(Json(ListBraidsResponse {
        braids: result.braids,
        total: result.total_count,
        has_more: result.has_more,
    }))
}

/// Delete a Braid.
///
/// # Errors
///
/// Returns an error if the store delete operation fails.
pub async fn delete_braid(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    let braid_id = BraidId::from_string(id);
    state.store.delete(&braid_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests;
