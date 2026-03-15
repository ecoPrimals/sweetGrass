// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability domain handler: list.
//!
//! Required by wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD`:
//! every spring must respond to `capability.list` so that Songbird
//! and other primals can discover what this primal offers at runtime.

use super::{to_value, DispatchResult, METHODS};
use crate::state::AppState;

/// `capability.list` — advertise every domain and operation this primal serves.
///
/// Returns a structured response with the primal name, version, and a
/// domain-grouped map of all registered JSON-RPC methods.  This is the
/// primary mechanism for runtime capability discovery.
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
        "domains": domains,
        "methods": methods,
    }))
}
