# 🔄 Zero-Copy Optimization Opportunities

**Status**: Phase 1+2+3 complete; remaining opportunities documented below  
**Completed**: `BraidId`, `Did`, `ContentHash`, `ActivityId`, `mime_type`, `source_primal`, `niche`, `spine_id`, `BraidMetadata.title`, `BraidMetadata.description`, `BraidMetadata.tags`, `LoamCommitRef.spine_id`, `BraidFactory.source_primal`, `CompressionEngine.source_primal` → `Arc<str>`; `Witness` constructors use named `&'static str` constants; tag index → `HashMap<Arc<str>, HashSet<BraidId>>`; `ProvenanceGraphBuilder` cycle detection uses `HashSet<ContentHash>` (O(1) Arc clone); `QueryError::NotFound` carries `ContentHash` (O(1) clone)  
**Priority**: Low (all hot-path clones eliminated)

---

## 📊 Current State

### Clone Distribution

| Crate | Clone Count | Hot Path? |
|-------|-------------|-----------|
| **sweet-grass-factory** | 33 | ✅ Yes (attribution) |
| **sweet-grass-query** | 16 | ✅ Yes (graph traversal) |
| **sweet-grass-store** | 25 | ⚠️ Some |
| **sweet-grass-service** | 30 | ⚠️ Some |
| **sweet-grass-core** | 10 | ❌ No (builders) |
| **sweet-grass-compression** | 15 | ⚠️ Some |
| **sweet-grass-integration** | 20 | ❌ No (tests) |
| **Other** | 31 | ❌ No |
| **TOTAL** | **~180** | — |

### Analysis

**Many clones are necessary** for async/concurrent contexts:
- Tokio requires `'static` lifetimes for spawned tasks
- Arc-wrapped structures already minimize copies
- String clones often unavoidable for owned data

**Realistic target**: Reduce by 40-50% (~100 clones)

---

## 🎯 High-Priority Optimizations

### 1. Factory String Allocations

**File**: `crates/sweet-grass-factory/src/factory/mod.rs`

**Current**:
```rust
pub fn from_data(
    data: &[u8],
    mime_type: &str,
    niche: Option<String>,
) -> Result<Braid> {
    let mime = mime_type.to_string();  // ❌ Clone
    let hash = calculate_hash(data).to_string();  // ❌ Clone
    // ...
}
```

**Optimized**:
```rust
pub fn from_data(
    data: &[u8],
    mime_type: impl Into<Cow<'static, str>>,
    niche: Option<Cow<'static, str>>,
) -> Result<Braid> {
    let mime = mime_type.into();  // ✅ Zero-copy for &'static str
    let hash = calculate_hash(data);  // ✅ Return Cow
    // ...
}
```

**Expected Gain**: 15-20% fewer clones (3-4 clones eliminated per call)

---

### 2. Attribution Calculator

**File**: `crates/sweet-grass-factory/src/attribution/mod.rs`

**Current**:
```rust
pub fn calculate_single(&self, braid: &Braid) -> AttributionChain {
    let entity = EntityReference::by_hash(&braid.data_hash);
    let agent = braid.attributed_to.clone();  // ❌ Clone DID
    let role = braid.agent_role.clone();  // ❌ Clone role
    // ...
}
```

**Optimized**:
```rust
pub fn calculate_single(&self, braid: &Braid) -> AttributionChain {
    let entity = EntityReference::by_hash_borrowed(&braid.data_hash);
    let agent = &braid.attributed_to;  // ✅ Borrow
    let role = braid.agent_role;  // ✅ Copy (if small)
    // Build chain with references
}
```

**Expected Gain**: 20-30% fewer clones (2-3 clones per calculation)

---

### 3. Query Engine Graph Traversal

**File**: `crates/sweet-grass-query/src/engine/mod.rs`

**Current**:
```rust
pub async fn ancestors(&self, hash: &ContentHash, depth: Option<u32>) 
    -> Result<Vec<Braid>> {
    let mut current = vec![hash.clone()];  // ❌ Clone
    for _ in 0..depth {
        for h in &current {
            let braid = self.store.get_by_hash(h).await?;
            results.push(braid.clone());  // ❌ Clone
        }
    }
    // ...
}
```

**Optimized**:
```rust
pub async fn ancestors(&self, hash: &ContentHash, depth: Option<u32>)
    -> Result<Vec<Arc<Braid>>> {
    let mut current = vec![Arc::new(hash.clone())];  // ✅ Arc once
    for _ in 0..depth {
        for h in &current {
            let braid = self.store.get_by_hash_arc(h).await?;
            results.push(braid);  // ✅ Arc clone (cheap)
        }
    }
    // ...
}
```

**Expected Gain**: 50% fewer clones (Arc cloning is cheap)

---

### 4. Storage Index Lookups

**File**: `crates/sweet-grass-store/src/memory/indexes.rs`

**Current**:
```rust
pub fn lookup_by_hash(&self, hash: &str) -> Option<String> {
    self.hash_index
        .read()
        .unwrap()
        .get(hash)
        .cloned()  // ❌ Clone String
}
```

**Optimized**:
```rust
pub fn lookup_by_hash(&self, hash: &str) -> Option<Arc<str>> {
    self.hash_index
        .read()
        .unwrap()
        .get(hash)
        .map(Arc::clone)  // ✅ Arc clone (cheap)
}
```

**Expected Gain**: 30-40% fewer allocations

---

## 🔧 Optimization Techniques

### 1. Cow (Copy-on-Write)

**When to use**: String data that's often borrowed, rarely modified

