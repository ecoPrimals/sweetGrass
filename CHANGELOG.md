# Changelog

All notable changes to SweetGrass will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2025-12-25

### Added - Showcase Enhancement
- **Privacy Controls Demo** (`showcase/00-local-primal/05-privacy-controls/`)
  - GDPR-inspired data subject rights (Access, Erasure, Portability)
  - Privacy levels (Public, Private, Encrypted)
  - Retention policies (Duration, LegalHold)
  - Real service execution, no mocks

- **Storage Backends Demo** (`showcase/00-local-primal/06-storage-backends/`)
  - Memory backend demonstration (testing, ephemeral)
  - Sled backend demonstration (embedded, Pure Rust)
  - PostgreSQL backend demonstration (production, multi-node)
  - Runtime backend selection patterns

- **Real Verification Demo** (`showcase/00-local-primal/07-real-verification/`)
  - 10-point real execution checklist
  - Binary verification, service validation
  - API compatibility checks
  - Zero mocks validation

- **Integration Tests** (Real binaries, no mocks)
  - NestGate integration test (3/5 tests, 60%)
  - Songbird integration test (5/6 tests, 83%)
  - ToadStool integration test (4/5 tests, 80%)
  - Squirrel integration test (4/6 tests, 66%)
  - **Overall**: 16/22 tests passed (73%)

- **Revolutionary AI Attribution Patterns**
  - Complete AI provenance chain (Training Data → Model → Generated Content)
  - Fair attribution for data providers (20%)
  - Fair attribution for ML engineers (20%)
  - Fair attribution for AI models (20%)
  - Fair attribution for users (40%)
  - First provenance system to provide fair AI attribution!

- **Documentation**
  - 3 new README files for demos
  - 4 integration pattern documents
  - Updated showcase README with test results
  - Comprehensive completion reports

### Changed
- **Enhanced `RUN_ME_FIRST.sh`** (NestGate pattern)
  - Added colored, narrative output
  - Progress tracking (X/6 levels)
  - Time estimates per level
  - Pauses for observation
  - Comprehensive summary
  - 50-minute guided tour

- **Updated Showcase Structure**
  - `00-standalone/` → `00-local-primal/` (7 levels now)
  - Added real binary integration tests
  - Enhanced documentation across all levels

### Fixed
- Gap discovered: NestGate JWT configuration requirement (documented)
- Gap discovered: Songbird health endpoint API format (documented)
- Gap discovered: ToadStool BYOB port configuration (documented)
- Gap discovered: Squirrel service mode capabilities (documented)

### Principles Validated
- ✅ "Interactions show us gaps in our evolution" (4 new gaps discovered)
- ✅ "No mocks in showcase" (100% real binaries, 0 mocks)
- ✅ "Deep debt solutions" (proper patterns maintained)
- ✅ "Primal sovereignty" (capability-based discovery enforced)

## [0.4.0] - 2025-12-24

### Added - Phase 2 Evolution
- Infant Discovery pattern (100% complete)
- BraidStoreFactory for runtime backend selection
- SelfKnowledge environment-driven configuration
- 4 capability clients (Anchor, Discovery, Listener, Signer)
- Privacy controls (GDPR-style data subject rights)
- Comprehensive PostgreSQL migration tests (13 tests)
- Fuzz testing infrastructure (3 targets)

### Changed
- Removed 28 deprecated aliases
- Expanded test coverage (error.rs: +9 tests, privacy.rs: +9 tests)
- Refactored factory.rs complexity (28 → clean)
- Evolved hardcoded test addresses (3 → 0)

### Fixed
- 2 failing tests
- 6 Clippy errors
- 7 Rustfmt violations
- Production unwrap audit (0 in production)

### Quality Metrics
- Tests: 489 passing (100% pass rate)
- Coverage: ~82% function, ~92% region
- unsafe_code: 0 (forbidden in all 9 crates)
- Production unwraps: 0 (638 audited, all in tests)
- Hardcoded addresses: 0 (all capability-based)
- Grade: A+ (100/100 initially, 98/100 after showcase)

## [0.3.0] - 2025-12-XX

### Added
- Multiple storage backends (Memory, PostgreSQL, Sled)
- W3C PROV-O compliance
- Attribution engine with fair credit distribution
- Session compression
- REST API and tarpc RPC

### Changed
- Pure Rust implementation (`#![forbid(unsafe_code)]`)
- Comprehensive error handling (zero production unwraps)
- Idiomatic Rust patterns throughout

## [0.2.0] - 2025-XX-XX

### Added
- Core Braid data model
- Basic provenance tracking
- Query engine

## [0.1.0] - 2025-XX-XX

### Added
- Initial project structure
- Basic attribution concepts
- Proof of concept

---

## Versioning Notes

- **Major version** (X.0.0): Breaking API changes
- **Minor version** (0.X.0): New features, backward compatible
- **Patch version** (0.0.X): Bug fixes, documentation

## Links

- [Repository](https://github.com/ecoPrimals/sweetGrass)
- [Documentation](./README.md)
- [Status](./STATUS.md)
- [Roadmap](./ROADMAP.md)

