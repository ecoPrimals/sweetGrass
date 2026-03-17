// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the `provenance.*` JSON-RPC handlers.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

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

// ==================== Extended Coverage ====================

#[tokio::test]
async fn test_provenance_graph_success() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:provgraphcov", "mime_type": "text/plain", "size": 1}),
    ).await.unwrap();

    let result = dispatch(
        &state,
        "provenance.graph",
        serde_json::json!({"entity": {"data_hash": "sha256:provgraphcov"}}),
    )
    .await;
    assert!(
        result.is_ok(),
        "provenance.graph should succeed: {result:?}"
    );
}

#[tokio::test]
async fn test_provenance_graph_with_depth() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:provdepth", "mime_type": "text/plain", "size": 1}),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "provenance.graph",
        serde_json::json!({"entity": {"data_hash": "sha256:provdepth"}, "depth": 3}),
    )
    .await;
    assert!(result.is_ok(), "provenance.graph with depth: {result:?}");
}

#[tokio::test]
async fn test_provenance_export_provo_success() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:provoexport", "mime_type": "text/plain", "size": 10}),
    ).await.unwrap();

    let result = dispatch(
        &state,
        "provenance.export_provo",
        serde_json::json!({"hash": "sha256:provoexport"}),
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_provenance_export_graph_provo_success() {
    let state = test_state();
    dispatch(
        &state,
        "braid.create",
        serde_json::json!({"data_hash": "sha256:graphexport", "mime_type": "text/plain", "size": 10}),
    ).await.unwrap();

    let result = dispatch(
        &state,
        "provenance.export_graph_provo",
        serde_json::json!({
            "entity": {"data_hash": "sha256:graphexport"},
            "depth": 5
        }),
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_provenance_export_provo_invalid_params() {
    let state = test_state();
    let result = dispatch(
        &state,
        "provenance.export_provo",
        serde_json::json!({"wrong": "params"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}

#[tokio::test]
async fn test_provenance_graph_invalid_params() {
    let state = test_state();
    let result = dispatch(
        &state,
        "provenance.graph",
        serde_json::json!({"wrong": "params"}),
    )
    .await;
    assert!(result.is_err());
    let (code, _) = result.unwrap_err();
    assert_eq!(code, error_code::INVALID_PARAMS);
}
