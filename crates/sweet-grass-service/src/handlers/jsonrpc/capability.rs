// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability domain handler: list.
//!
//! Required by wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`:
//! every spring must respond to `capability.list` so that Songbird
//! and other primals can discover what this primal offers at runtime.
//!
//! Evolved per airSpring niche pattern: capability descriptors include
//! operation dependencies and cost hints for intelligent dispatch.

use super::{DispatchResult, METHODS, to_value};
use crate::state::AppState;

/// Operation dependency and cost metadata for capability discovery.
///
/// Follows the airSpring niche architecture pattern where capabilities
/// include dependency information and cost estimates for intelligent
/// dispatch and graph construction.
fn capability_metadata() -> serde_json::Value {
    serde_json::json!({
        "braid.create":    { "depends_on": [], "cost": "low" },
        "braid.get":       { "depends_on": [], "cost": "low" },
        "braid.get_by_hash": { "depends_on": [], "cost": "low" },
        "braid.query":     { "depends_on": [], "cost": "medium" },
        "braid.delete":    { "depends_on": [], "cost": "low" },
        "braid.commit":    { "depends_on": ["braid.create"], "cost": "medium" },
        "anchoring.anchor":  { "depends_on": ["braid.create"], "cost": "high" },
        "anchoring.verify":  { "depends_on": [], "cost": "medium" },
        "provenance.graph":  { "depends_on": ["braid.create"], "cost": "medium" },
        "provenance.export_provo": { "depends_on": ["braid.create"], "cost": "medium" },
        "provenance.export_graph_provo": { "depends_on": ["braid.create"], "cost": "high" },
        "attribution.chain": { "depends_on": ["braid.create"], "cost": "high" },
        "attribution.calculate_rewards": { "depends_on": ["attribution.chain"], "cost": "high" },
        "attribution.top_contributors": { "depends_on": ["braid.create"], "cost": "medium" },
        "compression.compress_session": { "depends_on": ["braid.create"], "cost": "high" },
        "compression.create_meta_braid": { "depends_on": ["compression.compress_session"], "cost": "medium" },
        "contribution.record": { "depends_on": ["braid.create"], "cost": "low" },
        "contribution.record_session": { "depends_on": ["braid.create"], "cost": "medium" },
        "contribution.record_dehydration": { "depends_on": [], "cost": "medium" },
        "health.check":    { "depends_on": [], "cost": "low" },
        "capability.list": { "depends_on": [], "cost": "low" },
    })
}

/// `capability.list` — advertise every domain, operation, dependency,
/// and cost hint this primal serves.
///
/// Returns a structured response with primal identity, version, protocol
/// metadata, domain-grouped methods, and per-operation dependency/cost
/// information for intelligent niche dispatch.
pub(super) fn handle_capability_list(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    let methods: Vec<&str> = METHODS.iter().map(|m| m.name).collect();

    let mut domains = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for method in &methods {
        if let Some((domain, operation)) = method.split_once('.') {
            domains.entry(domain).or_default().push(operation);
        }
    }

    to_value(&serde_json::json!({
        "primal": sweet_grass_core::identity::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "jsonrpc-2.0",
        "transport": ["http", "uds"],
        "domains": domains,
        "methods": methods,
        "operations": capability_metadata(),
    }))
}
