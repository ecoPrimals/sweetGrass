# sweetGrass — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## Braid Model & PROV-O — W3C provenance attribution surface

HTTP REST endpoints → semantic JSON-RPC with W3C PROV-O braids, zero-copy `Arc<str>` identities, and structured domain metadata.

- **PROV-O braids** (v0.7.0): `BraidId`/`Did` newtypes with `Arc<str>` zero-copy; namespace URIs extracted to constants; contribution recording API (`recordContribution`, `recordSession`) for inter-primal attribution.
- **Braid factory evolution** (v0.5.0→v0.7.0): Hardcoded `"rhizoCrypt"` source → `with_source()` runtime discovery; Infant Discovery pattern eliminated 8 hardcoding violations.
- **CertificateRef type** (v0.7.64): Bare `String` certificate → structured type with `issuing_gate`, `sealed`, `minting_authority`, `content_hash` for cross-gate attestation; backward-compatible string deserialization.
- **Witness zero-copy** (Wave 155n+): `Witness` fields and `LoamAnchor.spine_id` evolved `String` → `Arc<str>`; eliminates per-clone allocations on hot braid paths.
- **Production certification** (v0.5.0): A+ (100/100) — 386 tests, 0 unsafe, 0 unwraps, 0 hardcoding; tied #1 with BearDog in ecosystem standing.

---

## Batch Provenance Pipeline — high-throughput ingestion

Sequential braid creation → bounded-concurrency batch operations with trailer-pattern alignment for convoy-scale ingestion.

- **braid.batch_create/commit** (Wave 155n): Bulk braid creation via `put_batch` + pipelined loamSpine forwarding; 10× throughput target for PDB/dataset ingestion (~3ms/object vs ~30ms sequential).
- **Trailer pattern** (Wave 155u/156b): Concurrent loamSpine commits via `join_all`; batch size guard (MAX 5,000) prevents unbounded memory; addresses 12× throughput divergence (74→6 files/s).
- **BraidId::to_uuid()** (Wave 155n): Deterministic UUID v5 derivation resolves braid_id→UUID mismatch on `braid.commit`.
- **G31 pipeline coordination** (Wave 155n): rhizoCrypt `dag.pipeline.ingest` coordinates create + batch append + optional dehydrate; sweetGrass receives batch dehydration notifications (N→1 RPC).

---

## Convergence & Verification — trust gates for data consumption

Ad-hoc provenance checks → one-call convergence verification with depth scoring and content integrity attestation.

- **convergence.check/batch_check** (Wave 156e): Stage-by-stage status (CAS → DAG → Spine → Braid → Signed) for content hashes; batch up to 1,000 hashes with depth 0–5 scoring.
- **braid.verify** (Wave 157a): P1 SHIPPED — content integrity + Ed25519 signature + ledger confirmation in single atomic call; 5 behavioral tests (unsigned, crypto-down permissive, not-found).
- **braid.list** (Wave 156e): Lightweight enumeration for observability — summary entries without full braid object transfer.
- **convergence.pressure** (Wave 156j): Backpressure signal from convergence lag — depth distribution, pressure ratio, throttle recommendation for convoy/bulk ingestion pipelines.

---

## LoamSpine Integration — ledger forwarding and anchor verification

Local-only braids → cross-primal ledger commits with graceful degradation when loamSpine unavailable.

- **LedgerClient module** (v0.8.0): JSON-RPC client over UDS/TCP for `braid.commit`, `certificate.verify`; capability-based socket resolution with env/family-scoped/standalone fallbacks.
- **braid.commit forwarding** (v0.8.0): Payloads forwarded to loamSpine when available; response includes `committed` flag and `ledger_commit` reference.
- **anchoring.verify** (v0.8.0): Cross-primal certificate verification via loamSpine; `ledger_verified` status in response; doc updated from "will be wired" to active.
- **Bootstrap Phase 4c** (v0.8.0): Automatic loamSpine discovery at startup; graceful local-only degradation.

---

## rootPulse Integration — graph executor trio steps

Standalone braid methods → rootPulse graph step handlers wired into biomeOS commit workflow.

- **rootpulse.attribute/query** (Wave 157k): Attribution braids from `rootpulse_commit` graph executor; `braid.attribute` wire-name alias; 13 rootPulse tests (full attribution, multi-filter queries, limit enforcement).
- **Registry sync** (Wave 157k): 48→50 methods across niche CAPABILITIES, dispatch table, and `capability_registry.toml`.

---

## Transport & Protocol — BTSP, riboCipher, dual-socket composition

Plain JSON-RPC over UDS → multi-protocol accept with G65 negotiation, G66 transport abstraction, and C2 tarpc binary path.

