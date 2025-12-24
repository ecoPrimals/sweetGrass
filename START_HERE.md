# 🌾 SweetGrass — Start Here

Welcome to SweetGrass! This guide will get you up to speed quickly.

**Version**: 0.4.0 (Phase 2) | **Status**: ✅ Production Ready | **Tests**: 446 passing

---

## 📖 What is SweetGrass?

SweetGrass is the **Attribution Layer** for ecoPrimals Phase 2. It's the storyteller—weaving meaning into data by tracking provenance and attribution.

**Key insight**: SweetGrass doesn't store data; it stores stories *about* data—who created it, who contributed, how it was transformed, who gets credit.

---

## 🚀 Quick Start (5 Minutes)

### 1. Build
```bash
cd sweetGrass
cargo build --release
```

### 2. Test
```bash
cargo test --lib -- --test-threads=1  # 446 tests, all passing
```

### 3. Try the Showcase
```bash
cd showcase/00-standalone
./RUN_ME_FIRST.sh  # Interactive demos, no dependencies!
```

### 4. Check Quality
```bash
cargo clippy --all-targets --all-features -- -D warnings  # Zero warnings
cargo fmt --check                                          # All formatted
```

---

## 🎬 Interactive Demos

The best way to understand SweetGrass is to **see it in action**:

```bash
# Standalone demos (5-10 minutes each)
cd showcase/00-standalone
./RUN_ME_FIRST.sh

# What you'll learn:
#   1. Creating Braids (provenance records)
#   2. Fair attribution calculations
#   3. Querying provenance graphs
#   4. Exporting to PROV-O (W3C standard)
#   5. Privacy controls (GDPR-inspired)
```

All demos feature:
- 🎨 **Colored output** for clarity
- 📖 **Narrative explanations** at each step
- 🌍 **Real-world scenarios** (ML training, HIPAA, GDPR)
- ✅ **No dependencies** (uses in-memory storage)

---

## 🏗️ Architecture (60 Seconds)

```
┌─────────────────────────────────────────────────┐
│              SweetGrass Service                 │
│                                                 │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐    │
│  │  REST   │  │  tarpc   │  │  Showcase  │    │
│  │  API    │  │  RPC     │  │  Scripts   │    │
│  └────┬────┘  └────┬─────┘  └──────┬─────┘    │
│       │            │                │          │
│       └────────────┴────────────────┘          │
│                    │                            │
│           ┌────────▼───────────┐               │
│           │  Core Components   │               │
│           ├────────────────────┤               │
│           │  • Braids          │               │
│           │  • Attribution     │               │
│           │  • Queries         │               │
│           │  • PROV-O          │               │
│           │  • Privacy         │               │
│           └────────┬───────────┘               │
│                    │                            │
│           ┌────────▼───────────┐               │
│           │  Storage Backend   │               │
│           ├────────────────────┤               │
│           │  Memory│Postgres   │               │
│           │       │   Sled     │               │
│           └────────────────────┘               │
└─────────────────────────────────────────────────┘
```

**3 Layers**:
1. **API** — REST + tarpc RPC
2. **Core** — Braids, Attribution, Queries, Privacy
3. **Storage** — Memory, PostgreSQL, or Sled

---

## 🎯 Core Concepts

### 1. Braids (Provenance Records)

A **Braid** is a complete provenance record:

```rust
Braid {
    id:                  "urn:braid:uuid:...",
    data_hash:           "sha256:abc123...",  // What data
    was_generated_by:    Activity,            // How created
    was_attributed_to:   [Agent],             // Who contributed
    was_derived_from:    [Entity],            // What sources
    // + privacy, signatures, metadata
}
```

**Think of it like Git**, but for any data:
- Git tracks code changes → Braids track data transformations
- Git has commits → Braids have Activities
- Git has authors → Braids have Attributed Agents

### 2. Attribution

Attribution tracks **who contributed and how much**:

