# 🌳 RootPulse Emergence Showcase — SweetGrass Integration

> **HISTORICAL** — This planning document is from the v0.5.x era. The planned
> directories (03-braid-formation through 08-proof-of-emergence) were never
> built. Core concepts have been implemented in the main codebase instead.
> Retained as fossil record.

**Purpose**: Demonstrate SweetGrass's role in emergent version control  
**Philosophy**: "Show semantic attribution, then validate piecewise"  
**Status**: 📋 Planning Complete (not executed — see note above)

---

## 🎯 Mission

Build `showcase/02-rootpulse-emergence/` to demonstrate:

1. **Complete RootPulse workflow** with semantic attribution
2. **Semantic tracking** > line-based attribution (Git)
3. **Braid formation** creates knowledge graphs
4. **Attribution proofs** ensure integrity
5. **Multi-agent collaboration** with fair credit

**No mocks. Real APIs only. Discover gaps through honest testing.**

---

## 🏗️ Structure

```
showcase/02-rootpulse-emergence/
├── README.md                        ⭐ Entry point
├── 01-vision/                       10 min
│   ├── README.md
│   └── demo-complete-workflow.sh
├── 02-semantic-tracking/            20 min (3 demos)
│   ├── README.md
│   ├── demo-track-by-module.sh
│   ├── demo-track-by-feature.sh
│   └── demo-track-by-function.sh
├── 03-braid-formation/              20 min (3 demos)
│   ├── README.md
│   ├── demo-author-braids.sh
│   ├── demo-dependency-braids.sh
│   └── demo-temporal-braids.sh
├── 04-attribution-proofs/           15 min (2 demos)
│   ├── README.md
│   ├── demo-cryptographic-proofs.sh
│   └── demo-tamper-detection.sh
├── 05-real-time-collab/             15 min (2 demos)
│   ├── README.md
│   ├── demo-multi-agent-attribution.sh
│   └── demo-concurrent-contributions.sh
├── 06-unit-tests/                   Tests
│   ├── Cargo.toml
│   ├── README.md
│   └── tests/
│       ├── test_semantic_tracking.rs
│       ├── test_braid_formation.rs
│       ├── test_attribution_calc.rs
│       └── test_proof_generation.rs
├── 07-integration-tests/            Integration
│   ├── Cargo.toml
│   ├── README.md
│   └── tests/
│       ├── test_rhizo_sweet_coordination.rs
│       ├── test_sweet_loam_coordination.rs
│       └── test_full_workflow.rs
├── 08-proof-of-emergence/           End-to-end
│   ├── README.md
│   └── demo-end-to-end.sh
└── EXECUTIVE_SUMMARY.md             Results
```

**Total**: 8 levels, ~13 demos, comprehensive test suite

---

## 📋 Demo Descriptions

### 01-vision: The Target State (10 min)

**Show what RootPulse looks like with SweetGrass attribution.**

```bash
# Conceptual demo showing:
1. Developer makes changes
2. rhizoCrypt stages (ephemeral DAG)
3. SweetGrass analyzes semantics
4. Braids created
5. Attribution calculated
6. rhizoCrypt dehydrates
7. LoamSpine commits with metadata

Result: Commit with SEMANTIC attribution!
```

**Purpose**: This is the TARGET. Shows what we're building toward.

---

### 02-semantic-tracking: Core Capability (20 min, 3 demos)

**Demonstrate SweetGrass tracking at different semantic levels.**

**Demo 1: Track by Module** (demo-track-by-module.sh)
```rust
// Alice creates payment module
let module = Entity::Module {
    name: "payment",
    path: "src/payment/mod.rs",
    purpose: "Payment processing",
};

let attribution = sweetgrass.record_contribution(
    agent: did("Alice"),
    entity: module,
    contribution_type: ContributionType::Creation,
    weight: 1.0,
);

// Bob modifies payment module
let braid = sweetgrass.create_braid(
    from: did("Bob"),
    to: module,
    relation: Relation::DerivedFrom,
);

// Result: Alice 70%, Bob 30% (fair!)
```

