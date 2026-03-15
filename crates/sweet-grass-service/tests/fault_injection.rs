// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Fault injection tests — HTTP-level resilience.
//!
//! Tests that verify the system returns proper HTTP/JSON-RPC error codes when
//! the store fails, and that data integrity is preserved under faults.

#![expect(clippy::expect_used, reason = "test file: expect is standard in tests")]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
use sweet_grass_factory::BraidFactory;
use sweet_grass_service::{AppState, create_router};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult, Result, StoreError};

use axum_test::TestServer;
use serde_json::json;

/// Store wrapper that fails on specific operations for fault injection.
struct FailingStore {
    inner: sweet_grass_store::MemoryStore,
    fail_puts: AtomicBool,
    fail_gets: AtomicBool,
    fail_queries: AtomicBool,
}

impl FailingStore {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: sweet_grass_store::MemoryStore::new(),
            fail_puts: AtomicBool::new(false),
            fail_gets: AtomicBool::new(false),
            fail_queries: AtomicBool::new(false),
        })
    }

    fn set_fail_puts(&self, fail: bool) {
        self.fail_puts.store(fail, Ordering::SeqCst);
    }

    fn set_fail_gets(&self, fail: bool) {
        self.fail_gets.store(fail, Ordering::SeqCst);
    }

    fn set_fail_queries(&self, fail: bool) {
        self.fail_queries.store(fail, Ordering::SeqCst);
    }

    fn fault_error() -> StoreError {
        StoreError::Internal("injected fault".to_string())
    }
}

#[async_trait]
impl BraidStore for FailingStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        if self.fail_puts.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.put(braid).await
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        if self.fail_gets.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get(id).await
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        if self.fail_gets.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get_by_hash(hash).await
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        self.inner.delete(id).await
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        if self.fail_gets.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.exists(id).await
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        if self.fail_queries.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.query(filter, order).await
    }

    async fn count(&self, filter: &QueryFilter) -> Result<usize> {
        if self.fail_queries.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.count(filter).await
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        if self.fail_queries.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.by_agent(agent).await
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        if self.fail_queries.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.derived_from(hash).await
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        if self.fail_puts.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.put_activity(activity).await
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        if self.fail_gets.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get_activity(id).await
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        if self.fail_queries.load(Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.activities_for_braid(braid_id).await
    }
}

fn fault_injection_rest_body() -> serde_json::Value {
    json!({
        "data_hash": "sha256:faulttest",
        "mime_type": "application/json",
        "size": 256,
        "was_attributed_to": "did:key:z6MkFaultTest"
    })
}

fn fault_injection_test_server(store: Arc<FailingStore>) -> TestServer {
    let state = AppState::with_store(
        store as Arc<dyn BraidStore>,
        Did::new("did:key:z6MkFaultTest"),
    );
    let router = create_router(state);
    TestServer::new(router)
}

// --- Store Failure Injection (HTTP) ---

#[tokio::test]
async fn test_fault_store_put_failure() {
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
    let store = FailingStore::new();
    store.set_fail_queries(true);

    let server = fault_injection_test_server(store);

    let resp = server.get("/api/v1/braids").await;

    resp.assert_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_fault_recovery_after_transient_failure() {
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
    let store = FailingStore::new();
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
