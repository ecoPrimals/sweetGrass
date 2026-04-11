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
- **Tests:** 1,315 passing
- **Coverage:** 90%+ region coverage (llvm-cov, excluding Postgres which needs runtime)
- **BTSP:** Phase 2 — server handshake on accept (UDS + TCP) when `FAMILY_ID` set
- **MSRV:** 1.87 (Edition 2024)
- **Crate count:** 10 workspace crates
- **Unsafe code:** 0 blocks (`#![forbid(unsafe_code)]` on all crates)
- **Clippy:** pedantic + nursery, zero warnings

## Key Capabilities (JSON-RPC methods)

28 semantic methods across 9 domains:

- `braid.create`, `braid.get`, `braid.get_by_hash`, `braid.query`, `braid.delete`, `braid.commit` — provenance record CRUD
- `contribution.record`, `contribution.record_session`, `contribution.record_dehydration` — inter-primal contribution tracking
- `attribution.chain`, `attribution.calculate_rewards`, `attribution.top_contributors` — fair credit assignment
- `compression.compress_session`, `compression.create_meta_braid` — session compression (0/1/Many)
- `provenance.graph`, `provenance.export_provo`, `provenance.export_graph_provo` — W3C PROV-O export
- `anchoring.anchor`, `anchoring.verify` — LoamSpine anchoring
- `health.check`, `health.liveness`, `health.readiness` — health probes
- `identity.get` — Wire Standard L2 primal identity
- `capabilities.list`, `tools.list`, `tools.call` — self-knowledge and MCP tool exposure
- `pipeline.attribute` — provenance trio pipeline coordination

## What This Does NOT Do

- Does not manage cryptographic keys or signing (that is BearDog / Songbird)
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
