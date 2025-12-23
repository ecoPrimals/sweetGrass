# 🌾 SweetGrass — Project Status

**Last Updated**: December 22, 2025  
**Version**: 0.1.0  
**Status**: 🌱 **Scaffolded** — Ready for Core Implementation  
**Grade**: N/A (Pre-implementation)

---

## 📊 Current State

### Build Status
| Metric | Status |
|--------|--------|
| **Compilation** | ✅ Clean |
| **Tests** | ✅ 0/0 (scaffold only) |
| **Linting** | ✅ Clean (pedantic clippy) |
| **Documentation** | 🟡 Scaffold docs only |

### Implementation Progress

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Traits** | ✅ Done | `PrimalLifecycle`, `PrimalHealth` |
| **Configuration** | ✅ Done | Basic `SweetGrassConfig` |
| **Error Types** | ✅ Done | Basic `SweetGrassError` |
| **Braid Structure** | ⬜ Not Started | PROV-O entities |
| **Attribution Types** | ⬜ Not Started | Agent roles, compute |
| **Provenance Graph** | ⬜ Not Started | DAG traversal |
| **Query Engine** | ⬜ Not Started | Attribution queries |
| **PROV-O Export** | ⬜ Not Started | JSON-LD output |
| **GraphQL API** | ⬜ Not Started | Query interface |

---

## 🎯 What SweetGrass Does

SweetGrass is the **Attribution Layer** — the storyteller of data provenance:

```
┌─────────────────────────────────────────────────────────────────┐
│                        SweetGrass                                │
│                    (Attribution Layer)                           │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Braids    │  │  Queries    │  │     PROV-O Export      │  │
│  │ (provenance)│  │(attribution)│  │   (interoperability)   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Key Concepts**:
- **Braids** — cryptographically signed provenance records
- **Attribution chains** — who contributed what
- **PROV-O compatible** — W3C standard export
- **sunCloud integration** — powers economic distribution

---

## 📁 Project Structure

```
sweetGrass/
├── Cargo.toml                    # Workspace manifest
├── README.md                     # Project overview
├── STATUS.md                     # This file
├── WHATS_NEXT.md                # Roadmap
├── START_HERE.md                # Developer guide
├── crates/
│   └── sweet-grass-core/        # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs           # Main entry
│           ├── config.rs        # Configuration
│           └── error.rs         # Error types
├── specs/
│   └── SWEETGRASS_SPECIFICATION.md  # Full spec (~900 lines)
└── showcase/                     # Demo applications
```

---

## 🔗 Dependencies

### Gen 1 Primals (Required)
| Primal | Purpose | Status |
|--------|---------|--------|
| **BearDog** | Braid Signing | ✅ Ready |
| **Songbird** | Service Discovery | ✅ Ready |

### Phase 2 Siblings
| Primal | Relationship | Status |
|--------|--------------|--------|
| **RhizoCrypt** | Traverses DAG | 🌱 Scaffolded |
| **LoamSpine** | Listens to commits | 🌱 Scaffolded |

### Downstream
| System | Relationship |
|--------|--------------|
| **sunCloud** | Attribution queries for rewards |
| **gAIa** | Trust assessment |

---

## 📈 Metrics

```
Lines of Code:       ~100 (scaffold)
Test Coverage:       0% (no tests yet)
Unsafe Blocks:       0
Files:               3 source files
Dependencies:        sourdough-core
```

---

## 🚀 Next Milestone

**Phase 1: Braid Structure** (Target: Week 5-6)

1. Implement `Braid` struct (PROV-O compatible)
2. Implement `Attribution` struct
3. Add braid signing with BearDog
4. Basic provenance queries

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | Project overview |
| [START_HERE.md](./START_HERE.md) | Developer onboarding |
| [WHATS_NEXT.md](./WHATS_NEXT.md) | Implementation roadmap |
| [specs/SWEETGRASS_SPECIFICATION.md](./specs/SWEETGRASS_SPECIFICATION.md) | Full specification |

---

*SweetGrass: Every piece of data has a story.*

