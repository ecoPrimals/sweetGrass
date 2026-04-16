// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid domain handlers: create, get, `get_by_hash`, query, delete, commit.

use base64::Engine;
use serde::Deserialize;
use sweet_grass_core::braid::{BraidId, BraidMetadata, ContentHash};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

use crate::state::AppState;

use super::{DispatchResult, error_code, internal, parse_params, to_value};

#[derive(Debug, Deserialize)]
pub(super) struct CreateBraidParams {
    data_hash: ContentHash,
    mime_type: String,
    size: u64,
    #[serde(default)]
    metadata: Option<BraidMetadata>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GetBraidParams {
    id: BraidId,
}

#[derive(Debug, Deserialize)]
pub(super) struct GetByHashParams {
    hash: ContentHash,
}

#[derive(Debug, Deserialize)]
pub(super) struct QueryBraidsParams {
    filter: QueryFilter,
    #[serde(default)]
    order: Option<QueryOrder>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BraidCommitParams {
    braid_id: BraidId,
    #[serde(default = "default_spine_id")]
    spine_id: String,
}

fn default_spine_id() -> String {
    "default".to_string()
}

pub(super) async fn handle_braid_create(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: CreateBraidParams = parse_params(params)?;
    let braid = state
        .factory
        .from_hash(p.data_hash, p.mime_type, p.size, p.metadata)
        .map_err(internal)?;
    state.store.put(&braid).await.map_err(internal)?;
    to_value(&braid)
}

pub(super) async fn handle_braid_get(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: GetBraidParams = parse_params(params)?;
    let braid = state.store.get(&p.id).await.map_err(internal)?;
    match braid {
        Some(b) => to_value(&b),
        None => Err((error_code::NOT_FOUND, format!("Braid not found: {}", p.id))),
    }
}

pub(super) async fn handle_braid_get_by_hash(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: GetByHashParams = parse_params(params)?;
    let filter = QueryFilter {
        data_hash: Some(p.hash.clone()),
        ..QueryFilter::default()
    };
    let result = state
        .store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .map_err(internal)?;
    match result.braids.into_iter().next() {
        Some(b) => to_value(&b),
        None => Err((
            error_code::NOT_FOUND,
            format!("No braid with hash: {}", p.hash),
        )),
    }
}

pub(super) async fn handle_braid_query(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: QueryBraidsParams = parse_params(params)?;
    let order = p.order.unwrap_or(QueryOrder::NewestFirst);
    let result = state
        .store
        .query(&p.filter, order)
        .await
        .map_err(internal)?;
    to_value(&result)
}

pub(super) async fn handle_braid_delete(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: GetBraidParams = parse_params(params)?;
    let deleted = state.store.delete(&p.id).await.map_err(internal)?;
    to_value(&deleted)
}

/// Package a Braid for `LoamSpine` anchoring.
///
/// Extracts UUID from `BraidId` and converts `ContentHash` to `[u8; 32]`
/// for the `LoamSpine` `braid.commit` wire format.
pub(super) async fn handle_braid_commit(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: BraidCommitParams = parse_params(params)?;

    let braid = state
        .store
        .get(&p.braid_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| {
            (
                error_code::NOT_FOUND,
                format!("Braid not found: {}", p.braid_id),
            )
        })?;

    let uuid = braid
        .id
        .extract_uuid()
        .map_or_else(|| braid.id.as_str().to_string(), |u| u.to_string());

    let hash_bytes = braid
        .data_hash
        .to_bytes32()
        .map(|b| base64::engine::general_purpose::STANDARD.encode(b));

    to_value(&serde_json::json!({
        "braid_id": braid.id.as_str(),
        "uuid": uuid,
        "data_hash": braid.data_hash.as_str(),
        "data_hash_bytes": hash_bytes,
        "spine_id": p.spine_id,
        "mime_type": braid.mime_type,
        "size": braid.size,
        "attributed_to": braid.was_attributed_to.as_str(),
        "generated_at": braid.generated_at_time,
        "is_signed": braid.is_signed(),
    }))
}
