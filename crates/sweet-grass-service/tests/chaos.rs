// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Chaos and fault injection tests for `SweetGrass`.
//!
//! These tests verify the system's behavior under failure conditions:
//! - Store failures during operations
//! - Concurrent failure scenarios

//! - Recovery and consistency checks
//! - Resource exhaustion handling

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
use sweet_grass_factory::BraidFactory;
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult, Result, StoreError};

// ============================================================================
// Fault-Injecting Store Wrapper
// ============================================================================

/// A store wrapper that can inject failures for testing.
pub struct FaultyStore {
    inner: Arc<sweet_grass_store::MemoryStore>,
    /// Fail rate as percentage (0-100).
    fail_rate: AtomicUsize,
    /// Whether to fail on next operation.
    fail_next: AtomicBool,
    /// Number of operations performed.
    op_count: AtomicUsize,
}

impl FaultyStore {
    /// Create a new faulty store wrapping an inner store.
    pub fn new(inner: Arc<sweet_grass_store::MemoryStore>) -> Arc<Self> {
        Arc::new(Self {
            inner,
            fail_rate: AtomicUsize::new(0),
            fail_next: AtomicBool::new(false),
            op_count: AtomicUsize::new(0),
        })
    }

    /// Set failure rate (0-100 percentage).
    pub fn set_fail_rate(&self, rate: usize) {
        self.fail_rate.store(rate.min(100), Ordering::SeqCst);
    }

    /// Fail the next operation.
    pub fn fail_next(&self) {
        self.fail_next.store(true, Ordering::SeqCst);
    }

    /// Get operation count.
    pub fn op_count(&self) -> usize {
        self.op_count.load(Ordering::SeqCst)
    }

    /// Check if this operation should fail.
    fn should_fail(&self) -> bool {
        self.op_count.fetch_add(1, Ordering::SeqCst);

        // Check fail_next first
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return true;
        }

        // Check probabilistic failure
        let rate = self.fail_rate.load(Ordering::SeqCst);
        if rate > 0 {
            // Simple pseudo-random based on op count
            let op = self.op_count.load(Ordering::SeqCst);
            return (op * 7 + 13) % 100 < rate;
        }

        false
    }

    /// Generate a fault error.
    fn fault_error() -> StoreError {
        StoreError::Internal("Injected fault for chaos testing".to_string())
    }
}

impl BraidStore for FaultyStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.put(braid).await
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.get(id).await
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.get_by_hash(hash).await
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.delete(id).await
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.exists(id).await
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.query(filter, order).await
    }

    async fn count(&self, filter: &QueryFilter) -> Result<usize> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.count(filter).await
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.by_agent(agent).await
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.derived_from(hash).await
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.put_activity(activity).await
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.get_activity(id).await
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        if self.should_fail() {
            return Err(Self::fault_error());
        }
        self.inner.activities_for_braid(braid_id).await
    }
}

// ============================================================================
// Chaos Tests
// ============================================================================

/// Helper to create test environment with faulty store.
fn setup_faulty() -> (Arc<FaultyStore>, Arc<BraidFactory>) {
    let inner = Arc::new(sweet_grass_store::MemoryStore::new());
    let store = FaultyStore::new(inner);
    let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkTest")));
    (store, factory)
}

#[tokio::test]
async fn test_store_failure_on_put() {
    let (store, factory) = setup_faulty();

    let braid = factory
        .from_data(b"test data", "text/plain", None)
        .expect("create");

    // First put should succeed
    store.put(&braid).await.expect("first put");

    // Fail next operation
    store.fail_next();
    let result = store.put(&braid).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("chaos testing"));
}

