// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Core tests for JSON-RPC 2.0 dispatch: protocol, braid CRUD, health,
//! helpers, and `DispatchOutcome` classification.
//!
//! Domain-specific handler tests live in sibling modules:
//! - `tests_anchoring` — `anchoring.*`
//! - `tests_attribution` — `attribution.*`
//! - `tests_compression` — `compression.*`
//! - `tests_contribution` — `contribution.*` + `pipeline.*`
//! - `tests_provenance` — `provenance.*`

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::test_fixtures::TEST_SOURCE_PRIMAL;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

#[test]
fn test_parse_error_response() {
    let resp = JsonRpcResponse::error(
        serde_json::Value::Null,
        error_code::PARSE_ERROR,
        "test parse error",
    );
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_some());
    assert!(resp.result.is_none());
    assert_eq!(resp.error.unwrap().code, error_code::PARSE_ERROR);
}

#[test]
fn test_success_response() {
    let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_method_not_found() {
    let state = test_state();
    let result = dispatch(&state, "nonexistent.method", serde_json::json!({})).await;
    assert!(result.is_err());
    let (code, _msg) = result.unwrap_err();
    assert_eq!(code, error_code::METHOD_NOT_FOUND);
}

#[test]
fn test_invalid_version_detection() {
    let request = serde_json::json!({
        "jsonrpc": "1.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let parsed: JsonRpcRequest = serde_json::from_value(request).unwrap();
    assert_ne!(parsed.jsonrpc, "2.0");
}

#[tokio::test]
async fn test_health_method() {
    let state = test_state();
    let result = dispatch(&state, "health.check", serde_json::json!({})).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["status"], "healthy");
    assert_eq!(value["braid_count"], 0);
}

#[tokio::test]
async fn test_create_and_get_braid() {
    let state = test_state();

    let create_params = serde_json::json!({
        "data_hash": "sha256:testjsonrpc",
        "mime_type": "application/json",
        "size": 512
    });
    let result = dispatch(&state, "braid.create", create_params).await;
    assert!(result.is_ok());
    let braid = result.unwrap();
    let braid_id = braid["@id"].as_str().unwrap().to_string();

    let get_result = dispatch(&state, "braid.get", serde_json::json!({"id": braid_id})).await;
    assert!(get_result.is_ok());
}

#[tokio::test]
async fn test_get_braid_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({"id": "nonexistent"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_query_braids() {
    let state = test_state();
    let result = dispatch(&state, "braid.query", serde_json::json!({"filter": {}})).await;
    assert!(result.is_ok());
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
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}

#[tokio::test]
async fn test_delete_braid() {
    let state = test_state();

    let create_result = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:deleteme",
            "mime_type": "text/plain",
            "size": 10
        }),
    )
    .await
    .unwrap();
    let braid_id = create_result["@id"].as_str().unwrap().to_string();

    let delete_result = dispatch(&state, "braid.delete", serde_json::json!({"id": braid_id})).await;
    assert!(delete_result.is_ok());
}

#[test]
fn test_all_error_codes() {
    assert_eq!(error_code::PARSE_ERROR, -32700);
    assert_eq!(error_code::INVALID_REQUEST, -32600);
    assert_eq!(error_code::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_code::INVALID_PARAMS, -32602);
    assert_eq!(error_code::INTERNAL_ERROR, -32603);
    assert_eq!(error_code::NOT_FOUND, -32004);
    assert_eq!(error_code::PERMISSION_DENIED, -32001);
    assert_eq!(error_code::UNAUTHORIZED, -32000);
}

#[tokio::test]
async fn test_record_contribution_dispatch() {
    let state = test_state();
    let params = serde_json::json!({
        "agent": "did:key:z6MkContributor",
        "role": "Creator",
        "content_hash": "sha256:rpc-contrib-test",
        "mime_type": "application/json",
        "size": 64
    });

    let result = dispatch(&state, "contribution.record", params).await;
    assert!(result.is_ok());
    let braid = result.unwrap();
    assert_eq!(braid["data_hash"], "sha256:rpc-contrib-test");
    assert!(braid["@id"].as_str().unwrap().starts_with("urn:braid:"));
}

#[tokio::test]
async fn test_record_session_dispatch() {
    let state = test_state();
    let params = serde_json::json!({
        "session_id": "rpc-session-123",
        "source_primal": TEST_SOURCE_PRIMAL,
        "contributions": [
            {
                "agent": "did:key:z6MkAgent1",
                "role": "Creator",
                "content_hash": "sha256:session-hash-1",
                "mime_type": "text/plain",
                "size": 10
            },
            {
                "agent": "did:key:z6MkAgent2",
                "role": "Contributor",
                "content_hash": "sha256:session-hash-2",
                "mime_type": "application/json",
                "size": 20
            }
        ]
    });

    let result = dispatch(&state, "contribution.record_session", params).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["session_id"], "rpc-session-123");
    assert_eq!(response["braids_created"], 2);
    let braid_ids = response["braid_ids"].as_array().unwrap();
    assert_eq!(braid_ids.len(), 2);
}

