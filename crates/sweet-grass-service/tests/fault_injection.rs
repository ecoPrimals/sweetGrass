// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Fault injection tests — HTTP-level resilience.
//!
//! Tests that verify the system returns proper HTTP/JSON-RPC error codes when
//! the store fails, and that data integrity is preserved under faults.

#![expect(clippy::expect_used, reason = "test file: expect is standard in tests")]

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::json;
use sweet_grass_core::agent::Did;
use sweet_grass_factory::BraidFactory;
use sweet_grass_service::{AppState, BraidBackend, FaultInjectionStore, create_router};
use sweet_grass_store::BraidStore;

fn fault_injection_rest_body() -> serde_json::Value {
    json!({
        "data_hash": "sha256:faulttest",
        "mime_type": "application/json",
        "size": 256,
        "was_attributed_to": "did:key:z6MkFaultTest"
    })
}

fn fault_injection_test_server(store: Arc<FaultInjectionStore>) -> TestServer {
    let state = AppState::with_store(
        Arc::new(BraidBackend::FaultInjection(store)),
        Did::new("did:key:z6MkFaultTest"),
    );
    let router = create_router(state);
    TestServer::new(router)
}

// --- Store Failure Injection (HTTP) ---

#[tokio::test]
async fn test_fault_store_put_failure() {
    let store = FaultInjectionStore::new();
    store.set_fail_puts(true);

    let server = fault_injection_test_server(store);

    let resp = server
        .post("/api/v1/braids")
        .json(&fault_injection_rest_body())
        .await;

    resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_fault_store_get_failure() {
    let store = FaultInjectionStore::new();
    let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkFaultTest")));
    let braid = factory
        .from_data(b"get fault test", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("seed");
    let braid_id = braid.id.to_string();

    store.set_fail_gets(true);

    let server = fault_injection_test_server(store);

    let resp = server.get(&format!("/api/v1/braids/{braid_id}")).await;

    resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_fault_store_query_failure() {
    let store = FaultInjectionStore::new();
    store.set_fail_queries(true);

    let server = fault_injection_test_server(store);

    let resp = server.get("/api/v1/braids").await;

    resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_fault_recovery_after_transient_failure() {
    let store = FaultInjectionStore::new();
    let server = fault_injection_test_server(Arc::clone(&store));

    store.set_fail_puts(true);
    let fail_resp = server
        .post("/api/v1/braids")
        .json(&fault_injection_rest_body())
        .await;
    fail_resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

    store.set_fail_puts(false);
    let ok_resp = server
        .post("/api/v1/braids")
        .json(&json!({
            "data_hash": "sha256:recovered",
            "mime_type": "text/plain",
            "size": 10,
            "was_attributed_to": "did:key:z6MkFaultTest"
        }))
        .await;
    ok_resp.assert_status(axum::http::StatusCode::CREATED);

    let body: serde_json::Value = ok_resp.json();
    let braid_id = body["id"].as_str().expect("id");
    let get_resp = server.get(&format!("/api/v1/braids/{braid_id}")).await;
    get_resp.assert_status_ok();
}

#[tokio::test]
async fn test_fault_partial_batch_failure() {
    let store = FaultInjectionStore::new();
    let server = fault_injection_test_server(Arc::clone(&store));

    let body1 = json!({
        "data_hash": "sha256:batch1",
        "mime_type": "text/plain",
        "size": 6,
        "was_attributed_to": "did:key:z6MkFaultTest"
    });
    let body2 = json!({
        "data_hash": "sha256:batch2",
        "mime_type": "text/plain",
        "size": 6,
        "was_attributed_to": "did:key:z6MkFaultTest"
    });
    let body3 = json!({
        "data_hash": "sha256:batch3",
        "mime_type": "text/plain",
        "size": 6,
        "was_attributed_to": "did:key:z6MkFaultTest"
    });

    let resp1 = server.post("/api/v1/braids").json(&body1).await;
    resp1.assert_status(axum::http::StatusCode::CREATED);

    let resp2 = server.post("/api/v1/braids").json(&body2).await;
    resp2.assert_status(axum::http::StatusCode::CREATED);

    store.set_fail_puts(true);

    let resp3 = server.post("/api/v1/braids").json(&body3).await;
    resp3.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

    let id1: String = resp1.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("id1")
        .to_string();
    let id2: String = resp2.json::<serde_json::Value>()["id"]
        .as_str()
        .expect("id2")
        .to_string();

    store.set_fail_puts(false);

    let get1 = server.get(&format!("/api/v1/braids/{id1}")).await;
    get1.assert_status_ok();
    let get2 = server.get(&format!("/api/v1/braids/{id2}")).await;
    get2.assert_status_ok();
}

// --- Load and Malformed Request Tests ---

#[tokio::test]
async fn test_graceful_degradation_under_load() {
    let store = FaultInjectionStore::new();
    let server = fault_injection_test_server(store);

    for i in 0..100 {
        let resp = server
            .get(&format!("/api/v1/braids?limit={}", 1 + (i % 5)))
            .await;
        let status = resp.status_code();
        assert!(
            status.is_success() || status.is_redirection() || status.is_client_error(),
            "Request {i} returned unexpected status: {status}"
        );
    }
}

#[tokio::test]
async fn test_malformed_request_resilience() {
    let store = FaultInjectionStore::new();
    let server = fault_injection_test_server(store);

    let parse_error_resp = server.post("/jsonrpc").json(&json!(123)).await;
    parse_error_resp.assert_status_ok();
    let parse_body: serde_json::Value = parse_error_resp.json();
    assert_eq!(parse_body["error"]["code"], -32700);

    let invalid_params_create = server
        .post("/jsonrpc")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "braid.create",
            "params": {"wrong": "params"},
            "id": 1
        }))
        .await;
    invalid_params_create.assert_status_ok();
    let create_body: serde_json::Value = invalid_params_create.json();
    assert_eq!(create_body["error"]["code"], -32602);

    let invalid_params_get = server
        .post("/jsonrpc")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "braid.get",
            "params": {},
            "id": 2
        }))
        .await;
    invalid_params_get.assert_status_ok();
    let get_body: serde_json::Value = invalid_params_get.json();
    assert_eq!(get_body["error"]["code"], -32602);
}

