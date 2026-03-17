// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for the `contribution.*` and `pipeline.*` JSON-RPC handlers.

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::test_fixtures::TEST_SOURCE_PRIMAL;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

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

// ==================== pipeline domain ====================

#[tokio::test]
async fn test_pipeline_attribute_creates_braids() {
    let state = test_state();
    let params = serde_json::json!({
        "session_id": "sess-pipeline-001",
        "agent_did": "did:key:z6MkPipelineAgent",
        "agent_summaries": [
            {"agent_did": "did:key:z6MkContributor1", "description": "primary", "weight": 0.7},
            {"agent_did": "did:key:z6MkContributor2", "description": "reviewer", "weight": 0.3}
        ]
    });

    let result = dispatch(&state, "pipeline.attribute", params).await;
    assert!(result.is_ok(), "pipeline.attribute should succeed");
    let val = result.unwrap();
    assert!(val["braid_ref"].is_string(), "should have a braid_ref");
}

#[tokio::test]
async fn test_pipeline_attribute_empty_summaries() {
    let state = test_state();
    let params = serde_json::json!({
        "session_id": "sess-empty-001",
        "agent_did": "did:key:z6MkEmptyAgent",
        "agent_summaries": []
    });

    let result = dispatch(&state, "pipeline.attribute", params).await;
    assert!(
        result.is_ok(),
        "pipeline.attribute with empty summaries should succeed"
    );
    let val = result.unwrap();
    assert!(
        val["braid_ref"].is_null(),
        "no braid_ref when no contributions"
    );
}

// ==================== Extended Coverage ====================

#[tokio::test]
async fn test_contribution_record_dehydration_with_operations() {
    let state = test_state();
    let result = dispatch(
        &state,
        "contribution.record_dehydration",
        serde_json::json!({
            "session_id": "session-001",
            "merkle_root": "sha256:merkle001",
            "source_primal": TEST_SOURCE_PRIMAL,
            "agents": ["did:key:z6MkAgent1"],
            "vertex_count": 10,
            "branch_count": 2,
            "operations": [
                {
                    "op_type": "create",
                    "content_hash": "sha256:op1hash",
                    "agent": "did:key:z6MkAgent1"
                }
            ]
        }),
    )
    .await;
    assert!(result.is_ok(), "dehydration with ops: {result:?}");
    let v = result.unwrap();
    assert_eq!(v["session_id"], "session-001");
    assert!(v["braids_created"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_contribution_record_dehydration_empty_operations() {
    let state = test_state();
    let result = dispatch(
        &state,
        "contribution.record_dehydration",
        serde_json::json!({
            "session_id": "session-empty",
            "merkle_root": "sha256:merkle_empty",
            "source_primal": TEST_SOURCE_PRIMAL,
            "agents": ["did:key:z6MkAgent1"],
            "vertex_count": 0,
            "branch_count": 0,
            "operations": []
        }),
    )
    .await;
    assert!(result.is_ok(), "dehydration empty ops: {result:?}");
    let v = result.unwrap();
    assert_eq!(v["braids_created"], 1);
}

#[tokio::test]
async fn test_contribution_pipeline_attribute() {
    let state = test_state();
    let result = dispatch(
        &state,
        "pipeline.attribute",
        serde_json::json!({
            "session_id": "pipeline-session",
            "agent_did": "did:key:z6MkCommitter",
            "agent_summaries": [
                {"agent_did": "did:key:z6MkPipeline1", "description": "tester", "weight": 1.0}
            ]
        }),
    )
    .await;
    assert!(result.is_ok(), "pipeline.attribute: {result:?}");
    let v = result.unwrap();
    assert!(v["braid_ref"].is_string());
}

#[tokio::test]
async fn test_contribution_pipeline_multiple_agents() {
    let state = test_state();
    let result = dispatch(
        &state,
        "pipeline.attribute",
        serde_json::json!({
            "session_id": "multi-pipeline",
            "agent_did": "did:key:z6MkCommitter",
            "agent_summaries": [
                {"agent_did": "did:key:z6MkAgent1", "description": "agent 1"},
                {"agent_did": "did:key:z6MkAgent2", "description": "agent 2"}
            ]
        }),
    )
    .await;
    assert!(result.is_ok(), "pipeline multi-agent: {result:?}");
}
