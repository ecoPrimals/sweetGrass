// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Privacy visibility edge-case tests for `braid.get` access control.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

async fn create_braid_with_privacy(
    state: &AppState,
    data_hash: &str,
    privacy: Option<serde_json::Value>,
) -> String {
    let mut params = serde_json::json!({
        "data_hash": data_hash,
        "mime_type": "application/json",
        "size": 128
    });
    if let Some(pm) = privacy {
        params["privacy"] = pm;
    }

    let result = dispatch(state, "braid.create", params).await;
    assert!(result.is_ok(), "braid.create failed: {:?}", result.err());
    result.unwrap()["@id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn test_authenticated_braid_denied_without_token() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:auth-denied",
        Some(serde_json::json!({ "visibility": "authenticated" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_code::PERMISSION_DENIED);
}

#[tokio::test]
async fn test_authenticated_braid_allowed_with_token() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:auth-allowed",
        Some(serde_json::json!({ "visibility": "authenticated" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({
            "id": braid_id,
            "_bearer_token": "any-token-string"
        }),
    )
    .await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched["@id"], braid_id);
    assert_eq!(
        fetched["metadata"]["privacy"]["visibility"],
        "authenticated"
    );
}

#[tokio::test]
async fn test_private_braid_denied_wrong_did() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:private-denied",
        Some(serde_json::json!({ "visibility": "private" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({
            "id": braid_id,
            "_caller_did": "did:key:z6MkWrongDid"
        }),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_code::PERMISSION_DENIED);
}

#[tokio::test]
async fn test_private_braid_allowed_owner_did() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:private-owner",
        Some(serde_json::json!({ "visibility": "private" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({
            "id": braid_id,
            "_caller_did": "did:key:z6MkTest"
        }),
    )
    .await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched["@id"], braid_id);
    assert_eq!(fetched["metadata"]["privacy"]["visibility"], "private");
}

#[tokio::test]
async fn test_encrypted_braid_denied_no_did() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:encrypted-denied",
        Some(serde_json::json!({ "visibility": "encrypted" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_code::PERMISSION_DENIED);
}

#[tokio::test]
async fn test_encrypted_braid_allowed_owner() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:encrypted-owner",
        Some(serde_json::json!({ "visibility": "encrypted" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({
            "id": braid_id,
            "_caller_did": "did:key:z6MkTest"
        }),
    )
    .await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched["@id"], braid_id);
    assert_eq!(fetched["metadata"]["privacy"]["visibility"], "encrypted");
}

#[tokio::test]
async fn test_public_braid_always_accessible() {
    let state = test_state();
    let braid_id = create_braid_with_privacy(
        &state,
        "sha256:public-access",
        Some(serde_json::json!({ "visibility": "public" })),
    )
    .await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched["@id"], braid_id);
    assert_eq!(fetched["metadata"]["privacy"]["visibility"], "public");
}

#[tokio::test]
async fn test_no_privacy_metadata_always_accessible() {
    let state = test_state();
    let braid_id =
        create_braid_with_privacy(&state, "sha256:no-privacy", None).await;

    let result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({ "id": braid_id }),
    )
    .await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert_eq!(fetched["@id"], braid_id);
    assert!(fetched["metadata"]["privacy"].is_null());
}
