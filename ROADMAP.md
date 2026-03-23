# SweetGrass Roadmap

**Current Version**: v0.7.26 (March 2026)

---

## Completed

### v0.7.26 — Ecosystem Absorption: scyBorg License, Sled Deprecation, Lint Evolution (March 2026)

- [x] **scyBorg Triple-Copyleft LICENSE** — adopted rhizoCrypt v0.13.0 format (AGPL + ORC + CC-BY-SA with Reserved Material section)
- [x] **Sled backend deprecated** — `#[deprecated(since = "0.7.26")]` on `SledStore`, all impl blocks annotated with `#[expect(deprecated)]`, migration docs pointing to redb; follows rhizoCrypt deprecation path
- [x] **3 unused `async-trait` deps removed** — factory, compression, query crates; remaining 6 documented with dyn-compatibility rationale (object safety requires boxing)
- [x] **`#[allow]` → `#[expect(reason)]` complete** — all 5 production `#[allow]` evolved; unfulfilled expectations removed (loamSpine v0.9.10 pattern: don't suppress what doesn't fire)
- [x] **SPDX headers on all 12 `Cargo.toml` files** — ludoSpring V29 ecosystem pattern
- [x] **`deny.toml` tightened** — `multiple-versions = "deny"` (was "warn"), aligned with BearDog v0.9.0
- [x] **Cast lints added to workspace** — `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_lossless` at warn level, aligned with loamSpine trio partner
- [x] **`normalize_method()`** — case-insensitive JSON-RPC dispatch (barraCuda → loamSpine → wetSpring ecosystem pattern)
- [x] **`cargo-llvm-cov` CI aliases** — `.cargo/config.toml`: `cargo coverage`, `cargo coverage-html`, `cargo coverage-json`
- [x] **`/tmp/` path audit** — 13 occurrences verified as config struct string fixtures (not filesystem ops); safe
- [x] **async-trait dyn-compatibility documented** — comments on all 6 retained Cargo.toml deps explaining object safety
- [x] **`primal_names::names` deprecated** — hardcoded other-primal constants deprecated; generic `socket_env_var()`/`address_env_var()` retained for runtime-discovered names
- [x] **Primal sovereignty audit** — production code has self-knowledge only (`niche.rs`, `primal_info.rs`); discovery is capability-based, no compile-time coupling to peer primals
- [x] **Mock audit** — all `Mock*` types gated behind `#[cfg(any(test, feature = "test"))]`; zero mocks in production paths
- [x] **Dependency audit** — production binary is 100% pure Rust (zero `-sys` crates); `ring`/`cc` only in dev-deps via testcontainers
- [x] **File size audit** — all files under 826 lines (max: `store-redb/tests.rs`); well under 1000-line ceiling
- [x] **Unsafe audit** — `#![forbid(unsafe_code)]` on all 10 crates + binary; zero `unsafe` blocks, `#[no_mangle]`, `extern "C"`, `.as_ptr()`, or `from_raw()`
- [x] 1,128 tests passing (unchanged), 0 clippy warnings, 0 doc warnings, 0 unsafe, 0 fmt issues

### v0.7.25 — Coverage Push, Test Hygiene, PUBLIC_SURFACE_STANDARD Compliance (March 2026)

- [x] **90% line coverage achieved** — pushed from ~78% to 90.47% via targeted tests for error variants, provenance traversal, handler gaps, and sled config
- [x] **Sled tests smart refactored** — 922-line monolith split into 3 focused modules (mod.rs + query.rs + edge.rs) by functional concern
- [x] **PII audit (Layer 4)** — clean: no emails, home paths, private IPs, or API keys in codebase
- [x] **README ecoPrimals footer** — per PUBLIC_SURFACE_STANDARD Layer 2
- [x] **Arc\<str\> filter verification** — confirmed `p.as_ref() == primal.as_str()` correctly compares `Arc<str>` ↔ `String` in `memory/filter.rs`
- [x] **Coverage artifacts cleaned** — phantom 0% entries from stale profraw data eliminated
- [x] 1,121 tests (was 1,106 — +15 new), max file 826 lines (was 922), 136 .rs files (39,903 LOC)

### v0.7.24 — Deep Debt: Zero-Copy Phase 2, Public Surface, Comprehensive Audit (March 2026)