- **BTSP handshake** (v0.7.63→151b): Consumer-side 4-step ClientHello; HMAC-SHA256 challenge-response; strict-mode detection via `BEARDOG_UDS_REQUIRE_BTSP`.
- **G65 protocol negotiation** (Wave 156m): Single-socket `PROTOCOLS: tarpc,jsonrpc\n` selection replaces C2 dual-socket as canonical entry; 14 unit tests; backward compatible with riboCipher/BTSP clients.
- **G66 transport abstraction** (Wave 156s): `TransportEndpoint::platform_default()`, `TransportListener`, `CryptoDelegate::with_endpoint()` — silicon-agnostic UDS/TCP without `#[cfg]` in callers.
- **C2 dual-socket** (Wave 156j): tarpc binary on `sweetgrass.tarpc.sock` alongside JSON-RPC; sub-ms intra-gate composition; 5-tier socket fallback resolution.
- **riboCipher signals** (Wave 113): Legacy peek removal; REJECT for invalid signals; transport signal convergence with ecosystem SSOT.
- **TransportEndpoint injection** (Wave 100→142b): Phase 2 abstraction — outbound IPC migrated to `connect_transport()`; ring elimination (Wave 98).

---

## Cross-Gate Trust & Attribution — weaving provenance across gates

Single-gate braids → cross-gate trust events, attribution weaving, and mito-beacon acceptance.

- **Cross-gate trust weaving** (Wave 77): Trust event braids linking gates; attribution metadata propagation.
- **Cross-gate attribution** (Wave 76): Deep debt + attribution integration for multi-gate provenance chains.
- **Genetics-layer wiring** (Wave 114): Mito-beacon signal acceptance on UDS transport.
- **braid.anchor method** (Wave 60): Anchor signing via Tower security provider; DH-1 `/tmp` hardcoding cleanup.

---

## Storage Backends & Persistence — from sled to Postgres purity

Multi-backend sprawl → Postgres-primary with sled removal, env snapshot isolation, and store parity.

- **Sled backend removal** (April 2026): Lockfile ghost elimination; pure Postgres path.
- **AppState env snapshots** (Wave 78): Zero hot-path env reads — factory + query + trust paths use startup-captured config.
- **Store parity** (Wave 67c→69): Stub elimination, error chains, PROV-O schema completeness, privacy edge cases.
- **Ring/async-trait elimination** (April 2026): Stadial parity gate; dyn-free production paths; entity tests extracted.

---

## Security, Auth & BTSP — method gates and encrypted sessions

Open JSON-RPC → BTSP encrypted framing, MethodGate authorization, and token extraction pipeline.

- **BTSP Phase 3** (May 2026): Server-side `btsp.negotiate` + ChaCha20-Poly1305 encrypted framing; first-line auto-detect; `family_seed` RPC param alignment.
- **JH-0 MethodGate** (v0.7.32): Pre-dispatch Public/Protected classification adopted from primalSpring reference.
- **PG-55/59 bind control** (v0.7.31): TCP bind address control; HTTP address docs; localhost-only default (Wave 79b).
- **Token extraction pipeline** (v0.7.33): Enriched auth, audit pipeline for multi-user hardening.

---

## Neural API & Discovery — self-registration and capability routing

Manual capability lists → automated `primal.announce`, `capability.call` routing envelope, and MCP tool schemas.

- **primal.announce** (Wave 43): Outbound startup registration with capabilities, methods, semantic mappings, cost hints.
- **capability.call handler** (Wave 157a): Neural API routing envelope for biomeOS dispatch.
- **Vertebrate self-audit** (Wave 157a–e): RPC surface aligned across dispatch table, `niche::CAPABILITIES`, and registry; MCP tool schemas corrected.
- **Composition readiness** (v0.7.34): Provenance trio pipeline validation for Nest atomic composition.

---

## Architecture & Engineering — codebase evolution (cross-cutting)

Audit-discovered debt → reference provenance primal with 90%+ coverage and zero production debt markers.

- **Deep debt evolution** (Wave 115→157k): Smart refactoring splits — `handlers/jsonrpc/tests.rs` (799L→9 modules), `uds.rs`, `btsp/transport.rs`; zero files >800L.
- **G72 dependency trim** (Wave 157g): tokio features explicit; dead deps excised; 155 unique transitive crates.
- **Coverage sprint** (Wave 157k): 88.09%→89.62% line; `braid_verify.rs` 33.8%→97.2%; 1,684→1,746 tests.
- **Pure Rust dogma** (Wave 133b): Shell-outs removed; hostname crate eliminated; UniBin musl-static deployment.
- **Clippy & fmt** (ongoing): Pedantic zero warnings; `#[must_use]` annotations; `#[non_exhaustive]` on forward-compat enums.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
