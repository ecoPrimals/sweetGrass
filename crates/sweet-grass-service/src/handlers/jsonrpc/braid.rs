// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid domain handlers: create, get, `get_by_hash`, query, delete, commit, anchor.

use std::sync::Arc;

use base64::Engine;
use serde::Deserialize;
use sweet_grass_core::activity::Activity;
use sweet_grass_core::braid::{BraidId, BraidMetadata, ContentHash, CrossGateAttribution};
use sweet_grass_core::dehydration::Witness;
use sweet_grass_core::privacy::{PrivacyLevel, PrivacyMetadata};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

use crate::method_gate::error_codes;
use crate::state::AppState;

use super::{DispatchError, DispatchResult, caller_context_from_params, caller_did_from_params, error_code, internal, parse_params, to_value};

/// Accepts both structured `metadata` and flattened convenience fields.
///
/// Composition callers (provenance trio pipeline, skunkBat Phase 3) send
/// `name`, `description` etc. as top-level params for ergonomics. These
/// are merged into `BraidMetadata` — structured `metadata` takes precedence
/// when both forms are present.
#[derive(Debug, Deserialize)]
pub(super) struct CreateBraidParams {
    data_hash: ContentHash,
    mime_type: String,
    size: u64,
    #[serde(default)]
    metadata: Option<BraidMetadata>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    source_session: Option<String>,
    #[serde(default)]
    source_merkle_root: Option<String>,
    #[serde(default)]
    privacy: Option<PrivacyMetadata>,
    #[serde(default)]
    cross_gate: Option<CrossGateAttribution>,
    #[serde(default)]
    source_gate: Option<String>,
    #[serde(default)]
    was_generated_by: Option<Activity>,
    #[serde(default)]
    witness: Option<Witness>,
}

impl CreateBraidParams {
    /// Merge flattened convenience fields into `BraidMetadata`.
    fn into_metadata(self) -> (ContentHash, String, u64, Option<BraidMetadata>) {
        let mut meta = self.metadata.unwrap_or_default();

        if meta.title.is_none() && let Some(name) = self.name {
            meta.title = Some(name.into());
        }
        if meta.description.is_none() && let Some(desc) = self.description {
            meta.description = Some(desc.into());
        }
        if meta.tags.is_empty() && let Some(tags) = self.tags {
            meta.tags = tags.into_iter().map(Into::into).collect();
        }
        if meta.privacy.is_none() && let Some(privacy) = self.privacy {
            meta.privacy = Some(privacy);
        }
        if meta.cross_gate.is_none() && let Some(cross_gate) = self.cross_gate {
            meta.cross_gate = Some(cross_gate);
        }
        if let Some(session) = self.source_session {
            meta.custom
                .entry("source_session".to_owned())
                .or_insert_with(|| serde_json::Value::String(session));
        }
        if let Some(root) = self.source_merkle_root {
            meta.custom
                .entry("source_merkle_root".to_owned())
                .or_insert_with(|| serde_json::Value::String(root));
        }

        (self.data_hash, self.mime_type, self.size, Some(meta))
    }
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

/// Enforce braid privacy visibility before returning stored content.
fn check_braid_privacy_access(braid: &sweet_grass_core::braid::Braid, params: &serde_json::Value) -> Result<(), DispatchError> {
    let Some(pm) = &braid.metadata.privacy else {
        return Ok(());
    };

    let caller = caller_context_from_params(params);

    match &pm.visibility {
        PrivacyLevel::Public | PrivacyLevel::AnonymizedPublic { .. } => Ok(()),
        PrivacyLevel::Authenticated => {
            if caller.bearer_token.is_some() {
                Ok(())
            } else {
                Err(DispatchError {
                    code: error_codes::PERMISSION_DENIED,
                    message: "Authentication required to access this braid".to_string(),
                    source_detail: None,
                })
            }
        }
        PrivacyLevel::Private | PrivacyLevel::Encrypted => {
            let requester = caller_did_from_params(params);
            match requester {
                Some(did) if pm.has_access(&did, &braid.was_attributed_to) => Ok(()),
                _ => Err(DispatchError {
                    code: error_codes::PERMISSION_DENIED,
                    message: "Access denied to private braid".to_string(),
                    source_detail: None,
                }),
            }
        }
        _ => Err(DispatchError {
            code: error_codes::PERMISSION_DENIED,
            message: "Unsupported privacy visibility level".to_string(),
            source_detail: None,
        }),
    }
}

pub(super) async fn handle_braid_create(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let mut p: CreateBraidParams = parse_params(params)?;
    let source_gate = p.source_gate.clone();
    let use_auto_sign = p.witness.is_none();
    let was_generated_by = p.was_generated_by.take();
    let witness = p.witness.take();
    let (data_hash, mime_type, size, metadata) = p.into_metadata();
    let mut braid = state
        .factory
        .from_hash(data_hash, mime_type, size, metadata)
        .map_err(internal)?;

    if let Some(gate) = source_gate {
        braid.ecop.source_gate = Some(Arc::from(gate.as_str()));
    }

    if let Some(activity) = was_generated_by {
        braid.was_generated_by = Some(activity);
    }

    if let Some(w) = witness {
        braid.witness = w;
    }

    #[cfg(unix)]
    if use_auto_sign
        && let Some(crypto) = &state.crypto
    {
        match crypto
            .sign(braid.compute_signing_hash().as_str().as_bytes())
            .await
        {
            Ok(result) => {
                let agent_did =
                    sweet_grass_core::agent::Did::from_public_key_bytes(&result.public_key);
                braid.witness = sweet_grass_core::dehydration::Witness::from_tower_ed25519(
                    &agent_did,
                    &result.signature,
                );
            }
            Err(e) => {
                tracing::warn!("crypto.sign unavailable, braid unsigned: {e}");
            }
        }
    }

    state.store.put(&braid).await.map_err(internal)?;
    to_value(&braid)
}

pub(super) async fn handle_braid_get(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: GetBraidParams = parse_params(params.clone())?;
    let braid = state.store.get(&p.id).await.map_err(internal)?;
    match braid {
        Some(b) => {
            check_braid_privacy_access(&b, &params)?;
            to_value(&b)
        }
        None => Err(DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found: {}", p.id),
            source_detail: None,
        }),
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
        None => Err(DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("No braid with hash: {}", p.hash),
            source_detail: None,
        }),
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
        .ok_or_else(|| DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found: {}", p.braid_id),
            source_detail: None,
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

#[derive(Debug, Deserialize)]
pub(super) struct BraidAnchorParams {
    braid_id: BraidId,
    branch_id: String,
}

/// Anchor a braid to a DAG branch point.
///
/// Establishes provenance at branch creation time rather than only at
/// commit time. Called by `rootpulse.branch` signal graphs when a DAG
/// branch is created so the attribution record is bound to the branch
/// context.
///
/// Signs the anchor via Tower/BearDog `crypto.sign` when available.
pub(super) async fn handle_braid_anchor(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: BraidAnchorParams = parse_params(params)?;

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

    let preimage = braid.compute_anchor_preimage(&p.branch_id);

    let mut response = serde_json::json!({
        "braid_id": braid.id.as_str(),
        "branch_id": p.branch_id,
        "content_hash": hash_bytes,
        "anchor_preimage": preimage.as_str(),
        "anchored_at_branch": true,
        "status": "anchored",
    });

    #[cfg(unix)]
    if let Some(crypto) = &state.crypto {
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
            }
            Err(e) => {
                tracing::warn!("crypto.sign unavailable, branch anchor unsigned: {e}");
            }
        }
    }

    to_value(&response)
}