- [x] **Zero-copy Phase 2: `EcoPrimalsAttributes` fields** — `source_primal` and `niche` evolved from `Option<String>` to `Option<Arc<str>>` across all 10 crates; every Braid shares source identity via O(1) atomic clone
- [x] **Zero-copy Phase 2: `LoamCommitRef.spine_id`** — `String` → `Arc<str>`
- [x] **Zero-copy Phase 2: `BraidFactory` + `CompressionEngine`** — internal `source_primal` and `niche` evolved to `Arc<str>`
- [x] **Zero-copy Phase 2: `LoamEntryParams`** — `spine_id` and `mime_type` evolved to `Arc<str>`
- [x] **`CONTEXT.md` created** — AI-readable context block per wateringHole `PUBLIC_SURFACE_STANDARD` Layer 3
- [x] **`CONTRIBUTING.md` created** — contributor guide with code standards, PR checklist, ecosystem principles
- [x] **Comprehensive audit verified** — 1,106 tests, 0 clippy warnings, 0 doc warnings, 0 unsafe, 0 production unwraps, 0 TODOs, all files <1000 LOC, all mocks test-gated, all deps pure Rust
- [x] **README metrics corrected** — accurate test count (1,106), honest coverage (78% excluding Postgres runtime), max file 922 lines
- [x] **Build environment documented** — `.cargo/config.toml` local target-dir override for `noexec` mount workaround

### v0.7.23 — Ecosystem Absorption: MCP Tool Exposure, Canonical Capabilities (March 2026)

- [x] **MCP `tools.list` + `tools.call`** — expose braid operations as MCP tools for Squirrel AI coordination (airSpring v0.10 pattern); McpTool descriptors with JSON Schema inputSchema
- [x] **`capabilities.list` canonical method** — wateringHole SEMANTIC_METHOD_NAMING v2.1 canonical name; `capability.list` retained as backward-compatible alias
- [x] **Niche self-knowledge expanded** — 24 → 27 capabilities, operation_dependencies, cost_estimates, semantic_mappings updated
- [x] **`DispatchOutcome` validated** — already aligned with rhizoCrypt v0.13.0 pattern (Success/ProtocolError/ApplicationError)
- [x] **Atomic socket test isolation validated** — already uses `tempdir()` + explicit `start_uds_listener_at` paths (ludoSpring V28 pattern)
- [x] **Pure Rust dependency stack confirmed** — cargo-deny: advisories ok, bans ok, licenses ok, sources ok; zero C/C++ deps
- [x] **8 new protocol tests** — canonical/alias equivalence, tools.list structure/contents, tools.call dispatch/error, DispatchOutcome classification
- [x] 1,099 tests (was 1,084 — +15 new), 27 JSON-RPC methods (was 24), 0 clippy warnings, 0 unsafe

### v0.7.22 — Sovereignty: Remove provenance-trio-types, Inline Wire Types (March 2026)

- [x] **`provenance-trio-types` dependency removed** — eliminated last cross-primal compile-time coupling; sweetGrass owns all its wire types
- [x] **`PipelineRequest` / `PipelineResult` inlined** — handler-local wire types in `contribution.rs`, minimum necessary serde derives
- [x] **Direct deserialization** — `handle_record_dehydration` deserializes into own `DehydrationSummary` directly; `From` impls and wire re-export deleted
- [x] **`deny.toml` sovereignty guard** — `provenance-trio-types` banned to prevent re-introduction
- [x] **Wire tolerance** — `#[serde(default)]` on `SessionOperation.timestamp` and `Attestation.attested_at` for forward compatibility
- [x] **wateringHole registry updated** — `PRIMAL_REGISTRY.md` and `genomeBin/manifest.toml` synced to v0.7.22

### v0.7.21 — Deep Audit: Zero-Copy, Handler Coverage, Test Refactor (March 2026)

- [x] **`Braid.mime_type: String` → `Arc<str>`** — zero-copy optimization across all 7 crates; MIME type indexes share `Arc<str>` eliminating per-query allocations on hot paths
- [x] **Hardcoded primal name eliminated** — `jsonrpc/contribution.rs` now uses canonical `sweet_grass_core::identity::PRIMAL_NAME` constant
- [x] **28 new JSON-RPC handler tests** — extended coverage across anchoring, attribution, braid commit, compression, provenance, contribution, and pipeline methods
- [x] **Smart test refactor** — `jsonrpc/tests.rs` (1,448 lines → 480) split into 5 domain test modules: `tests_anchoring`, `tests_attribution`, `tests_compression`, `tests_contribution`, `tests_provenance`
- [x] **`#[must_use]` on test port allocators** — clippy pedantic compliance for `allocate_test_port()` and `allocate_test_ports()`
- [x] **Float comparison fix** — epsilon-based `assert!` replacing strict `assert_eq!` on `f64` values
- [x] 1,084 tests (was 1,049 — +35 new), 133 .rs files, 0 clippy warnings, max file 808 lines

