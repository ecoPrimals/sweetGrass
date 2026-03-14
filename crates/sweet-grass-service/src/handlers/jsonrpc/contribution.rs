// SPDX-License-Identifier: AGPL-3.0-only
//! Contribution domain handlers: record, record_session, record_dehydration.

use sweet_grass_core::{
    braid::{CompressionMeta, EcoPrimalsAttributes},
    contribution::{ContributionRecord, SessionContribution},
    dehydration::DehydrationSummary,
};

use crate::state::AppState;

use super::{internal, parse_params, to_value, DispatchResult};

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
/// Converts a `DehydrationSummary` into Braids with full DAG metadata.
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
            .mime_type("application/octet-stream")
            .size(0)
            .attributed_to(op.agent.clone())
            .ecop(make_ecop(&summary))
            .build()
            .map_err(internal)?;

        state.store.put(&braid).await.map_err(internal)?;
        braids.push(braid);
    }

    if braids.is_empty() {
        let agent = summary
            .agents
            .first()
            .cloned()
            .unwrap_or_else(|| sweet_grass_core::agent::Did::new("did:key:unknown"));

        let braid = sweet_grass_core::Braid::builder()
            .data_hash(summary.merkle_root.clone())
            .mime_type("application/x-merkle-root")
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
