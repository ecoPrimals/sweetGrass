// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for `convergence.check`, `convergence.batch_check`, and `braid.list`.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTestConvergence"))
}

async fn create_braid(state: &AppState, hash: &str) -> serde_json::Value {
    let params = serde_json::json!({
        "data_hash": hash,
        "mime_type": "application/octet-stream",
        "size": 1024
    });
    dispatch(state, "braid.create", params).await.unwrap()
}

// ==================== convergence.check ====================

#[tokio::test]
async fn convergence_check_primordial() {
    let state = test_state();
    let result = dispatch(
        &state,
        "convergence.check",
        serde_json::json!({ "data_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000" }),
    )
    .await
    .unwrap();

    assert_eq!(result["converged"], false);
    assert_eq!(result["stages"][0]["stage"], "cas");
    assert_eq!(result["stages"][0]["present"], false);
    assert!(result["braid_id"].is_null());
}

#[tokio::test]
async fn convergence_check_cas_only() {
    let state = test_state();
    let hash = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    create_braid(&state, hash).await;

    let result = dispatch(
        &state,
        "convergence.check",
        serde_json::json!({ "data_hash": hash }),
    )
    .await
    .unwrap();

    assert_eq!(result["converged"], false);
    assert_eq!(result["stages"][0]["present"], true); // cas
    assert_eq!(result["stages"][1]["present"], false); // dag
    assert_eq!(result["stages"][2]["present"], false); // spine
    assert_eq!(result["stages"][3]["present"], true); // braid
    assert_eq!(result["stages"][4]["present"], false); // signed
    assert!(result["braid_id"].is_string());
}

#[tokio::test]
async fn convergence_check_invalid_params() {
    let state = test_state();
    let result = dispatch(&state, "convergence.check", serde_json::json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::INVALID_PARAMS);
}

// ==================== convergence.batch_check ====================

#[tokio::test]
async fn convergence_batch_check_empty() {
    let state = test_state();
    let result = dispatch(
        &state,
        "convergence.batch_check",
        serde_json::json!({ "data_hashes": [] }),
    )
    .await
    .unwrap();

    assert_eq!(result["summary"]["total"], 0);
    assert_eq!(result["summary"]["converged"], 0);
    assert_eq!(result["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn convergence_batch_check_mixed() {
    let state = test_state();
    let hash1 = "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let hash2 = "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    create_braid(&state, hash1).await;

    let result = dispatch(
        &state,
        "convergence.batch_check",
        serde_json::json!({ "data_hashes": [hash1, hash2] }),
    )
    .await
    .unwrap();

    assert_eq!(result["summary"]["total"], 2);
    assert_eq!(result["summary"]["partial"], 1); // hash1 has CAS+braid
    assert_eq!(result["summary"]["primordial"], 1); // hash2 has nothing
    assert_eq!(result["items"][0]["depth"], 4); // braid present
    assert_eq!(result["items"][1]["depth"], 0); // primordial
}

#[tokio::test]
async fn convergence_batch_check_exceeds_limit() {
    let state = test_state();
    let hashes: Vec<String> = (0..1001)
        .map(|i| format!("blake3:{i:064x}"))
        .collect();
    let result = dispatch(
        &state,
        "convergence.batch_check",
        serde_json::json!({ "data_hashes": hashes }),
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::INVALID_PARAMS);
}

// ==================== braid.list ====================

#[tokio::test]
async fn braid_list_empty() {
    let state = test_state();
    let result = dispatch(&state, "braid.list", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result["total"], 0);
    assert_eq!(result["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn braid_list_returns_summaries() {
    let state = test_state();
    create_braid(
        &state,
        "blake3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    )
    .await;
    create_braid(
        &state,
        "blake3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    )
    .await;

    let result = dispatch(&state, "braid.list", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result["total"], 2);
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);

    let first = &items[0];
    assert!(first["id"].is_string());
    assert!(first["data_hash"].is_string());
    assert_eq!(first["mime_type"], "application/octet-stream");
    assert_eq!(first["size"], 1024);
    assert!(first["attributed_to"].is_string());
    assert!(first["created_at"].is_number());
    assert_eq!(first["anchored"], false);
    assert_eq!(first["signed"], false);
}

#[tokio::test]
async fn braid_list_with_filter() {
    let state = test_state();
    let hash = "blake3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    create_braid(&state, hash).await;
    create_braid(
        &state,
        "blake3:1111111111111111111111111111111111111111111111111111111111111111",
    )
    .await;

    let result = dispatch(
        &state,
        "braid.list",
        serde_json::json!({ "filter": { "data_hash": hash } }),
    )
    .await
    .unwrap();

    assert_eq!(result["total"], 1);
    assert_eq!(
        result["items"][0]["data_hash"].as_str().unwrap(),
        hash
    );
}

#[tokio::test]
async fn braid_list_limit() {
    let state = test_state();
    for i in 0..5u8 {
        let hash = format!("blake3:{:0>64}", format!("{i:x}").repeat(64));
        create_braid(&state, &hash[..71]).await;
    }

    let result = dispatch(
        &state,
        "braid.list",
        serde_json::json!({ "filter": { "limit": 2 } }),
    )
    .await
    .unwrap();

    assert_eq!(result["total"], 2);
}