### v0.7.20 — Ecosystem Absorption: IPC Timeout, extract_rpc_error, Capability Parsing, Proptest (March 2026)

- [x] **`deny.toml` `yanked = "deny"`** — aligned with airSpring v0.8.7, neuralSpring S160 ecosystem standard; yanked crates now block builds
- [x] **`IpcErrorPhase::Timeout` variant** — explicit timeout phase aligned with neuralSpring S160 `IpcError::Timeout`; integrated into `is_retriable()` and `is_timeout_likely()` classification
- [x] **`extract_rpc_error()` helper** — extracts `(code, message)` from JSON-RPC 2.0 error responses; aligned with airSpring v0.8.7 and neuralSpring S160 patterns
- [x] **`extract_capabilities()` dual-format parser** — parses both flat array (`{"methods": [...]}`) and structured domain (`{"domains": {"braid": ["create"]}}`) formats from `capability.list` responses; handles `result` wrapper and `capabilities` alias for ecosystem compat
- [x] **Proptest properties** — `extract_rpc_error` roundtrip, never-panics fuzzing, `IpcErrorPhase` display/retriable consistency, `extract_capabilities` flat roundtrip and never-panics fuzzing
- [x] **`require_braid_by_hash()` refactor** — server/mod.rs: 4 methods deduplicated via shared helper; eliminates repeated `get_by_hash + ok_or_else(NotFound)` pattern
- [x] **`ValidatedFilter` + `bind_filter!` macro** — store-postgres: eliminated duplicated WHERE clause building and parameter binding between main query and count query
- [x] **`#[allow(unused_imports)]` removed** — aligned `lib.rs` mock re-exports to `#[cfg(any(test, feature = "test"))]`, eliminating 2 unnecessary `#[allow]` attributes
- [x] **`discovery` module public** — enables `sweet_grass_integration::discovery::extract_capabilities` path; doc fields added to `DiscoveryError::ConnectionFailed`
- [x] 1,049 tests (was 1,030 — +19 new), 0 clippy warnings, 0 unsafe

### v0.7.19 — Ecosystem Absorption: Health Probes, DispatchOutcome, OrExit (March 2026)

- [x] **`health.liveness` + `health.readiness`** — wateringHole `PRIMAL_IPC_PROTOCOL` v3.0 JSON-RPC + tarpc methods; aligned with coralReef + healthSpring
- [x] **`IpcErrorPhase` classification helpers** — `is_retriable()`, `is_timeout_likely()`, `is_method_not_found()`, `is_application_error()` for retry gating and circuit breaker integration
- [x] **`DispatchOutcome` enum** — protocol vs application error separation in JSON-RPC dispatch; aligned with rhizoCrypt/biomeOS pattern
- [x] **`OrExit<T>` trait** — zero-panic binary validation with structured exit codes; aligned with biomeOS pattern
- [x] **`exit` module** — centralized exit codes per wateringHole `UNIBIN_ARCHITECTURE_STANDARD`
- [x] **`eprintln!` → `tracing::error!`** — structured logging throughout binary entrypoint
- [x] **`#[allow]` audit** — retained only for conditionally-compiled items; all others verified
- [x] 1,030 tests (was 1,017), 24 JSON-RPC methods (was 22), 0 clippy warnings, 0 unsafe

### v0.7.18 — Deep Execution: tarpc 0.37 + Structured IPC + Pipeline Integration (March 2026)

- [x] **tarpc 0.34 → 0.37** — aligned with rhizoCrypt, biomeOS, barraCuda, coralReef; `tokio-serde` 0.9, `opentelemetry` 0.30
- [x] **Structured IPC errors** — `IpcErrorPhase` enum (Connect, Write, Read, InvalidJson, HttpStatus, NoResult, JsonRpcError) with `IntegrationError::Ipc { phase, message }` variant; aligned with rhizoCrypt + healthSpring V28
- [x] **All tarpc clients migrated** — signing, anchoring, listener clients now use `IpcErrorPhase` for connection and read errors instead of flat strings
- [x] **NDJSON streaming types** — `StreamItem` enum (Data, Progress, End, Error) with `to_ndjson_line()` / `parse_ndjson_line()`; aligned with rhizoCrypt streaming module
- [x] **`pipeline.attribute` handler** — new JSON-RPC method consuming `PipelineRequest`, creating attribution braids per agent contribution, returning `PipelineResult` with `braid_ref` *(wire types inlined in v0.7.22 — provenance-trio-types removed)*
- [x] **Smart refactor: store-postgres** — `row_mapping.rs` extracted (row_to_braid, row_to_activity, parse_activity_type, i64/u64 conversions); mod.rs 714→516 lines
- [x] 1,017 tests passing, 0 failures, 0 clippy warnings, 0 unsafe, docs build clean

