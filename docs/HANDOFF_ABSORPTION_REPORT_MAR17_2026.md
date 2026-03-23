# sweetGrass — Cross-Ecosystem Handoff Absorption Report

**Date**: March 17, 2026 (revised March 17, 2026)  
**Source**: wateringHole handoffs pulled March 16–17, 2026  
**sweetGrass Version**: v0.7.22  
**Purpose**: Summarize NEW handoff content and absorption priorities for sweetGrass

---

## 1. Handoff Summaries (What Changed)

### 1.1 groundSpring V112 — Deep Debt + OrExit

| Aspect | Change |
|--------|--------|
| **Version** | V111 → V112 (two sprints) |
| **Error types** | `thiserror` for InputError, BiomeOsError, IpcError |
| **RPC** | `DispatchOutcome` enum (Ok, ProtocolError, ApplicationError) |
| **Validation** | `OrExit<T>` trait, `parse_benchmark()`, 28 binaries migrated |
| **Discovery** | Generic `socket_env_var()` / `address_env_var()` |
| **Provenance trio** | RHIZOCRYPT, LOAMSPINE, SWEETGRASS in primal_names.rs |
| **Config** | `_with_env` DI pattern for env var reads |
| **Safe casts** | 25+ `as` → `crate::cast` or `#[expect(reason)]` |

### 1.2 groundSpring V112 — toadStool/barraCuda Evolution

- Same patterns; barraCuda API requests (GemmF64 transpose — **fulfilled in barraCuda v0.3.5**)
- Precision learnings: GPU ~1 ULP per transcendental, f32 accumulation biases ~28%
- 102 delegations (61 CPU + 41 GPU)

### 1.3 neuralSpring V110 — barraCuda/toadStool Evolution

| Aspect | Change |
|--------|--------|
| **Consumption** | 13+ barraCuda modules, 167 binaries, 80+ lib files |
| **Patterns** | OrExit, deny.toml, `#[expect(reason)]`, temp-env, ValidationHarness |
| **Discovery** | `discover_by_capability(cap, hint)` — never hardcoded paths |
| **blake3** | Requests `pure` feature in barraCuda (ecoBin compliance) |
| **P1 request** | sweetGrass generic `socket_env_var()` — **sweetGrass already provides** |

### 1.4 neuralSpring V110 S159 — Cross-Ecosystem Absorption Exec

- OrExit absorbed ✓
- deny.toml absorbed ✓
- Structured logging (eprintln → log::info/warn/debug) ✓
- **P1 remaining**: rhizoCrypt NDJSON, biomeOS typed CapabilityClient, **sweetGrass generic socket_env_var()** — **sweetGrass v0.7.18+ already has this**

### 1.5 barraCuda v0.3.5 — Cross-Ecosystem Absorption

| Aspect | Change |
|--------|--------|
| **GemmF64** | `execute_gemm_ex()` with transpose flags (groundSpring, airSpring request) |
| **Socket paths** | `$BIOMEOS_FAMILY_ID` in default_socket_path (PRIMAL_IPC_PROTOCOL) |
| **blake3** | `pure` feature — zero C deps |
| **deny.toml** | wildcards=deny, barracuda-core version pinned |
| **WGSL** | WGSL_MEAN_REDUCE re-exported from ops |

### 1.6 biomeOS v2.48 — Capability Registry + Evolution

| Aspect | Change |
|--------|--------|
| **Registry** | 5 new domains: compute.dispatch, secrets, relay, model, hardware |
| **Graph executor** | `fallback=skip` for optional nodes |
| **Hardcoding** | `"beardog"` → `primal_names::BEARDOG` |
| **Dependencies** | `once_cell` removed (LazyLock complete) |
| **Leverage guide** | +800 lines: per-spring recipes, emergent orchestration |

### 1.7 coralReef Phase 10 Iter 52 — Ecosystem Absorption

| Aspect | Change |
|--------|--------|
| **deny.toml** | yanked: warn → **deny** |
| **OrExit** | coralreef-core/src/or_exit.rs |
| **IpcServiceError** | `IpcPhase` enum, phase-aware JSON-RPC codes, `retryable()` |
| **Glowplug** | JSON-RPC 2.0 socket protocol |
| **GpuPersonality** | Trait-based (VfioPersonality, NouveauPersonality, etc.) |
| **Capability parsing** | Dual-format `CapabilityRef` (flat string OR nested object) |

### 1.8 squirrel v0.1.0-alpha.9 — Ecosystem Absorption

