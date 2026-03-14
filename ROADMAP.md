# SweetGrass Roadmap

**Current Version**: v0.7.4 (March 2026)

---

## Completed

### v0.7.4 — Deep Debt: parking_lot + Idiomatic Refactor (March 2026)

- [x] `parking_lot::RwLock` migration (MemoryStore, Indexes, mock impls — eliminates all lock poisoning)
- [x] Infallible index operations (removed `Result` wrapping from all `Indexes` methods)
- [x] `DEFAULT_QUERY_LIMIT` centralized in `sweet-grass-store::traits` (removed sled/postgres duplication)
- [x] `SIGNING_ALGORITHM` constant extracted (was hardcoded `"Ed25519Signature2020"`)
- [x] JSON-RPC `error_code` module promoted to `pub(crate)` (UDS uses named constants, not magic numbers)
- [x] Status subcommand evolved to real HTTP `/health` check (was raw TCP)
- [x] Attribution tests extracted to `tests.rs` (786→302L production + 484L tests)
- [x] Clippy 0 warnings, 746 tests passing, all files under 1000 LOC
- [x] Stale `DEPRECATED_ALIASES_REMOVAL_PLAN.md` references cleaned from source comments
- [x] Root docs and wateringHole handoff updated

### v0.7.3 — Comprehensive Audit + 94% Coverage (March 2026)

- [x] 176 new tests (570 → 746), 94% line coverage achieved
- [x] JSON-RPC dispatch coverage for all 20 methods
- [x] Server RPC, factory config, discovery, core model, store filter coverage
- [x] JSON-RPC test extraction (mod.rs 1103→280 LOC + tests.rs 824 LOC)
- [x] `get_batch` ordering bug fix (`buffer_unordered` → `buffered`)
- [x] Zero TODOs/FIXMEs in source
- [x] Root docs updated with current metrics

### v0.7.2 — Provenance Trio Coordination + biomeOS IPC + Tower Atomic (March 2026)

- [x] `DehydrationSummary` shared contract for rhizoCrypt dehydration handoff
- [x] `braid.commit` JSON-RPC method for LoamSpine anchoring (BraidId → UUID, ContentHash → `[u8; 32]`)
- [x] `contribution.recordDehydration` JSON-RPC method for rhizoCrypt session import
- [x] Unix domain socket transport for biomeOS IPC (XDG-compliant path resolution)
- [x] Centralized `hash` module (hex encode/decode/sha256 — eliminates 3x duplication)
- [x] Smart module refactoring: `attribution/` (chain.rs + mod.rs) and `listener/` (tarpc_client.rs + mod.rs)
- [x] `source_primal` field replaces hardcoded primal names in dehydration flow
- [x] Tower Atomic enforcement: `cargo deny` wrappers for ring/rustls dev-dependency exemption
- [x] `serial_test` for environment-dependent test isolation
- [x] 570 tests passing, zero clippy warnings

### v0.7.1 — Standards Compliance + Zero-Copy Evolution (March 2026)

- [x] JSON-RPC semantic naming aligned to wateringHole `{domain}.{operation}` standard
- [x] Dispatch table architecture (replaces giant match statement)
- [x] ContentHash evolved to zero-copy `Arc<str>` newtype (O(1) clone)
- [x] Bootstrap single-path through `BraidStoreFactory` (no dual env logic)
- [x] Primal lifecycle methods evolved from async to sync (no unnecessary runtime overhead)
- [x] `LoamEntryParams` struct replaces 7 positional arguments
- [x] Hardcoded postgres default URL removed (require explicit config)
- [x] Bootstrap test isolation fixed (8 env vars cleared)
- [x] 8 `unused_async` suppressions eliminated
- [x] `native-tls` banned in `deny.toml`
- [x] 554 tests passing, zero clippy warnings

### v0.7.0 — Deep Remediation + Contribution API (March 2026)

- [x] Inter-primal contribution recording API (`contribution.record`, `contribution.recordSession`)
- [x] ContributionRecord + SessionContribution types for rhizoCrypt/biomeOS integration
- [x] Extensible domain metadata (chemistry, ML, game domain keys)
- [x] JSON-RPC 2.0 handler with semantic method names
- [x] UniBin CLI (clap subcommands: `server`, `status`)
- [x] Arc<str> zero-copy for BraidId and Did
- [x] SPDX AGPL-3.0-only headers on all source files
- [x] Large file refactoring (mod.rs + tests.rs pattern)
- [x] Magic number elimination (named constants everywhere)
- [x] PROV-O namespace URIs extracted to constants
- [x] 19 HTTP-level E2E tests (REST + JSON-RPC + contribution flow)
- [x] Cross-compilation targets documented
- [x] deny.toml updated for AGPL-3.0-only
- [x] ecoBin full compliance verified

