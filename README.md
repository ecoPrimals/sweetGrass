# SweetGrass

**Semantic Provenance and Attribution Layer for ecoPrimals**

v0.7.31 | 1,500 tests | 91.7% coverage | Edition 2024 | scyBorg Triple-Copyleft | Pure Rust | ecoBin compliant | BTSP Phase 3 | Wire L3 | BearDog crypto.sign delegation | ChaCha20-Poly1305 AEAD framing | HKDF-SHA256 session keys | Stadial parity (zero async-trait, zero dyn dispatch, sled eliminated, libsqlite3-sys eliminated, hostname eliminated, BTSP first-line auto-detect, PG-52 EOF-resilient UDS, whitespace-tolerant TCP autodetect, --http-port, PG-55 TCP bind control, PG-59 --http-address docs)

---

## What is SweetGrass?

SweetGrass is the semantic layer that makes ecoPrimals activity visible and queryable. It tracks:

- **Provenance**: What created this data, how, and when?
- **Attribution**: Who contributed, and what roles did they play?
- **Lineage**: Where did this data come from originally?
- **Rewards**: Fair distribution based on contributions

Standards: W3C PROV-O | JSON-RPC 2.0 | tarpc binary RPC | REST | Pure Rust | No vendor lock-in

---

## Quick Start

```bash
# Build the UniBin
cargo build --release

# Start the server (UDS-only by default)
./target/release/sweetgrass server

# Or with explicit TCP port (opt-in) and socket path override
./target/release/sweetgrass server --port 9100 --http-address 0.0.0.0:8080 --socket /tmp/sweetgrass.sock

# Health check via REST
curl http://localhost:8080/health

# JSON-RPC over HTTP
curl -X POST http://localhost:8080/jsonrpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}'

# JSON-RPC over TCP (newline-delimited)
echo '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":1}' | nc localhost 9100

# REST API
curl http://localhost:8080/api/v1/braids

# Offline commands
./target/release/sweetgrass capabilities  # List all capabilities
./target/release/sweetgrass socket        # Print UDS socket path
```

---

## Architecture

```
           Applications (gAIa, sunCloud)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       SWEETGRASS  (you are here)
          Provenance & Attribution
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                 SOIL LINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       RhizoCrypt (ephemeral network)
       LoamSpine (permanent record)
```

### 10 Crates

| Crate | Purpose |
|-------|---------|
| `sweet-grass-core` | Braid, Agent, Activity, Entity, Contribution, DehydrationSummary, Config, niche.rs self-knowledge |
| `sweet-grass-store` | BraidStore trait + MemoryStore |
| `sweet-grass-store-postgres` | PostgreSQL backend |
| `sweet-grass-store-redb` | Embedded Pure Rust backend (redb, recommended) |
| `sweet-grass-factory` | Braid creation + attribution engine |
| `sweet-grass-query` | Graph traversal, PROV-O export |
| `sweet-grass-compression` | 0/1/Many session compression |
| `sweet-grass-store-nestgate` | Delegated storage via NestGate JSON-RPC over UDS (feature-gated) |
| `sweet-grass-integration` | Primal discovery + capability clients |
| `sweet-grass-service` | UniBin server (REST + JSON-RPC + tarpc + UDS + BTSP) |

### Protocol Stack

| Protocol | Env Var | Latency | Use Case |
|----------|---------|---------|----------|
| tarpc | `SWEETGRASS_TARPC_ADDRESS` | ~50μs | Primal-to-primal binary RPC |
| TCP JSON-RPC | `SWEETGRASS_PORT` | ~1ms | Composition (`--port`, UniBin standard) |
| UDS JSON-RPC | `SWEETGRASS_SOCKET` | ~0.5ms | biomeOS IPC (XDG-compliant) |
| HTTP JSON-RPC | `SWEETGRASS_HTTP_ADDRESS` | ~10ms | 32 methods, batch, MCP tools |
| REST | `SWEETGRASS_HTTP_ADDRESS` | ~10ms | Debug, admin (`/api/v1/braids`) |

