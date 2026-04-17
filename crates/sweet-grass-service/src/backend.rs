// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Concrete `BraidStore` backend — enum dispatch replacing `dyn` dispatch.
//!
//! `BraidBackend` enumerates all storage backends at compile time so the
//! `BraidStore` trait can use native `impl Future + Send` (RPITIT) instead
//! of `#[async_trait]` boxing.

#[cfg(any(test, feature = "test"))]
use std::sync::Arc;

use sweet_grass_core::{Activity, ActivityId, Braid, BraidId, ContentHash, agent::Did};
#[cfg(any(test, feature = "test"))]
use sweet_grass_store::StoreError;
use sweet_grass_store::{BraidStore, MemoryStore, QueryFilter, QueryOrder, QueryResult, Result};

/// Test-only store: delegates to [`MemoryStore`] but `count()` always fails.
/// Used by health/readiness handler tests (`SERVICE_UNAVAILABLE` paths).
#[cfg(any(test, feature = "test"))]
#[derive(Clone)]
pub struct CountFailingStore(pub Arc<MemoryStore>);

#[cfg(any(test, feature = "test"))]
impl BraidStore for CountFailingStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        self.0.put(braid).await
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        self.0.get(id).await
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        self.0.get_by_hash(hash).await
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        self.0.delete(id).await
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        self.0.exists(id).await
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        self.0.query(filter, order).await
    }

    async fn count(&self, _filter: &QueryFilter) -> Result<usize> {
        Err(StoreError::Internal("injected fault".to_string()))
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        self.0.by_agent(agent).await
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        self.0.derived_from(hash).await
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        self.0.put_activity(activity).await
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        self.0.get_activity(id).await
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        self.0.activities_for_braid(braid_id).await
    }
}

/// Test-only store for HTTP fault-injection integration tests (`tests/fault_injection.rs`).
#[cfg(any(test, feature = "test"))]
pub struct FaultInjectionStore {
    inner: MemoryStore,
    fail_puts: std::sync::atomic::AtomicBool,
    fail_gets: std::sync::atomic::AtomicBool,
    fail_queries: std::sync::atomic::AtomicBool,
}

#[cfg(any(test, feature = "test"))]
impl FaultInjectionStore {
    /// Create a new fault-injecting store backed by memory.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: MemoryStore::new(),
            fail_puts: std::sync::atomic::AtomicBool::new(false),
            fail_gets: std::sync::atomic::AtomicBool::new(false),
            fail_queries: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// When true, `put` / `put_activity` fail.
    pub fn set_fail_puts(&self, fail: bool) {
        self.fail_puts
            .store(fail, std::sync::atomic::Ordering::SeqCst);
    }

    /// When true, read paths that use `fail_gets` fail.
    pub fn set_fail_gets(&self, fail: bool) {
        self.fail_gets
            .store(fail, std::sync::atomic::Ordering::SeqCst);
    }

    /// When true, query-style operations fail.
    pub fn set_fail_queries(&self, fail: bool) {
        self.fail_queries
            .store(fail, std::sync::atomic::Ordering::SeqCst);
    }

    fn fault_error() -> sweet_grass_store::StoreError {
        sweet_grass_store::StoreError::Internal("injected fault".to_string())
    }
}

#[cfg(any(test, feature = "test"))]
impl BraidStore for FaultInjectionStore {
    async fn put(&self, braid: &Braid) -> Result<()> {
        if self.fail_puts.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.put(braid).await
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        if self.fail_gets.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get(id).await
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        if self.fail_gets.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get_by_hash(hash).await
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        self.inner.delete(id).await
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        if self.fail_gets.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.exists(id).await
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        if self.fail_queries.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.query(filter, order).await
    }

    async fn count(&self, filter: &QueryFilter) -> Result<usize> {
        if self.fail_queries.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.count(filter).await
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        if self.fail_queries.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.by_agent(agent).await
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        if self.fail_queries.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.derived_from(hash).await
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        if self.fail_puts.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.put_activity(activity).await
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        if self.fail_gets.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.get_activity(id).await
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        if self.fail_queries.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(Self::fault_error());
        }
        self.inner.activities_for_braid(braid_id).await
    }
}

/// Concrete backend enum — all storage variants known at compile time.
///
/// Replaces `Arc<dyn BraidStore>` with zero-cost enum dispatch.
#[expect(
    clippy::large_enum_variant,
    reason = "enum variants are large by design — each backend carries its state"
)]
pub enum BraidBackend {
    /// In-memory store (development, testing).
    Memory(MemoryStore),

    /// `redb` embedded Pure Rust store (recommended production).
    Redb(sweet_grass_store_redb::RedbStore),

    /// `PostgreSQL` store (multi-node production).
    Postgres(sweet_grass_store_postgres::PostgresStore),