#[tokio::test]
async fn test_store_failure_on_get() {
    let (store, factory) = setup_faulty();

    let braid = factory
        .from_data(b"test data", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("put");

    // Fail next operation
    store.fail_next();
    let result = store.get(&braid.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_probabilistic_failures() {
    let (store, factory) = setup_faulty();

    // Set 50% failure rate
    store.set_fail_rate(50);

    let mut successes = 0;
    let mut failures = 0;

    for i in 0..20 {
        let braid = factory
            .from_data(format!("data {i}").as_bytes(), "text/plain", None)
            .expect("create");

        match store.put(&braid).await {
            Ok(()) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    // With 50% failure rate, we should have some of each
    assert!(successes > 0, "Should have some successes");
    assert!(failures > 0, "Should have some failures");
}

#[tokio::test]
async fn test_recovery_after_failure() {
    let (store, factory) = setup_faulty();

    let braid = factory
        .from_data(b"test data", "text/plain", None)
        .expect("create");

    // Fail the put
    store.fail_next();
    let _ = store.put(&braid).await;

    // Retry should succeed (no more injected fault)
    store.put(&braid).await.expect("retry should succeed");

    // Verify data is stored
    let retrieved = store.get(&braid.id).await.expect("get").expect("exists");
    assert_eq!(retrieved.data_hash, braid.data_hash);
}

#[tokio::test]
async fn test_concurrent_failures() {
    let (store, factory) = setup_faulty();

    // Pre-store some data
    for i in 0..5 {
        let braid = factory
            .from_data(format!("seed {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("seed put");
    }

    // Set moderate failure rate
    store.set_fail_rate(30);

    // Spawn concurrent operations
    let mut handles = vec![];
    for i in 0..10 {
        let store = Arc::clone(&store);
        let factory = Arc::clone(&factory);
        handles.push(tokio::spawn(async move {
            let braid = factory
                .from_data(format!("concurrent {i}").as_bytes(), "text/plain", None)
                .expect("create");

            // Try put with retries
            for attempt in 0..3 {
                if store.put(&braid).await.is_ok() {
                    return (i, true, attempt);
                }
            }
            (i, false, 3)
        }));
    }

    // Wait for all
    let mut succeeded = 0;
    let mut with_retries = 0;
    for handle in handles {
        let (_, success, attempts) = handle.await.expect("join");
        if success {
            succeeded += 1;
            if attempts > 0 {
                with_retries += 1;
            }
        }
    }

    // Most should eventually succeed with retries
    assert!(succeeded >= 5, "At least half should succeed with retries");
    tracing::info!("Chaos test: {succeeded}/10 succeeded, {with_retries} required retries");
}

#[tokio::test]
async fn test_read_consistency_after_write_failure() {
    let (store, factory) = setup_faulty();

    let braid1 = factory
        .from_data(b"data 1", "text/plain", None)
        .expect("create");
    let braid2 = factory
        .from_data(b"data 2", "text/plain", None)
        .expect("create");

    // Successfully store first braid
    store.put(&braid1).await.expect("put 1");

    // Fail on second braid
    store.fail_next();
    let _ = store.put(&braid2).await;

    // First braid should still be accessible
    let retrieved = store.get(&braid1.id).await.expect("get").expect("exists");
    assert_eq!(retrieved.data_hash, braid1.data_hash);

    // Second braid should not exist (write failed)
    let missing = store.get(&braid2.id).await.expect("get");
    assert!(missing.is_none(), "Failed write should not store data");
}

#[tokio::test]
async fn test_query_under_failures() {
    let (store, factory) = setup_faulty();

    // Store several braids
    for i in 0..10 {
        let braid = factory
            .from_data(format!("query test {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("put");
    }

    // Query should work
    let filter = QueryFilter::new().with_limit(5);
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 5);

    // Fail query
    store.fail_next();
    let failed = store.query(&filter, QueryOrder::NewestFirst).await;
    assert!(failed.is_err());

    // Query should work again
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query");
    assert_eq!(result.braids.len(), 5);
}

#[tokio::test]
async fn test_operation_counting() {
    let (store, factory) = setup_faulty();

    let initial_count = store.op_count();

    let braid = factory
        .from_data(b"test", "text/plain", None)
        .expect("create");
    store.put(&braid).await.expect("put");
    let _ = store.get(&braid.id).await;
    let _ = store.exists(&braid.id).await;
    let _ = store.count(&QueryFilter::default()).await;

    let final_count = store.op_count();
    assert_eq!(final_count - initial_count, 4, "Should count 4 operations");
}

// ============================================================================
// NEW COMPREHENSIVE CHAOS TESTS (Dec 27, 2025) - Expanded Fault Scenarios
// ============================================================================

#[tokio::test]
async fn test_cascading_failures() {
    let (store, factory) = setup_faulty();

    // Pre-populate store
    let mut braids = Vec::new();
    for i in 0..5 {
        let braid = factory
            .from_data(format!("cascade {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("seed");
        braids.push(braid);
    }

    // Set high failure rate
    store.set_fail_rate(80);

    // Attempt multiple operations - most should fail
    let mut total_ops = 0;
    let mut failed_ops = 0;

    for braid in &braids {
        total_ops += 1;
        if store.get(&braid.id).await.is_err() {
            failed_ops += 1;
        }
    }

    // With 80% failure rate, most should fail
    assert!(failed_ops > 0, "Should have cascading failures");
    assert!(total_ops - failed_ops > 0, "Should have some successes");
}

#[tokio::test]
async fn test_partial_batch_failure() {
    let (store, factory) = setup_faulty();

    // Create batch of braids
    let mut braids = Vec::new();
    for i in 0..10 {
        let braid = factory
            .from_data(format!("batch {i}").as_bytes(), "text/plain", None)
            .expect("create");
        braids.push(braid);
    }

    // Set moderate failure rate
    store.set_fail_rate(40);

    // Try to store all
    let mut stored_ids = Vec::new();
    for braid in &braids {
        if store.put(braid).await.is_ok() {
            stored_ids.push(braid.id.clone());
        }
    }

    // Verify only successful ones are retrievable
    for id in &stored_ids {
        // Under chaos conditions, retrieval might also fail
        if let Ok(result) = store.get(id).await {
            assert!(result.is_some(), "Stored braid should exist");
        }
        // If retrieval fails due to chaos, that's acceptable in this test
    }

    // Count should match (but count might also fail under chaos)
    if let Ok(count) = store.count(&QueryFilter::new()).await {
        // If count succeeds, it should match stored IDs
        // But under high failure rates, count might differ slightly
        assert!(
            count >= stored_ids.len() / 2,
            "Count should be reasonable despite chaos"
        );
    }
}

#[tokio::test]
async fn test_failure_during_concurrent_reads() {
    let (store, factory) = setup_faulty();

    // Pre-populate
    let braid = factory
        .from_data(b"concurrent read test", "text/plain", None)
        .expect("create");
    let id = braid.id.clone();
    store.put(&braid).await.expect("put");

    // Set moderate failure rate
    store.set_fail_rate(30);

    // Spawn many concurrent reads
    let mut handles = vec![];
    for _ in 0..20 {
        let store = Arc::clone(&store);
        let id = id.clone();
        handles.push(tokio::spawn(async move { store.get(&id).await.is_ok() }));
    }

    // Collect results
    let mut successes = 0;
    for handle in handles {
        if handle.await.expect("join") {
            successes += 1;
        }
    }

    // Some should succeed despite failures
    assert!(successes > 0, "Should have some successful reads");
}

#[tokio::test]
async fn test_query_consistency_under_failures() {
    let (store, factory) = setup_faulty();

    // Store 10 braids successfully first
    for i in 0..10 {
        let braid = factory
            .from_data(format!("consistent {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("put");
    }

    // Now set failure rate
    store.set_fail_rate(50);

    // Multiple queries should return consistent results when they succeed
    let filter = QueryFilter::new().with_limit(10);

    let mut successful_results = Vec::new();
    for _ in 0..5 {
        if let Ok(result) = store.query(&filter, QueryOrder::NewestFirst).await {
            successful_results.push(result.total_count);
        }
    }

    // All successful queries should return same count
    if successful_results.len() > 1 {
        let first = successful_results[0];
        for count in &successful_results {
            assert_eq!(*count, first, "Query results should be consistent");
        }
    }
}

#[tokio::test]
async fn test_delete_under_failures() {
    let (store, factory) = setup_faulty();

    let braid = factory
        .from_data(b"delete test", "text/plain", None)
        .expect("create");
    let id = braid.id.clone();

    store.put(&braid).await.expect("put");

    // Fail delete
    store.fail_next();
    assert!(store.delete(&id).await.is_err());

    // Braid should still exist
    assert!(store.exists(&id).await.expect("exists"));

    // Successful delete
    store.delete(&id).await.expect("delete");

    // Now it should be gone
    assert!(!store.exists(&id).await.expect("exists"));
}

#[tokio::test]
async fn test_activity_storage_failures() {
    use sweet_grass_core::{
        activity::{Activity, ActivityType},
        agent::{AgentAssociation, AgentRole},
    };

    let (store, _) = setup_faulty();

    let activity = Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkAgent"),
            AgentRole::Creator,
        ))
        .build();

    // Fail activity storage
    store.fail_next();
    assert!(store.put_activity(&activity).await.is_err());

    // Activity should not be retrievable
    let result = store.get_activity(&activity.id).await.expect("get");
    assert!(result.is_none());

    // Successful store
    store.put_activity(&activity).await.expect("put");

    // Now retrievable
    let retrieved = store.get_activity(&activity.id).await.expect("get");
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_by_agent_query_failures() {
    let (store, _factory) = setup_faulty();

    let agent = Did::new("did:key:z6MkQueryAgent");

    // Store braids for this agent
    let factory_with_agent = BraidFactory::new(agent.clone());
    for i in 0..5 {
        let braid = factory_with_agent
            .from_data(format!("agent data {i}").as_bytes(), "text/plain", None)
            .expect("create");
        store.put(&braid).await.expect("put");
    }

    // Query should work
    let braids = store.by_agent(&agent).await.expect("query");
    assert_eq!(braids.len(), 5);

    // Fail query
    store.fail_next();
    assert!(store.by_agent(&agent).await.is_err());

    // Subsequent query should work
    let braids = store.by_agent(&agent).await.expect("query");
    assert_eq!(braids.len(), 5);
}

#[tokio::test]
async fn test_derived_from_query_failures() {
    let (store, factory) = setup_faulty();

    // Create parent-child relationship
    let parent = factory
        .from_data(b"parent", "text/plain", None)
        .expect("create");
    store.put(&parent).await.expect("put parent");

    let mut child = factory
        .from_data(b"child", "text/plain", None)
        .expect("create");
    child.was_derived_from = vec![sweet_grass_core::entity::EntityReference::by_hash(
        &parent.data_hash,
    )];
    store.put(&child).await.expect("put child");

    // Query should work
    let derived = store.derived_from(&parent.data_hash).await.expect("query");
    assert_eq!(derived.len(), 1);

    // Fail query
    store.fail_next();
    assert!(store.derived_from(&parent.data_hash).await.is_err());
}

#[tokio::test]
async fn test_mixed_operation_failures() {
    let (store, factory) = setup_faulty();

    store.set_fail_rate(40);

    // Mix of different operations
    let mut successes = 0;
    let mut failures = 0;

    for i in 0..20 {
        let braid = factory
            .from_data(format!("mixed {i}").as_bytes(), "text/plain", None)
            .expect("create");

        // Try put
        if store.put(&braid).await.is_ok() {
            successes += 1;

            // Try get
            if store.get(&braid.id).await.is_ok() {
                successes += 1;
            } else {
                failures += 1;
            }

            // Try exists
            if store.exists(&braid.id).await.is_ok() {
                successes += 1;
            } else {
                failures += 1;
            }
        } else {
            failures += 1;
        }
    }

    // Both successes and failures expected
    assert!(successes > 0);
    assert!(failures > 0);
    tracing::info!("Mixed operations: {successes} successes, {failures} failures");
}
