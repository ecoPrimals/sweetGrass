+++
title = "sweetGrass Validation Summary"
description = "Attribution primal — W3C PROV-O braids, provenance graphs, radiating attribution. 1,560 tests, 37 methods, zero production debt."
date = 2026-05-20

[taxonomies]
primals = ["sweetgrass", "beardog", "rhizocrypt", "loamspine", "nestgate"]
springs = []
+++

## Status

- **Version**: v0.7.38
- **1,560 tests** (all passed, 0 failed, 56 Docker CI integration tests)
- **37 registered capability methods** across 12 domains
- **194 source files** (55,496 LOC Rust), max 674 lines per file
- **Neural API `primal.announce`**: self-registers with biomeOS on startup (Wave 43)
- **Zero production debt**: 0 unsafe, 0 `#[allow]`, 0 TODO/FIXME, 0 `println!`, 0 production `unwrap()`, 0 `std::sync::Mutex`, 0 `Box<dyn Error>`, 0 `async_trait`, 0 `Rc<`, 0 missing SPDX
- **Clippy**: 0 warnings (pedantic + nursery)
- **`#![forbid(unsafe_code)]`** on all 11 crate roots
- **BTSP enforced** on TCP when `FAMILY_ID` set (v0.7.36+)
- **JH-0 Method Gate** adopted — permissive mode, `auth.check`/`auth.mode`/`auth.peer_info` registered
- **GAP-36 resolved** — 10 wire-name aliases for downstream compatibility
- **PID file** alongside UDS socket for instant liveness checks (v0.7.37)
- **Edition 2024**, resolver 3, Rust 1.87+

## Capability Domains

| Domain | Methods | Stability |
|--------|--------:|-----------|
| braid | 6 | Stable |
| anchoring | 2 | Stable |
| provenance | 3 | Stable |
| attribution | 4 | Stable |
| compression | 2 | Beta |
| contribution | 3 | Stable |
| health | 3 | Stable |
| identity | 1 | Stable |
| pipeline | 1 | Stable |
| composition | 4 | Beta |
| lifecycle | 1 | Stable |
| capabilities + tools | 4 | Stable |
| auth | 3 | Stable |

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
| Postgres | CI integration tests (Docker) |

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
