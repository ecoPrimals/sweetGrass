// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![cfg(unix)]
//! E2E integration tests for the `LedgerClient` (sweetGrass → loamSpine).
//!
//! Spins up a mock loamSpine UDS that speaks newline-delimited JSON-RPC 2.0,
//! then exercises `braid.commit` forwarding and `anchoring.verify` ledger
//! proof through the full handler chain.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::or_fun_call,
    reason = "test file: expect/unwrap are standard in tests"
)]

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;
use sweet_grass_core::agent::Did;
use sweet_grass_service::ledger_client::LedgerClient;
use sweet_grass_service::{AppState, create_router};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Spin up a mock loamSpine UDS that speaks newline-delimited JSON-RPC 2.0.
///
/// Responds to:
/// - `braid.commit` → `{ spine_id, entry_hash, index, sealed }`
/// - `certificate.verify` → `{ valid, detail }`
fn start_mock_loamspine(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
    let path = socket_path.to_path_buf();
    tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&path).expect("bind mock loamspine");
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let request: serde_json::Value =
                        serde_json::from_str(&line).unwrap_or_default();
                    let method = request
                        .get("method")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    let id = request.get("id").cloned().unwrap_or(json!(0));

                    let response = match method {
                        "braid.commit" => {
                            json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "spine_id": "default",
                                    "entry_hash": "sha256:ledger_abc123",
                                    "index": 42,
                                    "sealed": true
                                },
                                "id": id
                            })
                        },
                        "certificate.verify" => {
                            let cert_id = request
                                .pointer("/params/certificate_id")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("");

                            if cert_id.is_empty() || cert_id == "invalid" {
                                json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "valid": false,
                                        "detail": "certificate not found in ledger"
                                    },
                                    "id": id
                                })
                            } else {
                                json!({
                                    "jsonrpc": "2.0",
                                    "result": {
                                        "valid": true,
                                        "detail": "sealed in ledger at index 42"
                                    },
                                    "id": id
                                })
                            }
                        },
                        _ => {
                            json!({
                                "jsonrpc": "2.0",
                                "error": {
                                    "code": -32601,
                                    "message": format!("method not found: {method}")
                                },
                                "id": id
                            })
                        },
                    };

                    let mut buf = serde_json::to_string(&response).unwrap();
                    buf.push('\n');
                    if writer.write_all(buf.as_bytes()).await.is_err() {
                        break;
                    }
                    if writer.flush().await.is_err() {
                        break;
                    }
                }
            });
        }
    })
}

fn test_server_with_ledger(ledger: LedgerClient) -> TestServer {
    let state = AppState::new_memory(Did::new("did:key:z6MkLedgerE2E")).with_ledger_client(ledger);
    let router = create_router(state);
    TestServer::new(router)
}

fn test_server_without_ledger() -> TestServer {
    let state = AppState::new_memory(Did::new("did:key:z6MkLocalOnly"));
    let router = create_router(state);
    TestServer::new(router)
}

fn jsonrpc(method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
}

// ==================== braid.commit with loamSpine ====================

#[tokio::test]
async fn e2e_braid_commit_forwards_to_loamspine() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);
    let server = test_server_with_ledger(client);

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.create",
            json!({
                "data_hash": "sha256:e2e_ledger_test_001",
                "mime_type": "application/json",
                "size": 512
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let create_body: serde_json::Value = create_resp.json();
    let braid_id = create_body["result"]["@id"]
        .as_str()
        .expect("should have @id");

    let commit_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("braid.commit", json!({ "braid_id": braid_id })))
        .await;
    commit_resp.assert_status_ok();
    let commit_body: serde_json::Value = commit_resp.json();
    let result = &commit_body["result"];

    assert_eq!(result["committed"], true);
    assert_eq!(result["ledger_commit"]["spine_id"], "default");
    assert_eq!(
        result["ledger_commit"]["entry_hash"],
        "sha256:ledger_abc123"
    );
    assert_eq!(result["ledger_commit"]["index"], 42);
    assert_eq!(result["ledger_commit"]["sealed"], true);
}

