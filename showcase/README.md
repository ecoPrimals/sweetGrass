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

### Level 0: Standalone
SweetGrass core features without external dependencies.

```bash
cd 00-standalone/
./01-braid-basics/demo-create-braid.sh      # Create Braids
./02-attribution-engine/demo-attribution.sh  # Calculate shares
./03-provenance-queries/demo-queries.sh      # Query graphs
./04-provo-export/demo-export.sh             # W3C PROV-O
./05-privacy-controls/demo-privacy.sh        # Privacy controls
```

### Level 1: Primal Coordination
Integration with individual phase1 primals.

```bash
cd 01-primal-coordination/

# BearDog - Cryptographic signing
./01-sweetgrass-beardog/demo-signed-braid-live.sh

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

| Integration | Demo | Status |
|-------------|------|--------|
| SweetGrass Core | `demo-create-braid.sh` | ✅ Live |
| Attribution Engine | `demo-attribution.sh` | ✅ Live |
| BearDog Signing | `demo-signed-braid-live.sh` | ✅ Live |
| NestGate Storage | `demo-storage-live.sh` | ✅ Live |
| Songbird Discovery | `demo-discovery-live.sh` | ✅ Live |
| ToadStool Compute | `demo-compute-provenance.sh` | ✅ Live |
| Full Ecosystem | `demo-full-pipeline-live.sh` | ✅ Live |

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
- [x] SweetGrass demo runs with correct attribution (50%/25%/25%)
- [x] Individual primal demos show capabilities
- [x] Full ecosystem demo integrates all primals
- [ ] Live RPC connections (requires running services)
- [ ] End-to-end signed + stored + computed pipeline

🌾 Every piece of data has a story!
