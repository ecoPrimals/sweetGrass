+++
title = "sweetGrass Validation Summary"
description = "Attribution primal — W3C PROV-O braids, provenance graphs, radiating attribution. 1,746 tests, 50 methods, DH-0 clean. Pure Rust. G72 Tier 1 (155 transitive deps). rootPulse step handlers. 90% coverage. All files ≤710L. Zero TODO/unsafe/hardcoded names."
date = 2026-08-14

[taxonomies]
primals = ["sweetgrass", "beardog", "rhizocrypt", "loamspine", "nestgate"]
springs = []
+++

## Status

- **Version**: v0.8.0
- **1,746 tests** (all passed, 0 failed, pure Rust — no Docker required)
- **90%+ coverage** (line 89.62%, branch 90.70% via llvm-cov)
- **DH-0 clean**: Zero debt, zero unsafe, zero hardcoded primal names, zero dead_code
- **rootPulse step handlers**: `rootpulse.attribute` + `rootpulse.query` for trio pipeline (P2 #10)
- **G72 Tier 1 complete**: tokio trimmed from "full", dead deps excised, 155 unique transitive crates
- **braid.verify atomic**: Content integrity + Ed25519 signature + ledger confirmation in one call
- **G65 protocol negotiation**: Single-socket tarpc+jsonrpc (replaces dual-socket C2)
- **G66 transport abstraction**: Silicon-agnostic IPC (TransportEndpoint/Stream/Listener)
- **G68 platform substrate**: Cross-platform link creation, no raw Unix syscalls
- **Neural API routing**: `capability.call` handler for biomeOS dispatch
- **G31 batch pipeline**: `braid.batch_create` + `braid.batch_commit` for 10x throughput
- **Provenance Trio G3 WIRED**: `LedgerClient` closes sweetGrass→loamSpine triangle
- **50 registered capability methods** across 15 domains
- **All production files ≤710 lines** (3 large files smartly refactored this wave)
- **Zero production debt**: 0 unsafe, 0 TODO/FIXME, 0 `unwrap()`, 0 clippy warnings
- **`#![forbid(unsafe_code)]`** on all 9 crate roots
- **BTSP enforced** on TCP when `FAMILY_ID` set (v0.7.36+)
- **JH-0 Method Gate** adopted — permissive mode, `auth.check`/`auth.mode`/`auth.peer_info` registered
- **GAP-36 resolved** — 11 wire-name aliases for downstream compatibility
- **PID file** alongside UDS socket for instant liveness checks (v0.7.37)
- **Edition 2024**, resolver 3, Rust 1.87+

## Capability Domains

| Domain | Methods | Stability |
|--------|--------:|-----------|
| braid | 11 | Stable |
| anchoring | 2 | Stable |
| provenance | 3 | Stable |
| attribution | 4 | Stable |
| compression | 2 | Beta |
| contribution | 4 | Stable |
| health | 3 | Stable |
| identity | 1 | Stable |
| pipeline | 1 | Stable |
| composition | 4 | Beta |
| lifecycle | 1 | Stable |
| capabilities + tools | 4 | Stable |
| auth | 3 | Stable |
| rootpulse | 2 | Beta |
| trust | 1 | Beta |

## Key Binaries

- `sweetgrass server` — JSON-RPC 2.0 over UDS + TCP, BTSP auto-detect
- `sweetgrass status` — runtime health probe
- `sweetgrass capabilities` — offline capability metadata dump
- `sweetgrass socket` — resolved UDS path

## Architecture

sweetGrass is the attribution/provenance node of the **provenance trio**
(rhizoCrypt → loamSpine → sweetGrass). It creates, queries, and commits
semantic braids — W3C PROV-O provenance records with content-addressed
hashes, cryptographic witnesses, and radiating attribution.

### Provenance Trio Position

```
rhizoCrypt (DAG sessions) → loamSpine (ledger commits) → sweetGrass (braids)
```

### Tower Integration

- **BearDog**: `crypto.sign` Ed25519 delegation for `braid.create` and `anchoring.anchor`
- **BearDog**: `crypto.sha256` hash delegation for `braid.compute_signing_hash`
- **NestGate**: Artifact storage via `storage.artifact.store`/`get`

### Transport

| Surface | Protocol | Default |
|---------|----------|---------|
| UDS | Newline-delimited JSON-RPC 2.0 | Always on |
| TCP | BTSP handshake + length-prefixed framing | Opt-in (`--port`) |
| HTTP | Axum REST + JSON-RPC | `--http-address` |
| tarpc | Binary RPC | `--tarpc-address` |

## Storage Backends

| Backend | Use Case |
|---------|----------|
| Memory | Development, tests |
| redb | Local persistent (single-node) |
| NestGate | Distributed (IPC to nestGate primal) |

## Downstream Dependents

| Consumer | What They Use |
|----------|--------------|
| wetSpring | `braid.create` for ferment transcript provenance |
| lithoSpore | Braid verification artifacts |
| projectFOUNDATION | Attribution chain for thread evidence |
| primalSpring | Composition validation, trio pipeline |
| esotericWebb | Attribution metadata for game assets |

## Workload TOMLs

Not yet created — contribute to `projectNUCLEUS/workloads/sweetgrass/`.

## See Also

- [Provenance Trio Integration Guide](https://github.com/ecoPrimals/wateringHole/blob/main/PROVENANCE_TRIO_INTEGRATION_GUIDE.md)
- [sweetGrass Specification](specs/SWEETGRASS_SPECIFICATION.md)
- [Capability Registry](config/capability_registry.toml)