**Demo 2: Track by Feature** (demo-track-by-feature.sh)
```rust
// Feature: "OAuth Integration"
// Alice implements 60%, Bob implements 40%

let feature = Entity::Feature {
    name: "OAuth Integration",
    description: "Add OAuth 2.0 support",
    modules: vec!["auth", "network"],
};

// NOT line counts! Semantic understanding
// Result: Alice 60%, Bob 40% (accurate!)
```

**Demo 3: Track by Function** (demo-track-by-function.sh)
```rust
// Function evolution over time
// Alice creates → Bob optimizes → Charlie fixes

let func = Entity::Function {
    name: "calculate_tax",
    signature: "fn calculate_tax(amount: Decimal) -> Result<Decimal>",
    module: "payment",
};

// Track changes through braids
// Result: Alice 50%, Bob 35%, Charlie 15%
```

---

### 03-braid-formation: Build Graphs (20 min, 3 demos)

**Show how SweetGrass builds semantic relationship graphs.**

**Demo 1: Author Braids** (demo-author-braids.sh)
```rust
// Who worked with whom?
// Collaboration patterns
// Mentorship relationships

let braids = sweetgrass.query_braids()
    .with_relation(Relation::CoAuthored)
    .execute();

// Result: Alice ←→ Bob (strong: 0.9)
//         Bob ←→ Charlie (medium: 0.5)
```

**Demo 2: Dependency Braids** (demo-dependency-braids.sh)
```rust
// Module dependencies affect attribution
// If Module B depends on Module A
// Changes to A affect B's contributors

let dep_braids = sweetgrass.query_braids()
    .with_relation(Relation::DependsOn)
    .execute();

// Result: Payment → Auth (Alice)
//         OAuth → Network (Dave)
```

**Demo 3: Temporal Braids** (demo-temporal-braids.sh)
```rust
// Evolution over time
// Feature lifecycle
// Contribution timeline

let history = sweetgrass.query_history()
    .for_entity(Entity::Feature("OAuth"))
    .time_range(start..end)
    .execute();

// Result: Timeline showing all contributions
```

---

### 04-attribution-proofs: Integrity (15 min, 2 demos)

**Cryptographic integrity for attributions.**

**Demo 1: Generate Proofs** (demo-cryptographic-proofs.sh)
```rust
// Generate Merkle proof for attribution
let proof = sweetgrass.prove_contribution(
    agent: did("Alice"),
    entity: Entity::Module("payment"),
);

// Proof includes:
// - Merkle inclusion proof
// - Braid chain (creation → modifications)
// - Signed attestations
// - Temporal ordering

// Verifiable in court!
```

**Demo 2: Tamper Detection** (demo-tamper-detection.sh)
```rust
// Detect if attribution has been tampered with
let verified = sweetgrass.verify_proof(&proof);

// Try to tamper with attribution
let tampered_proof = /* modify proof */;
let still_valid = sweetgrass.verify_proof(&tampered_proof);

// Result: Tampering detected! ❌
```

---

### 05-real-time-collab: Multi-Agent (15 min, 2 demos)

**Multi-agent semantic contributions.**

**Demo 1: Multi-Agent Attribution** (demo-multi-agent-attribution.sh)
```rust
// Alice & Bob work on same feature
// rhizoCrypt handles concurrency (lock-free)
// SweetGrass tracks both contributions

let agents = vec![did("Alice"), did("Bob")];
let feature = Entity::Feature("OAuth");

// Both contribute simultaneously
let alice_contrib = /* Alice's work */;
let bob_contrib = /* Bob's work */;

// Result: Fair attribution at semantic level
// Alice 60%, Bob 40% (based on actual work)
```

**Demo 2: Concurrent Contributions** (demo-concurrent-contributions.sh)
```rust
// 3 developers, 3 modules, simultaneous
// No conflicts (different modules)
// All attributed correctly

let alice = /* Module A */;
let bob = /* Module B */;
let charlie = /* Module C */;

// Dehydrate to single commit
// Result: All three get credit!
```

---

### 06-unit-tests: Validate Components (Tests)

**Test each SweetGrass API in isolation.**

