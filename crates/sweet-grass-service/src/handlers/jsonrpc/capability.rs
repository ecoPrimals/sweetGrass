// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability domain handler: list.
//!
//! Required by wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`:
//! every spring must respond to `capability.list` so that Songbird
//! and other primals can discover what this primal offers at runtime.
//!
//! Delegates to `sweet_grass_core::niche` for the canonical source of
//! truth — no inline duplication of capability metadata.

use super::{DispatchResult, METHODS, to_value};
use crate::state::AppState;

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
    use sweet_grass_core::niche;

    let methods: Vec<&str> = METHODS.iter().map(|m| m.name).collect();

    let mut domains = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for method in &methods {
        if let Some((domain, operation)) = method.split_once('.') {
            domains.entry(domain).or_default().push(operation);
        }
    }

    let mut operations = serde_json::Map::new();
    for op in niche::operation_dependencies() {
        operations.insert(
            op.method.to_string(),
            serde_json::json!({
                "depends_on": op.depends_on,
                "cost": op.cost,
            }),
        );
    }

    to_value(&serde_json::json!({
        "primal": niche::NICHE_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "description": niche::NICHE_DESCRIPTION,
        "protocol": "jsonrpc-2.0",
        "transport": ["http", "uds"],
        "domains": domains,
        "methods": methods,
        "operations": operations,
        "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        "cost_estimates": niche::cost_estimates().into_iter().collect::<std::collections::BTreeMap<_, _>>(),
    }))
}
