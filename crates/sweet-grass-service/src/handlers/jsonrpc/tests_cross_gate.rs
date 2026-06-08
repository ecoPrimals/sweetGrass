// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for cross-gate attribution braids and `source_gate` query filtering.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::dehydration::{WITNESS_TIER_GATEWAY, Witness};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

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
    let fetched = dispatch(&state, "braid.get", serde_json::json!({ "id": braid_id }))
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

    let fetched = dispatch(&state, "braid.get", serde_json::json!({ "id": braid_id }))
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
    assert_eq!(iron_braids[0]["data_hash"], "sha256:iron-gate-query-001");

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

    let fetched = dispatch(&state, "braid.get", serde_json::json!({ "id": braid_id }))
        .await
        .unwrap();

    let activity = &fetched["was_generated_by"];
    assert_eq!(activity["@type"], "KeyExchange");
    assert_eq!(activity["@id"], "urn:activity:uuid:cross-gate-key-exchange");
}

// ==================== trust.event weaving ====================

#[tokio::test]
async fn test_trust_event_weaves_key_exchange() {
    let state = test_state();
    let result = dispatch(
        &state,
        "trust.event",
        serde_json::json!({
            "cross_gate": {
                "origin_gate": "ironGate",
                "target_gate": "strandGate",
                "trust_event": "key_exchange",
                "origin_agent": "did:key:z6MkIronGateAgent",
                "target_agent": "did:key:z6MkStrandGateAgent"
            }
        }),
    )
    .await;
    assert!(result.is_ok(), "trust.event failed: {result:?}");
    let braid = result.unwrap();

    assert_eq!(braid["mime_type"], "application/vnd.ecoprimals.trust-event");
    assert_eq!(braid["ecop"]["source_gate"], "ironGate");
    assert_eq!(braid["was_attributed_to"], "did:key:z6MkIronGateAgent");

    let activity = &braid["was_generated_by"];
    assert_eq!(activity["@type"], "KeyExchange");
    let assoc = &activity["was_associated_with"][0];
    assert_eq!(assoc["agent"], "did:key:z6MkIronGateAgent");
    assert_eq!(assoc["on_behalf_of"], "did:key:z6MkStrandGateAgent");

    let cross_gate = &braid["metadata"]["cross_gate"];
    assert_eq!(cross_gate["origin_gate"], "ironGate");
    assert_eq!(cross_gate["target_gate"], "strandGate");
    assert_eq!(cross_gate["trust_event"], "key_exchange");
}

#[tokio::test]
async fn test_trust_event_mesh_join() {
    let state = test_state();
    let result = dispatch(
        &state,
        "trust.event",
        serde_json::json!({
            "cross_gate": {
                "origin_gate": "westGate",
                "target_gate": "eastGate",
                "trust_event": "mesh_join",
                "origin_agent": "did:key:z6MkWestAgent",
                "family_id": "west-family"
            }
        }),
    )
    .await;
    assert!(result.is_ok(), "trust.event failed: {result:?}");
    let braid = result.unwrap();

    let activity = &braid["was_generated_by"];
    assert_eq!(activity["@type"], "MeshJoin");
    assert!(activity["was_associated_with"][0]["on_behalf_of"].is_null());
    assert_eq!(braid["metadata"]["cross_gate"]["family_id"], "west-family");
}

#[tokio::test]
async fn test_trust_event_with_gateway_witness() {
    use base64::Engine;
    let state = test_state();
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(b"fake-ed25519-signature-bytes");

    let result = dispatch(
        &state,
        "trust.event",
        serde_json::json!({
            "cross_gate": {
                "origin_gate": "ironGate",
                "target_gate": "strandGate",
                "trust_event": "cross_gate_attestation",
                "origin_agent": "did:key:z6MkWitnessAgent"
            },
            "signature": sig_b64
        }),
    )
    .await;
    assert!(result.is_ok(), "trust.event failed: {result:?}");
    let braid = result.unwrap();

    assert_eq!(braid["witness"]["tier"], WITNESS_TIER_GATEWAY);
    assert_eq!(braid["witness"]["context"], "ironGate->strandGate");
    assert_eq!(braid["witness"]["kind"], "signature");
}

#[tokio::test]
async fn test_trust_event_deterministic_hash() {
    let state = test_state();
    let params = serde_json::json!({
        "cross_gate": {
            "origin_gate": "gateA",
            "target_gate": "gateB",
            "trust_event": "gate_enrollment",
            "origin_agent": "did:key:z6MkDeterm"
        },
        "timestamp": 1_700_000_000_000_000_000_u64
    });

    let braid1 = dispatch(&state, "trust.event", params.clone())
        .await
        .unwrap();
    let hash1 = braid1["data_hash"].as_str().unwrap().to_string();

    assert!(
        hash1.starts_with("trust:gateA:gateB:gate_enrollment:"),
        "content hash should be deterministic seed: {hash1}"
    );
}

