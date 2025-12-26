# 🌾 SweetGrass — Documentation Index

**Version**: v0.5.0-evolution  
**Last Updated**: December 26, 2025  
**Status**: ✅ Production Ready (A+ Grade)

---

## 🚀 Quick Start

**New to SweetGrass?** Start here:
1. **[START_HERE.md](./START_HERE.md)** — Getting started guide
2. **[README.md](./README.md)** — Project overview
3. **[showcase/00-standalone/RUN_ME_FIRST.sh](./showcase/00-standalone/RUN_ME_FIRST.sh)** — Interactive demos

---

## 📚 Root Documentation

### Essential Operational Docs
| Document | Purpose |
|----------|---------|
| **[README.md](./README.md)** | Project overview and quick start |
| **[START_HERE.md](./START_HERE.md)** | Getting started guide for new developers |
| **[STATUS.md](./STATUS.md)** | Current build status and metrics |
| **[ROADMAP.md](./ROADMAP.md)** | Future development plans |
| **[CHANGELOG.md](./CHANGELOG.md)** | Version history and changes |
| **[ROOT_DOCS_INDEX.md](./ROOT_DOCS_INDEX.md)** | Comprehensive navigation guide |
| **[DOCUMENTATION_INDEX.md](./DOCUMENTATION_INDEX.md)** | This file |

### Configuration Files
| File | Purpose |
|------|---------|
| **[env.example](./env.example)** | Environment variable examples |
| **[deny.toml](./deny.toml)** | Dependency audit configuration |
| **[rustfmt.toml](./rustfmt.toml)** | Code formatting rules |

---

## 📖 Specifications (/specs/)

**Index**: [specs/00_SPECIFICATIONS_INDEX.md](./specs/00_SPECIFICATIONS_INDEX.md)

### Core Specifications
- **[PRIMAL_SOVEREIGNTY.md](./specs/PRIMAL_SOVEREIGNTY.md)** — Pure Rust principles, no gRPC
- **[SWEETGRASS_SPECIFICATION.md](./specs/SWEETGRASS_SPECIFICATION.md)** — Master specification
- **[ARCHITECTURE.md](./specs/ARCHITECTURE.md)** — System architecture and components
- **[DATA_MODEL.md](./specs/DATA_MODEL.md)** — Braid, Activity, Agent, Entity structures

### Integration Specifications
- **[API_SPECIFICATION.md](./specs/API_SPECIFICATION.md)** — tarpc, JSON-RPC, REST APIs
- **[INTEGRATION_SPECIFICATION.md](./specs/INTEGRATION_SPECIFICATION.md)** — Primal integrations
- **[BRAID_COMPRESSION.md](./specs/BRAID_COMPRESSION.md)** — 0/1/Many compression model
- **[ATTRIBUTION_GRAPH.md](./specs/ATTRIBUTION_GRAPH.md)** — Provenance for sunCloud
- **[NICHE_PATTERNS.md](./specs/NICHE_PATTERNS.md)** — Configurable semantic patterns

---

## 🎭 Showcase (/showcase/)

**Index**: [showcase/00_SHOWCASE_INDEX.md](./showcase/00_SHOWCASE_INDEX.md)

### Interactive Demonstrations (44 scripts)

**Level 0: Standalone** (no dependencies)
- [00-standalone/](./showcase/00-standalone/) — 5 demos showing core features
- [00-local-primal/](./showcase/00-local-primal/) — 7 demos with local primal

**Level 1: Primal Coordination**
- [01-primal-coordination/](./showcase/01-primal-coordination/) — 10+ multi-primal demos

**Level 2: Full Ecosystem**
- [02-full-ecosystem/](./showcase/02-full-ecosystem/) — Complete pipeline demos

**Level 3: Real World**
- [03-real-world/](./showcase/03-real-world/) — Production use cases

---

## 📊 Evolution Reports (/reports/)

**Index**: [reports/README.md](./reports/README.md)

### December 26, 2025 — Code Evolution & Audit
**Location**: [reports/dec-26-evolution/](./reports/dec-26-evolution/)

**Key Documents**:
- **COMPREHENSIVE_AUDIT_DEC_25_2025.md** (708 lines) — Full codebase audit
- **EVOLUTION_COMPLETE_DEC_26_2025.md** — Evolution summary
- **FINAL_STATUS_DEC_26_2025.md** — Final production status
- **SESSION_COMPLETE_DEC_26_2025.md** — Session summary
- **DEPLOYMENT_CHECKLIST_DEC_26_2025.md** — Deployment guide

**Summary**:
- 5 critical issues resolved
- Coverage verified: 78.39%
- Grade: A (91/100) → A+ (94/100)
- Status: Production Ready

### December 25, 2025 — Infant Discovery Evolution
**Location**: [reports/dec-25-evolution/](./reports/dec-25-evolution/)

**Key Documents**:
- **HARDCODING_EVOLUTION_PLAN.md** — Strategy and patterns
- **HARDCODING_EVOLUTION_COMPLETE.md** — Final summary
- **HARDCODING_FIXES_COMPLETED_DEC_25_2025.md** — Execution report