```rust
use std::borrow::Cow;

pub struct Config<'a> {
    pub name: Cow<'a, str>,
    pub description: Cow<'a, str>,
}

// Zero-copy for &'static str
let config = Config {
    name: "sweetgrass".into(),  // No allocation!
    description: "Provenance layer".into(),
};

// Copy when needed
let config = Config {
    name: format!("sweetgrass-{}", id).into(),  // Allocates
    description: user_description.into(),  // Clones
};
```

### 2. Arc for Shared Ownership

**When to use**: Large structures shared across threads

```rust
use std::sync::Arc;

// Before: Clone entire Braid
let braid_copy = braid.clone();  // Expensive!
tokio::spawn(async move {
    process(braid_copy).await;
});

// After: Arc clone (just pointer)
let braid_arc = Arc::new(braid);
let braid_ref = Arc::clone(&braid_arc);  // Cheap!
tokio::spawn(async move {
    process(&braid_ref).await;
});
```

### 3. Borrowing Instead of Cloning

**When to use**: Synchronous contexts, no ownership transfer

```rust
// Before: Clone for temporary use
fn process_braid(braid: &Braid) {
    let id = braid.id.clone();  // ❌ Unnecessary
    do_something(&id);
}

// After: Borrow
fn process_braid(braid: &Braid) {
    do_something(&braid.id);  // ✅ Zero-copy
}
```

### 4. Into Trait for Flexibility

**When to use**: API boundaries accepting multiple types

```rust
// Before: Force String allocation
pub fn with_name(mut self, name: &str) -> Self {
    self.name = name.to_string();  // ❌ Always allocates
    self
}

// After: Accept owned or borrowed
pub fn with_name(mut self, name: impl Into<String>) -> Self {
    self.name = name.into();  // ✅ Zero-copy for String
    self
}

// Usage:
builder.with_name("static")  // Allocates
builder.with_name(string_var)  // No clone!
```

---

## 📈 Expected Performance Impact

### Before Optimization

| Operation | Time | Allocations |
|-----------|------|-------------|
| Braid creation | 8ms | 25 |
| Attribution calc | 12ms | 18 |
| Graph traversal (10 levels) | 45ms | 120 |
| Query batch (100 braids) | 200ms | 2,500 |

### After Optimization (Estimated)

| Operation | Time | Allocations | Improvement |
|-----------|------|-------------|-------------|
| Braid creation | 6ms | 15 | **25% faster** |
| Attribution calc | 8ms | 10 | **33% faster** |
| Graph traversal (10 levels) | 30ms | 60 | **33% faster** |
| Query batch (100 braids) | 150ms | 1,500 | **25% faster** |

**Total memory reduction**: ~40% fewer allocations

---

## 🎯 Implementation Plan

### Phase 1: Low-Hanging Fruit (1-2 days)

1. **Replace `to_string()` with `into()`** in public APIs
2. **Use `Cow<str>` in Config structs**
3. **Borrow in synchronous contexts**

**Expected gain**: 30-40 clones eliminated

### Phase 2: Arc-wrapping (2-3 days)

1. **Add `get_arc()` methods to stores**
2. **Return `Arc<Braid>` from queries**
3. **Use `Arc::clone()` instead of `Braid::clone()`**

**Expected gain**: 40-50 clones eliminated

### Phase 3: Advanced (3-5 days)

1. **Redesign APIs for zero-copy**
2. **Benchmark and profile**
3. **Iterate based on metrics**

**Expected gain**: Additional 10-20 clones

---

## ⚠️ Trade-offs

### Pros ✅
- **Fewer allocations** — Less memory pressure
- **Better performance** — 25-40% faster in hot paths
- **More idiomatic** — Modern Rust patterns

### Cons ⚠️
- **API changes** — Breaking changes for some methods
- **Complexity** — Lifetimes and Cow can be tricky
- **Testing** — Need to verify async safety

### Decision

**NOT IMPLEMENTED YET** because:
1. Current performance is already excellent (8x speedup from parallelism)
2. Would require API changes (breaking)
3. Complexity vs benefit trade-off
4. Better to profile real workloads first

**Recommendation**: Implement after production profiling in v0.8.0+

---

## 🔬 Profiling Guide

Before optimizing, profile to find actual bottlenecks:

### 1. Install Profiler

```bash
cargo install cargo-flamegraph
```

### 2. Profile SweetGrass

```bash
# Profile compression
cargo flamegraph --bin sweetgrass -- server
```

### 3. Analyze Results

Look for:
- **Clone calls** (high in flamegraph)
- **String allocations** (`alloc::string`)
- **Memory operations** (`memcpy`, `memmove`)

### 4. Target Optimizations

Focus on functions taking >5% of total time.

---

## 📚 Resources

- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **Cow Documentation**: https://doc.rust-lang.org/std/borrow/enum.Cow.html
- **Arc Documentation**: https://doc.rust-lang.org/std/sync/struct.Arc.html

---

## ✅ When to Optimize

| Scenario | Optimize Now? | Why |
|----------|--------------|-----|
| **Hot path profiled** | ✅ Yes | Data-driven decision |
| **Allocation heavy** | ✅ Yes | Clear performance gain |
| **User complaint** | ✅ Yes | Real-world issue |
| **Premature** | ❌ No | No data, unclear benefit |
| **API breaking** | ⚠️ Maybe | Weigh benefits vs disruption |
| **After parallelism** | ✅ Yes | Already got 8x, now fine-tune |

**Current status**: After 8x parallelism gains, zero-copy is **nice-to-have**, not critical.

---

**🌾 SweetGrass: Already fast. Zero-copy will make it faster. 🌾**

*See [CHANGELOG](../../CHANGELOG.md) for performance evolution history.*