```
Original Data (Alice creates dataset)
  └─ Alice: 100%

↓ Bob processes the data

Processed Data
  ├─ Alice: 57% (DataProvider, weight 0.4)
  └─ Bob:   43% (Transformer, weight 0.3)

↓ Carol creates visualization

Final Output
  ├─ Alice:  24% (inherited)
  ├─ Bob:    18% (inherited)
  └─ Carol:  59% (Creator, weight 1.0)
```

Perfect for **sunCloud reward distribution** — everyone gets paid fairly!

### 3. Provenance Queries

Ask questions about data history:

```bash
# Where did this come from?
provenance_ancestors(braid_id, depth=5)

# What was created from this?
provenance_descendants(braid_id, depth=5)

# What has Alice created?
braids_by_agent(alice_did)

# What happened last week?
braids_by_time_range(start, end)
```

### 4. Privacy (GDPR-Inspired)

Built-in data subject rights:

- **Right to Access** — See all your data
- **Right to Rectification** — Fix incorrect data
- **Right to Erasure** — Delete your data ("right to be forgotten")
- **Right to Portability** — Export your data
- **Right to Object** — Opt-out of processing

**Privacy levels**: Public → Internal → Confidential → Secret

---

## 💻 Development Workflow

### Daily Development
```bash
# Make changes
vim crates/sweet-grass-core/src/braid.rs

# Test
cargo test -p sweet-grass-core

# Check quality
cargo clippy --package sweet-grass-core
cargo fmt --package sweet-grass-core

# Run all tests
cargo test --lib -- --test-threads=1
```

### Adding a Feature
```bash
# 1. Write tests first
vim crates/sweet-grass-core/src/my_feature.rs

# 2. Implement
cargo test -p sweet-grass-core

# 3. Document
cargo doc --package sweet-grass-core --open

# 4. Quality checks
cargo clippy --package sweet-grass-core -- -D warnings
cargo fmt --check
```

### Coverage
```bash
# Generate coverage report
cargo llvm-cov --workspace --html

# Open in browser
open target/llvm-cov/html/index.html
```

---

## 🔧 Configuration

SweetGrass uses **environment variables** (12-factor app):

### Minimal (Memory Storage)
```bash
export PRIMAL_NAME="sweetgrass"
cargo run --release
```

### Production (PostgreSQL)
```bash
export PRIMAL_NAME="sweetgrass-prod"
export PRIMAL_INSTANCE_ID="sweetgrass-01"
export STORAGE_BACKEND="postgres"
export DATABASE_URL="postgresql://user:pass@host/sweetgrass"
export PG_MAX_CONNECTIONS="20"
cargo run --release --features postgres
```

### Embedded (Sled)
```bash
export PRIMAL_NAME="sweetgrass"
export STORAGE_BACKEND="sled"
export STORAGE_PATH="./data/sweetgrass"
export SLED_CACHE_SIZE="512"
cargo run --release --features sled
```

---

## 🧪 Testing Philosophy

SweetGrass has **446 tests** across 9 crates with **80%+ coverage**:

| Test Type | Count | Purpose |
|-----------|-------|---------|
| **Unit** | 418 | Test individual functions |
| **Integration** | 20+ | Test full pipeline |
| **Chaos** | 8 | Test fault tolerance |
| **Property** | ✅ | Test invariants (proptest) |
| **Migration** | 13 | Test database schema |

### Test Categories
- ✅ **Correctness** — Does it work?
- ✅ **Safety** — Zero production unwraps
- ✅ **Performance** — Benchmarked operations
- ✅ **Chaos** — Fault injection
- ✅ **Property** — Invariants hold

---

## 🐛 Debugging Tips

### Enable Tracing
```bash
RUST_LOG=sweet_grass=debug cargo run
RUST_LOG=sweet_grass=trace cargo run  # Very verbose
```

### Test Specific Function
```bash
cargo test --lib test_attribution_basic -- --nocapture
```

