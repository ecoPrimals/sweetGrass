// SPDX-License-Identifier: AGPL-3.0-only
//! Health domain handler: check.

use sweet_grass_store::QueryFilter;

use crate::state::AppState;

use super::{internal, to_value, DispatchResult};

pub(super) async fn handle_health(state: &AppState, _params: serde_json::Value) -> DispatchResult {
    let count = state
        .store
        .count(&QueryFilter::default())
        .await
        .map_err(internal)?;
    to_value(&serde_json::json!({
        "status": "healthy",
        "store_status": "ok",
        "braid_count": count,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