#[tokio::test]
async fn test_trust_event_roundtrip_via_get() {
    let state = test_state();
    let created = dispatch(
        &state,
        "trust.event",
        serde_json::json!({
            "cross_gate": {
                "origin_gate": "eastGate",
                "target_gate": "westGate",
                "trust_event": "trust_issuer_registered",
                "origin_agent": "did:key:z6MkRoundTrip",
                "target_agent": "did:key:z6MkTarget"
            }
        }),
    )
    .await
    .unwrap();

    let braid_id = created["@id"].as_str().unwrap();
    let fetched = dispatch(&state, "braid.get", serde_json::json!({ "id": braid_id }))
        .await
        .unwrap();

    assert_eq!(fetched["was_attributed_to"], "did:key:z6MkRoundTrip");
    assert_eq!(
        fetched["metadata"]["cross_gate"]["trust_event"],
        "trust_issuer_registered"
    );
    assert_eq!(fetched["was_generated_by"]["@type"], "TrustEstablishment");
}

#[tokio::test]
async fn test_provenance_chain_beardog_to_sweetgrass() {
    let state = test_state();

    use base64::Engine as _;

    let trust_event = serde_json::json!({
        "cross_gate": {
            "origin_gate": "southGate",
            "target_gate": "strandGate",
            "trust_event": "trust_issuer_registered",
            "origin_agent": "did:key:z6MkBearDogSouth",
            "target_agent": "did:key:z6MkRhizoCryptStrand"
        },
        "signature": base64::engine::general_purpose::STANDARD.encode(b"mock-ed25519-sig-from-beardog"),
        "timestamp": 1_717_600_000
    });

    let braid = dispatch(&state, "trust.event", trust_event).await.unwrap();

    assert_eq!(braid["was_attributed_to"], "did:key:z6MkBearDogSouth");
    assert_eq!(braid["mime_type"], "application/vnd.ecoprimals.trust-event");
    assert!(braid["size"].as_u64().unwrap() == 0);

    let activity = &braid["was_generated_by"];
    assert_eq!(activity["@type"], "TrustEstablishment");
    let assoc = &activity["was_associated_with"];
    assert!(assoc.is_array(), "was_associated_with should be an array");
    assert_eq!(assoc[0]["agent"], "did:key:z6MkBearDogSouth");
    assert_eq!(
        assoc[0]["on_behalf_of"], "did:key:z6MkRhizoCryptStrand",
        "target_agent should be wired as on_behalf_of delegation"
    );

    let cga = &braid["metadata"]["cross_gate"];
    assert_eq!(cga["origin_gate"], "southGate");
    assert_eq!(cga["target_gate"], "strandGate");
    assert_eq!(cga["trust_event"], "trust_issuer_registered");

    assert_eq!(braid["ecop"]["source_gate"], "southGate");

    let witness = &braid["witness"];
    assert_eq!(witness["tier"], "gateway");

    let braid_id = braid["@id"].as_str().unwrap();
    let refetched = dispatch(&state, "braid.get", serde_json::json!({ "id": braid_id }))
        .await
        .unwrap();
    assert_eq!(refetched["was_attributed_to"], "did:key:z6MkBearDogSouth");
}

#[tokio::test]
async fn test_all_mesh_event_types_weave_braids() {
    let state = test_state();

    let event_types = [
        ("key_exchange", "KeyExchange"),
        ("trust_issuer_registered", "TrustEstablishment"),
        ("gate_enrollment", "GateEnrollment"),
        ("family_enrollment", "GateEnrollment"),
        ("cross_gate_attestation", "CrossGateAttestation"),
        ("mesh_join", "MeshJoin"),
        ("mesh_leave", "MeshLeave"),
    ];

    for (event, expected_activity) in &event_types {
        let result = dispatch(
            &state,
            "trust.event",
            serde_json::json!({
                "cross_gate": {
                    "origin_gate": "gateA",
                    "target_gate": "gateB",
                    "trust_event": event,
                    "origin_agent": "did:key:z6MkExhaustive"
                }
            }),
        )
        .await;

        let braid = result.unwrap_or_else(|e| {
            panic!("trust.event failed for {event}: {e:?}");
        });

        assert_eq!(
            braid["was_generated_by"]["@type"], *expected_activity,
            "event type {event} should map to {expected_activity}"
        );
        assert_eq!(
            braid["mime_type"], "application/vnd.ecoprimals.trust-event",
            "MIME should be trust-event for {event}"
        );
    }
}

#[tokio::test]
async fn test_trust_event_provenance_query_by_gate() {
    let state = test_state();

    for gate in ["ironGate", "biomeGate", "flockGate"] {
        dispatch(
            &state,
            "trust.event",
            serde_json::json!({
                "cross_gate": {
                    "origin_gate": gate,
                    "target_gate": "strandGate",
                    "trust_event": "key_exchange",
                    "origin_agent": format!("did:key:z6Mk{gate}Agent")
                }
            }),
        )
        .await
        .unwrap();
    }

    let iron_filter = QueryFilter::new().with_source_gate("ironGate");
    let iron_braids = state
        .store
        .query(&iron_filter, QueryOrder::NewestFirst)
        .await
        .unwrap();
    assert_eq!(
        iron_braids.braids.len(),
        1,
        "should find 1 ironGate trust braid"
    );
    assert_eq!(
        iron_braids.braids[0].ecop.source_gate.as_deref(),
        Some("ironGate")
    );

    let trust_filter = QueryFilter::new().with_mime_type("application/vnd.ecoprimals.trust-event");
    let all_trust = state
        .store
        .query(&trust_filter, QueryOrder::NewestFirst)
        .await
        .unwrap();
    assert_eq!(
        all_trust.braids.len(),
        3,
        "should find all 3 trust event braids"
    );
}
