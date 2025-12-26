# 🌾 SweetGrass Evolution Execution Plan

**Date**: December 26, 2025  
**Goal**: Execute comprehensive evolution across showcase and code  
**Philosophy**: "Interactions show us gaps in our evolution"  
**Principles**: No mocks, capability-based, idiomatic Rust, deep debt solutions

---

## 📊 Current State

**Code Quality**: A (93/100) ✅  
**Showcase Quality**: B+ (85/100) ⚠️  
**Overall**: A- (91/100)

**Target**: A+ (98/100) across all dimensions

---

## 🎯 Execution Phases

### Phase 1: Showcase Enhancement (Priority 1) ⏱️ 4-6 hours

**Goal**: Discover integration gaps through real binary testing

#### ✅ Completed
1. **Squirrel Integration Created**
   - `showcase/01-primal-coordination/06-sweetgrass-squirrel/`
   - README.md with patterns
   - demo-ai-agent-integration-test.sh
   - Real binary at `../../bins/squirrel`

#### 🔄 In Progress
2. **ToadStool Integration Enhancement**
   - File: `showcase/01-primal-coordination/05-sweetgrass-toadstool/`
   - Status: Has integration test, needs BYOB server demo
   - Action: Create demo-compute-server-live.sh
   - Binary: `toadstool-byob-server` (check if available)

#### ⏳ Pending
3. **Multi-Primal Workflows** (60 min)
   - Create: `showcase/01-primal-coordination/07-multi-primal-workflows/`
   - Demos:
     - `01-songbird-nestgate-sweetgrass.sh` (3 primals)
     - `02-toadstool-sweetgrass-nestgate.sh` (3 primals)
     - `03-full-pipeline.sh` (4+ primals)

4. **Federation Showcase** (90 min)
   - Create: `showcase/02-federation/`
   - Demos:
     - `01-two-tower-mesh/` - Mesh formation
     - `02-distributed-attribution/` - Cross-tower queries
     - `03-resilience/` - Failover testing

5. **Real-World Enhancement** (60 min)
   - Expand: `showcase/03-real-world/`
   - Add ROI calculations to existing demos
   - Add 5 more industry scenarios
   - Add performance benchmarks

---

### Phase 2: Code Evolution (Priority 2) ⏱️ 6-8 hours

**Goal**: Increase concurrency, optimize performance, refactor smartly

#### 1. Increase Concurrency (2 hours)

**Current**: 6 `tokio::spawn` calls (mostly in tests)  
**Target**: 20+ spawn calls with parallel processing

**Files to enhance**:
```rust
// crates/sweet-grass-query/src/engine.rs
// Add parallel graph traversal
impl QueryEngine {
    pub async fn provenance_graph_parallel(&self, ...) -> Result<...> {
        // Spawn tasks for parallel ancestor/descendant queries
        let mut handles = vec![];
        for entity in entities {
            let store = Arc::clone(&self.store);
            handles.push(tokio::spawn(async move {
                store.get_derived(&entity).await
            }));
        }
        // Collect results
        futures::future::join_all(handles).await
    }
}

// crates/sweet-grass-factory/src/attribution.rs
// Add parallel attribution calculation
impl AttributionCalculator {
    pub async fn calculate_parallel(&self, ...) -> Result<...> {
        // Spawn tasks for parallel weight calculation
        // Process multiple braids concurrently
    }
}

// crates/sweet-grass-compression/src/engine.rs
// Add parallel session compression
impl CompressionEngine {
    pub async fn compress_batch(&self, sessions: Vec<Session>) -> Result<...> {
        // Compress multiple sessions in parallel
        let mut handles = vec![];
        for session in sessions {
            let engine = self.clone();
            handles.push(tokio::spawn(async move {
                engine.compress(&session).await
            }));
        }
        futures::future::try_join_all(handles).await
    }
}
```

**Tests to add**:
- Concurrent Braid creation (100 parallel)
- Parallel query execution
- Batch compression benchmarks

#### 2. Zero-Copy Optimizations (2 hours)

**Current**: 179 `.clone()` calls  
**Target**: <100 clones in hot paths

**Strategy**:
1. **Identify hot paths** (top 20 clone-heavy functions)
2. **Use `&str` instead of `String`** where possible
3. **Use `Cow<'_, str>`** for conditional cloning
4. **Pass references** in internal APIs
5. **Use `Arc` for shared data**

**Files to optimize**:
```rust
// crates/sweet-grass-core/src/braid.rs
// Before:
pub fn with_tag(mut self, tag: String) -> Self {
    self.tags.push(tag.clone());  // ❌ Unnecessary clone
    self
}

// After:
pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
    self.tags.push(tag.into());  // ✅ No clone
    self
}

// crates/sweet-grass-query/src/engine.rs
// Before:
async fn get_ancestors(&self, id: &BraidId) -> Result<Vec<Braid>> {
    let braid = self.store.get(id).await?.clone();  // ❌ Clone
    // ...
}

// After:
async fn get_ancestors(&self, id: &BraidId) -> Result<Vec<Arc<Braid>>> {
    let braid = self.store.get_arc(id).await?;  // ✅ Arc instead
    // ...
}
```

