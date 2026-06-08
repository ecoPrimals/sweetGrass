// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Lifecycle and auth introspection JSON-RPC handlers (JH-0 method gate).

use sweet_grass_core::niche;

use crate::state::AppState;

use super::{DispatchResult, caller_context_from_params, registry::METHODS};

#[expect(clippy::unnecessary_wraps, reason = "must match DispatchFn signature")]
pub(super) fn handle_lifecycle_status(
    state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let version = env!("CARGO_PKG_VERSION");
    let name = state
        .self_knowledge
        .as_ref()
        .map_or("sweetgrass", |sk| sk.name.as_str());
    let uptime_secs = state
        .self_knowledge
        .as_ref()
        .map_or(0, |sk| sk.uptime().as_secs());
    let started_at = state
        .self_knowledge
        .as_ref()
        .map(|sk| chrono::DateTime::<chrono::Utc>::from(sk.established_at).to_rfc3339());
    let mut response = serde_json::json!({
        "status": "running",
        "primal": name,
        "version": version,
        "gate_mode": state.method_gate.mode().as_str(),
        "uptime_secs": uptime_secs,
        "method_count": METHODS.len(),
        "capabilities_count": niche::CAPABILITIES.len(),
        "store_backend": state.store_backend,
    });
    if let Some(started_at) = started_at {
        response["started_at"] = serde_json::Value::String(started_at);
    }
    Ok(response)
}

#[expect(clippy::unnecessary_wraps, reason = "must match DispatchFn signature")]
pub(super) fn handle_auth_mode(state: &AppState) -> DispatchResult {
    Ok(serde_json::json!({
        "mode": state.method_gate.mode().as_str(),
    }))
}

/// Enriched auth check per primalSpring later-term pattern.
///
/// Returns `{ authenticated, verified, enforcement, scopes, subject, expires_in }`.
/// Fields that require live `BearDog` token verification return `null` until
/// `auth.verify_ionic` is wired (JH-11).
#[expect(
    clippy::unnecessary_wraps,
    clippy::needless_pass_by_value,
    reason = "must match DispatchFn signature"
)]
pub(super) fn handle_auth_check(state: &AppState, params: serde_json::Value) -> DispatchResult {
    let caller = caller_context_from_params(&params);
    let has_token = caller.bearer_token.is_some();
    Ok(serde_json::json!({
        "authenticated": has_token,
        "verified": false,
        "enforcement": state.method_gate.mode().as_str(),
        "scopes": serde_json::Value::Null,
        "subject": serde_json::Value::Null,
        "expires_in": serde_json::Value::Null,
    }))
}

#[expect(
    clippy::unnecessary_wraps,
    clippy::needless_pass_by_value,
    reason = "must match DispatchFn signature"
)]
pub(super) fn handle_auth_peer_info(params: serde_json::Value) -> DispatchResult {
    let ctx = caller_context_from_params(&params);
    Ok(serde_json::json!({
        "origin": format!("{:?}", ctx.origin),
        "authenticated": ctx.bearer_token.is_some(),
        "peer": ctx.peer.as_ref().map(|p| serde_json::json!({
            "uid": p.uid,
            "pid": p.pid,
        })),
    }))
}
