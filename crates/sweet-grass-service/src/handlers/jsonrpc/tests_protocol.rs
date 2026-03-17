// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Protocol-level tests for JSON-RPC 2.0 compliance.
//!
//! Tests the entrypoint (`process_single`, `handle_jsonrpc`), batch support
//! (Section 6), notification handling (Section 4.1), and capability discovery.
//! Domain-specific handler tests live in `tests.rs`.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

// ==================== process_single ====================

#[tokio::test]
async fn test_process_single_parse_error() {
    let state = test_state();
    let resp = process_single(&state, serde_json::json!("not an object"))
        .await
        .unwrap();
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, error_code::PARSE_ERROR);
}

#[tokio::test]
async fn test_process_single_invalid_version() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "1.0",
            "method": "health.check",
            "params": {},
            "id": 1
        }),
    )
    .await
    .unwrap();
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, error_code::INVALID_REQUEST);
}

#[tokio::test]
async fn test_process_single_success() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 42
        }),
    )
    .await
    .unwrap();
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
    assert_eq!(resp.id, 42);
}

#[tokio::test]
async fn test_process_single_method_not_found() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "nonexistent.method",
            "params": {},
            "id": 99
        }),
    )
    .await
    .unwrap();
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().code, error_code::METHOD_NOT_FOUND);
}

// ==================== Notification support (JSON-RPC 2.0 Section 4.1) ===========

#[tokio::test]
async fn test_notification_returns_none() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {}
        }),
    )
    .await;
    assert!(resp.is_none(), "notifications must produce no response");
}

#[tokio::test]
async fn test_notification_with_null_id_is_not_notification() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": null
        }),
    )
    .await;
    assert!(
        resp.is_some(),
        "id: null is a valid request, not a notification"
    );
}

// ==================== Batch support (JSON-RPC 2.0 Section 6) ====================

#[tokio::test]
async fn test_batch_multiple_requests() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!([
            {"jsonrpc": "2.0", "method": "health.check", "params": {}, "id": 1},
            {"jsonrpc": "2.0", "method": "health.check", "params": {}, "id": 2}
        ])),
    )
    .await;

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].id, 1);
    assert_eq!(parsed[1].id, 2);
}

#[tokio::test]
async fn test_batch_empty_returns_invalid_request() {
    let state = test_state();
    let resp = handle_jsonrpc(State(state), Json(serde_json::json!([]))).await;

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: JsonRpcResponse = serde_json::from_slice(&body).unwrap();
    assert!(parsed.error.is_some());
    assert_eq!(parsed.error.unwrap().code, error_code::INVALID_REQUEST);
}

#[tokio::test]
async fn test_batch_all_notifications_returns_no_content() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!([
            {"jsonrpc": "2.0", "method": "health.check", "params": {}},
            {"jsonrpc": "2.0", "method": "health.check", "params": {}}
        ])),
    )
    .await;

    assert_eq!(resp.status(), axum::http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_batch_mixed_requests_and_notifications() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!([
            {"jsonrpc": "2.0", "method": "health.check", "params": {}},
            {"jsonrpc": "2.0", "method": "health.check", "params": {}, "id": 1},
            {"jsonrpc": "2.0", "method": "health.check", "params": {}}
        ])),
    )
    .await;

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: Vec<JsonRpcResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.len(), 1, "only the non-notification should respond");
    assert_eq!(parsed[0].id, 1);
}

#[tokio::test]
async fn test_single_notification_returns_no_content() {
    let state = test_state();
    let resp = handle_jsonrpc(
        State(state),
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {}
        })),
    )
    .await;

    assert_eq!(resp.status(), axum::http::StatusCode::NO_CONTENT);
}

// ==================== capability.list ====================

#[tokio::test]
async fn test_capability_list_returns_all_methods() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        }),
    )
    .await
    .unwrap();
    let result = resp.result.unwrap();
    assert_eq!(result["primal"], "sweetgrass");
    assert!(!result["version"].as_str().unwrap().is_empty());
    assert!(result["domains"].is_object());
    assert!(result["methods"].is_array());
    assert!(
        result["capabilities"].is_array(),
        "neuralSpring S156 ecosystem compat: capabilities key must be present"
    );
    assert_eq!(result["capabilities"], result["methods"]);
}

#[tokio::test]
async fn test_capability_list_domains_are_grouped() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        }),
    )
    .await
    .unwrap();
    let result = resp.result.unwrap();
    let domains = result["domains"].as_object().unwrap();
    let braid_ops = domains["braid"].as_array().unwrap();
    assert!(braid_ops.iter().any(|v| v == "create"));
    assert!(braid_ops.iter().any(|v| v == "get"));
    assert!(braid_ops.iter().any(|v| v == "query"));
    let health_ops = domains["health"].as_array().unwrap();
    assert!(health_ops.iter().any(|v| v == "check"));
    let capability_ops = domains["capability"].as_array().unwrap();
    assert!(capability_ops.iter().any(|v| v == "list"));
}

#[tokio::test]
async fn test_capability_list_has_expected_domains() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        }),
    )
    .await
    .unwrap();
    let result = resp.result.unwrap();
    let domains = result["domains"].as_object().unwrap();
    assert!(domains.contains_key("braid"));
    assert!(domains.contains_key("health"));
    assert!(domains.contains_key("attribution"));
    assert!(domains.contains_key("anchoring"));
    assert!(domains.contains_key("provenance"));
    assert!(domains.contains_key("compression"));
    assert!(domains.contains_key("contribution"));
    assert!(domains.contains_key("capability"));
}

#[tokio::test]
async fn test_capability_list_method_count() {
    let state = test_state();
    let resp = process_single(
        &state,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "params": {},
            "id": 1
        }),
    )
    .await
    .unwrap();
    let result = resp.result.unwrap();
    let methods = result["methods"].as_array().unwrap();
    assert_eq!(methods.len(), 24);
}
