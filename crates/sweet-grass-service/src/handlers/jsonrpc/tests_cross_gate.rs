// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for cross-gate attribution braids and `source_gate` query filtering.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::dehydration::{WITNESS_TIER_GATEWAY, Witness};
use sweet_grass_store::QueryFilter;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

#[tokio::test]
async fn test_create_cross_gate_trust_braid() {
    let state = test_state();
    let create_params = serde_json::json!({
        "data_hash": "sha256:cross-gate-key-exchange-001",
        "mime_type": "application/x-sweetgrass-trust-event",
        "size": 0,
        "source_gate": "ironGate",
        "cross_gate": {
            "origin_gate": "ironGate",
            "target_gate": "strandGate",
            "trust_event": "key_exchange",
            "origin_agent": "did:key:z6MkIronGateAgent"
        }
    });

    let result = dispatch(&state, "braid.create", create_params).await;
    assert!(result.is_ok(), "braid.create failed: {result:?}");
    let braid = result.unwrap();

    assert_eq!(braid["ecop"]["source_gate"], "ironGate");
    let cross_gate = &braid["metadata"]["cross_gate"];
    assert_eq!(cross_gate["origin_gate"], "ironGate");
    assert_eq!(cross_gate["target_gate"], "strandGate");
    assert_eq!(cross_gate["trust_event"], "key_exchange");
    assert_eq!(cross_gate["origin_agent"], "did:key:z6MkIronGateAgent");

    let braid_id = braid["@id"].as_str().unwrap();
    let fetched = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await
    .unwrap();
    assert_eq!(fetched["ecop"]["source_gate"], "ironGate");
    assert_eq!(
        fetched["metadata"]["cross_gate"]["trust_event"],
        "key_exchange"
    );
}

#[tokio::test]
async fn test_create_gate_enrollment_braid() {
    let state = test_state();
    let create_params = serde_json::json!({
        "data_hash": "sha256:gate-enrollment-strand-001",
        "mime_type": "application/x-sweetgrass-trust-event",
        "size": 0,
        "source_gate": "strandGate",
        "cross_gate": {
            "origin_gate": "strandGate",
            "target_gate": "eastGate",
            "trust_event": "gate_enrollment",
            "origin_agent": "did:key:z6MkStrandAgent",
            "target_agent": "did:key:z6MkEastAgent",
            "family_id": "test-family-001"
        }
    });

    let result = dispatch(&state, "braid.create", create_params).await;
    assert!(result.is_ok(), "braid.create failed: {result:?}");
    let braid = result.unwrap();

    assert_eq!(braid["ecop"]["source_gate"], "strandGate");
    let cross_gate = &braid["metadata"]["cross_gate"];
    assert_eq!(cross_gate["trust_event"], "gate_enrollment");
    assert_eq!(cross_gate["target_agent"], "did:key:z6MkEastAgent");
    assert_eq!(cross_gate["family_id"], "test-family-001");
}

#[tokio::test]
async fn test_cross_gate_witness_gateway_tier() {
    let state = test_state();
    let agent = "did:key:z6MkGatewayWitness";
    let witness = Witness::from_gateway_ed25519(
        &Did::new(agent),
        b"cross-gate-sig-bytes",
        "ironGate->strandGate",
    );
    let witness_json = serde_json::to_value(&witness).unwrap();

    let create_params = serde_json::json!({
        "data_hash": "sha256:cross-gate-witness-001",
        "mime_type": "application/x-sweetgrass-trust-event",
        "size": 0,
        "source_gate": "ironGate",
        "cross_gate": {
            "origin_gate": "ironGate",
            "target_gate": "strandGate",
            "trust_event": "key_exchange",
            "origin_agent": agent
        },
        "witness": witness_json
    });

    let created = dispatch(&state, "braid.create", create_params)
        .await
        .unwrap();
    let braid_id = created["@id"].as_str().unwrap();

    let fetched = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await
    .unwrap();

    assert_eq!(fetched["witness"]["tier"], WITNESS_TIER_GATEWAY);
    assert_eq!(fetched["witness"]["context"], "ironGate->strandGate");
    assert_eq!(fetched["witness"]["agent"], agent);
    assert_eq!(fetched["witness"]["kind"], "signature");
}

#[tokio::test]
async fn test_query_by_source_gate() {
    let state = test_state();

    dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:iron-gate-query-001",
            "mime_type": "application/x-sweetgrass-trust-event",
            "size": 0,
            "source_gate": "ironGate"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:strand-gate-query-001",
            "mime_type": "application/x-sweetgrass-trust-event",
            "size": 0,
            "source_gate": "strandGate"
        }),
    )
    .await
    .unwrap();

    let iron_query = dispatch(
        &state,
        "braid.query",
        serde_json::json!({
            "filter": QueryFilter::new().with_source_gate("ironGate"),
        }),
    )
    .await
    .unwrap();

    let iron_braids = iron_query["braids"].as_array().unwrap();
    assert_eq!(iron_braids.len(), 1);
    assert_eq!(iron_braids[0]["ecop"]["source_gate"], "ironGate");
    assert_eq!(
        iron_braids[0]["data_hash"],
        "sha256:iron-gate-query-001"
    );

    let strand_query = dispatch(
        &state,
        "braid.query",
        serde_json::json!({
            "filter": QueryFilter::new().with_source_gate("strandGate"),
        }),
    )
    .await
    .unwrap();

    let strand_braids = strand_query["braids"].as_array().unwrap();
    assert_eq!(strand_braids.len(), 1);
    assert_eq!(strand_braids[0]["ecop"]["source_gate"], "strandGate");
}

#[tokio::test]
async fn test_cross_gate_activity_types() {
    let state = test_state();
    let create_params = serde_json::json!({
        "data_hash": "sha256:cross-gate-activity-001",
        "mime_type": "application/x-sweetgrass-trust-event",
        "size": 0,
        "source_gate": "ironGate",
        "was_generated_by": {
            "@id": "urn:activity:uuid:cross-gate-key-exchange",
            "@type": "KeyExchange",
            "started_at_time": 1_700_000_000_000_000_000_u64
        }
    });

    let created = dispatch(&state, "braid.create", create_params)
        .await
        .unwrap();
    let braid_id = created["@id"].as_str().unwrap();

    let fetched = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await
    .unwrap();

    let activity = &fetched["was_generated_by"];
    assert_eq!(activity["@type"], "KeyExchange");
    assert_eq!(
        activity["@id"],
        "urn:activity:uuid:cross-gate-key-exchange"
    );
}
