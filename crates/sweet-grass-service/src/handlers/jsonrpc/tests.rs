// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the JSON-RPC 2.0 dispatch handler.

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
        21,
        "dispatch table should have all 21 methods"
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
        "health.check",
        "capability.list",
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

// ==================== anchoring domain ====================

#[tokio::test]
async fn test_anchor_braid() {
    let state = test_state();
    let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let braid = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": format!("sha256:{hex}"), "mime_type": "application/octet-stream", "size": 0}),
    )
    .await
    .unwrap();
    let braid_id = braid["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": braid_id, "spine_id": "main"}),
    )
    .await
    .unwrap();
    assert_eq!(result["spine_id"], "main");
    assert_eq!(result["anchored"], false);
    assert_eq!(result["status"], "prepared");
    assert!(result["content_hash"].is_string());
}

#[tokio::test]
async fn test_anchor_braid_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": "nonexistent"}),
    )
    .await;
    assert_eq!(result.unwrap_err().0, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_anchor_braid_non_sha256() {
    let state = test_state();
    let braid = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:tooshort", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();
    let braid_id = braid["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await;
    assert_eq!(result.unwrap_err().0, error_code::INVALID_PARAMS);
}

#[tokio::test]
async fn test_verify_anchor() {
    let state = test_state();
    let braid = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:verifytest", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();
    let braid_id = braid["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "anchoring.verify",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();
    assert_eq!(result["anchored"], false);
    assert_eq!(result["verification_status"], "pending_integration");
}

#[tokio::test]
async fn test_verify_anchor_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "anchoring.verify",
        serde_json::json!({"braid_id": "nonexistent"}),
    )
    .await;
    assert_eq!(result.unwrap_err().0, error_code::NOT_FOUND);
}

// ==================== provenance domain ====================

#[tokio::test]
async fn test_provenance_graph() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:provgraph", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "provenance.graph",
        serde_json::json!({"entity": {"data_hash": "sha256:provgraph"}}),
    )
    .await;
    assert!(
        result.is_ok(),
        "provenance.graph should succeed: {result:?}"
    );
}

#[tokio::test]
async fn test_export_provo() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:provohash", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "provenance.export_provo",
        serde_json::json!({"hash": "sha256:provohash"}),
    )
    .await;
    assert!(result.is_ok(), "exportProvo should succeed: {result:?}");
}

#[tokio::test]
async fn test_export_graph_provo() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:graphprovohash", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "provenance.export_graph_provo",
        serde_json::json!({"entity": {"data_hash": "sha256:graphprovohash"}}),
    )
    .await;
    assert!(
        result.is_ok(),
        "exportGraphProvo should succeed: {result:?}"
    );
}

// ==================== attribution domain ====================

#[tokio::test]
async fn test_attribution_chain() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:attrchain", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "attribution.chain",
        serde_json::json!({"hash": "sha256:attrchain"}),
    )
    .await;
    assert!(
        result.is_ok(),
        "attribution.chain should succeed: {result:?}"
    );
}

#[tokio::test]
async fn test_calculate_rewards() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:rewardshash", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "attribution.calculate_rewards",
        serde_json::json!({"hash": "sha256:rewardshash", "value": 100.0}),
    )
    .await
    .unwrap();
    assert!(result.is_array());
    let arr = result.as_array().unwrap();
    assert!(!arr.is_empty());
    let total: f64 = arr.iter().map(|r| r["amount"].as_f64().unwrap()).sum();
    assert!((total - 100.0).abs() < 0.01);
}

#[tokio::test]
async fn test_top_contributors() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:topcontrib", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "attribution.top_contributors",
        serde_json::json!({"hash": "sha256:topcontrib", "limit": 5}),
    )
    .await
    .unwrap();
    assert!(result.is_array());
}

#[tokio::test]
async fn test_top_contributors_default_limit() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:topdefault", "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "attribution.top_contributors",
        serde_json::json!({"hash": "sha256:topdefault"}),
    )
    .await
    .unwrap();
    assert!(result.is_array());
}

// ==================== compression domain ====================

#[tokio::test]
async fn test_compress_session() {
    let state = test_state();
    let params = serde_json::json!({
        "id": "compress-session-1",
        "vertices": [{
            "id": "v1",
            "data_hash": "sha256:compresstest",
            "mime_type": "text/plain",
            "agent": "did:key:z6MkTest",
            "size": 100,
            "parents": [],
            "timestamp": 1000,
            "activity_type": "Creation",
            "committed": true
        }],
        "started_at": 1000,
        "outcome": "Committed",
        "compression_hint": "Auto",
        "compute_units": 1.0
    });

    let result = dispatch(&state, "compression.compress_session", params).await;
    assert!(result.is_ok(), "compress should succeed: {result:?}");
}

