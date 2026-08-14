// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for rootPulse graph step handlers.
#![expect(clippy::unwrap_used, reason = "test module")]

use sweet_grass_core::agent::Did;

use crate::handlers::jsonrpc::dispatch;
use crate::state::AppState;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTestRootPulse"))
}

#[tokio::test]
async fn rootpulse_attribute_creates_braid() {
    let state = test_state();
    let result = dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:abc123",
            "cas_ref": "nestgate:cas:def456",
            "session_id": "session-42",
            "agent_did": "did:primal:sporeGate",
            "wave_id": "157k",
            "graph_name": "rootpulse_commit",
            "target_triple": "x86_64-unknown-linux-musl",
            "primal_name": "sweetgrass",
            "commit_sha": "cdabdad",
            "blake3_hash": "abc123def456"
        }),
    )
    .await
    .unwrap();

    let braid_ref = result.get("braid_ref").unwrap().as_str().unwrap();
    assert!(braid_ref.starts_with("urn:braid:"));

    let content_hash = result.get("content_hash").unwrap().as_str().unwrap();
    assert!(!content_hash.is_empty());
}

#[tokio::test]
async fn rootpulse_attribute_minimal_inputs() {
    let state = test_state();
    let result = dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:minimal",
            "cas_ref": "nestgate:cas:minimal"
        }),
    )
    .await
    .unwrap();

    let braid_ref = result.get("braid_ref").unwrap().as_str().unwrap();
    assert!(braid_ref.starts_with("urn:braid:"));
}

#[tokio::test]
async fn rootpulse_attribute_null_refs_graceful() {
    let state = test_state();
    let result = dispatch(&state, "rootpulse.attribute", serde_json::json!({}))
        .await
        .unwrap();

    let braid_ref = result.get("braid_ref").unwrap().as_str().unwrap();
    assert!(braid_ref.starts_with("urn:braid:"));
}

#[tokio::test]
async fn rootpulse_attribute_deterministic_hash() {
    let s1 = test_state();
    let s2 = test_state();
    let params = serde_json::json!({
        "ledger_ref": "loamspine:entry:deterministic",
        "cas_ref": "nestgate:cas:deterministic",
        "wave_id": "157k"
    });

    let r1 = dispatch(&s1, "rootpulse.attribute", params.clone())
        .await
        .unwrap();
    let r2 = dispatch(&s2, "rootpulse.attribute", params).await.unwrap();

    assert_eq!(
        r1.get("content_hash").unwrap(),
        r2.get("content_hash").unwrap(),
    );
}

#[tokio::test]
async fn rootpulse_query_empty_store() {
    let state = test_state();
    let result = dispatch(&state, "rootpulse.query", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 0);
    assert!(result.get("braids").unwrap().as_array().unwrap().is_empty());
}

#[tokio::test]
async fn rootpulse_query_finds_attributed_braids() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:query-test",
            "cas_ref": "nestgate:cas:query-test",
            "wave_id": "157k",
            "graph_name": "rootpulse_commit",
            "target_triple": "x86_64-unknown-linux-musl",
            "primal_name": "sweetgrass"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:query-test-2",
            "cas_ref": "nestgate:cas:query-test-2",
            "wave_id": "157k",
            "graph_name": "rootpulse_commit",
            "target_triple": "aarch64-unknown-linux-musl",
            "primal_name": "beardog"
        }),
    )
    .await
    .unwrap();

    let result = dispatch(&state, "rootpulse.query", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 2);
}

#[tokio::test]
async fn rootpulse_query_filters_by_wave() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "a",
            "wave_id": "157k",
            "graph_name": "rootpulse_commit"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "b",
            "wave_id": "156d",
            "graph_name": "rootpulse_commit"
        }),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "rootpulse.query",
        serde_json::json!({ "wave_id": "157k" }),
    )
    .await
    .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 1);
    let braids = result.get("braids").unwrap().as_array().unwrap();
    assert_eq!(braids[0].get("wave_id").unwrap().as_str().unwrap(), "157k");
}

#[tokio::test]
async fn rootpulse_query_filters_by_target() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "t1",
            "target_triple": "x86_64-unknown-linux-musl",
            "wave_id": "157k"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "t2",
            "target_triple": "aarch64-apple-darwin",
            "wave_id": "157k"
        }),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "rootpulse.query",
        serde_json::json!({ "target_triple": "aarch64-apple-darwin" }),
    )
    .await
    .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 1);
}

#[tokio::test]
async fn rootpulse_query_filters_by_primal() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "p1",
            "primal_name": "sweetgrass"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "p2",
            "primal_name": "beardog"
        }),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "rootpulse.query",
        serde_json::json!({ "primal_name": "beardog" }),
    )
    .await
    .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 1);
}

#[tokio::test]
async fn rootpulse_query_filters_by_graph() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "g1",
            "graph_name": "rootpulse_commit"
        }),
    )
    .await
    .unwrap();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "g2",
            "graph_name": "rootpulse_harvest"
        }),
    )
    .await
    .unwrap();

    let result = dispatch(
        &state,
        "rootpulse.query",
        serde_json::json!({ "graph_name": "rootpulse_harvest" }),
    )
    .await
    .unwrap();

    assert_eq!(result.get("total").unwrap().as_u64().unwrap(), 1);
}

#[tokio::test]
async fn rootpulse_query_respects_limit() {
    let state = test_state();

    for i in 0..5 {
        dispatch(
            &state,
            "rootpulse.attribute",
            serde_json::json!({
                "ledger_ref": format!("limit-{i}"),
                "wave_id": "157k"
            }),
        )
        .await
        .unwrap();
    }

    let result = dispatch(&state, "rootpulse.query", serde_json::json!({ "limit": 3 }))
        .await
        .unwrap();

    let braids = result.get("braids").unwrap().as_array().unwrap();
    assert!(braids.len() <= 3);
}

#[tokio::test]
async fn braid_attribute_alias_routes_to_rootpulse() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:alias-test",
            "cas_ref": "nestgate:cas:alias-test"
        }),
    )
    .await
    .unwrap();

    let braid_ref = result.get("braid_ref").unwrap().as_str().unwrap();
    assert!(braid_ref.starts_with("urn:braid:"));
}

#[tokio::test]
async fn rootpulse_attribute_metadata_preserved() {
    let state = test_state();

    dispatch(
        &state,
        "rootpulse.attribute",
        serde_json::json!({
            "ledger_ref": "loamspine:entry:meta-test",
            "cas_ref": "nestgate:cas:meta-test",
            "wave_id": "157k",
            "graph_name": "rootpulse_commit",
            "target_triple": "x86_64-unknown-linux-musl",
            "primal_name": "sweetgrass",
            "commit_sha": "cdabdad",
            "blake3_hash": "abc123"
        }),
    )
    .await
    .unwrap();

    let query_result = dispatch(
        &state,
        "rootpulse.query",
        serde_json::json!({ "wave_id": "157k" }),
    )
    .await
    .unwrap();

    let braids = query_result.get("braids").unwrap().as_array().unwrap();
    assert_eq!(braids.len(), 1);

    let braid = &braids[0];
    assert_eq!(braid.get("wave_id").unwrap().as_str().unwrap(), "157k");
    assert_eq!(
        braid.get("graph_name").unwrap().as_str().unwrap(),
        "rootpulse_commit"
    );
    assert_eq!(
        braid.get("target_triple").unwrap().as_str().unwrap(),
        "x86_64-unknown-linux-musl"
    );
    assert_eq!(
        braid.get("primal_name").unwrap().as_str().unwrap(),
        "sweetgrass"
    );
}
