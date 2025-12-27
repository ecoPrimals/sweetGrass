# 🧠 Semantic Tracking — Track by Module, Feature, Function

**Purpose**: Demonstrate SweetGrass tracking at different semantic levels  
**Type**: Real API demonstrations  
**Time**: 20 minutes (3 demos)

---

## 🎯 What This Level Proves

**Git tracks**: Files and line numbers (syntactic)  
**SweetGrass tracks**: Modules, features, functions (semantic)

This is the CORE capability that makes semantic attribution possible!

---

## 📋 Three Demos

### Demo 1: Track by Module
**File**: `demo-track-by-module.sh`  
**Shows**: How SweetGrass tracks module-level contributions

### Demo 2: Track by Feature
**File**: `demo-track-by-feature.sh`  
**Shows**: How SweetGrass tracks feature-level contributions

### Demo 3: Track by Function
**File**: `demo-track-by-function.sh`  
**Shows**: How SweetGrass tracks function-level contributions

---

## 🌟 The Difference

### Git's View (Syntactic)

```bash
$ git log --oneline src/payment/mod.rs
a3f2b1c Alice: Add payment module
b7e9d2a Bob: Optimize tax calculation
c1f4e8b Charlie: Fix typo

$ git blame src/payment/mod.rs
a3f2b1c (Alice)   pub mod payment {
b7e9d2a (Bob)       pub fn calculate_tax() {
c1f4e8b (Charlie)     amount * rate  // typo fix
```

**Result**: Alice 50%, Bob 30%, Charlie 20% (based on lines)

---

### SweetGrass View (Semantic)

```rust
// SweetGrass tracks ENTITIES
Entity::Module {
    name: "payment",
    created_by: Alice,
    modified_by: [Bob, Charlie],
}

Entity::Function {
    name: "calculate_tax",
    created_by: Alice,
    optimized_by: Bob,
    maintained_by: Charlie,
}

// SweetGrass calculates FAIR attribution
Attribution {
    Alice: 60%   (created module + function)
    Bob: 30%     (critical optimization)
    Charlie: 10% (maintenance)
}
```

**Result**: Fair attribution based on semantic contribution!

---

## 🔧 Real APIs Used

These demos will use actual SweetGrass APIs:

```rust
// From sweet-grass-core
use sweet_grass_core::{Entity, Braid, Did, Attribution};

// From sweet-grass-store
use sweet_grass_store::BraidStore;

// From sweet-grass-attribution
use sweet_grass_attribution::AttributionCalculator;
```

---

## 🎓 What You'll Learn

### Demo 1: Module Tracking
- How to create module entities
- How to record module creation
- How to track module modifications
- How attribution flows through module hierarchy

### Demo 2: Feature Tracking
- How to create feature entities (cross-module)
- How to track feature implementation progress
- How to attribute multi-module features
- How percentage calculation works

### Demo 3: Function Tracking
- How to create function entities
- How to track function evolution
- How optimization vs maintenance is weighted
- How temporal attribution works

---

## 🚀 Run the Demos

```bash
# Demo 1: Track by Module
./demo-track-by-module.sh

# Demo 2: Track by Feature
./demo-track-by-feature.sh

# Demo 3: Track by Function
./demo-track-by-function.sh
```

---

## 📊 Expected Gaps to Discover

As we build these demos with REAL APIs, we'll discover:

### API Gaps
- [ ] Do we have Entity creation APIs?
- [ ] Can we track at module level?
- [ ] Can we track at feature level?
- [ ] Can we track at function level?
- [ ] Attribution calculation APIs complete?

### Data Structure Gaps
- [ ] Entity types sufficient?
- [ ] Braid relations comprehensive?
- [ ] Attribution weights defined?
- [ ] Temporal tracking supported?

### Performance Gaps
- [ ] Entity creation fast enough?
- [ ] Query performance acceptable?
- [ ] Storage efficient?

---

## 🔗 Next Level

After semantic tracking, we'll explore:
- **[../03-braid-formation/](../03-braid-formation/)** — Building relationship graphs

---

🌱 **Let's see what SweetGrass can REALLY do!**

---

**Status**: 🏗️ Building with real APIs  
**Goal**: Discover gaps through honest testing

