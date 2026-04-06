// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Provenance domain handlers: graph, `export_provo`, `export_graph_provo`.

use serde::Deserialize;
use sweet_grass_core::braid::ContentHash;
use sweet_grass_core::entity::EntityReference;

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

#[derive(Debug, Deserialize)]
pub(super) struct ProvenanceParams {
    entity: EntityReference,
    #[serde(default)]
    depth: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ExportProvoParams {
    hash: ContentHash,
}

#[derive(Debug, Deserialize)]
pub(super) struct ExportGraphProvoParams {
    entity: EntityReference,
    #[serde(default = "default_provo_depth")]
    depth: u32,
}

const fn default_provo_depth() -> u32 {
    10
}

/// # Errors
///
/// Returns an error if params parsing fails, the provenance query fails, or serialization fails.
pub(super) async fn handle_provenance_graph(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ProvenanceParams = parse_params(params)?;
    let graph = state
        .query
        .provenance_graph(p.entity, p.depth)
        .await
        .map_err(internal)?;
    to_value(&graph)
}

/// # Errors
///
/// Returns an error if params parsing fails, the store/query fails, or serialization fails.
pub(super) async fn handle_export_provo(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ExportProvoParams = parse_params(params)?;
    let export = state
        .query
        .export_braid_provo(&p.hash)
        .await
        .map_err(internal)?;
    to_value(&export)
}

/// # Errors
///
/// Returns an error if params parsing fails, the provenance query fails, or serialization fails.
pub(super) async fn handle_export_graph_provo(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ExportGraphProvoParams = parse_params(params)?;
    let export = state
        .query
        .export_graph_provo(p.entity, Some(p.depth))
        .await
        .map_err(internal)?;
    to_value(&export)
}
