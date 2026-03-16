// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Niche self-knowledge for sweetGrass.
//!
//! Every ecoPrimals spring/primal exposes its niche identity: what
//! capabilities it offers, what it consumes, its dependencies, cost
//! estimates, and semantic mappings for biomeOS dispatch and deploy
//! graph construction.
//!
//! Follows the `SPRING_AS_NICHE_DEPLOYMENT_STANDARD` from wateringHole.

use crate::identity;

/// Canonical niche identifier (matches `identity::PRIMAL_NAME`).
pub const NICHE_ID: &str = identity::PRIMAL_NAME;

/// Human-readable niche description.
pub const NICHE_DESCRIPTION: &str =
    "Semantic provenance and attribution layer — braids, PROV-O, fair credit";

/// Capabilities this primal offers to the ecosystem.
pub const CAPABILITIES: &[&str] = &[
    "braid.create",
    "braid.get",
    "braid.get_by_hash",
    "braid.query",
    "braid.delete",
    "braid.commit",
    "anchoring.anchor",
    "anchoring.verify",
    "provenance.graph",
    "provenance.export_provo",
    "provenance.export_graph_provo",
    "attribution.chain",
    "attribution.calculate_rewards",
    "attribution.top_contributors",
    "compression.compress_session",
    "compression.create_meta_braid",
    "contribution.record",
    "contribution.record_session",
    "contribution.record_dehydration",
    "health.check",
    "capability.list",
];

/// Capabilities this primal consumes from other primals at runtime.
///
/// These are discovered via capability-based routing, never hardcoded
/// to specific primal names.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    "crypto.sign",
    "crypto.verify",
    "storage.artifact.store",
    "storage.artifact.get",
    "dag.session.create",
    "dag.dehydration.trigger",
    "spine.create",
    "commit.session",
];

/// Niche dependency descriptor for deploy graph construction.
#[derive(Debug, Clone)]
pub struct NicheDependency {
    /// Capability domain this dependency satisfies.
    pub capability: &'static str,
    /// Whether this dependency is required for startup.
    pub required: bool,
    /// Fallback behavior when unavailable: `"skip"`, `"warn"`, or `"fail"`.
    pub fallback: &'static str,
}

/// Dependencies for biomeOS deploy graph ordering.
///
/// sweetGrass is the last node in the provenance trio sequence:
/// `BearDog → Songbird → rhizoCrypt → LoamSpine → sweetGrass`
pub const DEPENDENCIES: &[NicheDependency] = &[
    NicheDependency {
        capability: "crypto.sign",
        required: false,
        fallback: "skip",
    },
    NicheDependency {
        capability: "discovery.register",
        required: false,
        fallback: "skip",
    },
    NicheDependency {
        capability: "dag.session.create",
        required: false,
        fallback: "skip",
    },
    NicheDependency {
        capability: "spine.create",
        required: false,
        fallback: "skip",
    },
];

/// Operation dependency graph for intelligent dispatch.
///
/// Each entry maps a capability to its prerequisite operations and
/// estimated cost (`"low"`, `"medium"`, `"high"`).
#[must_use]
pub fn operation_dependencies() -> Vec<OperationMeta> {
    vec![
        OperationMeta::new("braid.create", &[], "low"),
        OperationMeta::new("braid.get", &[], "low"),
        OperationMeta::new("braid.get_by_hash", &[], "low"),
        OperationMeta::new("braid.query", &[], "medium"),
        OperationMeta::new("braid.delete", &[], "low"),
        OperationMeta::new("braid.commit", &["braid.create"], "medium"),
        OperationMeta::new("anchoring.anchor", &["braid.create"], "high"),
        OperationMeta::new("anchoring.verify", &[], "medium"),
        OperationMeta::new("provenance.graph", &["braid.create"], "medium"),
        OperationMeta::new("provenance.export_provo", &["braid.create"], "medium"),
        OperationMeta::new("provenance.export_graph_provo", &["braid.create"], "high"),
        OperationMeta::new("attribution.chain", &["braid.create"], "high"),
        OperationMeta::new(
            "attribution.calculate_rewards",
            &["attribution.chain"],
            "high",
        ),
        OperationMeta::new("attribution.top_contributors", &["braid.create"], "medium"),
        OperationMeta::new("compression.compress_session", &["braid.create"], "high"),
        OperationMeta::new(
            "compression.create_meta_braid",
            &["compression.compress_session"],
            "medium",
        ),
        OperationMeta::new("contribution.record", &["braid.create"], "low"),
        OperationMeta::new("contribution.record_session", &["braid.create"], "medium"),
        OperationMeta::new("contribution.record_dehydration", &[], "medium"),
        OperationMeta::new("health.check", &[], "low"),
        OperationMeta::new("capability.list", &[], "low"),
    ]
}

/// Cost estimates for biomeOS scheduling.
///
/// Returns a map of domain → relative cost tier.
#[must_use]
pub fn cost_estimates() -> Vec<(&'static str, &'static str)> {
    vec![
        ("braid", "low"),
        ("anchoring", "medium"),
        ("provenance", "medium"),
        ("attribution", "high"),
        ("compression", "high"),
        ("contribution", "low"),
        ("health", "low"),
        ("capability", "low"),
    ]
}