| Aspect | Change |
|--------|--------|
| **deny.toml** | yanked: warn → **deny** |
| **tarpc** | 0.34 → 0.37 |
| **#[expect]** | 52 `#[allow(dead_code)]` → `#[expect(..., reason)]` |
| **Generic discovery** | `socket_env_var("rhizocrypt")` → `"RHIZOCRYPT_SOCKET"` |
| **Capability.list** | Flat `capabilities` array, `domains`, `locality` |
| **IpcErrorPhase** | Connect, Write, Read, JsonRpcError, NoResult + `is_retryable()` |
| **NDJSON** | StreamItem / StreamKind |

### 1.9 toadStool S157b — Deep Debt + Full CI Green

| Aspect | Change |
|--------|--------|
| **Edition 2024** | `set_var`/`remove_var` unsafe — wrapped in `unsafe {}` blocks |
| **serialport** | Pure Rust (libudev removed) |
| **Tests** | 21,156+ pass, 0 failures |
| **Clippy** | 0 warnings (--all-targets) |

### 1.10 toadStool S157 — Edition 2024 + Nursery Evolution

- Edition 2021 → 2024, MSRV 1.85
- Clippy nursery lints enabled
- ~500+ violations fixed across 56 crates

---

## 2. Patterns sweetGrass Could Absorb

### Already Absorbed (sweetGrass v0.7.18–v0.7.22)

| Pattern | Source | sweetGrass Status |
|---------|--------|-------------------|
| OrExit<T> | wetSpring, groundSpring, healthSpring | ✓ `sweet-grass-service/exit.rs` |
| socket_env_var() / address_env_var() | groundSpring V112 | ✓ `primal_names.rs` |
| IpcErrorPhase | rhizoCrypt, healthSpring | ✓ `sweet-grass-integration/error.rs` |
| DispatchOutcome | rhizoCrypt, biomeOS | ✓ `sweet-grass-service/handlers/jsonrpc/mod.rs` |
| #[expect(reason)] | groundSpring, neuralSpring | ✓ Workspace-wide |
| deny.toml wildcards=deny | airSpring V084 | ✓ |
| deny.toml yanked=deny | squirrel, coralReef, barraCuda | ✓ v0.7.20 |
| FAMILY_ID in socket paths | barraCuda, squirrel | ✓ `BIOMEOS_FAMILY_ID` in uds.rs |
| NDJSON streaming | rhizoCrypt | ✓ `StreamItem` in streaming.rs |
| tarpc 0.37 | rhizoCrypt, biomeOS | ✓ |
| Edition 2024 | toadStool | ✓ |
| Zero-copy `Braid.mime_type: Arc<str>` | internal audit | ✓ v0.7.21 |
| Dual-format capability parsing | coralReef, neuralSpring S156 | ✓ v0.7.20 `extract_capabilities()` |
| Sovereign wire types (no shared crates) | primalSpring sovereignty resolution | ✓ v0.7.22 — provenance-trio-types removed, banned in deny.toml |

### Not Yet Absorbed — Priority Gaps

| Priority | Pattern | Source | Effort |
|----------|---------|--------|--------|
| **P3** | IpcServiceError `retryable()` on wire | coralReef | SweetGrass has `IntegrationError::Ipc`; consider `retryable()` helper |

**Previously P1/P2 items — now resolved:**
- deny.toml `yanked = "deny"` — done in v0.7.20
- Dual-format capability parsing — done in v0.7.20 (`extract_capabilities()`)
- temp-env — not needed; sweetGrass v0.7.14 migrated fully to DI pattern, zero unsafe env mutation in tests

---

## 3. Ecosystem Standards sweetGrass Should Align With

### 3.1 PRIMAL_IPC_PROTOCOL compliance

- **FAMILY_ID** in socket paths: ✓ sweetGrass uses `BIOMEOS_FAMILY_ID` in `resolve_socket_path`
- **JSON-RPC 2.0**: ✓ sweetGrass already compliant
- **health.liveness / health.readiness**: ✓ sweetGrass v0.7.19

### 3.2 supply-chain hygiene (deny.toml)

| Setting | Ecosystem standard | sweetGrass |
|---------|--------------------|------------|
| wildcards | deny | ✓ deny |
| yanked | deny | ✓ deny (v0.7.20) |
| vulnerability | deny | (not in deny.toml — add if missing) |
| unknown-registry | deny | ✓ deny |

### 3.3 capability.list format (consumers)

