// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Concrete `BraidStore` backend — enum dispatch replacing `dyn` dispatch.
//!
//! `BraidBackend` enumerates all storage backends at compile time so the
//! `BraidStore` trait can use native `impl Future + Send` (RPITIT).

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
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;
    use sweet_grass_core::activity::ActivityType;
    use sweet_grass_core::agent::Did;
    use sweet_grass_core::entity::EntityReference;
    use sweet_grass_store_redb::RedbStore;
    use tempfile::TempDir;

    fn test_braid() -> Braid {
        Braid::builder()
            .data_hash("sha256:backend-test-001")
            .mime_type("text/plain")
            .size(128)
            .attributed_to(Did::new("did:key:z6MkBackendTest"))
            .build()
            .unwrap()
    }

    fn test_braid_with_hash(hash: &str, agent: &str) -> Braid {
        Braid::builder()
            .data_hash(hash)
            .mime_type("text/plain")
            .size(128)
            .attributed_to(Did::new(agent))
            .build()
            .unwrap()
    }

    fn open_redb_backend() -> (BraidBackend, TempDir) {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("backend-test.redb");
        let store = RedbStore::open_path(&db_path).unwrap();
        (BraidBackend::Redb(store), temp)
    }

    fn test_activity() -> Activity {
        Activity::builder(ActivityType::Computation)
            .compute_units(1.0)
            .build()
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

    #[tokio::test]
    async fn memory_backend_get_by_hash_and_query_paths() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let braid = test_braid();
        let hash = braid.data_hash.clone();
        let agent = Did::new("did:key:z6MkBackendTest");

        backend.put(&braid).await.unwrap();

        let by_hash = backend.get_by_hash(&hash).await.unwrap();
        assert!(by_hash.is_some());
        assert_eq!(by_hash.unwrap().id, braid.id);

        let by_agent = backend.by_agent(&agent).await.unwrap();
        assert_eq!(by_agent.len(), 1);

        let filter = QueryFilter::new().with_hash(hash.clone());
        let query = backend
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .unwrap();
        assert_eq!(query.total_count, 1);
    }

    #[tokio::test]
    async fn memory_backend_derived_from_and_get_all_by_hash() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let mut braid = test_braid_with_hash("sha256:derived-child", "did:key:z6MkBackendTest");
        braid
            .was_derived_from
            .push(EntityReference::by_hash("sha256:derived-source"));
        backend.put(&braid).await.unwrap();

        let derived = backend
            .derived_from(&"sha256:derived-source".into())
            .await
            .unwrap();
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].id, braid.id);

        let mut braid2 = test_braid_with_hash("sha256:converged-hash", "did:key:z6MkAlice");
        let mut braid3 = test_braid_with_hash("sha256:converged-hash", "did:key:z6MkBob");
        braid2.id = BraidId::new();
        braid3.id = BraidId::new();
        backend.put(&braid2).await.unwrap();
        backend.put(&braid3).await.unwrap();

        let all = backend.get_all_by_hash(&braid2.data_hash).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn memory_backend_activity_roundtrip() {
        let backend = BraidBackend::Memory(MemoryStore::new());
        let braid = test_braid();
        let activity = test_activity();

        backend.put(&braid).await.unwrap();
        backend.put_activity(&activity).await.unwrap();

        let retrieved = backend.get_activity(&activity.id).await.unwrap();
        assert!(retrieved.is_some());

        let for_braid = backend.activities_for_braid(&braid.id).await.unwrap();
        assert!(for_braid.is_empty());
    }

    #[tokio::test]
    async fn redb_backend_enum_dispatch_covers_store_paths() {
        let (backend, _temp) = open_redb_backend();
        let braid = test_braid_with_hash("sha256:redb-dispatch", "did:key:z6MkRedbTest");
        let hash = braid.data_hash.clone();
        let agent = Did::new("did:key:z6MkRedbTest");
        let activity = test_activity();

        backend.put(&braid).await.unwrap();
        backend.put_activity(&activity).await.unwrap();

        let by_id = backend.get(&braid.id).await.unwrap();
        assert!(by_id.is_some());

        let by_hash = backend.get_by_hash(&hash).await.unwrap();
        assert!(by_hash.is_some());

        let by_agent = backend.by_agent(&agent).await.unwrap();
        assert_eq!(by_agent.len(), 1);

        let filter = QueryFilter::new().with_hash(hash.clone());
        let query = backend
            .query(&filter, QueryOrder::NewestFirst)
            .await
            .unwrap();
        assert_eq!(query.total_count, 1);

        let mut derived_braid =
            test_braid_with_hash("sha256:redb-derived-child", "did:key:z6MkRedbTest");
        derived_braid
            .was_derived_from
            .push(EntityReference::by_hash("sha256:redb-derived-source"));
        backend.put(&derived_braid).await.unwrap();
        let derived = backend
            .derived_from(&"sha256:redb-derived-source".into())
            .await
            .unwrap();
        assert_eq!(derived.len(), 1);

        let mut braid2 = test_braid_with_hash("sha256:redb-converged", "did:key:z6MkAlice");
        let mut braid3 = test_braid_with_hash("sha256:redb-converged", "did:key:z6MkBob");
        braid2.id = BraidId::new();
        braid3.id = BraidId::new();
        backend.put(&braid2).await.unwrap();
        backend.put(&braid3).await.unwrap();
        let all = backend.get_all_by_hash(&braid2.data_hash).await.unwrap();
        assert_eq!(all.len(), 2);

        let retrieved_activity = backend.get_activity(&activity.id).await.unwrap();
        assert!(retrieved_activity.is_some());

        let for_braid = backend.activities_for_braid(&braid.id).await.unwrap();
        assert!(for_braid.is_empty());
    }

    #[tokio::test]
    async fn fault_injection_fail_puts_blocks_writes() {
        let store = FaultInjectionStore::new();
        let backend = BraidBackend::FaultInjection(Arc::clone(&store));
        let braid = test_braid();
        let activity = test_activity();

        store.set_fail_puts(true);
        assert!(backend.put(&braid).await.is_err());
        assert!(backend.put_activity(&activity).await.is_err());

        store.set_fail_puts(false);
        backend.put(&braid).await.unwrap();
        backend.put_activity(&activity).await.unwrap();
    }

    #[tokio::test]
    async fn fault_injection_fail_gets_blocks_reads() {
        let store = FaultInjectionStore::new();
        let backend = BraidBackend::FaultInjection(Arc::clone(&store));
        let braid = test_braid();
        let activity = test_activity();
        let hash = braid.data_hash.clone();

        backend.put(&braid).await.unwrap();
        backend.put_activity(&activity).await.unwrap();

        store.set_fail_gets(true);
        assert!(backend.get(&braid.id).await.is_err());
        assert!(backend.get_by_hash(&hash).await.is_err());
        assert!(backend.exists(&braid.id).await.is_err());
        assert!(backend.get_activity(&activity.id).await.is_err());

        store.set_fail_gets(false);
        assert!(backend.get(&braid.id).await.unwrap().is_some());
        assert!(backend.get_by_hash(&hash).await.unwrap().is_some());
        assert!(backend.exists(&braid.id).await.unwrap());
        assert!(backend.get_activity(&activity.id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn fault_injection_fail_queries_blocks_query_paths() {
        let store = FaultInjectionStore::new();
        let backend = BraidBackend::FaultInjection(Arc::clone(&store));
        let braid = test_braid();
        let agent = Did::new("did:key:z6MkBackendTest");
        let filter = QueryFilter::default();

        backend.put(&braid).await.unwrap();

        store.set_fail_queries(true);
        assert!(
            backend
                .query(&filter, QueryOrder::NewestFirst)
                .await
                .is_err()
        );
        assert!(backend.count(&filter).await.is_err());
        assert!(backend.by_agent(&agent).await.is_err());
        assert!(
            backend
                .derived_from(&"sha256:missing-source".into())
                .await
                .is_err()
        );
        assert!(backend.activities_for_braid(&braid.id).await.is_err());

        store.set_fail_queries(false);
        assert_eq!(backend.count(&filter).await.unwrap(), 1);
        assert_eq!(backend.by_agent(&agent).await.unwrap().len(), 1);
    }
}
