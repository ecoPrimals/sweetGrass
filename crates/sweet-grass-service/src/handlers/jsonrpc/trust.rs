// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Cross-gate trust event handler: auto-weaves a braid from a trust event.
//!
//! When a cross-gate trust relationship is established (e.g. bearDog on Gate A
//! trusts Gate B's key), the `trust.event` method weaves a fully-populated
//! PROV-O braid spanning both gates' provenance:
//!
//! - Maps `CrossGateTrustEvent` → `ActivityType` automatically
//! - Wires `origin_agent` as `wasAttributedTo` with `target_agent` delegation
//! - Builds a gateway-tier `Witness` when signature bytes are provided
//! - Sets `source_gate` and `cross_gate` metadata
//! - Uses `application/vnd.ecoprimals.trust-event` MIME type

use serde::Deserialize;
use sweet_grass_core::braid::cross_gate::CrossGateAttribution;
use sweet_grass_core::braid::Timestamp;
use sweet_grass_core::dehydration::Witness;
use sweet_grass_store::BraidStore;

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

/// Parameters for `trust.event`.
#[derive(Debug, Deserialize)]
pub(super) struct TrustEventParams {
    /// Cross-gate attribution context (required).
    cross_gate: CrossGateAttribution,
    /// Optional Ed25519 signature bytes (base64) for gateway witness.
    #[serde(default)]
    signature: Option<String>,
    /// Event timestamp override (defaults to now).
    #[serde(default)]
    timestamp: Option<u64>,
}

/// Weave a cross-gate trust braid from a `trust.event` invocation.
pub(super) async fn handle_trust_event(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: TrustEventParams = parse_params(params)?;

    let now = p
        .timestamp
        .map_or_else(Timestamp::now, Timestamp::new);

    let activity = p.cross_gate.to_activity(now);
    let content_hash = p.cross_gate.content_hash_seed();
    let gate_context = p.cross_gate.gate_context();
    let origin_agent = p.cross_gate.origin_agent.clone();
    let source_gate = std::sync::Arc::clone(&p.cross_gate.origin_gate);

    let witness = if let Some(sig_b64) = &p.signature {
        use base64::Engine;
        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(sig_b64)
            .map_err(|e| internal(format!("invalid base64 signature: {e}")))?;
        Witness::from_gateway_ed25519(&origin_agent, &sig_bytes, &gate_context)
    } else {
        Witness::unsigned()
    };

    let braid = sweet_grass_core::Braid::builder()
        .data_hash(content_hash)
        .mime_type(sweet_grass_core::identity::MIME_TRUST_EVENT)
        .size(0)
        .attributed_to(origin_agent)
        .generated_by(activity)
        .source_gate(source_gate)
        .cross_gate(p.cross_gate)
        .witness(witness)
        .generated_at_time(now)
        .build()
        .map_err(internal)?;

    state.store.put(&braid).await.map_err(internal)?;
    to_value(&braid)
}
