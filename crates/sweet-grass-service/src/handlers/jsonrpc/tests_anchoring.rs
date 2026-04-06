// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the `anchoring.*` JSON-RPC handlers.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

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

// ==================== Extended Coverage ====================

#[tokio::test]
async fn test_anchoring_anchor_success() {
    let state = test_state();
    let hex = "a".repeat(64);
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": format!("sha256:{hex}"), "mime_type": "text/plain", "size": 10}),
    ).await.unwrap();

    let filter = sweet_grass_store::QueryFilter::new().with_hash(format!("sha256:{hex}"));
    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .unwrap();
    let braid_id = result.braids[0].id.as_str();

    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await;
    assert!(result.is_ok());
    let v = result.unwrap();
    assert_eq!(v["status"], "prepared");
    assert_eq!(v["anchored"], false);
    assert_eq!(v["spine_id"], "default");
    assert!(v["content_hash"].is_string());
}

#[tokio::test]
async fn test_anchoring_anchor_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": "urn:braid:uuid:00000000-0000-0000-0000-000000000000"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_anchoring_anchor_custom_spine() {
    let state = test_state();
    let hex = "b".repeat(64);
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": format!("sha256:{hex}"), "mime_type": "text/plain", "size": 5}),
    ).await.unwrap();

    let filter = sweet_grass_store::QueryFilter::new().with_hash(format!("sha256:{hex}"));
    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .unwrap();
    let braid_id = result.braids[0].id.as_str();

    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": braid_id, "spine_id": "custom-spine"}),
    )
    .await
    .unwrap();
    assert_eq!(result["spine_id"], "custom-spine");
}

#[tokio::test]
async fn test_anchoring_anchor_invalid_hash() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "nothex:short", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();

    let filter = sweet_grass_store::QueryFilter::new().with_hash("nothex:short");
    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .unwrap();
    let braid_id = result.braids[0].id.as_str();

    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await;
    assert!(result.is_err());
    let (code, msg) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
    assert!(msg.contains("sha256"));
}

#[tokio::test]
async fn test_anchoring_verify_success() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:verifyanchor", "mime_type": "text/plain", "size": 1}),
    ).await.unwrap();

    let filter = sweet_grass_store::QueryFilter::new().with_hash("sha256:verifyanchor");
    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .unwrap();
    let braid_id = result.braids[0].id.as_str();

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
async fn test_anchoring_verify_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "anchoring.verify",
        serde_json::json!({"braid_id": "urn:braid:uuid:00000000-0000-0000-0000-000000000000"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_anchoring_anchor_invalid_params() {
    let state = test_state();
    let result = dispatch(
        &state,
        "anchoring.anchor",
        serde_json::json!({"not_a_real_field": true}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}