### v0.7.17 — Ecosystem Absorption + Lint Tightening + Capability Evolution (March 2026)

- [x] **Lint tightening** — `unwrap_used`/`expect_used` promoted from `warn` to `deny`, matching rhizoCrypt + loamSpine trio partners
- [x] **deny.toml hardened** — `wildcards = "allow"` → `"deny"` per airSpring V084 ecosystem standard
- [x] **capability.list evolved** — added `"capabilities"` key for neuralSpring S156 ecosystem compatibility (`parse_capability_list()` compat); test coverage added
- [x] **provenance-trio-types Edition 2024** — upgraded from Edition 2021 to 2024 + MSRV 1.87 alignment across trio
- [x] **Smart refactoring** — 4 more large files extracted to `mod.rs` + `tests.rs`:
  - `anchor.rs` (687→446 production + 230 tests)
  - `activity.rs` (621→494 production + 130 tests)
  - `privacy.rs` (642→377 production + 268 tests)
  - `engine.rs` (586→300 production + 281 tests)
- [x] **primal_names evolved** — replaced 5 dead per-primal `{NAME}_SOCKET` constants with generic `socket_env_var()`/`address_env_var()` functions; any primal works without code changes
- [x] **Storage path docs** — `DEFAULT_REDB_PATH`, `DEFAULT_SLED_PATH`, `DEFAULT_DB_PATH` documented as self-config fallbacks with env override guidance
- [x] **Large file analysis** — remaining 500+ LOC files (store/mod.rs 714, config/mod.rs 630, server/mod.rs 573) confirmed as single-concern cohesive modules; no further splitting warranted
- [x] 1,004 tests passing, 0 failures, 0 clippy warnings, 0 unsafe, docs build clean

### v0.7.16 — Deep Audit Remediation + Smart Refactoring (March 2026)

- [x] **SPDX header fix** — `memory/tests.rs` was the only file missing `SPDX-License-Identifier: AGPL-3.0-only`; now all 122 `.rs` files have headers
- [x] **`sign_placeholder` isolated** — gated behind `#[cfg(any(test, feature = "test"))]`, no mock in production code path
- [x] **Smart refactoring** — `provo.rs` (842→320 production + 522 tests), `session.rs` (759→329 production + 430 tests) extracted to `mod.rs` + `tests.rs` pattern
- [x] **Postgres placeholder evolved** — `test_health_check` renamed to `test_store_connectivity_via_count`, placeholder comment removed
- [x] **Testing constants documented** — `TEST_REST_URL`, `TEST_TARPC_ADDR`, `TEST_TARPC_URI` documented as mock fixture data (never bound to)
- [x] **rustfmt.toml edition** — documented Edition 2024 mismatch (stable rustfmt does not yet support `edition = "2024"`)
- [x] **Dependency audit verified** — zero C/C++ deps in production; `ring`/`cc` only in dev via testcontainers→bollard chain
- [x] **ROADMAP accuracy** — clarified `bytes::Bytes` status (integration tarpc clients already use it)
- [x] Full audit: 0 clippy warnings, 0 unsafe, 0 TODOs, 0 production unwraps, all files <1000 LOC, 1,001 tests passing

### v0.7.15 — Deep Debt Evolution + Coverage Expansion + Convergence Spec (March 2026)

- [x] **DI pattern extended** — `SweetGrassConfig::load_with_reader()`, `BraidStoreFactory::config_from_reader()`, `PostgresConfig::from_reader()`, server tests migrated to builder pattern
- [x] **All remaining unsafe eliminated** — 5 additional test files migrated to DI (factory, server, config, postgres, discovery), zero unsafe in entire workspace
- [x] **Coverage expansion** — redb error paths, entity decode, session DAG, PROV-O export, all tested
- [x] **PostgreSQL integration tests** — queries, schema, activities, concurrency modules implemented via `testcontainers`
- [x] **Smart refactor** — `memory/mod.rs` tests extracted to `memory/tests.rs` (717→246 LOC)
- [x] **deploy.sh hardened** — removed hardcoded DB credentials, fail-fast on missing `DATABASE_URL`, default port auto-allocate
- [x] **Hardcoding audit** — all production code confirmed env-driven and capability-based, mocks gated behind `#[cfg(test)]`
- [x] **Content Convergence specification** — `specs/CONTENT_CONVERGENCE.md`, ISSUE-013 in wateringHole, experiment guide for Springs
- [x] 1,001 tests passing (was 933), 0 failures, 0 clippy warnings, 0 unsafe