/// Semantic mappings for Neural API translation.
///
/// Maps human-readable intents to JSON-RPC method names for
/// natural-language routing via biomeOS Neural API.
#[must_use]
pub fn semantic_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        ("create provenance record", "braid.create"),
        ("get provenance", "braid.get"),
        ("find provenance by hash", "braid.get_by_hash"),
        ("search provenance", "braid.query"),
        ("commit provenance", "braid.commit"),
        ("anchor data", "anchoring.anchor"),
        ("verify anchor", "anchoring.verify"),
        ("show provenance graph", "provenance.graph"),
        ("export as PROV-O", "provenance.export_provo"),
        ("attribution chain", "attribution.chain"),
        ("calculate rewards", "attribution.calculate_rewards"),
        ("top contributors", "attribution.top_contributors"),
        ("compress session", "compression.compress_session"),
        ("record contribution", "contribution.record"),
        ("record dehydration", "contribution.record_dehydration"),
        ("health check", "health.check"),
        ("list capabilities", "capability.list"),
    ]
}

/// Metadata for a single operation in the niche dispatch graph.
#[derive(Debug, Clone)]
pub struct OperationMeta {
    /// Method name (`{domain}.{operation}`).
    pub method: &'static str,
    /// Methods that must complete before this one.
    pub depends_on: &'static [&'static str],
    /// Cost tier: `"low"`, `"medium"`, or `"high"`.
    pub cost: &'static str,
}

impl OperationMeta {
    /// Create a new operation metadata entry.
    #[must_use]
    pub const fn new(
        method: &'static str,
        depends_on: &'static [&'static str],
        cost: &'static str,
    ) -> Self {
        Self {
            method,
            depends_on,
            cost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niche_id_matches_identity() {
        assert_eq!(NICHE_ID, identity::PRIMAL_NAME);
    }

    #[test]
    fn capabilities_contains_required_methods() {
        assert!(CAPABILITIES.contains(&"health.check"));
        assert!(CAPABILITIES.contains(&"capability.list"));
        assert!(CAPABILITIES.contains(&"braid.create"));
        assert!(CAPABILITIES.contains(&"contribution.record_dehydration"));
    }

    #[test]
    fn consumed_capabilities_are_capability_based() {
        for cap in CONSUMED_CAPABILITIES {
            assert!(
                cap.contains('.'),
                "consumed capability should use domain.operation format: {cap}"
            );
        }
    }

    #[test]
    fn dependencies_all_have_fallback() {
        for dep in DEPENDENCIES {
            assert!(
                ["skip", "warn", "fail"].contains(&dep.fallback),
                "invalid fallback for {}: {}",
                dep.capability,
                dep.fallback
            );
        }
    }

    #[test]
    fn operation_dependencies_cover_all_capabilities() {
        let ops = operation_dependencies();
        let op_methods: Vec<&str> = ops.iter().map(|o| o.method).collect();
        for cap in CAPABILITIES {
            assert!(
                op_methods.contains(cap),
                "capability {cap} missing from operation_dependencies"
            );
        }
    }

    #[test]
    fn operation_dependencies_reference_valid_methods() {
        let ops = operation_dependencies();
        let all_methods: Vec<&str> = ops.iter().map(|o| o.method).collect();
        for op in &ops {
            for dep in op.depends_on {
                assert!(
                    all_methods.contains(dep),
                    "{} depends on unknown method {dep}",
                    op.method
                );
            }
        }
    }

    #[test]
    fn cost_estimates_cover_all_domains() {
        let costs = cost_estimates();
        let domains: Vec<&str> = costs.iter().map(|(d, _)| *d).collect();
        assert!(domains.contains(&"braid"));
        assert!(domains.contains(&"attribution"));
        assert!(domains.contains(&"health"));
    }

    #[test]
    fn cost_tiers_are_valid() {
        let ops = operation_dependencies();
        for op in &ops {
            assert!(
                ["low", "medium", "high"].contains(&op.cost),
                "invalid cost tier for {}: {}",
                op.method,
                op.cost
            );
        }
    }

    #[test]
    fn semantic_mappings_reference_valid_methods() {
        let mappings = semantic_mappings();
        for (intent, method) in &mappings {
            assert!(
                CAPABILITIES.contains(method),
                "semantic mapping '{intent}' → '{method}' references unknown capability"
            );
        }
    }

    #[test]
    fn no_duplicate_capabilities() {
        let mut seen = std::collections::HashSet::new();
        for cap in CAPABILITIES {
            assert!(seen.insert(cap), "duplicate capability: {cap}");
        }
    }

    #[test]
    fn no_duplicate_operation_methods() {
        let ops = operation_dependencies();
        let mut seen = std::collections::HashSet::new();
        for op in &ops {
            assert!(
                seen.insert(op.method),
                "duplicate operation method: {}",
                op.method
            );
        }
    }

    #[test]
    fn niche_description_is_not_empty() {
        assert!(!NICHE_DESCRIPTION.is_empty());
    }
}
