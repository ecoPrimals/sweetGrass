// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Composition contract tests.
//!
//! Validate the exact payload shapes from the provenance trio operational
//! handoff (`PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md`), the
//! skunkBat JH-5 Phase 3 audit pipeline, and downstream NFT seal workflows.
//!
//! These tests ensure that sweetGrass correctly accepts the flattened
//! parameter shapes used by rhizoCrypt, loamSpine, and wetSpring in
//! live compositions.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

#[tokio::test]
async fn test_composition_braid_create_flattened_name_description() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e",
            "name": "abg-pipeline-20260504",
            "mime_type": "application/x-provenance-pipeline",
            "description": "ABG Full Pipeline - 24 events across wetSpring validators",
            "size": 24
        }),
    )
    .await
    .unwrap();

    assert!(result["@id"].as_str().unwrap().starts_with("urn:braid:"));
    assert_eq!(
        result["metadata"]["title"], "abg-pipeline-20260504",
        "flattened name should map to metadata.title"
    );
    assert!(
        result["metadata"]["description"]
            .as_str()
            .unwrap()
            .contains("ABG Full Pipeline"),
        "flattened description should map to metadata.description"
    );
    assert_eq!(result["mime_type"], "application/x-provenance-pipeline");
    assert_eq!(result["size"], 24);
}

#[tokio::test]
async fn test_composition_braid_create_structured_metadata_takes_precedence() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:aabbccdd",
            "mime_type": "text/plain",
            "size": 100,
            "name": "flattened-name",
            "description": "flattened-desc",
            "metadata": {
                "title": "structured-title",
                "description": "structured-desc"
            }
        }),
    )
    .await
    .unwrap();

    assert_eq!(
        result["metadata"]["title"], "structured-title",
        "structured metadata.title should take precedence over flattened name"
    );
    assert_eq!(
        result["metadata"]["description"], "structured-desc",
        "structured metadata.description should take precedence over flattened description"
    );
}

#[tokio::test]
async fn test_composition_braid_create_source_session_and_merkle_root() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:sourcesession",
            "mime_type": "application/x-provenance-session",
            "size": 10,
            "source_session": "019df42d-0fba-7170-a216-2f3b282e3fb9",
            "source_merkle_root": "292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e"
        }),
    )
    .await
    .unwrap();

    let custom = &result["metadata"]["custom"];
    assert_eq!(
        custom["source_session"],
        "019df42d-0fba-7170-a216-2f3b282e3fb9"
    );
    assert_eq!(
        custom["source_merkle_root"],
        "292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e"
    );
}

#[tokio::test]
async fn test_composition_braid_create_hex_hash_without_prefix() {
    let state = test_state();
    let merkle = "292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e";
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": merkle,
            "mime_type": "application/octet-stream",
            "size": 1
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["data_hash"], merkle);
    assert!(result["@id"].as_str().unwrap().contains(merkle));
}

#[tokio::test]
async fn test_composition_braid_create_with_tags() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:tagged",
            "mime_type": "text/plain",
            "size": 5,
            "tags": ["provenance", "pipeline", "wetspring"]
        }),
    )
    .await
    .unwrap();

    let tags = result["metadata"]["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&serde_json::json!("provenance")));
}

#[tokio::test]
async fn test_composition_nft_seal_braid_commit_round_trip() {
    let state = test_state();
    let hex = "a1".repeat(32);
    let hash = format!("sha256:{hex}");
    let created = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": &hash,
            "mime_type": "application/x-nft-certificate",
            "size": 256,
            "name": "NFT pipeline seal",
            "source_session": "session-uuid-abc"
        }),
    )
    .await
    .unwrap();

    let braid_id = created["@id"].as_str().unwrap();
    assert!(braid_id.starts_with("urn:braid:"));

    let committed = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": braid_id, "spine_id": "nft-spine"}),
    )
    .await
    .unwrap();

    assert_eq!(committed["spine_id"], "nft-spine");
    assert!(
        committed["data_hash_bytes"].is_string(),
        "commit should produce base64 hash bytes for loamSpine"
    );
    assert_eq!(committed["data_hash"], hash);
}

#[tokio::test]
async fn test_composition_skunkbat_attribution_witness() {
    let state = test_state();
    let hex = "b2".repeat(32);
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": format!("sha256:{hex}"),
            "mime_type": "application/x-security-event",
            "size": 1,
            "name": "security-event-001"
        }),
    )
    .await
    .unwrap();

    let witness = dispatch(
        &state,
        "attribution.witness",
        serde_json::json!({
            "hash": format!("sha256:{hex}"),
            "witness_agent": "did:key:z6MkSkunkBatSecurity",
            "event_type": "security",
            "payload": {
                "severity": "high",
                "source": "defense.log",
                "event_kind": "intrusion_attempt",
                "forwarded_via": "dag.event.append"
            }
        }),
    )
    .await
    .unwrap();

    assert_eq!(witness["event_type"], "security");
    assert_eq!(witness["witness_agent"], "did:key:z6MkSkunkBatSecurity");
    assert!(witness["witnessed_at"].is_string());
    assert_eq!(witness["payload"]["source"], "defense.log");
}

#[tokio::test]
async fn test_composition_full_provenance_trio_pipeline() {
    let state = test_state();
    let merkle_hex = "292ebbcf8f02561aaa6c67b532ebbefc14c32192cf3dfb733ce81e45fba50f9e";

    let braid = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": merkle_hex,
            "mime_type": "application/x-provenance-pipeline",
            "size": 24,
            "name": "abg-pipeline",
            "description": "24 events from wetSpring validators",
            "source_session": "019df42d-0fba-7170-a216-2f3b282e3fb9",
            "source_merkle_root": merkle_hex,
            "tags": ["provenance", "pipeline"]
        }),
    )
    .await
    .unwrap();

    let braid_id = braid["@id"].as_str().unwrap();
    assert!(braid_id.starts_with("urn:braid:"));
    assert_eq!(braid["metadata"]["title"], "abg-pipeline");
    assert_eq!(
        braid["metadata"]["custom"]["source_session"],
        "019df42d-0fba-7170-a216-2f3b282e3fb9"
    );
    assert_eq!(
        braid["metadata"]["custom"]["source_merkle_root"],
        merkle_hex
    );

    let chain = dispatch(
        &state,
        "attribution.chain",
        serde_json::json!({"hash": merkle_hex}),
    )
    .await
    .unwrap();
    assert!(chain["contributors"].is_array());

    let graph = dispatch(
        &state,
        "provenance.graph",
        serde_json::json!({"entity": {"data_hash": merkle_hex}}),
    )
    .await
    .unwrap();
    assert!(graph.is_object());
}
