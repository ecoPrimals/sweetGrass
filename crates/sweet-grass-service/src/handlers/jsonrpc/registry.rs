// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! JSON-RPC method registry: dispatch table, wire-name aliases, and lookup.

use super::DispatchFn;

pub(super) struct MethodEntry {
    pub(super) name: &'static str,
    handler: DispatchFn,
}

/// Static dispatch table — each domain.operation maps to a handler fn.
///
/// Replaces the former giant match statement with a scannable, extendable table.
pub(super) static METHODS: &[MethodEntry] = &[
    // Braid operations
    MethodEntry {
        name: "braid.create",
        handler: |s, p| Box::pin(super::braid::handle_braid_create(s, p)),
    },
    MethodEntry {
        name: "braid.get",
        handler: |s, p| Box::pin(super::braid::handle_braid_get(s, p)),
    },
    MethodEntry {
        name: "braid.get_by_hash",
        handler: |s, p| Box::pin(super::braid::handle_braid_get_by_hash(s, p)),
    },
    MethodEntry {
        name: "braid.query",
        handler: |s, p| Box::pin(super::braid::handle_braid_query(s, p)),
    },
    MethodEntry {
        name: "braid.delete",
        handler: |s, p| Box::pin(super::braid::handle_braid_delete(s, p)),
    },
    MethodEntry {
        name: "braid.commit",
        handler: |s, p| Box::pin(super::braid::handle_braid_commit(s, p)),
    },
    MethodEntry {
        name: "braid.anchor",
        handler: |s, p| Box::pin(super::braid::handle_braid_anchor(s, p)),
    },
    MethodEntry {
        name: "anchoring.anchor",
        handler: |s, p| Box::pin(super::anchoring::handle_anchor_braid(s, p)),
    },
    MethodEntry {
        name: "anchoring.verify",
        handler: |s, p| Box::pin(super::anchoring::handle_verify_anchor(s, p)),
    },
    // Provenance
    MethodEntry {
        name: "provenance.graph",
        handler: |s, p| Box::pin(super::provenance::handle_provenance_graph(s, p)),
    },
    MethodEntry {
        name: "provenance.export_provo",
        handler: |s, p| Box::pin(super::provenance::handle_export_provo(s, p)),
    },
    MethodEntry {
        name: "provenance.export_graph_provo",
        handler: |s, p| Box::pin(super::provenance::handle_export_graph_provo(s, p)),
    },
    // Attribution
    MethodEntry {
        name: "attribution.chain",
        handler: |s, p| Box::pin(super::attribution::handle_attribution_chain(s, p)),
    },
    MethodEntry {
        name: "attribution.calculate_rewards",
        handler: |s, p| Box::pin(super::attribution::handle_calculate_rewards(s, p)),
    },
    MethodEntry {
        name: "attribution.top_contributors",
        handler: |s, p| Box::pin(super::attribution::handle_top_contributors(s, p)),
    },
    MethodEntry {
        name: "attribution.witness",
        handler: |s, p| Box::pin(super::attribution::handle_attribution_witness(s, p)),
    },
    // Compression
    MethodEntry {
        name: "compression.compress_session",
        handler: |s, p| {
            Box::pin(async move { super::compression::handle_compress_session_sync(s, p) })
        },
    },
    MethodEntry {
        name: "compression.create_meta_braid",
        handler: |s, p| Box::pin(super::compression::handle_create_meta_braid(s, p)),
    },
    // Contribution recording
    MethodEntry {
        name: "contribution.record",
        handler: |s, p| Box::pin(super::contribution::handle_record_contribution(s, p)),
    },
    MethodEntry {
        name: "contribution.record_session",
        handler: |s, p| Box::pin(super::contribution::handle_record_session(s, p)),
    },
    MethodEntry {
        name: "contribution.record_dehydration",
        handler: |s, p| Box::pin(super::contribution::handle_record_dehydration(s, p)),
    },
    MethodEntry {
        name: "contribution.record_provenance",
        handler: |s, p| Box::pin(super::contribution::handle_record_provenance(s, p)),
    },
    // Pipeline (provenance trio coordination)
    MethodEntry {
        name: "pipeline.attribute",
        handler: |s, p| Box::pin(super::contribution::handle_pipeline_attribute(s, p)),
    },
    // Health (wateringHole PRIMAL_IPC_PROTOCOL v3.0)
    MethodEntry {
        name: "health.check",
        handler: |s, p| Box::pin(super::health::handle_health(s, p)),
    },
    MethodEntry {
        name: "health.liveness",
        handler: |s, p| Box::pin(async move { super::health::handle_liveness(s, p) }),
    },
    MethodEntry {
        name: "health.readiness",
        handler: |s, p| Box::pin(super::health::handle_readiness(s, p)),
    },
    // Identity (biomeOS Neural API probes this for primal name + version)
    MethodEntry {
        name: "identity.get",
        handler: |s, p| Box::pin(async move { super::health::handle_identity_get(s, p) }),
    },
    // Capability discovery (wateringHole SEMANTIC_METHOD_NAMING v2.1)
    // `capabilities.list` is canonical; `capability.list` retained as alias
    MethodEntry {
        name: "capabilities.list",
        handler: |s, p| Box::pin(async move { super::capability::handle_capability_list(s, p) }),
    },
    MethodEntry {
        name: "capability.list",
        handler: |s, p| Box::pin(async move { super::capability::handle_capability_list(s, p) }),
    },
    // MCP tool exposure (airSpring v0.10 pattern for Squirrel AI coordination)
    MethodEntry {
        name: "tools.list",
        handler: |s, p| Box::pin(async move { super::capability::handle_tools_list(s, p) }),
    },
    MethodEntry {
        name: "tools.call",
        handler: |s, p| Box::pin(super::capability::handle_tools_call(s, p)),
    },
    MethodEntry {
        name: "composition.tower_health",
        handler: |s, p| Box::pin(super::composition::handle_tower_health(s, p)),
    },
    MethodEntry {
        name: "composition.node_health",
        handler: |s, p| Box::pin(super::composition::handle_node_health(s, p)),
    },
    MethodEntry {
        name: "composition.nest_health",
        handler: |s, p| Box::pin(super::composition::handle_nest_health(s, p)),
    },
    MethodEntry {
        name: "composition.nucleus_health",
        handler: |s, p| Box::pin(super::composition::handle_nucleus_health(s, p)),
    },
    // Lifecycle (wateringHole public surface, classified public in method gate)
    MethodEntry {
        name: "lifecycle.status",
        handler: |s, p| Box::pin(async move { super::lifecycle::handle_lifecycle_status(s, p) }),
    },
    // Cross-gate trust events
    MethodEntry {
        name: "trust.event",
        handler: |s, p| Box::pin(super::trust::handle_trust_event(s, p)),
    },
    // Auth introspection (JH-0 method gate)
    MethodEntry {
        name: "auth.mode",
        handler: |s, _p| Box::pin(async move { super::lifecycle::handle_auth_mode(s) }),
    },
    MethodEntry {
        name: "auth.check",
        handler: |s, p| Box::pin(async move { super::lifecycle::handle_auth_check(s, p) }),
    },
    MethodEntry {
        name: "auth.peer_info",
        handler: |_s, p| Box::pin(async move { super::lifecycle::handle_auth_peer_info(p) }),
    },
];