- **JSON-RPC 2.0**: 32 semantic methods (`braid.create`, `braid.commit`, `contribution.record`, `identity.get`, `capabilities.list`, `tools.list`, `tools.call`, `health.check`, `composition.tower_health`, etc.) with batch requests and notification support
- **MCP tool exposure**: `tools.list` + `tools.call` for Squirrel AI coordination
- **Capability-domain symlink**: `provenance.sock -> sweetgrass.sock` for Tier 3 filesystem discovery

### UniBin

Single binary with subcommands (`sweetgrass server`, `sweetgrass status`, `sweetgrass capabilities`, `sweetgrass socket`), graceful shutdown, runtime backend selection. The `--port` flag binds a newline-delimited TCP JSON-RPC listener per UniBin standard v1.1. The `capabilities` subcommand dumps capability metadata offline; `socket` prints the resolved UDS path.

---

## Features

### Provenance Tracking
- Full W3C PROV-O compliance (JSON-LD)
- Activity, Agent, Entity model
- Derivation chains and dependencies
- Content-addressed braids (URN format)

### scyBorg Types
- **ContentCategory**, **LicenseId**, **LicenseExpression**, **AttributionNotice** — License and attribution metadata types

### Attribution and Rewards
- 12 configurable agent roles with weights
- Time-decay models
- Recursive derivation chain propagation
- sunCloud integration ready

### Error Types
- **`CapabilityProvider { capability, message }`** — Ecosystem-consistent capability provider error variant
- **`ServiceError::Transport` and `ServiceError::Discovery`** — IPC error variants for trio partner communication

### Cryptographic Provenance
- **Tower-delegated Ed25519 signing** — `braid.create` and `anchoring.anchor` delegate to BearDog `crypto.sign` over UDS JSON-RPC
- **`Witness::from_tower_ed25519`** — Tower-tier witnesses (`tier: "tower"`) distinguish BearDog-signed from local
- **Graceful degradation** — unsigned witnesses when BearDog is unavailable

### Resilience
- **CircuitBreaker + RetryPolicy** — `with_resilience()` async helper for trio partner IPC

### Storage Flexibility
- **Memory**: Testing and development
- **PostgreSQL**: Production scale with migrations
- **redb**: Embedded Pure Rust, ACID transactions, actively maintained (recommended)
- **NestGate**: Ecosystem-delegated storage via JSON-RPC over UDS (feature-gated, `--features nestgate`)
- Runtime selection via environment

### Privacy and Consent
- GDPR-inspired 5-level privacy controls
- Data subject rights (access, erasure, portability)
- Retention policies
- Selective disclosure

---

## Building

### Prerequisites
- Rust 1.87+ (stable, Edition 2024)
- Docker (optional, for PostgreSQL)

### From Source

```bash
cargo build --release
```

### Testing

```bash
# All tests
cargo test --workspace --all-features

# Pre-commit checks
./scripts/check.sh

# Coverage
cargo llvm-cov --workspace
```

### Configuration

```bash
STORAGE_BACKEND=redb                     # or: memory, postgres
DATABASE_URL=postgresql://...            # for postgres backend
SWEETGRASS_HTTP_ADDRESS=0.0.0.0:8080    # REST + HTTP JSON-RPC endpoint
SWEETGRASS_PORT=9100                     # TCP JSON-RPC (UniBin --port)
SWEETGRASS_TARPC_ADDRESS=0.0.0.0:8091   # Binary RPC endpoint
SWEETGRASS_SOCKET=/run/user/1000/biomeos/sweetgrass.sock  # UDS JSON-RPC
```

See [DEVELOPMENT.md](./DEVELOPMENT.md) for all options.

---

## Documentation

| Doc | Purpose |
|-----|---------|
| [DEVELOPMENT.md](./DEVELOPMENT.md) | Dev setup, testing, code standards |
| [QUICK_COMMANDS.md](./QUICK_COMMANDS.md) | Command reference |
| [ROADMAP.md](./ROADMAP.md) | Future plans |
| [CHANGELOG.md](./CHANGELOG.md) | Version history |
| [specs/](./specs/) | Technical specifications (11 docs including Content Convergence) |
| [docs/guides/](./docs/guides/) | Zero-copy, Tokio Console guides |
| [showcase/](./showcase/) | Interactive demos |

