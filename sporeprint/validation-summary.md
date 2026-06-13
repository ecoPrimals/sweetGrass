+++
title = "sweetGrass Validation Summary"
description = "Attribution primal — W3C PROV-O braids, provenance graphs, radiating attribution. 1,647 tests, 40 methods, zero production debt. riboCipher reference impl, BTSP E2E ready, HEALTH-01, zero bare env vars."
date = 2026-06-13

[taxonomies]
primals = ["sweetgrass", "beardog", "rhizocrypt", "loamspine", "nestgate"]
springs = []
+++

## Status

- **Version**: v0.7.57
- **1,647 tests** (all passed, 0 failed, 56 Docker CI integration tests)
- **40 registered capability methods** across 13 domains
- **210+ source files**, max 796 lines per file
- **riboCipher**: Reference implementation in `peek.rs` — signal detection for `0xEC`/`0xED`/`0xEE` before legacy peek (Wave 111, Stream 7)
- **HEALTH-01**: Bare `health` method alias, enriched `health.check` with `primal` + `uptime_secs`
- **BTSP E2E ready**: `BEARDOG_SOCKET` in security socket resolution tier 2
- **Zero bare env var strings**: All `env::var()` calls use `primal_names::env_vars` constants
- **Zero production debt**: 0 unsafe, 0 `#[allow]`, 0 TODO/FIXME, 0 `println!`, 0 production `unwrap()`, 0 `std::sync::Mutex`, 0 `Box<dyn Error>`, 0 `async_trait`, 0 `Rc<`, 0 missing SPDX
- **Clippy**: 0 warnings (pedantic + nursery)
- **`#![forbid(unsafe_code)]`** on all 10 crate roots
- **BTSP enforced** on TCP when `FAMILY_ID` set (v0.7.36+)
- **JH-0 Method Gate** adopted — permissive mode, `auth.check`/`auth.mode`/`auth.peer_info` registered
- **GAP-36 resolved** — 11 wire-name aliases for downstream compatibility
- **PID file** alongside UDS socket for instant liveness checks (v0.7.37)
- **Edition 2024**, resolver 3, Rust 1.87+

## Capability Domains

| Domain | Methods | Stability |
|--------|--------:|-----------|
| braid | 7 | Stable |
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
