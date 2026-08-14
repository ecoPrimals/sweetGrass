// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `contribution.*` dispatch tests.

use super::super::*;
use super::helpers::test_state;
use sweet_grass_core::test_fixtures::TEST_SOURCE_PRIMAL;

#[tokio::test]
async fn test_record_contribution_dispatch() {
    let state = test_state();
    let params = serde_json::json!({
        "agent": "did:key:z6MkContributor",
        "role": "Creator",
        "content_hash": "sha256:rpc-contrib-test",
        "mime_type": "application/json",
        "size": 64
    });

    let result = dispatch(&state, "contribution.record", params).await;
    assert!(result.is_ok());
    let braid = result.unwrap();
    assert_eq!(braid["data_hash"], "sha256:rpc-contrib-test");
    assert!(braid["@id"].as_str().unwrap().starts_with("urn:braid:"));
}

#[tokio::test]
async fn test_record_session_dispatch() {
    let state = test_state();
    let params = serde_json::json!({
        "session_id": "rpc-session-123",
        "source_primal": TEST_SOURCE_PRIMAL,
        "contributions": [
            {
                "agent": "did:key:z6MkAgent1",
                "role": "Creator",
                "content_hash": "sha256:session-hash-1",
                "mime_type": "text/plain",
                "size": 10
            },
            {
                "agent": "did:key:z6MkAgent2",
                "role": "Contributor",
                "content_hash": "sha256:session-hash-2",
                "mime_type": "application/json",
                "size": 20
            }
        ]
    });

    let result = dispatch(&state, "contribution.record_session", params).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["session_id"], "rpc-session-123");
    assert_eq!(response["braids_created"], 2);
    let braid_ids = response["braid_ids"].as_array().unwrap();
    assert_eq!(braid_ids.len(), 2);
}