### Check Lints
```bash
# Pedantic mode
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Nursery lints (experimental)
cargo clippy --workspace --all-targets --all-features -- \
    -W clippy::nursery \
    -D warnings
```

---

## 📚 Next Steps

### Understand the System
1. Read [README.md](./README.md) — Overview
2. Check [STATUS.md](./STATUS.md) — Current metrics
3. Browse [specs/](./specs/) — Detailed specs
4. Review [FINAL_HANDOFF.md](./FINAL_HANDOFF.md) — Production handoff

### Explore the Code
```bash
# Core provenance model
crates/sweet-grass-core/src/braid.rs

# Attribution engine
crates/sweet-grass-factory/src/attribution.rs

# Storage trait
crates/sweet-grass-store/src/lib.rs

# PostgreSQL implementation
crates/sweet-grass-store-postgres/src/store.rs

# REST API
crates/sweet-grass-service/src/server.rs
```

### Run Demos
```bash
# Standalone (no dependencies)
cd showcase/00-standalone && ./RUN_ME_FIRST.sh

# Primal coordination (requires phase1 bins)
cd showcase/01-primal-coordination && ./RUN_ME_FIRST.sh
```

---

## 🎓 Learning Resources

### Provenance Concepts
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — Official standard
- [specs/SWEETGRASS_SPECIFICATION.md](./specs/SWEETGRASS_SPECIFICATION.md) — Our implementation
- [specs/ATTRIBUTION_GRAPH.md](./specs/ATTRIBUTION_GRAPH.md) — Attribution algorithm

### Architecture
- [specs/ARCHITECTURE.md](./specs/ARCHITECTURE.md) — System design
- [specs/PRIMAL_SOVEREIGNTY.md](./specs/PRIMAL_SOVEREIGNTY.md) — Design principles
- [specs/INTEGRATION_SPECIFICATION.md](./specs/INTEGRATION_SPECIFICATION.md) — How primals connect

### API
- [specs/API_SPECIFICATION.md](./specs/API_SPECIFICATION.md) — REST + tarpc

---

## 🤝 Integration with Other Primals

SweetGrass works with:

| Primal | What It Does | How SweetGrass Uses It |
|--------|--------------|------------------------|
| **Songbird** | Discovery | Find primals by capability |
| **Beardog** | Compute | Track ML training provenance |
| **Nestgate** | Sessions | Link Braids to authenticated users |
| **Squirrel** | State | Distributed provenance state |
| **sunCloud** | Rewards | Fair payment based on attribution |

See `showcase/01-primal-coordination/` for live examples!

---

## ❓ FAQ

**Q: What's the difference between SweetGrass and a database?**  
A: Databases store data. SweetGrass stores the *story* of data—who created it, how it was transformed, who gets credit.

**Q: Do I need to use PostgreSQL?**  
A: No! Use memory (testing), Sled (embedded), or PostgreSQL (production). It's configurable.

**Q: How is this different from Git?**  
A: Git is for code. SweetGrass is for *any* data. Git has linear history. SweetGrass supports DAGs (multiple parents).

**Q: What's a "Braid"?**  
A: A provenance record—the complete story of how a piece of data came to be.

**Q: Why "Attribution"?**  
A: To answer: "Who gets credit (and payment) for this data?" Essential for fair reward distribution in sunCloud.

**Q: Is it GDPR compliant?**  
A: SweetGrass has GDPR-*inspired* features (data subject rights, retention policies). Full compliance requires organizational processes too.

---

## 🌾 Philosophy

> "Every piece of data has a story. SweetGrass tells it."

Provenance isn't an afterthought—it's fundamental. Attribution isn't accounting—it's recognizing contribution. Privacy isn't compliance—it's human dignity.

**Welcome to SweetGrass!**

---

*Need help? Check [STATUS.md](./STATUS.md) for build status and metrics.*  
*Ready to dive deep? See [README.md](./README.md) for complete documentation.*
