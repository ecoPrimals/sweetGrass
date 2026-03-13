// SPDX-License-Identifier: AGPL-3.0-only
//! HTTP-level E2E tests exercising REST and JSON-RPC 2.0 endpoints.
//!
//! These tests start the actual Axum router (without a network socket)
//! and verify the full request→handler→store→response pipeline.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value
)]

use axum_test::TestServer;
use serde_json::json;
use sweet_grass_core::agent::Did;
use sweet_grass_service::{create_router, AppState};

fn test_server() -> TestServer {
    let state = AppState::new_memory(Did::new("did:key:z6MkE2ETest"));
    let router = create_router(state);
    TestServer::new(router)
}

fn rest_create_body() -> serde_json::Value {
    json!({
        "data_hash": "sha256:e2etest123",
        "mime_type": "application/json",
        "size": 1024,
        "was_attributed_to": "did:key:z6MkE2ETest"
    })
}

// ==================== Health Endpoints ====================

#[tokio::test]
async fn e2e_health_check() {
    let server = test_server();
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn e2e_liveness() {
    let server = test_server();
    server.get("/live").await.assert_status_ok();
}

#[tokio::test]
async fn e2e_readiness() {
    let server = test_server();
    server.get("/ready").await.assert_status_ok();
}

// ==================== REST API ====================

#[tokio::test]
async fn e2e_rest_create_and_get_braid() {
    let server = test_server();

    let create_resp = server
        .post("/api/v1/braids")
        .json(&rest_create_body())
        .await;
    create_resp.assert_status(axum::http::StatusCode::CREATED);
    let braid: serde_json::Value = create_resp.json();
    let braid_id = braid["id"].as_str().expect("should have id");

    let get_resp = server.get(&format!("/api/v1/braids/{braid_id}")).await;
    get_resp.assert_status_ok();
    let fetched: serde_json::Value = get_resp.json();
    assert_eq!(fetched["data_hash"], "sha256:e2etest123");
}

#[tokio::test]
async fn e2e_rest_list_braids() {
    let server = test_server();
    let resp = server.get("/api/v1/braids").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["braids"].is_array());
}

#[tokio::test]
async fn e2e_rest_get_braid_not_found() {
    let server = test_server();
    let resp = server.get("/api/v1/braids/nonexistent-id").await;
    resp.assert_status_not_found();
}

#[tokio::test]
async fn e2e_rest_delete_braid() {
    let server = test_server();

    let create_resp = server
        .post("/api/v1/braids")
        .json(&rest_create_body())
        .await;
    let braid: serde_json::Value = create_resp.json();
    let braid_id = braid["id"].as_str().unwrap();

    let delete_resp = server.delete(&format!("/api/v1/braids/{braid_id}")).await;
    delete_resp.assert_status(axum::http::StatusCode::NO_CONTENT);

    let get_resp = server.get(&format!("/api/v1/braids/{braid_id}")).await;
    get_resp.assert_status_not_found();
}

#[tokio::test]
async fn e2e_rest_get_by_hash() {
    let server = test_server();

    server
        .post("/api/v1/braids")
        .json(&json!({
            "data_hash": "sha256:hashtest999",
            "mime_type": "text/plain",
            "size": 5,
            "was_attributed_to": "did:key:z6MkE2ETest"
        }))
        .await;

    let resp = server.get("/api/v1/braids/hash/sha256:hashtest999").await;
    resp.assert_status_ok();
}

// ==================== JSON-RPC 2.0 ====================

fn jsonrpc(method: &str, params: serde_json::Value, id: u64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id
    })
}

#[tokio::test]
async fn e2e_jsonrpc_health() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("sweetgrass.health", json!({}), 1))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"].is_object());
    assert_eq!(body["result"]["status"], "healthy");
    assert_eq!(body["id"], 1);
    assert!(body["error"].is_null());
}

#[tokio::test]
async fn e2e_jsonrpc_create_and_get_braid() {
    let server = test_server();

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.createBraid",
            json!({
                "data_hash": "sha256:jsonrpctest",
                "mime_type": "application/json",
                "size": 256
            }),
            1,
        ))
        .await;

    create_resp.assert_status_ok();
    let create_body: serde_json::Value = create_resp.json();
    assert!(
        create_body["error"].is_null(),
        "Expected no error: {create_body}"
    );
    let braid_id = create_body["result"]["@id"].as_str().expect("braid @id");

    let get_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("sweetgrass.getBraid", json!({"id": braid_id}), 2))
        .await;

    get_resp.assert_status_ok();
    let get_body: serde_json::Value = get_resp.json();
    assert!(get_body["error"].is_null());
    assert_eq!(get_body["result"]["@id"], braid_id);
    assert_eq!(get_body["id"], 2);
}

#[tokio::test]
async fn e2e_jsonrpc_method_not_found() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("nonexistent.method", json!({}), 99))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["result"].is_null());
    assert!(body["error"].is_object());
    assert_eq!(body["error"]["code"], -32601);
    assert_eq!(body["id"], 99);
}