**Summary**:
- Zero hardcoding achieved
- 100% Infant Discovery compliance
- SelfKnowledge pattern established

---

## 💻 Source Code (/crates/)

### Core Crates
- **sweet-grass-core** — Core data structures (Braid, Activity, Agent, Entity)
- **sweet-grass-factory** — Braid creation and attribution
- **sweet-grass-query** — Query engine and PROV-O export
- **sweet-grass-compression** — Session compression (0/1/Many)

### Storage Crates
- **sweet-grass-store** — Storage trait and memory backend
- **sweet-grass-store-postgres** — PostgreSQL backend
- **sweet-grass-store-sled** — Sled backend (pure Rust, embedded)

### Integration Crates
- **sweet-grass-integration** — Primal integration (discovery, signing, etc.)
- **sweet-grass-service** — REST + tarpc service binary

---

## 🧪 Testing (/fuzz/, /tests/)

### Fuzz Testing
**Location**: [fuzz/](./fuzz/)

**Targets**:
- `fuzz_attribution.rs` — Attribution calculation
- `fuzz_braid_deserialize.rs` — Braid deserialization
- `fuzz_query_filter.rs` — Query filter parsing

### Test Coverage
- **489 tests** (100% passing)
- **78.39%** line coverage (verified with llvm-cov)
- **78.84%** function coverage
- **88.74%** region coverage

---

## 🎯 Documentation by Audience

### For New Developers
1. [START_HERE.md](./START_HERE.md) — Getting started
2. [specs/PRIMAL_SOVEREIGNTY.md](./specs/PRIMAL_SOVEREIGNTY.md) — Core principles
3. [showcase/00-standalone/](./showcase/00-standalone/) — Interactive demos
4. [specs/ARCHITECTURE.md](./specs/ARCHITECTURE.md) — System design

### For DevOps/SRE
1. [reports/dec-26-evolution/DEPLOYMENT_CHECKLIST_DEC_26_2025.md](./reports/dec-26-evolution/DEPLOYMENT_CHECKLIST_DEC_26_2025.md) — Deployment guide
2. [STATUS.md](./STATUS.md) — Build status
3. [env.example](./env.example) — Configuration
4. [specs/API_SPECIFICATION.md](./specs/API_SPECIFICATION.md) — API reference

### For Architects
1. [specs/ARCHITECTURE.md](./specs/ARCHITECTURE.md) — System architecture
2. [reports/dec-26-evolution/COMPREHENSIVE_AUDIT_DEC_25_2025.md](./reports/dec-26-evolution/COMPREHENSIVE_AUDIT_DEC_25_2025.md) — Full audit
3. [specs/INTEGRATION_SPECIFICATION.md](./specs/INTEGRATION_SPECIFICATION.md) — Integration patterns
4. [specs/DATA_MODEL.md](./specs/DATA_MODEL.md) — Data structures

### For Product Managers
1. [README.md](./README.md) — Overview
2. [ROADMAP.md](./ROADMAP.md) — Future plans
3. [showcase/03-real-world/](./showcase/03-real-world/) — Use cases
4. [reports/dec-26-evolution/FINAL_STATUS_DEC_26_2025.md](./reports/dec-26-evolution/FINAL_STATUS_DEC_26_2025.md) — Current status

---

## 📈 Current Status

```
Version:          v0.5.0-evolution
Grade:            A+ (94/100)
Tests:            489/489 passing
Coverage:         78.39% line (verified)
Unsafe Code:      0 blocks
Hardcoding:       0 violations
Status:           ✅ PRODUCTION READY
```

---

## 🔗 Related Resources

### Phase1 Primals
- **BearDog** — Identity & Signing (`../../phase1/bearDog/`)
- **NestGate** — Storage (`../../phase1/nestGate/`)
- **Songbird** — Discovery (binaries in `../bins/`)

### External Standards
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — Provenance Ontology
- [JSON-LD](https://json-ld.org/) — Linked Data
- [W3C DIDs](https://www.w3.org/TR/did-core/) — Decentralized Identifiers

---

## 📝 Documentation Standards

### Writing Guidelines
- Use markdown format
- Include front matter (version, date, status)
- Keep under 1000 lines where possible
- Use clear headings (H1-H4)
- Include code examples where helpful
- Add emojis sparingly for visual navigation

### File Organization
- Root: Essential operational docs only
- `/specs/`: Technical specifications
- `/showcase/`: Interactive demonstrations
- `/reports/`: Evolution and audit reports
- `/crates/`: Source code with inline docs

### Maintenance
- Update dates when modifying
- Keep indexes current
- Archive old reports to `/reports/archive/`
- Update STATUS.md with each release

---

## 🏆 Documentation Achievements

- ✅ 10 comprehensive specifications
- ✅ 44 working showcase scripts
- ✅ 6 evolution reports (2,000+ lines)
- ✅ Complete API documentation
- ✅ Inline code documentation
- ✅ Multiple navigation indexes

---

**Last Updated**: December 26, 2025  
**Maintained By**: SweetGrass Team  
**Status**: ✅ Current and Complete

🌾 **Each primal knows only itself. Network effects through universal adapter.** 🌾
