// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Attribution domain handlers: chain, `calculate_rewards`, `top_contributors`,
//! `witness` (JH-5 Phase 3 audit pipeline).

use serde::Deserialize;
use sweet_grass_core::braid::ContentHash;

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

#[derive(Debug, Deserialize)]
pub(super) struct AttributionParams {
    hash: ContentHash,
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

/// # Errors
///
/// Returns an error if params parsing fails, the store query fails, or serialization fails.
pub(super) async fn handle_attribution_chain(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: AttributionParams = parse_params(params)?;
    let chain = state
        .query
        .attribution_chain(&p.hash)
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

    let chain = state
        .query
        .attribution_chain(&p.hash)
        .await
        .map_err(internal)?;

    let witness_record = serde_json::json!({
        "hash": p.hash.as_str(),
        "witness_agent": p.witness_agent,
        "event_type": p.event_type,
        "payload": p.payload,
        "chain_depth": chain.contributors.len(),
        "witnessed_at": chrono::Utc::now().to_rfc3339(),
    });

    tracing::info!(
        hash = p.hash.as_str(),
        witness_agent = %p.witness_agent,
        event_type = %p.event_type,
        "attribution.witness: recorded audit attestation"
    );

    Ok(witness_record)
}