/// Normalize a JSON-RPC method name for case-insensitive lookup.
///
/// Lowercases the method name so that `Braid.Create` matches `braid.create`.
/// Underscores within operation names are preserved (e.g. `get_by_hash`).
/// Adopted from barraCuda `normalize_method` pattern via loamSpine / wetSpring.
pub(super) fn normalize_method(raw: &str) -> String {
    raw.to_ascii_lowercase()
}

/// Wire-name aliases for downstream compatibility (GAP-36 reconciliation).
///
/// Downstream springs and integration guides reference method names that
/// diverge from the canonical wire names. This table maps every known
/// variant to the canonical name so callers get a valid handler instead
/// of `-32601 Method not found`.
///
/// Sources: `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`, `CAPABILITY_DOMAIN_REGISTRY.md`,
/// ludoSpring trio graph, primalSpring handoffs, biomeOS Neural API translation
/// errors (GAP-MATRIX-09).
static ALIASES: &[(&str, &str)] = &[
    ("braid.attribution.create", "braid.create"),
    ("attribution.create_braid", "braid.create"),
    ("provenance.create_braid", "braid.create"),
    ("attribution.braid", "braid.create"),
    ("attribution.add_contribution", "contribution.record"),
    ("attribution.calculate", "attribution.calculate_rewards"),
    ("attribution.seal", "braid.commit"),
    ("attribution.export_prov", "provenance.export_provo"),
    ("provenance.lineage", "attribution.chain"),
    ("attribution.anchor", "anchoring.anchor"),
];

pub(super) fn resolve_alias(method: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|(alias, _)| *alias == method)
        .map(|(_, canonical)| *canonical)
}

pub(super) fn find_handler(method: &str) -> Option<DispatchFn> {
    find_handler_normalized(&normalize_method(method))
}

pub(super) fn find_handler_normalized(normalized: &str) -> Option<DispatchFn> {
    METHODS
        .iter()
        .find(|m| m.name == normalized)
        .or_else(|| {
            let canonical = resolve_alias(normalized)?;
            METHODS.iter().find(|m| m.name == canonical)
        })
        .map(|m| m.handler)
}
