# 🔧 Smart Refactoring Analysis - Dec 26, 2025

## 📊 Files Requiring Refactoring (>600 LOC)

### 1. `crates/sweet-grass-store-postgres/src/store.rs` (762 lines)

**Current Structure:**
- Single file implementing `PostgresStore` and `BraidStore` trait
- Mix of configuration, connection management, and CRUD operations
- Query building logic inline

**Refactoring Strategy:**
```
crates/sweet-grass-store-postgres/src/
├── store.rs (main struct, ~200 lines)
├── queries.rs (SQL query builders, ~200 lines)
├── conversions.rs (DB ↔ Braid conversions, ~150 lines)
├── config.rs (already exists, good!)
└── migrations.rs (already exists, good!)
```

**Benefits:**
- **Separation of concerns:** SQL, conversions, and API separated
- **Testability:** Each module can be unit tested independently
- **Maintainability:** Easier to find and update specific functionality
- **Code review:** Smaller, focused files

**Priority:** High (production storage layer)

---

### 2. `crates/sweet-grass-store-sled/src/store.rs` (767 lines)

**Current Structure:**
- Similar to PostgresStore - single file with all logic
- Sled-specific operations mixed with generic store logic

**Refactoring Strategy:**
```
crates/sweet-grass-store-sled/src/
├── store.rs (main struct, ~200 lines)
├── keys.rs (Key generation and parsing, ~150 lines)
├── serialization.rs (Braid ↔ bytes, ~150 lines)
├── indices.rs (Index management, ~150 lines)
└── transactions.rs (Sled transactions, ~100 lines)
```

**Benefits:**
- **Key management isolation:** Clearer key schema
- **Serialization clarity:** bincode logic separate
- **Index optimization:** Easier to add new indices
- **Transaction safety:** Focused transaction logic

**Priority:** Medium (alternative storage, less critical)

---

### 3. `crates/sweet-grass-store-postgres/tests/integration.rs` (800 lines)

**Current Structure:**
- All PostgreSQL integration tests in one file
- Tests for CRUD, queries, migrations, edge cases

**Refactoring Strategy:**
```
crates/sweet-grass-store-postgres/tests/
├── crud.rs (Create, Read, Update, Delete tests, ~200 lines)
├── queries.rs (Search and filter tests, ~200 lines)
├── migrations.rs (Migration and schema tests, ~150 lines)
├── edge_cases.rs (Error handling, boundaries, ~150 lines)
└── performance.rs (Perf and concurrency tests, ~100 lines)
```

**Benefits:**
- **Test organization:** Easy to find specific test categories
- **Parallel execution:** Smaller test files run concurrently
- **Maintenance:** Add tests to appropriate category
- **CI optimization:** Run specific test suites

**Priority:** Medium (tests, not production code)

---

## 🎯 Refactoring Principles

### 1. **Preserve Behavior**
- All refactoring must be behavior-preserving
- Run full test suite before and after
- No functional changes during refactoring

### 2. **Incremental Approach**
- Refactor one file at a time
- Commit after each successful refactoring
- Keep main branch stable

### 3. **Clear Module Boundaries**
- Each module has single responsibility
- Public API remains unchanged
- Internal structure improves

### 4. **Test Coverage Maintained**
- No reduction in test coverage
- Add tests if coverage gaps found
- Integration tests verify end-to-end

---

## 📋 Refactoring Checklist

### PostgresStore (Priority: High)

- [ ] Extract query builders to `queries.rs`
  - [ ] `build_insert_query()`
  - [ ] `build_select_query()`
  - [ ] `build_update_query()`
  - [ ] `build_delete_query()`
  - [ ] `build_search_query()`

- [ ] Extract conversions to `conversions.rs`
  - [ ] `braid_to_row()`
  - [ ] `row_to_braid()`
  - [ ] `json_to_braid()`
  - [ ] `braid_to_json()`

- [ ] Keep in `store.rs`:
  - [ ] `PostgresStore` struct
  - [ ] `BraidStore` trait implementation
  - [ ] Public API methods
  - [ ] Connection management

- [ ] Verify:
  - [ ] All tests pass
  - [ ] No performance regression
  - [ ] API unchanged
  - [ ] Documentation updated

### SledStore (Priority: Medium)

- [ ] Extract key generation to `keys.rs`
- [ ] Extract serialization to `serialization.rs`
- [ ] Extract indices to `indices.rs`
- [ ] Extract transactions to `transactions.rs`
- [ ] Verify all tests pass

### Integration Tests (Priority: Medium)

- [ ] Split by test category
- [ ] Ensure all tests still run
- [ ] Update CI configuration if needed
- [ ] Verify parallel execution works

---

## 🔍 Example: PostgresStore Query Extraction

### Before (in store.rs):
```rust
impl PostgresStore {
    pub async fn put(&self, braid: &Braid) -> Result<()> {
        let query = sqlx::query(
            "INSERT INTO braids (id, data_hash, content, created_at) 
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (id) DO UPDATE SET 
             content = EXCLUDED.content, 
             updated_at = NOW()"
        );
        // ... bind parameters and execute
    }
}
```

### After (queries.rs):
```rust
pub struct BraidQueries;

impl BraidQueries {
    pub fn insert() -> &'static str {
        "INSERT INTO braids (id, data_hash, content, created_at) 
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (id) DO UPDATE SET 
         content = EXCLUDED.content, 
         updated_at = NOW()"
    }
}
```

### After (store.rs):
```rust
impl PostgresStore {
    pub async fn put(&self, braid: &Braid) -> Result<()> {
        let query = sqlx::query(BraidQueries::insert());
        // ... bind parameters and execute
    }
}
```

**Benefits:**
- SQL queries centralized and easy to find
- Store.rs focuses on orchestration
- SQL can be tested independently
- Easier to optimize queries

---

## 📈 Expected Outcomes

### Metrics
| Metric | Before | After (Target) |
|--------|--------|----------------|
| Max file size | 800 LOC | <400 LOC |
| Modules per crate | 3-4 | 6-8 |
| Test discoverability | Low | High |
| Maintainability score | Medium | High |

### Benefits
- ✅ Easier code navigation
- ✅ Better separation of concerns
- ✅ Improved testability
- ✅ Clearer module boundaries
- ✅ Reduced cognitive load
- ✅ Faster onboarding for new contributors

---

## 🚀 Implementation Plan

### Phase 1: PostgresStore (Week 1)
1. Extract `queries.rs`
2. Extract `conversions.rs`
3. Update tests
4. Verify performance

### Phase 2: SledStore (Week 2)
1. Extract `keys.rs`
2. Extract `serialization.rs`
3. Extract `indices.rs`
4. Extract `transactions.rs`

### Phase 3: Test Organization (Week 3)
1. Split integration tests
2. Update CI configuration
3. Verify parallel execution
4. Document test categories

---

## 💡 Key Insights

1. **Not Just Line Count:** Refactoring isn't about hitting a line limit arbitrarily. It's about improving code organization and maintainability.

2. **Smart Splitting:** Group related functionality together. Don't split mid-function or create artificial boundaries.

3. **API Stability:** Public API should remain unchanged. Refactoring is internal restructuring only.

4. **Test First:** Ensure comprehensive test coverage before refactoring. Tests are your safety net.

5. **Incremental Progress:** Small, incremental changes are safer and easier to review than massive restructuring.

---

**Status:** ✅ Analysis Complete - Ready for implementation  
**Priority Files:** PostgresStore (High), SledStore (Medium), Tests (Medium)  
**Estimated Effort:** 2-3 weeks for all three  
**Risk Level:** Low (with good test coverage)

*Generated: Dec 26, 2025*

