// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Contribution domain handlers: record, `record_session`, `record_dehydration`,
//! `pipeline_attribute`.
//!
//! Wire types for the pipeline handler are defined locally — sweetGrass owns
//! its own types and communicates with trio partners via JSON-RPC, not shared
//! crates. See `PRIMAL_SOVEREIGNTY_STANDARD` in wateringHole.

use sweet_grass_core::{
    braid::{CompressionMeta, EcoPrimalsAttributes},
    contribution::{ContributionRecord, SessionContribution},
    dehydration::DehydrationSummary,
};

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

// ==================== Pipeline wire types (sweetGrass-owned) ====================

/// Input to the `pipeline.attribute` JSON-RPC method.
///
/// Wire-compatible with the biomeOS provenance pipeline graph parameters.
/// Deserialized from JSON — no compile-time coupling to other primals.
#[derive(Clone, Debug, serde::Deserialize)]
#[allow(dead_code)]
struct PipelineRequest {
    session_id: String,
    agent_did: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    niche: Option<String>,
    #[serde(default)]
    agent_summaries: Vec<AgentContribution>,
}

/// Per-agent contribution data for attribution braids.
///
/// Fields beyond `agent_did` are deserialized for wire compatibility and
/// available for future attribution weight logic.
#[derive(Clone, Debug, serde::Deserialize)]
#[allow(dead_code)]
struct AgentContribution {
    agent_did: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_weight")]
    weight: f64,
}

const fn default_weight() -> f64 {
    1.0
}

/// Output of the `pipeline.attribute` JSON-RPC method.
///
/// Wire-compatible with biomeOS pipeline result expectations.
#[derive(Clone, Debug, serde::Serialize)]
struct PipelineResult {
    dehydration_merkle_root: String,
    commit_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    braid_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_ref: Option<String>,
}

// ==================== Handlers ====================

/// Handle pipeline attribution request from the provenance trio pipeline.
///
/// Accepts a [`PipelineRequest`] and creates attribution braids for the
/// session's agent contributions, returning a [`PipelineResult`] with the
/// `braid_ref` set.
pub(super) async fn handle_pipeline_attribute(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let request: PipelineRequest = parse_params(params)?;

    let mut braid_ids = Vec::new();

    for contribution in &request.agent_summaries {
        let agent = sweet_grass_core::agent::Did::new(&contribution.agent_did);
        let braid = sweet_grass_core::Braid::builder()
            .data_hash(format!(
                "pipeline:{}:{}",
                request.session_id, contribution.agent_did
            ))
            .mime_type(sweet_grass_core::identity::MIME_OCTET_STREAM)
            .size(0)
            .attributed_to(agent)
            .ecop(EcoPrimalsAttributes {
                source_primal: Some(sweet_grass_core::identity::PRIMAL_NAME.to_owned()),
                rhizo_session: Some(request.session_id.clone()),
                niche: request.niche.clone(),
                ..EcoPrimalsAttributes::default()
            })
            .build()
            .map_err(internal)?;

        state.store.put(&braid).await.map_err(internal)?;
        braid_ids.push(braid.id.to_string());
    }

    let braid_ref = braid_ids.first().cloned();

    to_value(&PipelineResult {
        dehydration_merkle_root: String::new(),
        commit_ref: String::new(),
        braid_ref,
        signature: None,
        content_ref: None,
    })
}

pub(super) async fn handle_record_contribution(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let record: ContributionRecord = parse_params(params)?;
    let braid = state.factory.from_contribution(&record).map_err(internal)?;
    state.store.put(&braid).await.map_err(internal)?;
    to_value(&braid)
}

pub(super) async fn handle_record_session(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let session: SessionContribution = parse_params(params)?;
    let braids = state.factory.from_session(&session).map_err(internal)?;
    for braid in &braids {
        state.store.put(braid).await.map_err(internal)?;
    }
    to_value(&serde_json::json!({
        "session_id": session.session_id,
        "braids_created": braids.len(),
        "braid_ids": braids.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
    }))
}

/// Record provenance from a rhizoCrypt dehydration event.
///
/// Deserializes directly into sweetGrass's own [`DehydrationSummary`] — no
/// shared crate needed. Unknown wire fields (e.g., `session_type`, `outcome`)
/// are silently ignored by serde, maintaining forward compatibility with
/// evolving trio partners.
pub(super) async fn handle_record_dehydration(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let summary: DehydrationSummary = parse_params(params)?;

    let make_ecop = |summary: &DehydrationSummary| {
        let compression = summary.compression_ratio.map(|ratio| CompressionMeta {
            vertex_count: summary.vertex_count,
            branch_count: summary.branch_count,
            ratio,
            summarizes: Vec::new(),
        });
        EcoPrimalsAttributes {
            source_primal: Some(summary.source_primal.clone()),
            rhizo_session: Some(summary.session_id.clone()),
            niche: summary.niche.clone(),
            compression,
            ..EcoPrimalsAttributes::default()
        }
    };

    let mut braids = Vec::new();
    for op in &summary.operations {
        let braid = sweet_grass_core::Braid::builder()
            .data_hash(op.content_hash.clone())
            .mime_type(sweet_grass_core::identity::MIME_OCTET_STREAM)
            .size(0)
            .attributed_to(op.agent.clone())
            .ecop(make_ecop(&summary))
            .build()
            .map_err(internal)?;

        state.store.put(&braid).await.map_err(internal)?;
        braids.push(braid);
    }

    if braids.is_empty() {
        let agent = summary.agents.first().cloned().unwrap_or_else(|| {
            sweet_grass_core::agent::Did::new(sweet_grass_core::identity::UNKNOWN_AGENT_DID)
        });

        let braid = sweet_grass_core::Braid::builder()
            .data_hash(summary.merkle_root.clone())
            .mime_type(sweet_grass_core::identity::MIME_MERKLE_ROOT)
            .size(32)
            .attributed_to(agent)
            .ecop(make_ecop(&summary))
            .build()
            .map_err(internal)?;

        state.store.put(&braid).await.map_err(internal)?;
        braids.push(braid);
    }

    to_value(&serde_json::json!({
        "session_id": summary.session_id,
        "merkle_root": summary.merkle_root.as_str(),
        "braids_created": braids.len(),
        "braid_ids": braids.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
        "vertex_count": summary.vertex_count,
        "agents": summary.agents.iter().map(sweet_grass_core::Did::as_str).collect::<Vec<_>>(),
    }))
}
