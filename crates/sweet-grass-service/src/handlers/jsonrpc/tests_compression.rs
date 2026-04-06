// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the `compression.*` JSON-RPC handlers.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

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

// ==================== Extended Coverage ====================

#[tokio::test]
async fn test_compression_create_meta_braid() {
    let state = test_state();

    let id1 = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:metac1", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap()["@id"]
        .as_str()
        .unwrap()
        .to_string();

    let id2 = dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:metac2", "mime_type": "text/plain", "size": 2}),
    )
    .await
    .unwrap()["@id"]
        .as_str()
        .unwrap()
        .to_string();

    let result = dispatch(
        &state,
        "compression.create_meta_braid",
        serde_json::json!({
            "braid_ids": [id1, id2],
            "summary_type": {"Session": {"session_id": "test-session"}}
        }),
    )
    .await;
    assert!(result.is_ok());
    let meta = result.unwrap();
    assert!(meta["@id"].is_string());
}

#[tokio::test]
async fn test_compression_compress_session_invalid() {
    let state = test_state();
    let result = dispatch(
        &state,
        "compression.compress_session",
        serde_json::json!({"not_a_session": true}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}
