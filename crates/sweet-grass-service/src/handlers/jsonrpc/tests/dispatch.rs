// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Dispatch table completeness and `DispatchOutcome` classification tests.

use super::super::*;
use super::helpers::test_state;

#[tokio::test]
async fn test_method_not_found() {
    let state = test_state();
    let result = dispatch(&state, "nonexistent.method", serde_json::json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::METHOD_NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_params() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"wrong": "params"}),
    )
    .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::INVALID_PARAMS);
}

#[test]
fn test_dispatch_table_completeness() {
    assert_eq!(
        METHODS.len(),
        48,
        "dispatch table should have all 48 methods (43 domain + capability.call + lifecycle + 3 auth)"
    );

    let expected = [
        "braid.create",
        "braid.get",
        "braid.get_by_hash",
        "braid.query",
        "braid.delete",
        "braid.commit",
        "braid.anchor",
        "braid.batch_create",
        "braid.batch_commit",
        "braid.list",
        "anchoring.anchor",
        "anchoring.verify",
        "convergence.check",
        "convergence.batch_check",
        "convergence.pressure",
        "provenance.graph",
        "provenance.export_provo",
        "provenance.export_graph_provo",
        "attribution.chain",
        "attribution.calculate_rewards",
        "attribution.top_contributors",
        "attribution.witness",
        "compression.compress_session",
        "compression.create_meta_braid",
        "contribution.record",
        "contribution.record_session",
        "contribution.record_dehydration",
        "contribution.record_provenance",
        "pipeline.attribute",
        "health.check",
        "health.liveness",
        "health.readiness",
        "identity.get",
        "composition.tower_health",
        "composition.node_health",
        "composition.nest_health",
        "composition.nucleus_health",
        "trust.event",
        "lifecycle.status",
        "capabilities.list",
        "capability.list",
        "tools.list",
        "tools.call",
        "auth.mode",
        "auth.check",
        "auth.peer_info",
    ];
    for name in expected {
        assert!(find_handler(name).is_some(), "missing handler for: {name}");
    }
}

#[tokio::test]
async fn test_dispatch_outcome_protocol_error_for_unknown_method() {
    let state = test_state();
    let outcome = dispatch_classified(&state, "no.such.method", serde_json::json!({})).await;
    assert!(outcome.is_protocol_error());
}

#[tokio::test]
async fn test_dispatch_outcome_success_for_health() {
    let state = test_state();
    let outcome = dispatch_classified(&state, "health.check", serde_json::json!({})).await;
    assert!(!outcome.is_protocol_error());
    assert!(matches!(outcome, DispatchOutcome::Success(_)));
}

#[tokio::test]
async fn test_dispatch_outcome_application_error_for_not_found() {
    let state = test_state();
    let outcome =
        dispatch_classified(&state, "braid.get", serde_json::json!({"id": "missing"})).await;
    assert!(!outcome.is_protocol_error());
    assert!(outcome.is_application_error());
}