### v0.6.0 — Production Hardening (January 2026)

- [x] Comprehensive audit and remediation
- [x] E2E and chaos testing expansion
- [x] Documentation consolidation
- [x] PostgreSQL integration tests with testcontainers
- [x] Fuzz testing infrastructure (3 targets)

### v0.5.0 — Infant Discovery (December 2025)

- [x] Capability-based discovery (zero hardcoded primal names)
- [x] SelfKnowledge environment-driven configuration
- [x] BraidStoreFactory for runtime backend selection
- [x] 4 capability clients (Anchor, Discovery, Listener, Signer)
- [x] Privacy controls (GDPR-style data subject rights)
- [x] Production certification (A+)

### v0.4.0 — Phase 2 Evolution (December 2025)

- [x] Service binary with REST API
- [x] Multiple storage backends (Memory, PostgreSQL, Sled)
- [x] tarpc RPC (pure Rust, no gRPC/protobuf)
- [x] Showcase with 37 demo scripts
- [x] 12 agent roles with attribution weights

---

## Next

### v0.8.0 — Real Deployment (Q2 2026)

**Goal**: Connect to production-deployed primals

- [ ] Connect to deployed signing service (via Capability::Signing)
- [ ] Connect to deployed session events service (via Capability::SessionEvents)
- [ ] Connect to deployed anchoring service (via Capability::Anchoring)
- [ ] End-to-end multi-primal integration testing
- [ ] Chemistry entity types for wetSpring (Molecule, BasisSet, DftCampaign)
- [ ] Chemistry braid relations (DependsOn, ValidatedBy, ComputedWith, TrainedOn)
- [x] Coverage target: 90%+ with llvm-cov *(done in v0.7.3 — 94%)*

### v0.9.0 — sunCloud Integration (Q3 2026)

**Goal**: Fair reward distribution based on attribution

- [ ] Attribution API for sunCloud
- [ ] Real-time attribution updates
- [ ] Historical attribution queries
- [ ] Payment flow integration

### v1.0.0 — Production GA (Q4 2026)

**Goal**: Stable public API

- [ ] API versioning strategy finalized
- [ ] Full W3C PROV-O spec compliance (PROV-DM, PROV-N, PROV-AQ)
- [ ] Distributed provenance (multi-node federation via Squirrel)
- [ ] Performance benchmarks published
- [ ] Kubernetes deployment manifests

---

## Ongoing

### Testing
- [x] Coverage to 90%+ *(done in v0.7.3 — 94%)*
- [ ] Expand chaos testing scenarios
- [ ] Property-based testing expansion (proptest)
- [ ] Fuzz testing campaigns
- [ ] Load testing for production scenarios

### Performance
- [x] Zero-copy expansion (ContentHash to newtype with Arc<str>) *(done in v0.7.1)*
- [ ] Zero-copy: tarpc `Vec<u8>` → `bytes::Bytes` (wire protocol change, needs cross-primal coordination)
- [ ] Query performance benchmarks
- [ ] PostgreSQL index tuning
- [ ] Lazy loading for large provenance graphs

### Infrastructure
- [ ] CI/CD pipeline
- [ ] Automated performance regression testing
- [ ] Monitoring and observability

---

## Guiding Principles

### Primal Sovereignty
- Pure Rust (no C/C++ dependencies)
- tarpc for RPC (no gRPC/protobuf)
- Capability-based discovery (zero hardcoded addresses)
- Environment-driven configuration
- Zero-knowledge startup

### Human Dignity
- Privacy by design (GDPR-inspired)
- Consent management
- Data subject rights
- Transparency and auditability
- Fair attribution

### Code Quality
- `#![forbid(unsafe_code)]` in all crates
- Zero production unwraps
- Comprehensive testing (90% coverage target)
- Clean Clippy (pedantic + nursery, `-D warnings`)
- All files under 1000 LOC
- AGPL-3.0-only

---

| Version | Target | Focus |
|---------|--------|-------|
| v0.7.4 | **March 2026** | Deep Debt: parking_lot + Refactor (DONE) |
| v0.7.3 | March 2026 | Audit + 94% Coverage (DONE) |
| v0.7.2 | March 2026 | Provenance Trio + biomeOS IPC (DONE) |
| v0.7.1 | March 2026 | Standards + Zero-Copy Evolution (DONE) |
| v0.7.0 | March 2026 | Deep Remediation (DONE) |
| v0.8.0 | Q2 2026 | Real Deployment |
| v0.9.0 | Q3 2026 | sunCloud Integration |
| v1.0.0 | Q4 2026 | Production GA |