#[tokio::test]
async fn e2e_braid_commit_without_loamspine_is_local_only() {
    let server = test_server_without_ledger();

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.create",
            json!({
                "data_hash": "sha256:local_only_test",
                "mime_type": "text/plain",
                "size": 64
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let create_body: serde_json::Value = create_resp.json();
    let braid_id = create_body["result"]["@id"]
        .as_str()
        .expect("should have @id");

    let commit_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("braid.commit", json!({ "braid_id": braid_id })))
        .await;
    commit_resp.assert_status_ok();
    let commit_body: serde_json::Value = commit_resp.json();
    let result = &commit_body["result"];

    assert!(result.get("committed").is_none());
    assert!(result.get("ledger_commit").is_none());
    assert!(result["data_hash"].is_string());
}

// ==================== anchoring.verify with loamSpine ====================

#[tokio::test]
async fn e2e_anchoring_verify_with_ledger_proof() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);
    let server = test_server_with_ledger(client);

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.create",
            json!({
                "data_hash": "sha256:verify_test_001",
                "mime_type": "application/octet-stream",
                "size": 256
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let create_body: serde_json::Value = create_resp.json();
    let braid_id = create_body["result"]["@id"]
        .as_str()
        .expect("should have @id");

    let verify_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "anchoring.verify",
            json!({ "braid_id": braid_id }),
        ))
        .await;
    verify_resp.assert_status_ok();
    let verify_body: serde_json::Value = verify_resp.json();
    let result = &verify_body["result"];

    assert_eq!(result["ledger_verified"], true);
    assert_eq!(result["verification_status"], "ledger_verified");
    assert_eq!(result["ledger_detail"], "sealed in ledger at index 42");
}

#[tokio::test]
async fn e2e_anchoring_verify_without_loamspine_is_local_only() {
    let server = test_server_without_ledger();

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.create",
            json!({
                "data_hash": "sha256:local_verify_test",
                "mime_type": "text/plain",
                "size": 32
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let create_body: serde_json::Value = create_resp.json();
    let braid_id = create_body["result"]["@id"]
        .as_str()
        .expect("should have @id");

    let verify_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "anchoring.verify",
            json!({ "braid_id": braid_id }),
        ))
        .await;
    verify_resp.assert_status_ok();
    let verify_body: serde_json::Value = verify_resp.json();
    let result = &verify_body["result"];

    assert!(result.get("ledger_verified").is_none());
    assert_eq!(result["verification_status"], "unanchored");
}

// ==================== LedgerClient unit-level integration ====================

#[tokio::test]
async fn e2e_ledger_client_commit_braid() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);

    let payload = json!({
        "braid_id": "urn:braid:test-123",
        "data_hash": "sha256:testpayload",
        "spine_id": "default"
    });

    let resp = client.commit_braid(payload).await.unwrap();
    assert_eq!(resp.spine_id, "default");
    assert_eq!(resp.entry_hash, "sha256:ledger_abc123");
    assert_eq!(resp.index, 42);
    assert!(resp.sealed);
}

#[tokio::test]
async fn e2e_ledger_client_verify_certificate_valid() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);

    let resp = client.verify_certificate("cert-abc-123").await.unwrap();
    assert!(resp.valid);
    assert_eq!(resp.detail.unwrap(), "sealed in ledger at index 42");
}

#[tokio::test]
async fn e2e_ledger_client_verify_certificate_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);

    let resp = client.verify_certificate("invalid").await.unwrap();
    assert!(!resp.valid);
    assert_eq!(resp.detail.unwrap(), "certificate not found in ledger");
}

#[tokio::test]
async fn e2e_ledger_client_unknown_method_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);

    let params = json!({ "foo": "bar" });
    let result = client.commit_braid(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn e2e_ledger_client_connection_refused() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("nonexistent.sock");

    let client = LedgerClient::from_socket_path(&sock);

    let result = client.commit_braid(json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("unavailable") || err.to_string().contains("io:"));
}

// ==================== Graceful degradation under load ====================

#[tokio::test]
async fn e2e_multiple_commits_sequential() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);
    let server = test_server_with_ledger(client);

    for i in 0..5 {
        let hash = format!("sha256:sequential_test_{i:03}");
        let create_resp = server
            .post("/jsonrpc")
            .json(&jsonrpc(
                "braid.create",
                json!({
                    "data_hash": hash,
                    "mime_type": "text/plain",
                    "size": i * 100
                }),
            ))
            .await;
        create_resp.assert_status_ok();
        let body: serde_json::Value = create_resp.json();
        let braid_id = body["result"]["@id"].as_str().unwrap();

        let commit_resp = server
            .post("/jsonrpc")
            .json(&jsonrpc("braid.commit", json!({ "braid_id": braid_id })))
            .await;
        commit_resp.assert_status_ok();
        let commit_body: serde_json::Value = commit_resp.json();
        assert_eq!(commit_body["result"]["committed"], true);
    }
}

