# Context — SweetGrass

## What This Is

SweetGrass is a pure Rust binary that provides semantic provenance tracking
and fair attribution for data flowing through the ecoPrimals sovereign
computing ecosystem. It creates **Braids** — cryptographically signed,
machine-readable provenance documents following the W3C PROV-O standard —
and calculates fair attribution weights for economic distribution.

SweetGrass is part of the ecoPrimals sovereign computing ecosystem: a
collection of self-contained binaries that coordinate via JSON-RPC 2.0 over
Unix sockets, with zero compile-time coupling between components.

## Role in the Ecosystem

SweetGrass sits above the "soil line" (rhizoCrypt ephemeral network +
LoamSpine permanent ledger) and below applications (gAIa, sunCloud). It
answers the fundamental question: **"What is the story of this data?"** —
who created it, how it was transformed, who contributed, and what they are
owed. Other primals call SweetGrass to record provenance and query
attribution before distributing rewards.

## Technical Facts

- **Language:** 100% Rust, zero C dependencies in production
- **Architecture:** Single binary (UniBin), multiple operational modes
- **Communication:** JSON-RPC 2.0 (required) + tarpc (optional high-perf) + REST + UDS
- **License:** scyBorg Triple-Copyleft (AGPL-3.0-or-later + ORC-1.0 + CC-BY-SA-4.0)
- **Tests:** 1,544 local + 56 Docker CI
- **Coverage:** 90%+ line (91.7% with Postgres Docker, llvm-cov)
- **BTSP:** Phase 3 — server-side `btsp.negotiate` handler with ChaCha20-Poly1305 AEAD encrypted framing; `detect_protocol` three-way multiplexer (JSON-RPC, JSON-line BTSP, length-prefixed BTSP) when `FAMILY_ID` set; HKDF-SHA256 directional session keys from BearDog's `session_key`; NULL cipher graceful fallback; `family_seed` forwarded to BearDog for crypto; EOF-resilient first-line detection for shell callers; whitespace-tolerant autodetect (leading `\n`/`\r`/` `/`\t` skipped before classification)
- **UDS contract:** Newline-delimited JSON-RPC 2.0; compositions should use `\n`-terminated requests and >=10s read timeout (`braid.create`/`provenance.graph` may touch storage)
- **Transport ports:** `--port` = TCP JSON-RPC (opt-in, newline-delimited; accepts `host:port` or bare port number — bare port binds `127.0.0.1` localhost-only per PG-55; use `0.0.0.0:PORT` for all-interfaces in Docker/production), `--http-port` / `--http-address host:port` = HTTP REST+JSON-RPC (primary integration surface, default `0.0.0.0:0` = dynamic), `--tarpc-address` = tarpc (default dynamic). Recommended TCP allocation: **9850** (avoids biomeOS TCP fallback range at 9800)
- **Discovery tiers supported:** Tier 3 (UDS filesystem convention: `sweetgrass.sock` / `sweetgrass-{family}.sock` + `provenance.sock` capability symlink) and Tier 4 (registry announce via `DISCOVERY_ADDRESS` / `DISCOVERY_BOOTSTRAP`). Tiers 1/2/5 (Songbird `ipc.resolve`, biomeOS Neural API, TCP probing) not yet implemented — sweetGrass is UDS-primary
- **Version:** 0.7.34
- **Method gate:** JH-0 pre-dispatch capability gate with token extraction — `auth.mode`, `auth.check` (enriched: `authenticated`, `verified`, `enforcement`, `scopes`, `subject`, `expires_in`), `auth.peer_info` methods; `_bearer_token` extracted from JSON-RPC params and threaded through gate; `SWEETGRASS_AUTH_MODE=permissive|enforced` env var; public whitelist (`health.*`, `auth.*`, `identity.get`, `capabilities.list`, `capability.list`, `lifecycle.status`, `tools.list`); all other methods protected; starts permissive
- **Audit pipeline:** `attribution.witness` method for JH-5 Phase 3 (`defense.log` -> `dag.event.append` -> `attribution.witness`)
- **Composition path:** `braid.create` accepts flattened convenience fields (`name`, `description`, `tags`, `source_session`, `source_merkle_root`) for provenance trio pipeline callers — merged into `BraidMetadata`; structured `metadata` takes precedence
- **Source files:** 191 `.rs` files (54,879 LOC), max 764 lines
- **Property testing:** 25 proptest strategies across 7 crates
- **Chaos/fault:** 11 attribution chaos + 17 service chaos + 9 fault injection
- **Edition:** 2024 (`resolver = "3"`), MSRV 1.87
- **Crate count:** 10 workspace crates
- **Unsafe code:** 0 blocks (`#![forbid(unsafe_code)]` on all crates)
- **Lint policy:** `#[expect(...)]` only — zero `#[allow(...)]` in source
- **Clippy:** pedantic + nursery, zero warnings
- **Dependency audit:** `cargo-deny` clean (3 RUSTSEC dev-dep ignores); `ring` dev-only via testcontainers; `sled` eliminated; `hostname` eliminated (pure Rust `/etc/hostname` read); `chacha20poly1305`+`hkdf`+`zeroize` added for BTSP Phase 3 in-process AEAD
- **Wire Standard:** L3 compliant, ecoBin static binary, Stadial parity