#[tokio::test]
async fn test_create_meta_braid() {
    let state = test_state();
    let b1 = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:meta1", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();
    let b2 = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:meta2", "mime_type": "text/plain", "size": 2}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "compression.create_meta_braid",
        serde_json::json!({
            "braid_ids": [b1["@id"], b2["@id"]],
            "summary_type": {"Session": {"session_id": "meta-session"}}
        }),
    )
    .await;
    assert!(result.is_ok(), "createMetaBraid should succeed: {result:?}");
}

// ==================== contribution domain extended ====================

#[tokio::test]
async fn test_record_dehydration_with_operations() {
    let state = test_state();
    let params = serde_json::json!({
        "source_primal": TEST_SOURCE_PRIMAL,
        "session_id": "dehydrate-session-1",
        "merkle_root": "sha256:merkleroot01",
        "vertex_count": 10,
        "branch_count": 3,
        "agents": ["did:key:z6MkAlice"],
        "operations": [{
            "op_type": "create",
            "content_hash": "sha256:op1hash",
            "agent": "did:key:z6MkAlice",
            "timestamp": 500_000
        }],
        "session_start": 100_000,
        "dehydrated_at": 200_000,
        "niche": "rootpulse",
        "compression_ratio": 0.42
    });

    let result = dispatch(&state, "contribution.record_dehydration", params).await;
    assert!(
        result.is_ok(),
        "recordDehydration should succeed: {result:?}"
    );
    let resp = result.unwrap();
    assert_eq!(resp["session_id"], "dehydrate-session-1");
    assert_eq!(resp["braids_created"], 1);
}

#[tokio::test]
async fn test_record_dehydration_empty_operations() {
    let state = test_state();
    let params = serde_json::json!({
        "source_primal": TEST_SOURCE_PRIMAL,
        "session_id": "dehydrate-empty-ops",
        "merkle_root": "sha256:emptymerkle",
        "vertex_count": 5,
        "branch_count": 1,
        "agents": ["did:key:z6MkSolo"],
        "operations": [],
        "session_start": 100_000,
        "dehydrated_at": 200_000
    });

    let result = dispatch(&state, "contribution.record_dehydration", params).await;
    assert!(
        result.is_ok(),
        "dehydration with empty ops should succeed: {result:?}"
    );
    let resp = result.unwrap();
    assert_eq!(resp["braids_created"], 1);
    assert_eq!(resp["merkle_root"], "sha256:emptymerkle");
}

#[tokio::test]
async fn test_record_dehydration_no_agents_fallback() {
    let state = test_state();
    let params = serde_json::json!({
        "source_primal": TEST_SOURCE_PRIMAL,
        "session_id": "dehydrate-no-agents",
        "merkle_root": "sha256:noagentmerkle",
        "vertex_count": 1,
        "branch_count": 0,
        "agents": [],
        "operations": [],
        "session_start": 0,
        "dehydrated_at": 1
    });

    let result = dispatch(&state, "contribution.record_dehydration", params).await;
    assert!(
        result.is_ok(),
        "dehydration with no agents should use fallback DID"
    );
}

// ==================== handle_jsonrpc entrypoint ====================

#[tokio::test]
async fn test_handle_jsonrpc_parse_error() {
    let state = test_state();
    let resp = handle_jsonrpc(State(state), Json(serde_json::json!("not an object"))).await;
    assert!(resp.0.error.is_some());
    assert_eq!(resp.0.error.unwrap().code, error_code::PARSE_ERROR);
}

#[tokio::test]
async fn test_handle_jsonrpc_invalid_version() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!({
            "jsonrpc": "1.0",
            "method": "health.check",
            "params": {},
            "id": 1
        })),
    )
    .await;
    assert!(resp.0.error.is_some());
    assert_eq!(resp.0.error.unwrap().code, error_code::INVALID_REQUEST);
}

#[tokio::test]
async fn test_handle_jsonrpc_success() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 42
        })),
    )
    .await;
    assert!(resp.0.result.is_some());
    assert!(resp.0.error.is_none());
    assert_eq!(resp.0.id, 42);
}

#[tokio::test]
async fn test_handle_jsonrpc_method_not_found() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "nonexistent.method",
            "params": {},
            "id": 99
        })),
    )
    .await;
    assert!(resp.0.error.is_some());
    assert_eq!(resp.0.error.unwrap().code, error_code::METHOD_NOT_FOUND);
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
