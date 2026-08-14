// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `braid.*` CRUD, commit, query, and verify dispatch tests.

use super::super::*;
use super::helpers::test_state;

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
async fn test_create_and_get_braid_with_privacy_metadata() {
    let state = test_state();

    let create_params = serde_json::json!({
        "data_hash": "sha256:privacy-test",
        "mime_type": "application/json",
        "size": 128,
        "privacy": { "visibility": "private" }
    });
    let result = dispatch(&state, "braid.create", create_params).await;
    assert!(result.is_ok());
    let braid = result.unwrap();
    assert_eq!(braid["metadata"]["privacy"]["visibility"], "private");
    let braid_id = braid["@id"].as_str().unwrap().to_string();

    let get_result = dispatch(
        &state,
        "braid.get",
        serde_json::json!({
            "id": braid_id,
            "_caller_did": "did:key:z6MkTest"
        }),
    )
    .await;
    assert!(get_result.is_ok());
    let fetched = get_result.unwrap();
    assert_eq!(fetched["metadata"]["privacy"]["visibility"], "private");
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
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_query_braids() {
    let state = test_state();
    let result = dispatch(&state, "braid.query", serde_json::json!({"filter": {}})).await;
    assert!(result.is_ok());
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
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::NOT_FOUND);
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
    let uuid_str = commit["uuid"].as_str().unwrap();
    assert!(
        uuid_str.len() == 36 && uuid_str.chars().filter(|c| *c == '-').count() == 4,
        "uuid field must be a valid UUID, got: {uuid_str}"
    );
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
    assert_eq!(result.unwrap_err().code, error_code::NOT_FOUND);
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

#[tokio::test]
async fn test_braid_verify_unsigned_braid() {
    let state = test_state();
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:verify-unsigned",
            "mime_type": "text/plain",
            "size": 64
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();

    assert_eq!(result["verified"], false);
    assert_eq!(result["braid_id"], braid_id);
    assert_eq!(result["data_hash"], "sha256:verify-unsigned");

    let checks = result["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 3);

    assert_eq!(checks[0]["check"], "content_integrity");
    assert_eq!(checks[0]["status"], "pass");
    assert!(
        checks[0]["signing_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );

    assert_eq!(checks[1]["check"], "signature");
    assert_eq!(checks[1]["status"], "unsigned");

    assert_eq!(checks[2]["check"], "ledger");
    assert_eq!(checks[2]["status"], "skipped");
}

#[tokio::test]
async fn test_braid_verify_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({"braid_id": "urn:braid:uuid:nonexistent"}),
    )
    .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::NOT_FOUND);
}

#[tokio::test]
async fn test_braid_verify_content_integrity_format() {
    let state = test_state();
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:content-integrity-check",
            "mime_type": "application/octet-stream",
            "size": 1024
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();

    let checks = result["checks"].as_array().unwrap();
    let integrity = &checks[0];
    assert_eq!(integrity["check"], "content_integrity");
    assert_eq!(integrity["status"], "pass");

    let signing_hash = integrity["signing_hash"].as_str().unwrap();
    assert!(
        signing_hash.starts_with("sha256:"),
        "signing hash must be sha256 prefixed"
    );
    assert!(
        signing_hash.len() > 10,
        "signing hash must have meaningful content"
    );
}

#[tokio::test]
async fn test_braid_verify_crypto_down_permissive() {
    let state = test_state();

    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:crypto-down-test",
            "mime_type": "text/plain",
            "size": 32
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await;

    assert!(
        result.is_ok(),
        "braid.verify must not error when crypto is unavailable"
    );
    let val = result.unwrap();
    let checks = val["checks"].as_array().unwrap();

    let sig_check = &checks[1];
    assert_eq!(sig_check["check"], "signature");
    assert_ne!(
        sig_check["status"].as_str().unwrap(),
        "fail",
        "unsigned braid should not 'fail' signature — it's 'unsigned'"
    );

    let ledger_check = &checks[2];
    assert_eq!(ledger_check["check"], "ledger");
    assert_eq!(
        ledger_check["status"], "skipped",
        "ledger check should be skipped when no ledger client"
    );
}

#[tokio::test]
async fn test_braid_verify_returns_attribution_metadata() {
    let state = test_state();
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:verify-metadata",
            "mime_type": "application/json",
            "size": 256
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({"braid_id": braid_id}),
    )
    .await
    .unwrap();

    assert!(
        result["attributed_to"]
            .as_str()
            .unwrap()
            .starts_with("did:")
    );
    assert!(result["generated_at_time"].as_u64().is_some());
    assert_eq!(result["data_hash"], "sha256:verify-metadata");
}
