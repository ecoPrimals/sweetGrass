# SweetGrass Roadmap

**Current Version**: v0.7.0 (March 2026)

---

## Completed

### v0.7.0 — Deep Remediation (March 2026)

- [x] JSON-RPC 2.0 handler with semantic method names
- [x] UniBin CLI (clap subcommands: `server`, `status`)
- [x] Arc<str> zero-copy for BraidId and Did
- [x] SPDX AGPL-3.0-only headers on all source files
- [x] Large file refactoring (mod.rs + tests.rs pattern)
- [x] Magic number elimination (named constants everywhere)
- [x] PROV-O namespace URIs extracted to constants
- [x] 16 HTTP-level E2E tests (REST + JSON-RPC)
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
- [ ] Coverage target: 90%+ with llvm-cov

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
- [ ] Coverage to 90%+ (currently ~88%)
- [ ] Expand chaos testing scenarios
- [ ] Property-based testing expansion (proptest)
- [ ] Fuzz testing campaigns
- [ ] Load testing for production scenarios

### Performance
- [ ] Zero-copy expansion (ContentHash to newtype with Arc<str>)
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
| v0.7.0 | **March 2026** | Deep Remediation (DONE) |
| v0.8.0 | Q2 2026 | Real Deployment |
| v0.9.0 | Q3 2026 | sunCloud Integration |
| v1.0.0 | Q4 2026 | Production GA |
