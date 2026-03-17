# sweetGrass — Cross-Ecosystem Handoff Absorption Report

**Date**: March 17, 2026  
**Source**: wateringHole handoffs pulled March 16–17, 2026  
**sweetGrass Version**: v0.7.19  
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

### Already Absorbed (sweetGrass v0.7.18–v0.7.19)

| Pattern | Source | sweetGrass Status |
|---------|--------|-------------------|
| OrExit<T> | wetSpring, groundSpring, healthSpring | ✓ `sweet-grass-service/exit.rs` |
| socket_env_var() / address_env_var() | groundSpring V112 | ✓ `primal_names.rs` |
| IpcErrorPhase | rhizoCrypt, healthSpring | ✓ `sweet-grass-integration/error.rs` |
| DispatchOutcome | rhizoCrypt, biomeOS | ✓ `sweet-grass-service/handlers/jsonrpc/mod.rs` |
| #[expect(reason)] | groundSpring, neuralSpring | ✓ Workspace-wide |
| deny.toml wildcards=deny | airSpring V084 | ✓ |
| FAMILY_ID in socket paths | barraCuda, squirrel | ✓ `BIOMEOS_FAMILY_ID` in uds.rs |
| NDJSON streaming | rhizoCrypt | ✓ `StreamItem` in streaming.rs |
| tarpc 0.37 | rhizoCrypt, biomeOS | ✓ |
| Edition 2024 | toadStool | ✓ |

### Not Yet Absorbed — Priority Gaps

| Priority | Pattern | Source | Effort |
|----------|---------|--------|--------|
| **P1** | deny.toml `yanked = "deny"` | squirrel, coralReef, barraCuda | 1-line change |
| **P2** | Dual-format capability parsing | coralReef, neuralSpring S156 | Only if sweetGrass consumes capability.list from others |
| **P2** | temp-env for env testing | neuralSpring | Replace unsafe set_var in tests |
| **P3** | IpcServiceError `retryable()` on wire | coralReef | SweetGrass has `IntegrationError::Ipc`; consider `retryable()` helper |

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
| yanked | deny | ⚠ **warn** — upgrade to deny |
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

### P1 — Immediate (1–2 days)

1. **deny.toml: yanked = "deny"**
   - Change `yanked = "warn"` → `yanked = "deny"` in deny.toml
   - Aligns with squirrel alpha.9, coralReef Iter 52, barraCuda v0.3.5
   - Run `cargo deny check` after change

### P2 — Near-term (1 sprint)

2. **temp-env for test env mutation**
   - If sweetGrass has tests using `std::env::set_var` (Rust 2024 makes it unsafe)
   - Use `temp-env = "0.3"` with `with_var()` / `with_var_unset()` for safe scoped env mutation
   - Check: sweetGrass v0.7.14 migrated to DI pattern — verify no remaining unsafe env in tests

3. **Dual-format capability parsing** (only if consuming)
   - If sweetGrass ever parses `capability.list` responses from other primals (e.g. neuralSpring, squirrel), add support for both flat and nested `provides` formats
   - Current: sweetGrass does not appear to consume capability.list from others

### P3 — Awareness / Future

4. **biomeOS typed CapabilityClient**
   - neuralSpring, rhizoCrypt waiting on biomeOS
   - sweetGrass can consume when available

5. **Content Convergence (ISSUE-013)**
   - Collision-preserving provenance may affect barraCuda shaders::provenance
   - sweetGrass is attribution layer; monitor for downstream impact

6. **Leverage guide**
   - Consider adding sweetGrass-specific section to wateringHole/SWEETGRASS_LEVERAGE_GUIDE.md (if exists) per biomeOS pattern

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

- **sweetGrass entry**: v0.7.6 (may be stale); update to v0.7.19 with recent absorption (OrExit, health probes, DispatchOutcome, IpcErrorPhase, primal_names, socket_env_var)
- **Spring versions**: ToadStool S155b, neuralSpring V112/S161 — registry may need refresh
- **BarraCuda**: v0.3.5, blake3 pure feature
- **coralReef**: Phase 10 Iter 52

---

## 7. Summary

**sweetGrass v0.7.19 is already well-aligned** with the ecosystem. The handoffs confirm:

- OrExit, socket_env_var, IpcErrorPhase, DispatchOutcome, FAMILY_ID, NDJSON, tarpc 0.37, Edition 2024 — all present
- neuralSpring P1 request for "sweetGrass generic socket_env_var()" is **already satisfied** in v0.7.18+

**Single actionable P1**: Upgrade `deny.toml` yanked from `warn` to `deny`.

**P2/P3**: temp-env (if tests still use unsafe env), dual-format capability parsing (if ever consumed), and PRIMAL_REGISTRY.md update for sweetGrass v0.7.19.

---

**Report generated by handoff analysis of 10 wateringHole handoffs + 3 leverage guides + PRIMAL_REGISTRY.md**
