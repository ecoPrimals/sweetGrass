# SweetGrass

**Semantic Provenance and Attribution Layer for ecoPrimals**

v0.7.12 | 903 tests | Edition 2024 | AGPL-3.0-only | Pure Rust | ecoBin compliant

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

# Start the server
./target/release/sweetgrass server

# Health check
curl http://localhost:8080/health

# Create a braid via JSON-RPC
curl -X POST http://localhost:8080/jsonrpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}'

# Or use REST
curl http://localhost:8080/api/v1/braids
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
| `sweet-grass-core` | Braid, Agent, Activity, Entity, Contribution, DehydrationSummary, Config |
| `sweet-grass-store` | BraidStore trait + MemoryStore |
| `sweet-grass-store-postgres` | PostgreSQL backend |
| `sweet-grass-store-redb` | Embedded Pure Rust backend (redb, recommended) |
| `sweet-grass-store-sled` | Embedded Pure Rust backend (sled, legacy) |
| `sweet-grass-factory` | Braid creation + attribution engine |
| `sweet-grass-query` | Graph traversal, PROV-O export |
| `sweet-grass-compression` | 0/1/Many session compression |
| `sweet-grass-integration` | Primal discovery + capability clients |
| `sweet-grass-service` | UniBin server (REST + JSON-RPC + tarpc + UDS) |

### Protocol Stack

- **JSON-RPC 2.0** (primary): `POST /jsonrpc` with 21 semantic methods, batch requests, and notification support (`braid.create`, `braid.commit`, `contribution.record`, `capability.list`, `health.check`, etc.)
- **Unix domain socket** (biomeOS IPC): Newline-delimited JSON-RPC 2.0 over UDS with XDG-compliant path resolution
- **tarpc** (high-performance binary): Pure Rust RPC, no gRPC/protobuf
- **REST** (HTTP/JSON): `/api/v1/braids` for debugging and admin

### UniBin

Single binary with subcommands (`sweetgrass server`, `sweetgrass status`), graceful shutdown, runtime backend selection.

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

### Storage Flexibility
- **Memory**: Testing and development
- **PostgreSQL**: Production scale with migrations
- **redb**: Embedded Pure Rust, ACID transactions, actively maintained (recommended)
- **Sled**: Embedded Pure Rust, legacy (feature-gated, `--features sled`)
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
cargo test --workspace

# Pre-commit checks
./scripts/check.sh

# Coverage
cargo llvm-cov --workspace
```

### Configuration

```bash
STORAGE_BACKEND=redb          # or: memory, postgres, sled (with --features sled)
DATABASE_URL=postgresql://... # for postgres backend
HTTP_LISTEN=0.0.0.0:8080
TARPC_LISTEN=0.0.0.0:8091
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
| [specs/](./specs/) | Technical specifications |
| [docs/guides/](./docs/guides/) | Zero-copy, Tokio Console guides |
| [showcase/](./showcase/) | Interactive demos |

---

## Quality

| Metric | Value |
|--------|-------|
| Version | v0.7.12 |
| Tests | 903 passing |
| Edition | 2024 (MSRV 1.87) |
| Unsafe code | 0 (`#![forbid(unsafe_code)]` all crates) |
| Production unwraps | 0 |
| Clippy | 0 warnings (pedantic + nursery + `missing_errors_doc` + `missing_const_for_fn`, `-D warnings`) |
| Max file size | 808 lines (limit: 1000) |
| TODOs in source | 0 |
| SPDX + copyright | All 112 .rs files |
| License | AGPL-3.0-only |
| Benchmarks | 7 criterion groups |
| JSON-RPC methods | 21 (batch + notification support) |
| Property-based tests | proptest (11 strategies) |
| Chaos/fault tests | 11 attribution + 17 service scenarios |

### ecoBin Compliance

- Pure Rust (zero C/C++ dependencies in production)
- Cross-compilation ready (ARM64, musl, RISC-V targets documented)
- Platform-agnostic IPC (JSON-RPC + tarpc + UDS, no gRPC/protobuf)
- `cargo-deny` enforced (tonic, prost, openssl banned)

### Synchronization

`parking_lot::RwLock` throughout (Pure Rust, no poisoning, better perf than `std`).

### Zero-Copy

`ContentHash`, `BraidId`, `Did`, and `ActivityId` use `Arc<str>` internally — `.clone()` is O(1) atomic refcount increment. `BraidSignature` fields use `Cow<'static, str>` for zero-allocation static values. `BraidContext.imports` uses `IndexMap` for deterministic serialization.

### Configuration

TOML config file support with full hierarchy: CLI args > env vars > config file > defaults. XDG-compliant config search (`$SWEETGRASS_CONFIG`, `$XDG_CONFIG_HOME/sweetgrass/config.toml`, `~/.config/sweetgrass/config.toml`).

---

## License

AGPL-3.0-only. See [LICENSE](./LICENSE).

---

**Fair attribution. Complete transparency. Human dignity preserved.**
