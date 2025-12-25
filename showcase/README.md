# 🌾 SweetGrass Showcase

Interactive demonstrations of SweetGrass attribution and provenance capabilities,
with live integration to phase1 primals.

## Quick Start

```bash
# Full ecosystem demo (all primals)
./02-full-ecosystem/01-complete-pipeline/demo-full-pipeline-live.sh

# Or the 5-minute overview
./scripts/quick-demo.sh
```

## Phase 1 Primal Binaries

All phase1 primal binaries are available in `../../bins/`:

| Binary | Primal | Capability |
|--------|--------|------------|
| `beardog` | BearDog | Security, signing, HSM |
| `nestgate` | NestGate | Storage, ZFS, backup |
| `songbird-cli` | Songbird | Discovery, mesh, federation |
| `toadstool-cli` | ToadStool | Compute, runtime, WASM |
| `squirrel` | Squirrel | AI, MCP, inference |

## Progressive Levels

### Level 0: Local Primal
**Philosophy**: "SweetGrass BY ITSELF is Amazing"

SweetGrass core features without external dependencies. Start here!

```bash
cd 00-local-primal/

# Automated tour (recommended) - ~50 minutes
./RUN_ME_FIRST.sh

# Or run individual levels:
./01-hello-provenance/demo-first-braid.sh      # Create your first Braid
./02-attribution-basics/demo-fair-credit.sh    # Calculate fair shares
./03-query-engine/demo-filters.sh              # Query provenance graphs
./04-prov-o-standard/demo-prov-o-export.sh     # W3C PROV-O export
./05-privacy-controls/demo-privacy.sh          # GDPR-inspired privacy
./06-storage-backends/demo-backends.sh         # Memory/Sled/Postgres
./07-real-verification/demo-no-mocks.sh        # Real execution validation
```

**Time**: ~50 minutes for full tour  
**Mocks**: Zero (all real execution)  
**Pattern**: Inspired by NestGate's local-first approach

### Level 1: Primal Coordination
**Philosophy**: "Interactions show us gaps in our evolution"

Integration with individual phase1 primals using **real binaries** (no mocks).

```bash
cd 01-primal-coordination/

# BearDog - Cryptographic signing
./01-sweetgrass-beardog/demo-signed-braid-live.sh

# NestGate - Storage with ZFS integrity
./02-sweetgrass-nestgate/demo-storage-integration-test.sh

# Songbird - Capability-based discovery
./04-sweetgrass-songbird/demo-discovery-integration-test.sh

# ToadStool - Compute provenance
./05-sweetgrass-toadstool/demo-compute-integration-test.sh

# Squirrel - AI attribution (revolutionary!)
./06-sweetgrass-squirrel/demo-ai-attribution-test.sh

# NestGate - Sovereign storage
./02-sweetgrass-nestgate/demo-storage-live.sh

# RhizoCrypt - Session compression (phase2)
./02-sweetgrass-rhizocrypt/demo-session-compression.sh

# LoamSpine - Immutable anchoring (phase2)
./03-sweetgrass-loamspine/demo-anchor.sh

# Songbird - Discovery mesh
./04-sweetgrass-songbird/demo-discovery-live.sh
```

### Level 2: Full Ecosystem
Complete multi-primal workflows.

```bash
cd 02-full-ecosystem/

# All primals working together
./01-complete-pipeline/demo-full-pipeline-live.sh

# Cross-primal provenance
./02-multi-primal-provenance/demo-cross-primal.sh

# ToadStool compute integration
./03-toadstool-compute/demo-compute-provenance.sh
```

## Live Integration Status

