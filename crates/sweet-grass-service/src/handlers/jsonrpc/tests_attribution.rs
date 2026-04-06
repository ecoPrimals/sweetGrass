// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the `attribution.*` JSON-RPC handlers.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

async fn create_braid_for_attribution(state: &AppState, hash: &str) {
    dispatch(
        state,
        "braid.create",
        serde_json::json!({"data_hash": hash, "mime_type": "text/plain", "size": 10}),
    )
    .await
    .unwrap();
}

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

// ==================== Extended Coverage ====================

#[tokio::test]
async fn test_attribution_chain_success() {
    let state = test_state();
    create_braid_for_attribution(&state, "sha256:attrchaintest").await;
    let result = dispatch(
        &state,
        "attribution.chain",
        serde_json::json!({"hash": "sha256:attrchaintest"}),
    )
    .await;
    assert!(result.is_ok());
    let v = result.unwrap();
    assert!(v["contributors"].is_array());
}

#[tokio::test]
async fn test_attribution_calculate_rewards_success() {
    let state = test_state();
    create_braid_for_attribution(&state, "sha256:rewardtest").await;
    let result = dispatch(
        &state,
        "attribution.calculate_rewards",
        serde_json::json!({"hash": "sha256:rewardtest", "value": 100.0}),
    )
    .await;
    assert!(result.is_ok());
    let v = result.unwrap();
    assert!(v.is_array());
    if let Some(first) = v.as_array().and_then(|a| a.first()) {
        assert!(first["agent"].is_string());
        assert!(first["share"].is_number());
        assert!(first["amount"].is_number());
        assert!(first["role"].is_string());
    }
}

#[tokio::test]
async fn test_attribution_calculate_rewards_zero_value() {
    let state = test_state();
    create_braid_for_attribution(&state, "sha256:zeroval").await;
    let result = dispatch(
        &state,
        "attribution.calculate_rewards",
        serde_json::json!({"hash": "sha256:zeroval", "value": 0.0}),
    )
    .await
    .unwrap();
    if let Some(rewards) = result.as_array() {
        for r in rewards {
            assert!(r["amount"].as_f64().unwrap().abs() < f64::EPSILON);
        }
    }
}

#[tokio::test]
async fn test_attribution_top_contributors_success() {
    let state = test_state();
    create_braid_for_attribution(&state, "sha256:toptest").await;
    let result = dispatch(
        &state,
        "attribution.top_contributors",
        serde_json::json!({"hash": "sha256:toptest"}),
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_attribution_top_contributors_with_limit() {
    let state = test_state();
    create_braid_for_attribution(&state, "sha256:toplimit").await;
    let result = dispatch(
        &state,
        "attribution.top_contributors",
        serde_json::json!({"hash": "sha256:toplimit", "limit": 1}),
    )
    .await
    .unwrap();
    let arr = result.as_array().unwrap();
    assert!(arr.len() <= 1);
}

#[tokio::test]
async fn test_attribution_chain_invalid_params() {
    let state = test_state();
    let result = dispatch(
        &state,
        "attribution.chain",
        serde_json::json!({"wrong_field": "value"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}