### v0.7.14 — DI Pattern + Unsafe Elimination + Dynamic Reconnection (March 2026)

- [x] **DI-based environment reading** — `SelfKnowledge::from_reader()`, `infant_bootstrap_with_config_and_reader()`, `check_integrations_with_reader()` for testable env-free code paths
- [x] **Unsafe code eliminated from tests** — All `unsafe { std::env::set_var }` and `std::env::remove_var` removed from `primal_info.rs`, `bootstrap.rs`, `health.rs`, `uds.rs` via DI pattern
- [x] **`#[serial_test::serial]` removed** — Tests now thread-safe via injected readers, no global env mutation
- [x] **Dynamic reconnection** — `AnchorManager::reconnect()` and `EventHandler::reconnect()` use `parking_lot::RwLock` for hot-swappable clients via capability discovery
- [x] **Resilience compile-time safety** — `with_resilience()` refactored to eliminate `unwrap()` and `#[allow]` via `try_once()` helper
- [x] **UDS explicit-path API** — `start_uds_listener_at()` and `cleanup_socket_at()` for direct path control
- [x] **Redundant tests consolidated** — 8 duplicative env-based tests removed; remaining tests more robust
- [x] 933 tests passing (was 941), 0 clippy warnings, 0 unsafe in tests or production

### v0.7.13 — Self-Knowledge Module + Resilience + biomeOS Deploy (March 2026)

- [x] **niche.rs self-knowledge module** — `sweet_grass_core::niche` with CAPABILITIES (21 methods), CONSUMED_CAPABILITIES (8), DEPENDENCIES (4), operation_dependencies(), cost_estimates(), semantic_mappings()
- [x] **primal_names.rs** — centralized external primal identifiers (rhizocrypt, loamspine, beardog, nestgate, songbird, toadstool, squirrel, biomeos) + env var constants
- [x] **config/capability_registry.toml** — biomeOS-compatible capability registry with all 21 methods, 8 domains, per-operation depends_on/cost
- [x] **graphs/sweetgrass_deploy.toml** — biomeOS BYOB deploy graph with dependency ordering
- [x] **UniBin subcommands** — `sweetgrass capabilities` (offline capability dump) + `sweetgrass socket` (print resolved socket path)
- [x] **SocketConfig DI pattern** — `resolve_socket_path_with(config)` for env-free socket resolution in tests (airSpring V082 pattern)
- [x] **Resilience module** — CircuitBreaker + RetryPolicy + with_resilience() async helper for trio partner IPC
- [x] **#[non_exhaustive]** on ALL 10 error enums across workspace
- [x] **ServiceError::Transport and ServiceError::Discovery** — new IPC error variants
- [x] **capability.list evolution** — now delegates to niche.rs, includes consumed_capabilities and cost_estimates in response
- [x] **UDS module** uses primal_names::env_vars constants instead of string literals
- [x] 941 tests passing (was 903), 0 clippy warnings, 0 rustfmt issues, 0 cargo doc warnings, Edition 2024, MSRV 1.87

### v0.7.12 — Edition 2024 Migration + Spring Absorption + Chaos Tests (March 2026)

- [x] Edition 2024 + MSRV 1.87 + resolver 3
- [x] Let-chains adoption (8 collapsible_if patterns modernized)
- [x] Test env var safety (unsafe wrappers, cfg_attr forbid/deny pattern)
- [x] `capability.list` evolved with dependency/cost metadata (airSpring niche pattern)
- [x] 11 chaos/fault tests for attribution weights (groundSpring pattern)
- [x] Remaining hardcoded paths extracted to identity constants
- [x] `#[expect(reason)]` for benchmark lint suppressions
- [x] 903 tests passing (was 892), 0 failures, 0 unsafe, 0 clippy warnings

### v0.7.11 — JSON-RPC 2.0 Spec Compliance + Deep Debt + Coverage Push (March 2026)