| Integration | Demo | Status | Tests |
|-------------|------|--------|-------|
| **Local Primal** |
| Hello Provenance | `01-hello-provenance/demo-first-braid.sh` | ✅ Live | Real service |
| Fair Credit | `02-attribution-basics/demo-fair-credit.sh` | ✅ Live | Real attribution |
| Query Engine | `03-query-engine/demo-filters.sh` | ✅ Live | Real queries |
| PROV-O Export | `04-prov-o-standard/demo-prov-o-export.sh` | ✅ Live | W3C standard |
| Privacy Controls | `05-privacy-controls/demo-privacy.sh` | ✅ Live | GDPR rights |
| Storage Backends | `06-storage-backends/demo-backends.sh` | ✅ Live | Memory/Sled/PG |
| Real Verification | `07-real-verification/demo-no-mocks.sh` | ✅ Live | 10-pt checklist |
| **Primal Coordination** |
| BearDog Signing | `demo-signed-braid-live.sh` | ✅ Live | Real binary |
| NestGate Storage | `demo-storage-integration-test.sh` | ✅ Tested | 3/5 (60%) |
| Songbird Discovery | `demo-discovery-integration-test.sh` | ✅ Tested | 5/6 (83%) |
| ToadStool Compute | `demo-compute-integration-test.sh` | ✅ Tested | 4/5 (80%) |
| Squirrel AI | `demo-ai-attribution-test.sh` | ✅ Tested | 4/6 (66%) |
| **Full Ecosystem** |
| Complete Pipeline | `demo-full-pipeline-live.sh` | ✅ Live | Multi-primal |

**Overall**: 16/22 integration tests passed (73%)  
**Mocks Used**: 0 (ZERO - all real binaries!)  
**Gaps Discovered**: 4 (documented, not hidden)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ecoPrimals Ecosystem                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│     ┌──────────┐    ┌──────────┐    ┌──────────┐           │
│     │ BearDog  │    │ NestGate │    │ ToadStool│           │
│     │ Security │    │ Storage  │    │ Compute  │           │
│     └────┬─────┘    └────┬─────┘    └────┬─────┘           │
│          │               │               │                  │
│          └───────────────┼───────────────┘                  │
│                          │                                  │
│                   ┌──────┴──────┐                           │
│                   │  Songbird   │                           │
│                   │   Mesh      │                           │
│                   └──────┬──────┘                           │
│                          │                                  │
│                   ┌──────┴──────┐                           │
│                   │ SweetGrass  │                           │
│                   │ Provenance  │                           │
│                   └─────────────┘                           │
│                                                             │
│  Phase 2 Primals:                                           │
│  • RhizoCrypt (Session Compression)                         │
│  • LoamSpine (Immutable Anchoring)                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Starting Services

To enable full live integration, start the primal services:

```bash
# BearDog (signing) - needs HSM
../bins/beardog hsm discover

# NestGate (storage)
../bins/nestgate service start --port 8093

# Songbird (discovery mesh)
../bins/songbird-cli tower start

# ToadStool (compute)
../bins/toadstool-cli up
```

## Success Criteria

- [x] All 5 phase1 binaries available
- [x] SweetGrass demo runs with correct attribution
- [x] Individual primal demos show capabilities
- [x] **NEW**: Privacy controls demo (GDPR-inspired)
- [x] **NEW**: Storage backends demo (Memory/Sled/Postgres)
- [x] **NEW**: Real verification checklist (no mocks)
- [x] **NEW**: NestGate integration tested (real binary)
- [x] **NEW**: Songbird integration tested (real binary)
- [x] **NEW**: ToadStool integration tested (real binary)
- [x] **NEW**: Squirrel integration tested (AI attribution!)
- [x] Live RPC connections tested (real services)
- [x] End-to-end provenance tracking validated
- [x] Zero mocks principle enforced (100%)
- [x] Gaps discovered and documented (not hidden)

## Principles Validated

✅ **"Interactions show us gaps in our evolution"**
- 4 real gaps discovered through testing
- All documented, none hidden by mocks
- Used to improve integration design

✅ **"No mocks in showcase"**
- 0 mocked services
- 0 fake responses
- 100% real binary execution

✅ **"Primal sovereignty"**
- Capability-based discovery
- No hardcoded primal names
- Runtime configuration

✅ **"Fair attribution for AI"**
- Complete AI provenance chain
- Data providers get credit
- ML engineers get credit
- AI models get credit
- Revolutionary ethical AI! 🌾🐿️

---

🌾 **Every piece of data has a story - and now AI gets fair credit too!**