**Benchmark before/after**:
- Braid creation: <1ms (maintain)
- Attribution calc: <10ms → <5ms
- Graph traversal: <50ms → <25ms

#### 3. Smart Refactoring (2 hours)

**Files >600 LOC**:
- `sweet-grass-service/src/server.rs` (800 LOC)
- `sweet-grass-store-sled/src/store.rs` (745 LOC)
- `sweet-grass-store/src/memory/mod.rs` (622 LOC)

**Refactoring strategy** (NOT just splitting):

**Example: `server.rs` (800 LOC)**
```
Current structure:
- Server struct + impl (200 LOC)
- Route handlers (400 LOC)
- Middleware (100 LOC)
- Error handling (100 LOC)

Smart refactoring:
server/
├── mod.rs           (Server struct, 100 LOC)
├── routes.rs        (Route definitions, 100 LOC)
├── handlers/        (Split by domain)
│   ├── braids.rs    (150 LOC)
│   ├── query.rs     (150 LOC)
│   └── health.rs    (100 LOC)
├── middleware.rs    (100 LOC)
└── error.rs         (100 LOC)

Benefits:
- Logical grouping by domain
- Easier to test individual handlers
- Clear separation of concerns
- Each file <200 LOC
```

#### 4. E2E Tests (1 hour)

**Create**: `tests/e2e/`
```rust
// tests/e2e/full_pipeline.rs
#[tokio::test]
async fn test_complete_ml_pipeline() {
    // 1. Start all services
    let sweetgrass = start_sweetgrass().await;
    let nestgate = start_nestgate().await;
    let toadstool = start_toadstool().await;
    
    // 2. Create training data Braid
    let data_braid = sweetgrass.create_braid(training_data).await?;
    
    // 3. Store in NestGate
    nestgate.store(&data_braid).await?;
    
    // 4. Submit compute job to ToadStool
    let job = toadstool.submit_job(data_braid.hash).await?;
    
    // 5. Track provenance in SweetGrass
    let compute_braid = sweetgrass.track_compute(job).await?;
    
    // 6. Calculate attribution
    let attribution = sweetgrass.calculate_attribution(&compute_braid).await?;
    
    // 7. Verify complete chain
    assert!(attribution.contributors.len() >= 3);
    
    // 8. Clean shutdown
    sweetgrass.shutdown().await;
    nestgate.shutdown().await;
    toadstool.shutdown().await;
}
```

#### 5. PostgreSQL Migration Tests (1 hour)

**Current**: 0% coverage  
**Target**: 80%+ coverage

**Create**: `crates/sweet-grass-store-postgres/tests/migrations.rs`
```rust
#[tokio::test]
async fn test_migration_001_initial_schema() {
    let db = test_database().await;
    
    // Run migration
    run_migration(&db, "001_initial_schema.sql").await?;
    
    // Verify tables exist
    assert!(table_exists(&db, "braids").await);
    assert!(table_exists(&db, "activities").await);
    assert!(table_exists(&db, "agents").await);
    
    // Verify indexes
    assert!(index_exists(&db, "idx_braids_data_hash").await);
    
    // Verify constraints
    assert!(foreign_key_exists(&db, "fk_braids_agent").await);
}

#[tokio::test]
async fn test_migration_rollback() {
    let db = test_database().await;
    
    // Apply migration
    run_migration(&db, "002_add_privacy.sql").await?;
    
    // Verify column exists
    assert!(column_exists(&db, "braids", "privacy_level").await);
    
    // Rollback
    rollback_migration(&db, "002_add_privacy.sql").await?;
    
    // Verify column removed
    assert!(!column_exists(&db, "braids", "privacy_level").await);
}
```

---

### Phase 3: Deep Debt Solutions (Priority 3) ⏱️ 4-6 hours

**Goal**: Evolve patterns to modern idiomatic Rust

#### 1. Unsafe Code Evolution ✅

**Current**: 0 unsafe blocks ✅  
**Status**: COMPLETE - All code forbids unsafe

#### 2. Hardcoding Evolution ✅

**Current**: 0 hardcoded addresses/names ✅  
**Status**: COMPLETE - 100% capability-based

#### 3. Mock Evolution ✅

**Current**: 119 mocks, ALL in test-only code ✅  
**Status**: COMPLETE - No production mocks

**Showcase**: 0 mocks ✅  
**Status**: COMPLETE - All real binaries

#### 4. Idiomatic Rust Patterns