---

## Quality

| Metric | Value |
|--------|-------|
| Version | v0.7.30 |
| Tests | 1,495 local + 56 Docker CI |
| Coverage | 90%+ line (91.7% with Postgres Docker) |
| Edition | 2024 (MSRV 1.87) |
| Unsafe code | 0 (`#![forbid(unsafe_code)]` workspace-level + all crate roots) |
| Production unwraps | 0 (`unwrap_used`/`expect_used` = `deny`) |
| Clippy | 0 warnings (pedantic + nursery, `-D warnings`) |
| Max file size | 763 lines (limit: 1000) |
| .rs files | 199 (55,960 LOC) |
| TODOs in source | 0 |
| SPDX + copyright | All .rs files |
| License | scyBorg Triple-Copyleft (AGPL-3.0-or-later + ORC-1.0 + CC-BY-SA-4.0) |
| cargo deny | advisories ok, bans ok, licenses ok, sources ok |
| Benchmarks | 7 criterion groups |
| JSON-RPC methods | 32 (batch + notification + MCP tool exposure + Wire Standard L3 + composition health) |
| Property-based tests | proptest (25 strategies across 7 crates) |
| Chaos/fault tests | 11 attribution chaos + 17 service chaos + 9 fault injection |
| BTSP | Phase 3 — `btsp.negotiate` + ChaCha20-Poly1305 AEAD framing; `detect_protocol` three-way multiplexer (EOF-resilient, PG-52) when `FAMILY_ID` set |

### ecoBin Compliance

- Pure Rust (zero C/C++ dependencies in production)
- musl-static builds verified (4.5 MB stripped, plasmidBin / benchScale ready)
- Platform-agnostic IPC (JSON-RPC + tarpc + UDS, no gRPC/protobuf)
- `cargo-deny` enforced (tonic, prost, openssl banned)

### Synchronization

`parking_lot::RwLock` throughout (Pure Rust, no poisoning, better perf than `std`).

### Zero-Copy

`ContentHash`, `BraidId`, `Did`, `ActivityId`, `Braid.mime_type`, `BraidMetadata.title`, `BraidMetadata.description`, `BraidMetadata.tags`, `EcoPrimalsAttributes.source_primal`, `EcoPrimalsAttributes.niche`, `LedgerCommitRef.spine_id`, `BraidFactory.source_primal`, and `CompressionEngine.source_primal` use `Arc<str>` internally — `.clone()` is O(1) atomic refcount increment. MIME type and tag indexes (`MemoryStore`, `AgentContributions`) share the same `Arc<str>`, eliminating per-query allocations on hot paths. `Witness` constructors use named `&'static str` constants (`WITNESS_KIND_SIGNATURE`, `WITNESS_ENCODING_BASE64`, etc.) for the `WireWitnessRef`-aligned provenance vocabulary. `BraidContext.imports` uses `IndexMap` for deterministic serialization. `DEFAULT_MAX_PROVENANCE_DEPTH` is a single shared constant used by all graph traversal and attribution components.

### Configuration

TOML config file support with full hierarchy: CLI args > env vars > config file > defaults. XDG-compliant config search (`$SWEETGRASS_CONFIG`, `$XDG_CONFIG_HOME/sweetgrass/config.toml`, `~/.config/sweetgrass/config.toml`).

---

## License

scyBorg Triple-Copyleft: AGPL-3.0-or-later (software), ORC-1.0 (game mechanics), CC-BY-SA-4.0 (creative content/documentation). See [LICENSE](./LICENSE).

---

## Part of ecoPrimals

This repo is part of the [ecoPrimals](https://github.com/ecoPrimals) sovereign
computing ecosystem — a collection of pure Rust binaries that coordinate via
JSON-RPC, capability-based routing, and zero compile-time coupling.

See [wateringHole](https://github.com/ecoPrimals/wateringHole) for ecosystem
documentation, standards, and the primal registry.

---

**Fair attribution. Complete transparency. Human dignity preserved.**
