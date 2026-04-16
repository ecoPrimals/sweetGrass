// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Anchoring domain handlers: anchor, verify.

use base64::Engine;
use serde::Deserialize;
use sweet_grass_core::braid::BraidId;

use sweet_grass_store::BraidStore;

use crate::state::AppState;

use super::{DispatchResult, error_code, internal, parse_params, to_value};

#[derive(Debug, Deserialize)]
pub(super) struct AnchorBraidParams {
    braid_id: BraidId,
    #[serde(default = "default_spine_id")]
    spine_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct VerifyAnchorParams {
    braid_id: BraidId,
}

fn default_spine_id() -> String {
    "default".to_string()
}

pub(super) async fn handle_anchor_braid(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: AnchorBraidParams = parse_params(params)?;

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

    let hash_bytes = braid
        .data_hash
        .to_bytes32()
        .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
        .ok_or_else(|| {
            (
                error_code::INVALID_PARAMS,
                "Content hash must be sha256 (32 bytes)".to_string(),
            )
        })?;

    let uuid_str = p
        .braid_id
        .as_str()
        .strip_prefix("urn:braid:")
        .unwrap_or(p.braid_id.as_str());

    to_value(&serde_json::json!({
        "braid_id": uuid_str,
        "spine_id": p.spine_id,
        "content_hash": hash_bytes,
        "anchored": false,
        "status": "prepared",
    }))
}

pub(super) async fn handle_verify_anchor(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: VerifyAnchorParams = parse_params(params)?;

    let exists = state.store.exists(&p.braid_id).await.map_err(internal)?;

    if !exists {
        return Err((
            error_code::NOT_FOUND,
            format!("Braid not found: {}", p.braid_id),
        ));
    }

    to_value(&serde_json::json!({
        "braid_id": p.braid_id.as_str(),
        "anchored": false,
        "verification_status": "pending_integration",
    }))
}