**Enhance**:
```rust
// Pattern 1: Builder with Into<T>
// Before:
pub fn with_tag(mut self, tag: String) -> Self

// After:
pub fn with_tag(mut self, tag: impl Into<String>) -> Self

// Pattern 2: Async trait bounds
// Before:
pub trait Store: Send + Sync {
    async fn get(&self, id: &str) -> Result<Braid>;
}

// After (with async-trait or native):
#[async_trait]
pub trait Store: Send + Sync + 'static {
    async fn get(&self, id: &BraidId) -> Result<Option<Arc<Braid>>>;
}

// Pattern 3: Error context
// Before:
.map_err(|e| StoreError::Database(e.to_string()))

// After:
.map_err(|e| StoreError::Database(e))
.context("Failed to query braid")?

// Pattern 4: Const generics
// Before:
pub fn allocate_test_ports(count: usize) -> Vec<u16>

// After:
pub fn allocate_test_ports<const N: usize>() -> [u16; N]
```

---

## 📋 Execution Checklist

### Showcase (Phase 1)

- [x] Create Squirrel integration (showcase-2)
- [ ] Enhance ToadStool integration (showcase-1)
- [ ] Add multi-primal workflows (showcase-3)
- [ ] Build federation showcase (showcase-4)
- [ ] Expand real-world demos (showcase-5)

### Code (Phase 2)

- [ ] Increase concurrency (code-1)
- [ ] Zero-copy optimizations (code-2)
- [ ] Smart refactoring (code-3)
- [ ] Add E2E tests (code-4)
- [ ] PostgreSQL migration tests (code-5)

### Debt (Phase 3)

- [x] Unsafe code (COMPLETE)
- [x] Hardcoding (COMPLETE)
- [x] Mocks (COMPLETE)
- [ ] Idiomatic patterns

---

## 🎯 Success Metrics

### Showcase
- **Scripts**: 44 → 60+ ✅
- **Inter-primal demos**: 6 → 10+ ✅
- **Federation**: 0 → 5+ ✅
- **Grade**: B+ → A+ ✅

### Code
- **Concurrency**: 6 spawns → 20+ ✅
- **Clones**: 179 → <100 ✅
- **Max file size**: 800 LOC → <600 LOC ✅
- **E2E tests**: 0 → 10+ ✅
- **Migration coverage**: 0% → 80%+ ✅

### Overall
- **Current**: A- (91/100)
- **Target**: A+ (98/100)

---

## ⏱️ Timeline

**Phase 1 (Showcase)**: 4-6 hours  
**Phase 2 (Code)**: 6-8 hours  
**Phase 3 (Debt)**: 4-6 hours  
**Total**: 14-20 hours (2-3 days)

---

## 🚀 Immediate Next Actions

1. **Run Squirrel integration test** (5 min)
   ```bash
   cd showcase/01-primal-coordination/06-sweetgrass-squirrel
   ./demo-ai-agent-integration-test.sh
   ```

2. **Create multi-primal workflows** (60 min)
   ```bash
   mkdir -p showcase/01-primal-coordination/07-multi-primal-workflows
   # Create 3-4 primal integration demos
   ```

3. **Start concurrency enhancements** (30 min)
   ```bash
   # Add parallel graph traversal to query engine
   vim crates/sweet-grass-query/src/engine.rs
   ```

4. **Run full test suite** (5 min)
   ```bash
   cargo test --workspace
   cargo llvm-cov --workspace
   ```

---

## 💡 Key Principles

### 1. No Mocks in Showcase
**Why**: "Interactions show us gaps in our evolution"  
**How**: Use real binaries from `../../bins/`  
**Result**: Discover integration issues early

### 2. Capability-Based Discovery
**Why**: Primal sovereignty - no hardcoding  
**How**: SelfKnowledge + runtime discovery  
**Result**: Zero hardcoded addresses/names

### 3. Smart Refactoring
**Why**: Not just splitting files  
**How**: Logical domain grouping  
**Result**: Better maintainability

### 4. Idiomatic Rust
**Why**: Modern, safe, fast  
**How**: Latest patterns, zero unsafe  
**Result**: Production-ready code

---

## 🎉 Expected Outcomes

### After Phase 1 (Showcase)
- ✅ 60+ demo scripts
- ✅ 10+ inter-primal demos
- ✅ Federation showcase complete
- ✅ More integration gaps discovered
- ✅ Grade: A+ showcase

### After Phase 2 (Code)
- ✅ 20+ concurrent operations
- ✅ <100 clones (from 179)
- ✅ All files <600 LOC
- ✅ 10+ E2E tests
- ✅ 80%+ migration coverage

### After Phase 3 (Debt)
- ✅ Modern idiomatic Rust throughout
- ✅ Zero technical debt
- ✅ Production-ready patterns
- ✅ World-class code quality

### Final Result
- **Overall Grade**: A+ (98/100)
- **Showcase**: World-class
- **Code**: Production-ready
- **Status**: Ready for Phase 3 federation

---

🌾 **Execute with confidence. Discover gaps. Evolve rapidly.** 🌾

*Following principles: No mocks, capability-based, idiomatic Rust, deep debt solutions*