#[tokio::test]
async fn e2e_jsonrpc_invalid_params() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.createBraid",
            json!({"wrong": "params"}),
            5,
        ))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"], -32602);
}

#[tokio::test]
async fn e2e_jsonrpc_invalid_version() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&json!({
            "jsonrpc": "1.0",
            "method": "sweetgrass.health",
            "params": {},
            "id": 1
        }))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"], -32600);
}

#[tokio::test]
async fn e2e_jsonrpc_query_braids() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.queryBraids",
            json!({"filter": {}}),
            10,
        ))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["error"].is_null());
    assert!(body["result"]["braids"].is_array());
}

#[tokio::test]
async fn e2e_jsonrpc_delete_braid() {
    let server = test_server();

    let create_body: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.createBraid",
            json!({
                "data_hash": "sha256:jsonrpcdelete",
                "mime_type": "text/plain",
                "size": 5
            }),
            1,
        ))
        .await
        .json();
    let braid_id = create_body["result"]["@id"].as_str().unwrap();

    let delete_body: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.deleteBraid",
            json!({"id": braid_id}),
            2,
        ))
        .await
        .json();
    assert!(delete_body["error"].is_null());

    let get_body: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc("sweetgrass.getBraid", json!({"id": braid_id}), 3))
        .await
        .json();
    assert_eq!(get_body["error"]["code"], -32001);
}

// ==================== Contribution Recording ====================

#[tokio::test]
async fn e2e_jsonrpc_record_contribution() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.recordContribution",
            json!({
                "agent": "did:key:z6MkContributor1",
                "role": "Creator",
                "content_hash": "sha256:contrib001",
                "mime_type": "application/json",
                "size": 2048,
                "description": "Initial data creation",
                "source_primal": "rhizoCrypt",
                "session_id": "session-42",
                "domain": {
                    "chemistry.molecule": "caffeine",
                    "chemistry.basis_set": "6-31G*"
                }
            }),
            20,
        ))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["error"].is_null(), "Expected no error: {body}");
    let braid = &body["result"];
    assert!(braid["@id"].is_string());
    assert_eq!(braid["data_hash"], "sha256:contrib001");
}

#[tokio::test]
async fn e2e_jsonrpc_record_session() {
    let server = test_server();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.recordSession",
            json!({
                "session_id": "rhizo-session-99",
                "source_primal": "rhizoCrypt",
                "niche": "rootpulse",
                "session_start": 1_000_000,
                "session_end": 2_000_000,
                "contributions": [
                    {
                        "agent": "did:key:z6MkAlice",
                        "role": "Creator",
                        "content_hash": "sha256:alice-change",
                        "size": 512
                    },
                    {
                        "agent": "did:key:z6MkBob",
                        "role": "Contributor",
                        "content_hash": "sha256:bob-review",
                        "size": 128
                    }
                ]
            }),
            30,
        ))
        .await;

    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["error"].is_null(), "Expected no error: {body}");
    let result = &body["result"];
    assert_eq!(result["session_id"], "rhizo-session-99");
    assert_eq!(result["braids_created"], 2);
    assert_eq!(result["braid_ids"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn e2e_jsonrpc_record_contribution_then_query() {
    let server = test_server();

    // Record a contribution
    let create_body: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.recordContribution",
            json!({
                "agent": "did:key:z6MkQueryTest",
                "role": "Creator",
                "content_hash": "sha256:queryable",
                "size": 64
            }),
            1,
        ))
        .await
        .json();
    assert!(create_body["error"].is_null());

    // Verify it's queryable via getBraidByHash
    let query_body: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "sweetgrass.getBraidByHash",
            json!({"hash": "sha256:queryable"}),
            2,
        ))
        .await
        .json();
    assert!(query_body["error"].is_null());
    assert_eq!(query_body["result"]["data_hash"], "sha256:queryable");
}

// ==================== Cross-Protocol Consistency ====================

#[tokio::test]
async fn e2e_rest_and_jsonrpc_share_state() {
    let server = test_server();

    // Create via REST
    let rest_create: serde_json::Value = server
        .post("/api/v1/braids")
        .json(&json!({
            "data_hash": "sha256:crossprotocol",
            "mime_type": "text/plain",
            "size": 42,
            "was_attributed_to": "did:key:z6MkE2ETest"
        }))
        .await
        .json();
    let braid_id = rest_create["id"].as_str().unwrap();

    // Read via JSON-RPC (uses the same store)
    let jsonrpc_get: serde_json::Value = server
        .post("/jsonrpc")
        .json(&jsonrpc("sweetgrass.getBraid", json!({"id": braid_id}), 1))
        .await
        .json();
    assert!(jsonrpc_get["error"].is_null());
    assert_eq!(jsonrpc_get["result"]["data_hash"], "sha256:crossprotocol");
}