## Key Capabilities (JSON-RPC methods)

35 semantic methods across 11 domains:

- `braid.create`, `braid.get`, `braid.get_by_hash`, `braid.query`, `braid.delete`, `braid.commit` — provenance record CRUD
- `contribution.record`, `contribution.record_session`, `contribution.record_dehydration` — inter-primal contribution tracking
- `attribution.chain`, `attribution.calculate_rewards`, `attribution.top_contributors` — fair credit assignment
- `compression.compress_session`, `compression.create_meta_braid` — session compression (0/1/Many)
- `provenance.graph`, `provenance.export_provo`, `provenance.export_graph_provo` — W3C PROV-O export
- `anchoring.anchor`, `anchoring.verify` — LoamSpine anchoring
- `health.check`, `health.liveness`, `health.readiness` — health probes
- `identity.get` — Wire Standard L2 primal identity
- `capabilities.list`, `capability.list`, `tools.list`, `tools.call` — self-knowledge and MCP tool exposure
- `pipeline.attribute` — provenance trio pipeline coordination
- `auth.mode`, `auth.check`, `auth.peer_info` — JH-0 method gate introspection
- `composition.tower_health`, `composition.node_health`, `composition.nest_health`, `composition.nucleus_health` — ecosystem composition health probes

## What This Does NOT Do

- Does not manage cryptographic keys (that is BearDog / Songbird); signing is delegated to BearDog `crypto.sign` over UDS
- Does not store content itself (that is NestGate for content-addressed storage)
- Does not manage the permanent ledger (that is LoamSpine)
- Does not manage the ephemeral network (that is rhizoCrypt)
- Does not discover hardware or manage compute (that is toadStool / biomeOS)
- Does not perform AI inference or model coordination (that is Squirrel)

## Related Repositories

- [wateringHole](https://github.com/ecoPrimals/wateringHole) — ecosystem standards and registry
- rhizoCrypt — ephemeral DAG network (provenance trio partner)
- LoamSpine — permanent immutable ledger (provenance trio partner)
- NestGate — content-addressed storage
- Squirrel — AI coordination (consumes `tools.list` / `tools.call`)
- biomeOS — orchestration (health probes, capability discovery)
- sunCloud — reward distribution (consumes attribution queries)

## Design Philosophy

These binaries are built using AI-assisted constrained evolution. Rust's
compiler constraints (ownership, lifetimes, type system) reshape the fitness
landscape and drive specialization. Primals are self-contained — they know
what they can do, never what others can do. Complexity emerges from runtime
coordination, not compile-time coupling.