- **Flat `capabilities` array**: Squirrel, rhizoCrypt, toadStool, sweetGrass standardized
- **Dual-format**: coralReef, squirrel accept both `"gpu.dispatch"` and `{"id": "gpu.dispatch", "version": "0.1.0"}`
- sweetGrass **produces** capability.list; **consumes** only if it discovers other primals. Current discovery uses BearDog/LoamSpine/rhizoCrypt via direct IPC — no capability.list parsing from others.

### 3.4 biomeOS capability registry

- **New domains** (compute.dispatch, secrets, relay, model, hardware): sweetGrass does not provide these; no action needed
- **sweetGrass registry**: already has 8 domains (braid, anchoring, provenance, attribution, compression, contribution, health, capability); aligned

### 3.5 Provenance trio (rhizoCrypt, LoamSpine, sweetGrass)

- groundSpring V112: `RHIZOCRYPT`, `LOAMSPINE`, `SWEETGRASS` in primal_names
- sweetGrass: has RHIZOCRYPT, LOAMSPINE in primal_names; SWEETGRASS is self — no change needed

---

## 4. Absorption Priorities for sweetGrass

### All P1/P2 Resolved

As of v0.7.22, all high-priority absorption items from the original report are complete:
- deny.toml yanked=deny (v0.7.20)
- DI pattern replaces all unsafe env mutation (v0.7.14)
- Dual-format capability parsing via `extract_capabilities()` (v0.7.20)
- Sovereign wire types — provenance-trio-types removed, banned in deny.toml (v0.7.22)

### P3 — Awareness / Future

1. **biomeOS typed CapabilityClient**
   - neuralSpring, rhizoCrypt waiting on biomeOS
   - sweetGrass can consume when available

2. **Content Convergence (ISSUE-013)**
   - Collision-preserving provenance may affect barraCuda shaders::provenance
   - sweetGrass is attribution layer; monitor for downstream impact

3. **IpcServiceError `retryable()` on wire**
   - coralReef pattern; sweetGrass has `IntegrationError::Ipc` with `is_retriable()`
   - Consider exposing retryable hint over the wire if trio partners need it

---

## 5. Leverage Guide Updates (First 100 Lines)

### BIOMEOS_LEVERAGE_GUIDE.md
- **Version**: v2.47 (header may still say v2.47; handoff says v2.48)
- **New**: Section 7 (per-spring deep recipes), Section 8 (emergent orchestration)
- **Capability domains**: 19 → 24 (compute.dispatch, secrets, relay, model, hardware)
- **sweetGrass relevance**: Provenance trio (rhizoCrypt + LoamSpine + sweetGrass) wired; no new sweetGrass-specific changes

### SQUIRREL_LEVERAGE_GUIDE.md
- **Version**: v0.1.0-alpha.8 (header); handoff says alpha.9
- **Capability.list**: Now includes flat `capabilities` array, `domains`, `locality`
- **sweetGrass relevance**: Squirrel can consume sweetGrass provenance capabilities; format compatibility

### TOADSTOOL_LEVERAGE_GUIDE.md
- **Version**: S157 (March 16, 2026)
- **Edition**: 2024, MSRV 1.85, pedantic+nursery
- **sweetGrass relevance**: Provenance trio; compute.dispatch; no direct sweetGrass dependency

---

## 6. PRIMAL_REGISTRY.md (Full File)

- **sweetGrass entry**: updated to v0.7.22 (sovereign types, 1,077 tests, zero unsafe, deny.toml sovereignty guards)
- **Spring versions**: ToadStool S155b, neuralSpring V112/S161 — registry may need refresh
- **BarraCuda**: v0.3.5, blake3 pure feature
- **coralReef**: Phase 10 Iter 52

---

## 7. Summary

**sweetGrass v0.7.22 is fully aligned** with the ecosystem. All original P1/P2 items resolved:

- OrExit, socket_env_var, IpcErrorPhase, DispatchOutcome, FAMILY_ID, NDJSON, tarpc 0.37, Edition 2024 — all present since v0.7.18–v0.7.19
- deny.toml yanked=deny — v0.7.20
- Dual-format capability parsing — v0.7.20
- Zero-copy `Braid.mime_type: Arc<str>` — v0.7.21
- Sovereign wire types (provenance-trio-types removed and banned) — v0.7.22
- PRIMAL_REGISTRY.md updated to v0.7.22; genomeBin/manifest.toml updated

**Remaining P3**: biomeOS typed CapabilityClient (waiting on biomeOS), Content Convergence ISSUE-013, wire-level retryable hints.

---

**Report generated by handoff analysis of 10 wateringHole handoffs + 3 leverage guides + PRIMAL_REGISTRY.md**