- [x] JSON-RPC 2.0 batch request support (spec Section 6: array of requests/responses)
- [x] JSON-RPC 2.0 notification support (spec Section 4.1: absent `id` = no response)
- [x] `JsonRpcResponse`/`JsonRpcError` evolved to `Serialize + Deserialize` (`Cow<'static, str>` for version)
- [x] Hardcoded constants extracted: `UNKNOWN_AGENT_DID`, `MIME_MERKLE_ROOT`, `MIME_OCTET_STREAM`, `DEFAULT_STORAGE_BACKEND`, `DEFAULT_DB_PATH`
- [x] UDS transport evolved to use `process_single` (notification-aware)
- [x] Property-based tests (proptest): 6 strategies for `BraidId`, `ContentHash`, `Did`, hex, Braid builder, Arc clone
- [x] `capability.list` handler coverage: 4 tests (domains, methods, grouping, count)
- [x] Health handler coverage: 8 new tests (error paths, `PrimalStatus`, integrations)
- [x] `MemoryStore` coverage: 18 new tests (batch ops, edge cases, error paths, indexing)
- [x] Protocol-level test extraction: `tests_protocol.rs` (batch, notification, capability)
- [x] Smart refactor: `jsonrpc/tests.rs` 1053→768 LOC + `tests_protocol.rs` 302 LOC
- [x] 892 tests passing (was 847), 0 failures, 0 unsafe, 0 clippy warnings

### v0.7.10 — Typed Error Evolution + Lint Hardening + Platform-Agnostic IPC (March 2026)

- [x] `Result<_, String>` → typed error enums: `HexDecodeError`, `BootstrapEnvError`, `HealthCheckError`
- [x] Workspace lints: `missing_const_for_fn` and `missing_errors_doc` promoted from `allow` to `warn`
- [x] ~40 `missing_errors_doc` warnings resolved (added `# Errors` sections)
- [x] ~20 `missing_const_for_fn` warnings resolved (functions marked `const`)
- [x] UDS fallback paths: `/tmp` → `std::env::temp_dir()` (platform-agnostic)
- [x] `BraidFactory::sign()` → `sign_placeholder()` (naming clarity)
- [x] `config/tests.rs` flattened (removed `module_inception`)
- [x] `doc_markdown` cleanup across integration, postgres, service, benchmark crates
- [x] 847 tests passing, 0 failures, 0 unsafe, 0 clippy warnings

### v0.7.9 — Deep Debt Audit: Pedantic Quality + Capability Discovery + Spec Evolution (March 2026)

- [x] `capability.list` JSON-RPC method — wateringHole `SPRING_AS_NICHE_DEPLOYMENT_STANDARD` compliance
- [x] `#![warn(missing_docs)]` on all 10 crates (was 5)
- [x] `doc_markdown` lint enabled — all backtick warnings fixed via `cargo clippy --fix`
- [x] Cargo metadata (`readme`, `keywords`, `categories`) on all 10 crates
- [x] Copyright notice (`Copyright (C) 2024–2026 ecoPrimals Project`) on all 112 source files
- [x] `test-support` feature renamed to `test` per clippy::cargo (14 files)
- [x] `config.rs` (879L) smart-refactored → `config/mod.rs` (455L) + `config/tests.rs` (271L)
- [x] PostgreSQL test URLs centralized with env-var fallback pattern
- [x] `specs/SWEETGRASS_SPECIFICATION.md` Section 8.1 evolved: gRPC/protobuf → tarpc + JSON-RPC 2.0
- [x] `specs/SWEETGRASS_SPECIFICATION.md` Section 12 roadmap updated to reflect v0.7.x reality
- [x] `deploy.sh` evolved from hardcoded port to env-var based (`SWEETGRASS_HTTP_PORT`)
- [x] Redundant `#![allow]` removed from 3 crate lib.rs files (workspace lints handle them)
- [x] 857 tests passing (was 853), 0 failures, 0 unsafe, 0 clippy warnings

### v0.7.8 — Deep Debt Evolution: Zero-Copy + Benchmarks + Config (March 2026)

- [x] `ActivityId(String)` → `ActivityId(Arc<str>)` — O(1) clone, consistent zero-copy strategy
- [x] `BraidSignature` fields → `Cow<'static, str>` — zero heap allocation for static values
- [x] `BraidContext.imports` → `IndexMap` — deterministic JSON-LD serialization
- [x] `#[allow]` → `#[expect(..., reason)]` — ~50+ attributes across all 10 crates
- [x] Primal identity constants centralized (`identity::PRIMAL_NAME`, `PRIMAL_DISPLAY_NAME`)
- [x] Test address constants centralized (`TEST_BIND_ADDR`, `TEST_REST_URL`, etc.)
- [x] Criterion benchmarks — 7 groups (braid, store, hash, query, attribution, compression, traversal)
- [x] TOML config file support — `SweetGrassConfig::load()`, XDG-compliant, env > file > defaults
- [x] Smart refactoring: `factory.rs` 820→310+330, `listener/mod.rs` 703→320+testing+tests
- [x] 853 tests passing (was 849), 0 failures, 0 unsafe

