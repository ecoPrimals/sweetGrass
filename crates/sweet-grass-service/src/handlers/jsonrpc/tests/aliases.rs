// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Wire-name alias resolution (GAP-36) dispatch tests.

use super::super::*;
use super::helpers::test_state;

#[test]
fn test_alias_resolution_maps_all_downstream_names() {
    let aliases = [
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
        ("health", "health.check"),
    ];
    for (alias, canonical) in aliases {
        let handler = find_handler(alias);
        let canonical_handler = find_handler(canonical);
        assert!(
            handler.is_some(),
            "alias {alias} should resolve to a handler"
        );
        assert!(
            canonical_handler.is_some(),
            "canonical {canonical} should have a handler"
        );
    }
}

#[tokio::test]
async fn test_alias_braid_attribution_create_dispatches_correctly() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.attribution.create",
        serde_json::json!({
            "data_hash": "sha256:aliasresolution",
            "mime_type": "text/plain",
            "size": 42
        }),
    )
    .await
    .unwrap();
    assert!(result["@id"].as_str().unwrap().starts_with("urn:braid:"));
}

#[tokio::test]
async fn test_alias_attribution_create_braid_dispatches_correctly() {
    let state = test_state();
    let result = dispatch(
        &state,
        "attribution.create_braid",
        serde_json::json!({
            "data_hash": "sha256:legacyalias",
            "mime_type": "text/plain",
            "size": 10,
            "name": "legacy-caller"
        }),
    )
    .await
    .unwrap();
    assert_eq!(result["metadata"]["title"], "legacy-caller");
}

#[tokio::test]
async fn test_alias_provenance_lineage_maps_to_attribution_chain() {
    let state = test_state();
    let hex = "ab".repeat(32);
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": format!("sha256:{hex}"),
            "mime_type": "text/plain",
            "size": 1
        }),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "provenance.lineage",
        serde_json::json!({"hash": format!("sha256:{hex}")}),
    )
    .await
    .unwrap();
    assert!(result["contributors"].is_array());
}