    /// `NestGate` delegated store (ecosystem storage).
    #[cfg(feature = "nestgate")]
    NestGate(sweet_grass_store_nestgate::NestGateStore),

    /// Test-only: `count()` fails (health handler error paths).
    #[cfg(any(test, feature = "test"))]
    CountFailing(CountFailingStore),

    /// Test-only: togglable faults for HTTP fault-injection tests.
    #[cfg(any(test, feature = "test"))]
    FaultInjection(Arc<FaultInjectionStore>),
}

macro_rules! delegate_store {
    ($self:ident, $method:ident $(, $arg:expr)*) => {
        match $self {
            Self::Memory(s) => s.$method($($arg),*).await,
            Self::Redb(s) => s.$method($($arg),*).await,
            Self::Postgres(s) => s.$method($($arg),*).await,
            #[cfg(feature = "nestgate")]
            Self::NestGate(s) => s.$method($($arg),*).await,
            #[cfg(any(test, feature = "test"))]
            Self::CountFailing(s) => s.$method($($arg),*).await,
            #[cfg(any(test, feature = "test"))]
            Self::FaultInjection(s) => s.$method($($arg),*).await,
        }
    };
}

impl BraidStore for BraidBackend {
    async fn put(&self, braid: &Braid) -> Result<()> {
        delegate_store!(self, put, braid)
    }

    async fn get(&self, id: &BraidId) -> Result<Option<Braid>> {
        delegate_store!(self, get, id)
    }

    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Option<Braid>> {
        delegate_store!(self, get_by_hash, hash)
    }

    async fn get_all_by_hash(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        delegate_store!(self, get_all_by_hash, hash)
    }

    async fn delete(&self, id: &BraidId) -> Result<bool> {
        delegate_store!(self, delete, id)
    }

    async fn exists(&self, id: &BraidId) -> Result<bool> {
        delegate_store!(self, exists, id)
    }

    async fn query(&self, filter: &QueryFilter, order: QueryOrder) -> Result<QueryResult> {
        delegate_store!(self, query, filter, order)
    }

    async fn count(&self, filter: &QueryFilter) -> Result<usize> {
        delegate_store!(self, count, filter)
    }

    async fn by_agent(&self, agent: &Did) -> Result<Vec<Braid>> {
        delegate_store!(self, by_agent, agent)
    }

    async fn derived_from(&self, hash: &ContentHash) -> Result<Vec<Braid>> {
        delegate_store!(self, derived_from, hash)
    }

    async fn put_activity(&self, activity: &Activity) -> Result<()> {
        delegate_store!(self, put_activity, activity)
    }

    async fn get_activity(&self, id: &ActivityId) -> Result<Option<Activity>> {
        delegate_store!(self, get_activity, id)
    }

    async fn activities_for_braid(&self, braid_id: &BraidId) -> Result<Vec<Activity>> {
        delegate_store!(self, activities_for_braid, braid_id)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;

    fn test_braid() -> Braid {
        Braid::builder()
            .data_hash("sha256:backend-test-001")
            .mime_type("text/plain")
            .size(128)
            .attributed_to(Did::new("did:key:z6MkBackendTest"))
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn memory_backend_put_get_roundtrip() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let braid = test_braid();
        let id = braid.id.clone();

        backend.put(&braid).await.unwrap();
        let retrieved = backend.get(&id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[tokio::test]
    async fn memory_backend_exists() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let braid = test_braid();
        let id = braid.id.clone();

        assert!(!backend.exists(&id).await.unwrap());
        backend.put(&braid).await.unwrap();
        assert!(backend.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn memory_backend_delete() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let braid = test_braid();
        let id = braid.id.clone();

        backend.put(&braid).await.unwrap();
        let deleted = backend.delete(&id).await.unwrap();
        assert!(deleted);
        assert!(!backend.exists(&id).await.unwrap());
    }

    #[tokio::test]
    async fn memory_backend_count() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let filter = QueryFilter::default();

        let count = backend.count(&filter).await.unwrap();
        assert_eq!(count, 0);

        backend.put(&test_braid()).await.unwrap();
        let count = backend.count(&filter).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn memory_backend_get_nonexistent() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let id = BraidId::new();

        let result = backend.get(&id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn count_failing_store_delegates_reads() {
        let inner = Arc::new(MemoryStore::new());
        let backend = BraidBackend::CountFailing(CountFailingStore(inner));
        let braid = test_braid();
        let id = braid.id.clone();

        backend.put(&braid).await.unwrap();
        let retrieved = backend.get(&id).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn count_failing_store_fails_count() {
        let inner = Arc::new(MemoryStore::new());
        let backend = BraidBackend::CountFailing(CountFailingStore(inner));
        let filter = QueryFilter::default();

        let result = backend.count(&filter).await;
        assert!(result.is_err());
    }
}