### v0.7.7 — Deep Audit + Architecture Fix + UniBin Compliance (March 2026)

- [x] **CRITICAL**: `SweetGrassServer` evolved from `Arc<MemoryStore>` to `Arc<dyn BraidStore>` — tarpc now shares the same store as HTTP/JSON-RPC
- [x] `SweetGrassServer::from_app_state()` constructor — single shared state across all transports
- [x] Binary renamed from `sweet-grass-service` to `sweetgrass` (UniBin compliance)
- [x] `Box<dyn Error>` eliminated from production — `start_tarpc_server()`, `start_uds_listener()`, `handle_uds_connection()`, `http_health_check()` all use typed errors
- [x] `ServiceError::Io` variant added for IO error coverage
- [x] `specs/ARCHITECTURE.md` rewritten — removed stale gRPC/proto/GraphQL references, aligned with actual 10-crate structure
- [x] Flaky sled corruption test fixed (proper db handle flush + drop before re-open)
- [x] Clippy `--all-targets --all-features -D warnings` clean (was failing on scyborg.rs tests, discovery tests, server tests, state tests)
- [x] Root docs and deploy script updated for `sweetgrass` binary name
- [x] 849 tests passing (was 843), 0 failures, 0 unsafe

### v0.7.6 — redb Migration: Pure Rust Storage Evolution (March 2026)

- [x] `sweet-grass-store-redb` crate — full `BraidStore` implementation (redb 2.4, 42 tests)
- [x] `STORAGE_BACKEND=redb` in factory (env + config)
- [x] sled feature-gated: `--features sled` opt-in (was default)
- [x] 843 tests passing (was 794), 10 crates (was 9)
- [x] Follows rhizoCrypt/LoamSpine proven redb migration pattern

### v0.7.5 — Sovereignty Hardening + Coverage Push (March 2026)

- [x] JSON-RPC methods evolved to snake_case per `SEMANTIC_METHOD_NAMING_STANDARD`
- [x] `SongbirdDiscovery` → `RegistryDiscovery` (vendor-agnostic discovery)
- [x] UDS socket path derived from `SelfKnowledge` (was hardcoded)
- [x] tarpc `max_concurrent_requests` configurable via env/builder (was hardcoded)
- [x] `#[allow]` → `#[expect(..., reason)]` audit (11 production attrs evolved)
- [x] Safe casts: `as u64` → `u64::try_from()` (postgres store, signer client)
- [x] 34 new tests (760 → 794), region coverage 91%, line coverage 89%
- [x] `cargo-deny` fully passing (dev-only advisory ignores documented)
- [x] `# Errors` doc sections on anchor, listener, signer, discovery APIs

### v0.7.4 — Deep Debt: parking_lot + Idiomatic Refactor (March 2026)

- [x] `parking_lot::RwLock` migration (MemoryStore, Indexes, mock impls — eliminates all lock poisoning)
- [x] Infallible index operations (removed `Result` wrapping from all `Indexes` methods)
- [x] `DEFAULT_QUERY_LIMIT` centralized in `sweet-grass-store::traits` (removed sled/postgres duplication)
- [x] `SIGNING_ALGORITHM` constant extracted (was hardcoded `"Ed25519Signature2020"`)
- [x] JSON-RPC `error_code` module promoted to `pub(crate)` (UDS uses named constants, not magic numbers)
- [x] Status subcommand evolved to real HTTP `/health` check (was raw TCP)
- [x] Attribution tests extracted to `tests.rs` (786→302L production + 484L tests)
- [x] Clippy 0 warnings, all files under 1000 LOC
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

### v0.8.0 — Real Deployment + Content Convergence Phase 1 (Q2 2026)

**Goal**: Connect to production-deployed primals, implement convergence tracking

