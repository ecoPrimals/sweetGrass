// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability method extraction from `capabilities.list` JSON-RPC responses.
//!
//! Handles multiple wire formats across the ecosystem, normalizing them
//! into sorted `domain.operation` method name vectors.

/// Extract capability method names from a `capability.list` JSON-RPC response.
///
/// Handles multiple response formats across the ecosystem:
///
/// - **Format A** (flat array): `{"methods": ["braid.create", "health.check"]}`
/// - **Format B** (structured domains): `{"domains": {"braid": ["create"], "health": ["check"]}}`
/// - **`capabilities` alias**: Falls back to `capabilities` key if `methods` is absent
///   (neuralSpring S156 ecosystem compat)
/// - **`result` wrapper**: Unwraps `{"result": {...}}` if present
///
/// Returns a sorted, deduplicated `Vec<String>` of `domain.operation` method names.
///
/// # Examples
///
/// ```
/// # use serde_json::json;
/// # use sweet_grass_integration::discovery::extract_capabilities;
/// let flat = json!({"methods": ["braid.create", "health.check"]});
/// assert_eq!(extract_capabilities(&flat), vec!["braid.create", "health.check"]);
///
/// let structured = json!({"domains": {"braid": ["create", "get"], "health": ["check"]}});
/// let caps = extract_capabilities(&structured);
/// assert_eq!(caps, vec!["braid.create", "braid.get", "health.check"]);
/// ```
#[must_use]
pub fn extract_capabilities(response: &serde_json::Value) -> Vec<String> {
    let source = response.get("result").unwrap_or(response);

    if let Some(methods) = source
        .get("methods")
        .or_else(|| source.get("capabilities"))
        .and_then(serde_json::Value::as_array)
    {
        let mut caps: Vec<String> = methods
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(String::from)
            .collect();
        caps.sort();
        caps.dedup();
        return caps;
    }

    if let Some(domains) = source.get("domains").and_then(serde_json::Value::as_object) {
        let mut caps = Vec::new();
        for (domain, ops) in domains {
            if let Some(arr) = ops.as_array() {
                for op in arr {
                    if let Some(s) = op.as_str() {
                        caps.push(format!("{domain}.{s}"));
                    }
                }
            }
        }
        caps.sort();
        caps.dedup();
        return caps;
    }

    Vec::new()
}
