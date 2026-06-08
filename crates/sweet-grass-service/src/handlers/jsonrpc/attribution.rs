// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Attribution domain handlers: chain, `calculate_rewards`, `top_contributors`,
//! `witness` (JH-5 Phase 3 audit pipeline).

use std::sync::Arc;

use serde::Deserialize;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::{BraidMetadata, ContentHash};
use sweet_grass_factory::AttributionConfig;
use sweet_grass_store::BraidStore;

use crate::state::AppState;

use super::{DispatchError, DispatchResult, error_code, internal, parse_params, to_value};

#[derive(Debug, Deserialize, Default)]
struct ChainConfigOverride {
    #[serde(default)]
    max_depth: Option<u32>,
    #[serde(default)]
    decay_factor: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChainParams {
    hash: ContentHash,
    #[serde(default)]
    config: Option<ChainConfigOverride>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RewardsParams {
    hash: ContentHash,
    value: f64,
}

#[derive(Debug, Deserialize)]
pub(super) struct TopContributorsParams {
    hash: ContentHash,
    #[serde(default = "default_contributor_limit")]
    limit: u32,
}

const fn default_contributor_limit() -> u32 {
    10
}

fn merge_attribution_config(overrides: Option<ChainConfigOverride>) -> AttributionConfig {
    let mut config = AttributionConfig::default();
    if let Some(overrides) = overrides {
        if let Some(max_depth) = overrides.max_depth {
            config.max_depth = max_depth;
        }
        if let Some(decay_factor) = overrides.decay_factor {
            config.decay_factor = decay_factor;
        }
    }
    config
}

/// # Errors
///
/// Returns an error if params parsing fails, the store query fails, or serialization fails.
pub(super) async fn handle_attribution_chain(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ChainParams = parse_params(params)?;
    let config = merge_attribution_config(p.config);
    let chain = state
        .query
        .attribution_chain_with_config(&p.hash, config)
        .await
        .map_err(internal)?;
    to_value(&chain)
}

/// # Errors
///
/// Returns an error if params parsing fails, the store query fails, or serialization fails.
pub(super) async fn handle_calculate_rewards(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: RewardsParams = parse_params(params)?;
    let chain = state
        .query
        .attribution_chain(&p.hash)
        .await
        .map_err(internal)?;
    let rewards: Vec<serde_json::Value> = chain
        .contributors
        .iter()
        .map(|c| {
            serde_json::json!({
                "agent": c.agent.as_str(),
                "share": c.share,
                "amount": c.share * p.value,
                "role": format!("{:?}", c.role),
            })
        })
        .collect();
    to_value(&rewards)
}

/// # Errors
///
/// Returns an error if params parsing fails, the store query fails, or serialization fails.
pub(super) async fn handle_top_contributors(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: TopContributorsParams = parse_params(params)?;
    let chain = state
        .query
        .attribution_chain(&p.hash)
        .await
        .map_err(internal)?;
    let mut contributors: Vec<serde_json::Value> = chain
        .contributors
        .iter()
        .map(|c| {
            serde_json::json!({
                "agent": c.agent.as_str(),
                "share": c.share,
                "role": format!("{:?}", c.role),
            })
        })
        .collect();
    contributors.sort_by(|a, b| {
        b["share"]
            .as_f64()
            .partial_cmp(&a["share"].as_f64())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    contributors.truncate(p.limit as usize);
    to_value(&contributors)
}

/// Witness event for JH-5 Phase 3 audit pipeline.
///
/// Records an externally-sourced attestation in the attribution layer.
/// The full pipeline is: skunkBat `defense.log` -> rhizoCrypt
/// `dag.event.append` -> sweetGrass `attribution.witness`.
#[derive(Debug, Deserialize)]
pub(super) struct WitnessParams {
    /// Hash of the content being witnessed.
    hash: ContentHash,
    /// Agent DID attesting the event.
    witness_agent: String,
    /// Free-form event type (e.g. `"security"`, `"integrity"`, `"provenance"`).
    #[serde(default = "default_event_type")]
    event_type: String,
    /// Optional structured payload from the upstream event.
    #[serde(default)]
    payload: serde_json::Value,
}

fn default_event_type() -> String {
    "attestation".to_owned()
}

/// # Errors
///
/// Returns an error if params parsing fails, the braid is not found,
/// or the store query fails.
pub(super) async fn handle_attribution_witness(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: WitnessParams = parse_params(params)?;

    let attested_braid = state
        .query
        .get_by_hash(&p.hash)
        .await
        .map_err(internal)?
        .ok_or_else(|| DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found for hash: {}", p.hash.as_str()),
            source_detail: None,
        })?;

    let chain = state
        .query
        .attribution_chain(&p.hash)
        .await
        .map_err(internal)?;

    let attester = Did::new(&p.witness_agent);
    let attestation_statement = format!("{} attestation for {}", p.event_type, p.hash.as_str());

    let mut metadata = BraidMetadata {
        description: Some(Arc::from(attestation_statement.as_str())),
        ..BraidMetadata::default()
    };
    metadata.custom.insert(
        "attestation_type".to_string(),
        serde_json::json!(p.event_type),
    );
    metadata.custom.insert(
        "attested_braid_id".to_string(),
        serde_json::json!(attested_braid.id.as_str()),
    );

    let witness_braid = sweet_grass_core::Braid::builder()
        .data_hash(format!(
            "witness:{}:{}",
            attested_braid.id.as_str(),
            attester.as_str()
        ))
        .mime_type(sweet_grass_core::identity::MIME_OCTET_STREAM)
        .size(0)
        .attributed_to(attester)
        .metadata(metadata)
        .build()
        .map_err(internal)?;

    state.store.put(&witness_braid).await.map_err(internal)?;

    let witness_record = serde_json::json!({
        "hash": p.hash.as_str(),
        "witness_agent": p.witness_agent,
        "event_type": p.event_type,
        "payload": p.payload,
        "chain_depth": chain.contributors.len(),
        "witnessed_at": chrono::Utc::now().to_rfc3339(),
        "witness_braid_id": witness_braid.id.as_str(),
    });

    tracing::info!(
        hash = p.hash.as_str(),
        witness_agent = %p.witness_agent,
        event_type = %p.event_type,
        witness_braid_id = %witness_braid.id.as_str(),
        "attribution.witness: recorded audit attestation"
    );

    Ok(witness_record)
}