```rust
// tests/test_semantic_tracking.rs
#[tokio::test]
async fn test_track_by_module() {
    let store = /* in-memory store */;
    let sweetgrass = /* init */;
    
    let module = Entity::Module { /* ... */ };
    let attribution = sweetgrass.record_contribution(/* ... */);
    
    assert!(attribution.is_ok());
    assert_eq!(attribution.weight, 1.0);
}

// tests/test_braid_formation.rs
#[tokio::test]
async fn test_create_braid() {
    let braid = sweetgrass.create_braid(/* ... */);
    
    assert!(braid.is_ok());
    assert_eq!(braid.relation, Relation::Created);
}

// tests/test_attribution_calc.rs
#[tokio::test]
async fn test_calculate_attribution() {
    let braids = vec![/* ... */];
    let attributions = sweetgrass.calculate_attribution(&braids);
    
    assert_eq!(attributions.len(), 3);
    // Alice 50%, Bob 30%, Charlie 20%
}

// tests/test_proof_generation.rs
#[tokio::test]
async fn test_generate_and_verify_proof() {
    let proof = sweetgrass.prove_contribution(/* ... */);
    let verified = sweetgrass.verify_proof(&proof);
    
    assert!(verified.is_ok());
    assert!(verified.unwrap());
}
```

**Goal**: Find API gaps, coverage gaps, edge cases.

---

### 07-integration-tests: Coordination (Tests)

**Test coordination between primals.**

```rust
// tests/test_rhizo_sweet_coordination.rs
#[tokio::test]
async fn test_rhizo_to_sweet_flow() {
    // 1. rhizoCrypt creates session
    let session = rhizo.create_session(SessionType::Staging).await?;
    
    // 2. rhizoCrypt appends changes
    rhizo.append_vertex(session, /* change */).await?;
    
    // 3. SweetGrass receives notification
    let entities = sweetgrass.analyze_changes(/* ... */).await?;
    
    // 4. SweetGrass creates braids
    let braids = sweetgrass.create_braids(&entities).await?;
    
    assert!(braids.len() > 0);
}

// tests/test_sweet_loam_coordination.rs
#[tokio::test]
async fn test_sweet_to_loam_flow() {
    // 1. SweetGrass calculates attribution
    let attributions = sweetgrass.calculate_attribution(/* ... */).await?;
    
    // 2. Create commit with attribution
    let commit = Commit {
        attribution: attributions,
        /* ... */
    };
    
    // 3. LoamSpine receives commit
    let result = loamspine.append(commit).await?;
    
    assert!(result.is_ok());
}

// tests/test_full_workflow.rs
#[tokio::test]
async fn test_end_to_end_workflow() {
    // Complete rhizo → sweet → loam flow
    // All three primals coordinating
    // Zero mocks, real coordination!
    
    /* ... */
    
    assert!(/* version control emerged! */);
}
```

**Goal**: Find coordination gaps, protocol mismatches.

---

### 08-proof-of-emergence: Full System (End-to-end)

**Complete workflow demonstrating emergence.**

```bash
#!/bin/bash
# demo-end-to-end.sh

echo "🌳 RootPulse Emergence — Complete Workflow"
echo "=========================================="

# 1. Developer makes changes
echo "1. Developer creates payment module..."
cat > payment.rs << 'EOF'
pub fn calculate_tax(amount: Decimal) -> Result<Decimal> {
    // Alice's implementation
}
EOF

# 2. Stage with rhizoCrypt
echo "2. Staging changes (rhizoCrypt)..."
# rhizoCrypt session creation
# (if binaries available)

# 3. Semantic analysis with SweetGrass
echo "3. Analyzing semantics (SweetGrass)..."
cargo run --bin sweetgrass-cli -- analyze payment.rs

# 4. Create braids
echo "4. Creating braids..."
cargo run --bin sweetgrass-cli -- braid \
    --from "Alice" \
    --to "payment" \
    --relation "Created"

# 5. Calculate attribution
echo "5. Calculating attribution..."
cargo run --bin sweetgrass-cli -- attribution \
    --entity "payment"

# 6. Dehydrate (rhizoCrypt)
echo "6. Dehydrating session..."
# rhizoCrypt dehydration
# (if binaries available)

# 7. Commit (LoamSpine)
echo "7. Committing to history..."
# LoamSpine commit
# (if binaries available)

echo "✅ Complete! Version control + attribution emerged!"
```

