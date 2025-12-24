# SweetGrass — Specifications Index

**Version**: 1.0.0  
**Status**: Canonical  
**Last Updated**: December 2025

---

## Overview

SweetGrass is the **semantic provenance and attribution layer** of the ecoPrimals ecosystem. It grows from both RhizoCrypt (the living fungal network) and LoamSpine (the permanent geological record), making their activity visible and queryable.

**Pure Rust, Primal Sovereignty** — No gRPC, no protobuf, no vendor lock-in.

```
           ☀️ VISIBLE WORLD (Applications, gAIa, sunCloud)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       🌾 SWEETGRASS — Semantic layer above ground
           Braids, Attribution, Provenance Graphs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                      SOIL LINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       🍄 RHIZOCRYPT — Active fungal network (ephemeral)
───────────────────────────────────────────────────────────
       🦴 LOAMSPINE — Deep geological record (permanent)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Document Map

```
sweetGrass/specs/
├── 00_SPECIFICATIONS_INDEX.md     ← You are here
├── PRIMAL_SOVEREIGNTY.md          ← ⭐ Pure Rust principles
├── SWEETGRASS_SPECIFICATION.md    ← Master specification
├── ARCHITECTURE.md                ← System architecture
├── DATA_MODEL.md                  ← Braid & Entity structures
├── BRAID_COMPRESSION.md           ← 0/1/Many model, summaries
├── NICHE_PATTERNS.md              ← Configurable semantic patterns
├── ATTRIBUTION_GRAPH.md           ← Provenance for sunCloud
├── API_SPECIFICATION.md           ← tarpc, JSON-RPC, REST APIs
└── INTEGRATION_SPECIFICATION.md   ← Primal integrations via tarpc
```

---

## Reading Order

### 1. Core Principles
| Document | Purpose |
|----------|---------|
| [PRIMAL_SOVEREIGNTY.md](./PRIMAL_SOVEREIGNTY.md) | **⭐ START HERE** — Pure Rust, no gRPC, tarpc |
| [SWEETGRASS_SPECIFICATION.md](./SWEETGRASS_SPECIFICATION.md) | Master spec: principles, data model, full API |

### 2. System Design
| Document | Purpose |
|----------|---------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System components and data flow |
| [DATA_MODEL.md](./DATA_MODEL.md) | Braid, Activity, Agent, Entity structures |
| [BRAID_COMPRESSION.md](./BRAID_COMPRESSION.md) | How DAGs compress to Braids (0/1/many) |

### 3. Ecosystem Integration
| Document | Purpose |
|----------|---------|
| [NICHE_PATTERNS.md](./NICHE_PATTERNS.md) | How SweetGrass configures for biomeOS niches |
| [ATTRIBUTION_GRAPH.md](./ATTRIBUTION_GRAPH.md) | Provenance graphs for sunCloud attribution |
| [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) | RhizoCrypt, LoamSpine, BearDog integrations (tarpc) |

### 4. Implementation
| Document | Purpose |
|----------|---------|
| [API_SPECIFICATION.md](./API_SPECIFICATION.md) | tarpc service, JSON-RPC, REST endpoints |

---

## Quick Reference

### Primal Sovereignty Principles

```
❌ REJECTED                      ✅ ADOPTED
─────────────────────────────────────────────────────────
gRPC (requires protoc)           tarpc (pure Rust macros)
Protocol Buffers (Google)        serde + bincode (native)
.proto code generation           #[tarpc::service] macros
C/C++ toolchain deps             100% Rust compilation
Vendor lock-in                   Community-driven crates
```

### Protocol Stack

| Layer | Technology | Latency | Use Case |
|-------|------------|---------|----------|
| **Primary** | tarpc + bincode | ~50μs | Primal-to-primal |
| **Universal** | JSON-RPC 2.0 | ~2ms | Python, JS, curl |
| **Debug** | HTTP/REST | ~10ms | Admin, debugging |

### What SweetGrass Does

| Function | Description |
|----------|-------------|
| **Provenance** | Tracks what created data, who contributed, where it came from |
| **Attribution** | Calculates contributor shares for sunCloud rewards |
| **Semantic Linking** | Connects data across RhizoCrypt sessions and LoamSpine spines |
| **Query Engine** | PROV-O export, provenance graph traversal |

### Core Data Structures

| Structure | Purpose |
|-----------|---------|
| **Braid** | Provenance record following W3C PROV-O |
| **Activity** | Process that creates or transforms data |
| **Agent** | Person, software, or organization that acts |
| **Entity** | Data artifact with provenance |

### Braid Cardinality

| Count | Meaning |
|-------|---------|
| **0** | Session explored but discarded |
| **1** | Single coherent record (hardest case) |
| **Many** | Summary hierarchies, braids of braids |

### Standards

| Standard | Usage |
|----------|-------|
| **W3C PROV-O** | Provenance ontology (Entity, Activity, Agent) |
| **JSON-LD** | Linked data serialization |
| **DIDs** | Decentralized identifiers (via BearDog) |
| **tarpc** | Pure Rust RPC framework |

---

## Dependencies

### Required Crates (Pure Rust)

```toml
tarpc = { version = "0.34", features = ["full"] }  # RPC
serde = { version = "1.0", features = ["derive"] }  # Serialization
bincode = "1.3"                                     # Binary format
tokio = { version = "1.40", features = ["full"] }   # Async
axum = "0.7"                                        # HTTP fallback
```

### Forbidden Crates

```toml
# ❌ NEVER add:
# tonic, prost, protobuf, grpc (C++ deps, vendor lock-in)
```

### Required Primals

| Primal | Dependency Type |
|--------|-----------------|
| **BearDog** | Required: DID resolution, signing |
| **LoamSpine** | Required: Permanent Braid anchoring |
| **RhizoCrypt** | Required: Session activity source |

### Optional Primals

| Primal | Integration |
|--------|-------------|
| **ToadStool** | Activity events from compute tasks |
| **Songbird** | Service discovery |
| **Squirrel** | AI agent provenance |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | Dec 2025 | Added PRIMAL_SOVEREIGNTY, tarpc APIs |
| 0.2.0 | Dec 2025 | Added compression model, niche patterns |
| 0.1.0 | Dec 2025 | Initial specification |

---

*SweetGrass: Pure Rust semantic provenance — weaving the stories that give data its meaning.*