- [ ] Connect to deployed signing service (via Capability::Signing)
- [ ] Connect to deployed session events service (via Capability::SessionEvents)
- [ ] Connect to deployed anchoring service (via Capability::Anchoring)
- [ ] End-to-end multi-primal integration testing
- [ ] Chemistry entity types for wetSpring (Molecule, BasisSet, DftCampaign)
- [ ] Chemistry braid relations (DependsOn, ValidatedBy, ComputedWith, TrainedOn)
- [ ] `ContentConvergence` and `ConvergentArrival` types in `sweet-grass-core`
- [ ] Evolved `MemoryStore` indexes (collision-preserving)
- [ ] `convergence.query` JSON-RPC method
- [ ] PostgreSQL `content_convergence` table and migrations
- [x] Coverage target: 90%+ with llvm-cov *(region coverage 91% in v0.7.5)*

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
- [x] Coverage to 90%+ *(region coverage 91% in v0.7.5)*
- [ ] Expand chaos testing scenarios
- [ ] Property-based testing expansion (proptest)
- [ ] Fuzz testing campaigns
- [ ] Load testing for production scenarios

### Performance
- [x] Zero-copy expansion (ContentHash to newtype with Arc<str>) *(done in v0.7.1)*
- [x] Zero-copy: integration tarpc clients use `bytes::Bytes` for wire payloads (signer, anchor, listener)
- [x] Zero-copy: evolve `Braid.mime_type` from `String` to `Arc<str>` *(done in v0.7.21 — cross-crate migration across 7 crates)*
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
| v0.7.26 | **March 2026** | Ecosystem Absorption: scyBorg License, Sled Deprecation, Lint Evolution (DONE) |
| v0.7.25 | March 2026 | Coverage Push, Test Hygiene, PUBLIC_SURFACE_STANDARD Compliance (DONE) |
| v0.7.24 | March 2026 | Deep Debt: Zero-Copy Phase 2, Public Surface, Audit (DONE) |
| v0.7.23 | March 2026 | Ecosystem Absorption: MCP Tool Exposure, Canonical Capabilities (DONE) |
| v0.7.22 | March 2026 | Sovereignty: Remove provenance-trio-types, Inline Wire Types (DONE) |
| v0.7.21 | March 2026 | Deep Audit: Zero-Copy, Handler Coverage, Test Refactor (DONE) |
| v0.7.20 | March 2026 | Ecosystem Absorption: IPC Timeout, extract_rpc_error, Proptest (DONE) |
| v0.7.19 | March 2026 | Ecosystem Absorption: Health Probes, DispatchOutcome, OrExit (DONE) |
| v0.7.18 | March 2026 | Deep Execution: tarpc 0.37 + Structured IPC + Pipeline Integration (DONE) |
| v0.7.17 | March 2026 | Ecosystem Absorption + Lint Tightening + Capability Evolution (DONE) |
| v0.7.16 | March 2026 | Deep Audit Remediation + Smart Refactoring (DONE) |
| v0.7.15 | March 2026 | Deep Debt Evolution + Coverage Expansion + Convergence Spec (DONE) |
| v0.7.14 | March 2026 | DI Pattern + Unsafe Elimination + Dynamic Reconnection (DONE) |
| v0.7.13 | March 2026 | Self-Knowledge Module + Resilience + biomeOS Deploy (DONE) |
| v0.7.12 | March 2026 | Edition 2024 Migration + Spring Absorption + Chaos Tests (DONE) |
| v0.7.11 | March 2026 | JSON-RPC 2.0 Spec Compliance + Deep Debt + Coverage Push (DONE) |
| v0.7.10 | March 2026 | Typed Error Evolution + Lint Hardening + Platform-Agnostic IPC (DONE) |
| v0.7.9 | March 2026 | Deep Debt Audit: Pedantic Quality + Capability Discovery (DONE) |
| v0.7.8 | March 2026 | Deep Debt Evolution: Zero-Copy + Benchmarks + Config (DONE) |
| v0.7.7 | March 2026 | Deep Audit + Architecture Fix + UniBin (DONE) |
| v0.7.6 | March 2026 | redb Migration: Pure Rust Storage Evolution (DONE) |
| v0.7.5 | March 2026 | Sovereignty Hardening + Coverage Push (DONE) |
| v0.7.4 | March 2026 | Deep Debt: parking_lot + Refactor (DONE) |
| v0.7.3 | March 2026 | Audit + Coverage (DONE) |
| v0.7.2 | March 2026 | Provenance Trio + biomeOS IPC (DONE) |
| v0.7.1 | March 2026 | Standards + Zero-Copy Evolution (DONE) |
| v0.7.0 | March 2026 | Deep Remediation (DONE) |
| v0.8.0 | Q2 2026 | Real Deployment |
| v0.9.0 | Q3 2026 | sunCloud Integration |
| v1.0.0 | Q4 2026 | Production GA |