#[tokio::test]
async fn e2e_concurrent_commits() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);
    let server = Arc::new(test_server_with_ledger(client));

    let mut handles = Vec::new();
    for i in 0..3 {
        let srv = Arc::clone(&server);
        let hash = format!("sha256:concurrent_{i:03}");
        handles.push(tokio::spawn(async move {
            let create_resp = srv
                .post("/jsonrpc")
                .json(&jsonrpc(
                    "braid.create",
                    json!({ "data_hash": hash, "mime_type": "text/plain", "size": 100 }),
                ))
                .await;
            create_resp.assert_status_ok();
            let body: serde_json::Value = create_resp.json();
            let braid_id = body["result"]["@id"].as_str().unwrap().to_string();

            let commit_resp = srv
                .post("/jsonrpc")
                .json(&jsonrpc("braid.commit", json!({ "braid_id": braid_id })))
                .await;
            commit_resp.assert_status_ok();
            let commit_body: serde_json::Value = commit_resp.json();
            assert_eq!(commit_body["result"]["committed"], true);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}

// ==================== Batch Operations (G31) ====================

#[tokio::test]
async fn e2e_batch_create_multiple_braids() {
    let server = test_server_without_ledger();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.batch_create",
            json!({
                "braids": [
                    { "data_hash": "sha256:batch_001", "mime_type": "text/plain", "size": 100 },
                    { "data_hash": "sha256:batch_002", "mime_type": "application/json", "size": 200 },
                    { "data_hash": "sha256:batch_003", "mime_type": "application/octet-stream", "size": 300 },
                    { "data_hash": "sha256:batch_004", "mime_type": "text/csv", "size": 400 },
                    { "data_hash": "sha256:batch_005", "mime_type": "chemical/x-pdb", "size": 500 },
                ]
            }),
        ))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let result = &body["result"];

    assert_eq!(result["created"], 5);
    assert_eq!(result["total"], 5);
    assert_eq!(result["errors"], 0);
    let results = result["results"].as_array().unwrap();
    assert_eq!(results.len(), 5);
    for r in results {
        assert_eq!(r["status"], "created");
        assert!(r["id"].is_string());
    }
}

#[tokio::test]
async fn e2e_batch_create_empty() {
    let server = test_server_without_ledger();

    let resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("braid.batch_create", json!({ "braids": [] })))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["result"]["created"], 0);
}

#[tokio::test]
async fn e2e_batch_commit_with_loamspine() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine.sock");
    let _mock = start_mock_loamspine(&sock);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = LedgerClient::from_socket_path(&sock);
    let server = test_server_with_ledger(client);

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.batch_create",
            json!({
                "braids": [
                    { "data_hash": "sha256:commit_batch_a", "mime_type": "text/plain", "size": 10 },
                    { "data_hash": "sha256:commit_batch_b", "mime_type": "text/plain", "size": 20 },
                    { "data_hash": "sha256:commit_batch_c", "mime_type": "text/plain", "size": 30 },
                ]
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let created: serde_json::Value = create_resp.json();
    let ids: Vec<String> = created["result"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();

    let commit_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("braid.batch_commit", json!({ "braid_ids": ids })))
        .await;
    commit_resp.assert_status_ok();
    let commit_body: serde_json::Value = commit_resp.json();
    let result = &commit_body["result"];

    assert_eq!(result["committed"], 3);
    assert_eq!(result["total"], 3);
    let results = result["results"].as_array().unwrap();
    for r in results {
        assert_eq!(r["status"], "committed");
        assert!(r["ledger_commit"].is_object());
    }
}

#[tokio::test]
async fn e2e_batch_commit_without_loamspine() {
    let server = test_server_without_ledger();

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.batch_create",
            json!({
                "braids": [
                    { "data_hash": "sha256:local_batch_1", "mime_type": "text/plain", "size": 10 },
                    { "data_hash": "sha256:local_batch_2", "mime_type": "text/plain", "size": 20 },
                ]
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let created: serde_json::Value = create_resp.json();
    let ids: Vec<String> = created["result"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();

    let commit_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc("braid.batch_commit", json!({ "braid_ids": ids })))
        .await;
    commit_resp.assert_status_ok();
    let commit_body: serde_json::Value = commit_resp.json();
    let result = &commit_body["result"];

    assert_eq!(result["committed"], 0);
    assert_eq!(result["total"], 2);
    let results = result["results"].as_array().unwrap();
    for r in results {
        assert_eq!(r["status"], "local_only");
    }
}

#[tokio::test]
async fn e2e_batch_commit_mixed_found_and_missing() {
    let server = test_server_without_ledger();

    let create_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.batch_create",
            json!({
                "braids": [
                    { "data_hash": "sha256:mixed_batch_1", "mime_type": "text/plain", "size": 10 },
                ]
            }),
        ))
        .await;
    create_resp.assert_status_ok();
    let created: serde_json::Value = create_resp.json();
    let real_id = created["result"]["results"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let commit_resp = server
        .post("/jsonrpc")
        .json(&jsonrpc(
            "braid.batch_commit",
            json!({ "braid_ids": [real_id, "nonexistent_braid_id"] }),
        ))
        .await;
    commit_resp.assert_status_ok();
    let commit_body: serde_json::Value = commit_resp.json();
    let results = commit_body["result"]["results"].as_array().unwrap();

    assert_eq!(results[0]["status"], "local_only");
    assert_eq!(results[1]["status"], "not_found");
}
