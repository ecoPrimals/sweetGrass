// SPDX-License-Identifier: AGPL-3.0-only
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
    assert_eq!(error_code::NOT_FOUND, -32001);
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
        27,
        "dispatch table should have all 27 methods"
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
        "compression.compress_session",
        "compression.create_meta_braid",
        "contribution.record",
        "contribution.record_session",
        "contribution.record_dehydration",
        "pipeline.attribute",
        "health.check",
        "health.liveness",
        "health.readiness",
        "capabilities.list",
        "capability.list",
        "tools.list",
        "tools.call",
    ];
    for name in expected {
        assert!(find_handler(name).is_some(), "missing handler for: {name}");
    }
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
