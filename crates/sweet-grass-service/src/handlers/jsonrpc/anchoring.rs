// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Anchoring domain handlers: anchor, verify.

use base64::Engine;
use serde::Deserialize;
use sweet_grass_core::braid::BraidId;

use sweet_grass_store::BraidStore;

use crate::state::AppState;

use super::{DispatchError, DispatchResult, error_code, internal, parse_params, to_value};

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
        .ok_or_else(|| DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found: {}", p.braid_id),
            source_detail: None,
        })?;

    let hash_bytes = braid
        .data_hash
        .to_bytes32()
        .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
        .ok_or_else(|| DispatchError {
            code: error_code::INVALID_PARAMS,
            message: "Content hash must be sha256 (32 bytes)".to_string(),
            source_detail: None,
        })?;

    let uuid_str = p
        .braid_id
        .as_str()
        .strip_prefix("urn:braid:")
        .unwrap_or(p.braid_id.as_str());

    let mut response = serde_json::json!({
        "braid_id": uuid_str,
        "spine_id": p.spine_id,
        "content_hash": hash_bytes,
        "anchored": false,
        "status": "prepared",
    });

    #[cfg(unix)]
    if let Some(crypto) = &state.crypto {
        let preimage = braid.compute_anchor_preimage(&p.spine_id);
        match crypto.sign(preimage.as_str().as_bytes()).await {
            Ok(result) => {
                let agent_did =
                    sweet_grass_core::agent::Did::from_public_key_bytes(&result.public_key);
                let witness = sweet_grass_core::dehydration::Witness::from_tower_ed25519(
                    &agent_did,
                    &result.signature,
                );
                if let Ok(w) = serde_json::to_value(&witness) {
                    response["witness"] = w;
                }
            },
            Err(e) => {
                tracing::warn!("crypto.sign unavailable, anchor unsigned: {e}");
            },
        }
    }

    to_value(&response)
}

/// Verify the anchor status of a braid.
///
/// Retrieves the full braid and inspects its witness field to determine
/// whether it has been cryptographically signed (tower-level anchoring).
/// Returns `"signed"` when a witness signature is present, or
/// `"unanchored"` when the braid exists but has no witness.
///
/// Full loamSpine ledger verification (cross-primal anchor proof) will
/// be wired in v0.8.0 via outbound trio clients.
pub(super) async fn handle_verify_anchor(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: VerifyAnchorParams = parse_params(params)?;

    let braid = state
        .store
        .get(&p.braid_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found: {}", p.braid_id),
            source_detail: None,
        })?;

    let has_witness = braid.witness.is_signed();

    let verification_status = if has_witness { "signed" } else { "unanchored" };

    let mut response = serde_json::json!({
        "braid_id": p.braid_id.as_str(),
        "anchored": has_witness,
        "verification_status": verification_status,
        "data_hash": braid.data_hash.as_str(),
        "generated_at_time": braid.generated_at_time.nanos(),
    });

    if has_witness && let Ok(w) = serde_json::to_value(&braid.witness) {
        response["witness"] = w;
    }

    to_value(&response)
}
