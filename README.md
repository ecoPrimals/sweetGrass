# 🌾 SweetGrass

**Attribution Layer — Semantic Provenance & PROV-O**

SweetGrass is the storyteller of ecoPrimals Phase 2. It weaves meaning into data by tracking provenance, attribution, and contribution flows. Every piece of data has a story—SweetGrass tells it.

---

## 🚀 Status

**✅ Production Ready (Phase 2)** — Infant Discovery Architecture + Full Attribution Pipeline

| Metric | Value |
|--------|-------|
| **Version** | v0.4.0 (Phase 2 Production Ready) |
| **Tests** | 446 passing (100%) |
| **Function Coverage** | ~80% |
| **Migration Coverage** | 80%+ (PostgreSQL) |
| **Production Unwraps** | 0 (A+ Safety) |
| **Crates** | 9 |
| **Lines of Code** | ~19,200 |
| **Clippy** | Clean (pedantic + nursery, `-D warnings`) |
| **unsafe** | Forbidden (`#![forbid(unsafe_code)]`) |
| **Architecture** | Infant Discovery (zero-knowledge startup) |
| **Showcase** | 26 scripts (standalone + primal coordination) |

---

## ⚡ Quick Start

```bash
# Clone and build
cd sweetGrass
cargo build --release

# Run tests
cargo test --lib -- --test-threads=1

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check

# Try the showcase (no dependencies)
cd showcase/00-standalone
./RUN_ME_FIRST.sh

# Run with PostgreSQL
export DATABASE_URL="postgresql://localhost/sweetgrass"
cargo run --release --features postgres

# Run with coverage
cargo llvm-cov --workspace
```

---

## 🎯 What Makes SweetGrass Special?

### 1. **Provenance as a First-Class Citizen**
Every piece of data has a complete history—who created it, how it was transformed, who contributed. Not as metadata, but as the core data model (PROV-O compatible).

### 2. **Fair Attribution**
Attribution flows through derivation chains with configurable weights:
- Creator: 1.0
- Contributor: 0.5
- DataProvider: 0.4
- Transformer: 0.3
- Curator: 0.2
- Publisher: 0.1

Perfect for **sunCloud reward distribution**—contributors get paid fairly.

### 3. **Privacy by Design (GDPR-Inspired)**
Built-in data subject rights:
- Right to Access
- Right to Rectification
- Right to Erasure ("right to be forgotten")
- Right to Portability
- Right to Object

### 4. **Primal Sovereignty**
- **Zero-knowledge startup**: No hardcoded addresses
- **Capability-based discovery**: Find primals by what they can do
- **Pure Rust**: No C/C++ dependencies, no gRPC
- **Environment-driven**: 12-factor app compliant

### 5. **Multiple Storage Backends**
- **Memory**: Fast, ephemeral
- **PostgreSQL**: Production-grade, durable
- **Sled**: Embedded, pure Rust

---

## 🏗️ Architecture

```
SweetGrass (Attribution Layer)
    │
    ├── 📦 Braids (provenance records)
    │   ├── Entity — what (data hash, MIME, size)
    │   ├── Activity — how (creation, transformation)
    │   └── Attribution — who (DIDs, roles, weights)
    │
    ├── 🧮 Attribution Engine
    │   ├── Role-based weights
    │   ├── Derivation chains
    │   └── Time decay (optional)
    │
    ├── 🔍 Query Engine
    │   ├── Filter by agent, activity, time
    │   ├── Provenance graph traversal
    │   └── Ancestor/descendant queries
    │
    ├── 📤 PROV-O Export
    │   ├── W3C standard compliance
    │   ├── JSON-LD format
    │   └── Interoperability
    │
    ├── 🔐 Privacy Controls
    │   ├── Privacy levels (Public → Secret)
    │   ├── Data subject rights (GDPR)
    │   └── Retention policies
    │
    └── 💾 Storage Backends
        ├── Memory (testing)
        ├── PostgreSQL (production)
        └── Sled (embedded)
```

---

## 🎬 Showcase

SweetGrass includes a comprehensive showcase demonstrating all capabilities:

### Standalone Demos (`showcase/00-standalone/`)
1. **Braid Basics** — Creating and querying Braids
2. **Attribution Engine** — Fair contribution tracking
3. **Provenance Queries** — DAG traversal
4. **PROV-O Export** — W3C standard export
5. **Privacy Controls** — GDPR-inspired data rights

### Primal Coordination (`showcase/01-primal-coordination/`)
1. **Discovery Integration** — Capability-based discovery with Songbird
2. **ML Training Provenance** — Full ML pipeline with Beardog
3. **Session-Aware Braids** — Attestations with Nestgate

```bash
# Run all standalone demos (no dependencies)
cd showcase/00-standalone && ./RUN_ME_FIRST.sh

# Run primal coordination (requires phase1 bins)
cd showcase/01-primal-coordination && ./RUN_ME_FIRST.sh
```

---

## 🔗 Integration

SweetGrass integrates with other ecoPrimals:

| Primal | Capability | Integration |
|--------|-----------|-------------|
| **Songbird** | Discovery | Find primals by capability |
| **Beardog** | Compute | Track ML training provenance |
| **Nestgate** | Session | Link Braids to authenticated sessions |
| **Squirrel** | State | Distributed provenance state |
| **sunCloud** | Rewards | Fair attribution for payments |

All via **capability-based discovery** — zero hardcoded addresses!

---

## 📚 Documentation

- **[START_HERE.md](./START_HERE.md)** — Getting started guide
- **[STATUS.md](./STATUS.md)** — Current build status and metrics
- **[ROADMAP.md](./ROADMAP.md)** — Future development plans
- **[FINAL_HANDOFF.md](./FINAL_HANDOFF.md)** — Complete production handoff
- **[specs/](./specs/)** — Full specifications
- **[showcase/](./showcase/)** — Live demonstrations

### Key Docs
- **[EXECUTION_COMPLETE_DEC_24_2025.md](./EXECUTION_COMPLETE_DEC_24_2025.md)** — Complete audit execution summary
- **[COMPREHENSIVE_CODE_AUDIT_DEC_24_2025.md](./COMPREHENSIVE_CODE_AUDIT_DEC_24_2025.md)** — Full audit report

---

## 🧪 Testing

```bash
# All tests
cargo test --lib -- --test-threads=1

# Specific crate
cargo test -p sweet-grass-core

# With coverage
cargo llvm-cov --workspace

# PostgreSQL migrations (requires Docker)
cargo test -p sweet-grass-store-postgres --features integration-tests -- --ignored

# Clippy (pedantic)
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --check
```

---

## 🌐 API

### REST API (Axum)
```
POST   /braids              - Create Braid
GET    /braids/:id          - Get Braid
GET    /braids/hash/:hash   - Get by content hash
GET    /braids              - Query Braids (filters)
GET    /attribution/:id     - Calculate attribution
GET    /provenance/:id      - Get provenance graph
GET    /export/provo/:id    - Export to PROV-O

GET    /health              - Health check
GET    /health/detailed     - Detailed status
GET    /live                - Liveness probe
GET    /ready               - Readiness probe
GET    /status              - Service status
```

### tarpc RPC (Pure Rust)
```rust
trait SweetGrassService {
    async fn create_braid(braid: Braid) -> Result<BraidId>;
    async fn get_braid(id: BraidId) -> Result<Option<Braid>>;
    async fn query_braids(filter: QueryFilter) -> Result<Vec<Braid>>;
    async fn calculate_attribution(id: BraidId) -> Result<HashMap<Did, f64>>;
}
```

---

## 🔐 Security & Privacy

### Safety
- `#![forbid(unsafe_code)]` in all 9 crates
- **Zero production unwraps** (638 audited, all in tests)
- Comprehensive error handling
- Input validation

### Privacy
- Privacy levels: Public, Internal, Confidential, Secret
- Consent management (purpose-based)
- Retention policies (automatic deletion)
- Data subject rights (GDPR Article 15-21)
- Privacy-preserving queries

### Authentication
- DID-based identity
- Capability-based access control
- Session attestations (via Nestgate)
- Cryptographic signatures (Ed25519)

---

## 🚀 Deployment

### Environment Variables
```bash
# Primal Identity (Infant Discovery)
export PRIMAL_NAME="sweetgrass"
export PRIMAL_INSTANCE_ID="sweetgrass-prod-01"
export PRIMAL_CAPABILITIES="provenance,attribution"

# Storage Backend
export STORAGE_BACKEND="postgres"  # memory | postgres | sled
export DATABASE_URL="postgresql://user:pass@host/sweetgrass"

# Optional: PostgreSQL tuning
export PG_MAX_CONNECTIONS="20"
export PG_MIN_CONNECTIONS="5"

# Optional: Sled configuration
export STORAGE_PATH="./data/sweetgrass"
export SLED_CACHE_SIZE="512"  # MB

# Discovery (optional)
export DISCOVERY_URL="http://songbird:9000"

# Server
export REST_PORT="8080"
export TARPC_PORT="0"  # 0 = auto-allocate
```

### Docker (Example)
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features postgres

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/sweet-grass-service /usr/local/bin/
CMD ["sweet-grass-service"]
```

---

## 🤝 Contributing

SweetGrass follows ecoPrimals principles:

1. **Primal Sovereignty** — Pure Rust, no hardcoding
2. **Human Dignity** — Privacy & consent built-in
3. **Fair Attribution** — Everyone gets credit
4. **Test Quality** — 80%+ coverage
5. **Code Quality** — Clippy pedantic, zero warnings

---

## 📜 License

Copyright © 2024-2025 ecoPrimals  
All rights reserved.

---

## 🌾 Philosophy

> "Every piece of data has a story. SweetGrass tells it."

Provenance isn't metadata—it's the story of how data came to be. Attribution isn't accounting—it's recognizing contribution. Privacy isn't compliance—it's respect for human dignity.

**SweetGrass makes these principles real.**

---

*For detailed status and metrics, see [STATUS.md](./STATUS.md)*  
*For getting started, see [START_HERE.md](./START_HERE.md)*  
*For future plans, see [ROADMAP.md](./ROADMAP.md)*