// --- Data Integrity Under Fault ---

#[tokio::test]
async fn test_data_integrity_after_failed_write() {
    let store = FaultInjectionStore::new();
    let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkFaultTest")));

    let braid1 = factory
        .from_data(b"integrity test data", "text/plain", None)
        .expect("create");
    store.put(&braid1).await.expect("put braid1");

    store.set_fail_puts(true);
    let braid2 = factory
        .from_data(b"should fail", "text/plain", None)
        .expect("create");
    let put_result = store.put(&braid2).await;
    assert!(put_result.is_err());

    let retrieved = store.get(&braid1.id).await.expect("get").expect("exists");
    assert_eq!(retrieved.data_hash, braid1.data_hash);
}

#[tokio::test]
async fn test_concurrent_reads_during_write_failure() {
    let store = FaultInjectionStore::new();
    let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkFaultTest")));

    let braid = factory
        .from_data(b"concurrent read test", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("put");
    let id = braid.id.clone();

    store.set_fail_puts(true);

    let mut read_handles = Vec::new();
    for _ in 0..20 {
        let store = Arc::clone(&store);
        let id = id.clone();
        read_handles.push(tokio::spawn(async move {
            store.get(&id).await.map(|o| o.is_some())
        }));
    }

    let mut read_successes = 0;
    for handle in read_handles {
        if matches!(handle.await, Ok(Ok(true))) {
            read_successes += 1;
        }
    }
    assert_eq!(
        read_successes, 20,
        "Reads should succeed even when writes fail"
    );
}
