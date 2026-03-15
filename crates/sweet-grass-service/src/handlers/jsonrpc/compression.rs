// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Compression domain handlers: `compress_session`, `create_meta_braid`.

use serde::Deserialize;
use sweet_grass_compression::Session;
use sweet_grass_core::braid::{BraidId, SummaryType};

use crate::state::AppState;

use super::{internal, parse_params, to_value, DispatchResult};

#[derive(Debug, Deserialize)]
pub(super) struct MetaBraidParams {
    braid_ids: Vec<BraidId>,
    summary_type: SummaryType,
}

pub(super) fn handle_compress_session_sync(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let session: Session = parse_params(params)?;
    let result = state.compression.compress(&session).map_err(internal)?;
    to_value(&result)
}

pub(super) async fn handle_create_meta_braid(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: MetaBraidParams = parse_params(params)?;
    let braid = state
        .factory
        .meta_braid(p.braid_ids, p.summary_type, None)
        .map_err(internal)?;
    state.store.put(&braid).await.map_err(internal)?;
    to_value(&braid)
}
