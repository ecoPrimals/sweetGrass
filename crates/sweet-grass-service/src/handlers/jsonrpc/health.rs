// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Health domain handlers: check, liveness, readiness.
//!
//! `health.liveness` and `health.readiness` implement the wateringHole
//! `PRIMAL_IPC_PROTOCOL` v3.0 health methods aligned with coralReef's
//! and healthSpring's implementations.

use sweet_grass_store::{BraidStore, QueryFilter};

use crate::state::AppState;

use super::{DispatchResult, internal, to_value};

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

/// Lightweight liveness probe — always true if the process is running.
///
/// Zero-cost: no store queries, no allocations beyond the JSON envelope.
/// Async signature required by the `DispatchFn` type.
pub(super) fn handle_liveness(_state: &AppState, _params: serde_json::Value) -> DispatchResult {
    to_value(&serde_json::json!({ "alive": true }))
}

/// Readiness probe — checks whether the store backend is reachable.
///
/// Used by orchestrators and circuit breakers to gate traffic.
pub(super) async fn handle_readiness(
    state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let ready = state.store.count(&QueryFilter::default()).await.is_ok();
    to_value(&serde_json::json!({ "ready": ready }))
}

/// `identity.get` — Wire Standard L2 primal identity.
///
/// Returns `{primal, version, domain, license}` per
/// `wateringHole/CAPABILITY_WIRE_STANDARD.md` v1.0 §4.
/// biomeOS Neural API probes this alongside `capabilities.list`.
pub(super) fn handle_identity_get(_state: &AppState, _params: serde_json::Value) -> DispatchResult {
    use sweet_grass_core::niche;
    to_value(&serde_json::json!({
        "primal": sweet_grass_core::identity::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": niche::PRIMARY_DOMAIN,
        "license": "AGPL-3.0-or-later",
    }))
}