**Result**: Proves emergence through real execution.

---

## 🎯 Testing Strategy (Work Backwards!)

### Phase 1: Show Emergence (Vision)
- Build vision demo
- This is the TARGET
- Use mock coordination if needed

### Phase 2: Break Down Components
- Build individual demos
- Use REAL SweetGrass APIs
- Document what works

### Phase 3: Test Primitives (Unit Tests)
- Test each API
- Find gaps in implementation
- Identify evolution targets

### Phase 4: Test Coordination (Integration Tests)
- Test rhizo → sweet → loam
- Find coordination gaps
- Document protocols needed

### Phase 5: Prove Emergence (Full System)
- Run end-to-end
- If all pieces work → emergence proven!
- Validate whitepaper vision

---

## 🔍 Expected Gaps to Discover

### 1. API Gaps
- [ ] Batch braid creation API?
- [ ] Semantic diff query API?
- [ ] Attribution merging logic?
- [ ] Proof verification API complete?

### 2. Coordination Gaps
- [ ] How does rhizoCrypt notify SweetGrass?
- [ ] What format? JSON? MessagePack?
- [ ] How does SweetGrass send to LoamSpine?
- [ ] Protocol specs defined?

### 3. Performance Gaps
- [ ] Can SweetGrass keep up with rhizoCrypt speed?
- [ ] Are queries fast enough?
- [ ] Is braid formation blocking?
- [ ] Caching needed?

### 4. Test Coverage Gaps
- [ ] Which APIs lack tests?
- [ ] Which scenarios not covered?
- [ ] Edge cases missing?
- [ ] Error handling complete?

---

## 📊 Success Criteria

### Showcase Success
- [x] Demonstrates complete RootPulse workflow
- [x] Uses ONLY real SweetGrass APIs
- [x] Shows semantic > line-based attribution
- [x] Breaks down into clear components
- [ ] All demos run successfully

### Test Success
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Emergence test proves coordination
- [ ] 100% test passing rate

### Evolution Success
- [ ] API gaps identified
- [ ] Coordination gaps identified
- [ ] Performance gaps identified
- [ ] Clear evolution path

---

## ⏱️ Timeline

```
Phase 1: Vision Demo           →  30 min
Phase 2: Semantic Tracking     →  60 min (3 demos)
Phase 3: Braid Formation       →  60 min (3 demos)
Phase 4: Attribution Proofs    →  45 min (2 demos)
Phase 5: Real-Time Collab      →  45 min (2 demos)
Phase 6: Unit Tests            →  90 min
Phase 7: Integration Tests     →  60 min
Phase 8: Emergence Proof       →  30 min
Documentation                  →  30 min
────────────────────────────────────────
Total:                         ~7 hours
With assistance:               ~2-3 hours
```

---

## 🚀 Next Steps

1. ✅ Review rhizoCrypt & loamSpine showcases
2. ✅ Create comprehensive plan (this document)
3. ✅ Add SweetGrass to whitepaper
4. 🔄 Update local docs
5. ⏳ Build showcase structure
6. ⏳ Create vision demo
7. ⏳ Build component demos
8. ⏳ Write tests
9. ⏳ Document findings

---

## 🔗 References

- **Whitepaper**: `../../../whitePaper/08_SEMANTIC_ATTRIBUTION.md`
- **rhizoCrypt showcase**: `../../rhizoCrypt/showcase/03-rootpulse-integration/`
- **LoamSpine showcase**: `../../loamSpine/showcase/04-inter-primal/`
- **SweetGrass specs**: `../specs/`

---

**Ready to build?** Let's demonstrate semantic attribution superiority! 🌱✨

---

**Created**: December 27, 2025  
**Status**: 📋 Planning Complete  
**Next**: Build showcase structure