#[test]
fn test_dispatch_table_completeness() {
    assert_eq!(
        METHODS.len(),
        37,
        "dispatch table should have all 37 methods (33 domain + lifecycle + 3 auth)"
    );

    let expected = [
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
        "attribution.witness",
        "compression.compress_session",
        "compression.create_meta_braid",
        "contribution.record",
        "contribution.record_session",
        "contribution.record_dehydration",
        "pipeline.attribute",
        "health.check",
        "health.liveness",
        "health.readiness",
        "identity.get",
        "composition.tower_health",
        "composition.node_health",
        "composition.nest_health",
        "composition.nucleus_health",
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

// ==================== Wire-Name Alias Resolution (GAP-36) ====================

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

// ==================== lifecycle.status ====================

#[tokio::test]
async fn test_lifecycle_status_returns_running() {
    let state = test_state();
    let result = dispatch(
        &state,
        "lifecycle.status",
        serde_json::json!({}),
    )
    .await
    .unwrap();
    assert_eq!(result["status"], "running");
    assert!(result["version"].is_string());
    assert!(result["gate_mode"].is_string());
}

// ==================== braid domain extended ====================

#[tokio::test]
async fn test_braid_get_by_hash() {
    let state = test_state();
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:byHashTest", "mime_type": "text/plain", "size": 64}),
    )
    .await
    .unwrap();
    let hash = create["data_hash"].as_str().unwrap();

    let found = dispatch(
        &state,
        "braid.get_by_hash",
        serde_json::json!({"hash": hash}),
    )
    .await
    .unwrap();
    assert_eq!(found["data_hash"], hash);
}

#[tokio::test]
async fn test_braid_get_by_hash_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.get_by_hash",
        serde_json::json!({"hash": "sha256:nonexistent"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_braid_commit() {
    let state = test_state();
    let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let hash = format!("sha256:{hex}");
    let braid = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": hash, "mime_type": "application/octet-stream", "size": 0}),
    )
    .await
    .unwrap();
    let braid_id = braid["@id"].as_str().unwrap();

    let commit = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();
    assert_eq!(commit["spine_id"], "default");
    assert!(commit["data_hash_bytes"].is_string());
    assert_eq!(commit["is_signed"], false);
}

#[tokio::test]
async fn test_braid_commit_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": "nonexistent"}),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_braid_query_with_order() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:order1", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:order2", "mime_type": "text/plain", "size": 2}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "braid.query",
        serde_json::json!({"filter": {}, "order": "OldestFirst"}),
    )
    .await
    .unwrap();
    assert_eq!(result["total_count"], 2);
}

// ==================== health domain extended ====================

#[tokio::test]
async fn test_health_liveness() {
    let state = test_state();
    let result = dispatch(&state, "health.liveness", serde_json::json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["alive"], true);
}

#[tokio::test]
async fn test_health_readiness() {
    let state = test_state();
    let result = dispatch(&state, "health.readiness", serde_json::json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["ready"], true);
}

// ==================== helper unit tests ====================

#[test]
fn test_parse_params_valid() {
    let val = serde_json::json!({"id": "test-id"});
    let result: Result<super::braid::GetBraidParams, _> = parse_params(val);
    assert!(result.is_ok());
}

#[test]
fn test_parse_params_invalid() {
    let val = serde_json::json!({"wrong_field": 123});
    let result: Result<super::braid::GetBraidParams, _> = parse_params(val);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, error_code::INVALID_PARAMS);
}

#[test]
fn test_to_value_success() {
    let data = serde_json::json!({"key": "value"});
    let result = to_value(&data);
    assert!(result.is_ok());
}

#[test]
fn test_internal_error() {
    let err = internal("something went wrong");
    assert_eq!(err.0, error_code::INTERNAL_ERROR);
    assert!(err.1.contains("something went wrong"));
}

// ==================== DispatchOutcome ====================

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

// ==================== Braid Commit Coverage ====================

#[tokio::test]
async fn test_braid_commit_success() {
    let state = test_state();
    let hex = "c".repeat(64);
    let created = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": format!("sha256:{hex}"), "mime_type": "text/plain", "size": 10}),
    ).await.unwrap();
    let braid_id = created["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();
    assert_eq!(result["spine_id"], "default");
    assert!(result["uuid"].is_string());
    assert!(result["data_hash_bytes"].is_string());
    assert_eq!(result["is_signed"], false);
}

#[tokio::test]
async fn test_braid_commit_missing_braid() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": "urn:braid:uuid:00000000-0000-0000-0000-000000000000"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_braid_commit_custom_spine() {
    let state = test_state();
    let hex = "d".repeat(64);
    let created = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": format!("sha256:{hex}"), "mime_type": "text/plain", "size": 5}),
    ).await.unwrap();
    let braid_id = created["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.commit",
        serde_json::json!({"braid_id": braid_id, "spine_id": "my-spine"}),
    )
    .await
    .unwrap();
    assert_eq!(result["spine_id"], "my-spine");
}

// ==================== Composition Contract Tests ====================
// Validate the exact payload shapes from the provenance trio operational
// handoff (PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md) and the
// skunkBat JH-5 Phase 3 audit pipeline.

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
        result["metadata"]["title"],
        "abg-pipeline-20260504",
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
    assert_eq!(braid["metadata"]["custom"]["source_merkle_root"], merkle_hex);

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
