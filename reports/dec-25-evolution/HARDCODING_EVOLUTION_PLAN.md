# 🌾 Hardcoding Evolution Plan — SweetGrass

**Date**: December 25, 2025  
**Target**: v0.5.0 (Q1 2026)  
**Status**: In Progress  
**Principle**: **Infant Discovery** — Zero knowledge at birth, discover everything

---

## EXECUTIVE SUMMARY

SweetGrass currently has **7 hardcoding violations** that must be evolved to full Infant Discovery compliance. Each primal must start with zero knowledge and discover the network through the universal adapter (Songbird).

### Current Status

| Category | Violations | Status | Priority |
|----------|-----------|--------|----------|
| **Primal Names** | 3 instances | 🔴 Critical | P0 |
| **Port Numbers** | 4 instances | 🟡 Medium | P1 |
| **Vendor Names** | 1 instance | 🟢 Low | P2 |
| **Total** | **8** | **In Progress** | |

---

## PRINCIPLE: INFANT DISCOVERY

### What Infant Discovery Means

```
AT BIRTH:
❌ "Connect to BearDog at beardog:8091 for signing"
❌ "RhizoCrypt sessions come from rhizocrypt:8092"
❌ "Use port 8080 for REST API"

✅ Read environment: PRIMAL_NAME, PRIMAL_CAPABILITIES
✅ Discover network via: SONGBIRD_ADDRESS or local fallback
✅ Find capabilities: "signing", "session_events", not primal names
✅ Allocate ports: OS-assigned (port 0) or environment-specified
```

### Why This Matters

1. **N² Connection Problem**: With 8 primals, hardcoding = 64 connections to manage
2. **Universal Adapter**: With Songbird, = 8 connections (each primal ↔ Songbird)
3. **Deployment Flexibility**: Same binary works in dev, staging, prod, multi-tower
4. **Primal Sovereignty**: No vendor lock-in, no assumption about network topology

---

## VIOLATION 1: 🔴 CRITICAL — "rhizoCrypt" Hardcoding

### Location
`crates/sweet-grass-compression/src/engine.rs:213`

```rust
// ❌ CURRENT (hardcoded primal name)
let ecop = EcoPrimalsAttributes {
    source_primal: Some("rhizoCrypt".to_string()),
    rhizo_session: Some(session.id.clone()),
    // ...
};
```

### Problem
- Assumes sessions always come from a primal named "rhizoCrypt"
- Violates capability-based architecture
- Any primal with `SessionEvents` capability should work

### Solution
```rust
// ✅ EVOLVED (capability-based)
pub struct CompressionEngine {
    factory: Arc<BraidFactory>,
    config: CompressionConfig,
    source_primal: String,  // NEW: from SelfKnowledge or discovery
}

impl CompressionEngine {
    pub fn new(factory: Arc<BraidFactory>, config: CompressionConfig) -> Self {
        Self {
            factory,
            config,
            source_primal: "unknown".to_string(),  // Default
        }
    }
    
    // NEW: Accept source from discovered primal
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_primal = source.into();
        self
    }
}

// Usage in compression:
let ecop = EcoPrimalsAttributes {
    source_primal: Some(self.source_primal.clone()),
    rhizo_session: Some(session.id.clone()),
    // ...
};
```

### Migration Path
1. Add `source_primal` field to `CompressionEngine`
2. Accept from discovery: `engine.with_source(discovered_primal.name)`
3. Default to "unknown" if not provided
4. Update all call sites to pass discovered source
5. Remove hardcoded "rhizoCrypt" string

**Status**: 🔴 Not started  
**Blocker**: None  
**Effort**: 2 hours

---

## VIOLATION 2: 🟡 MEDIUM — Test Port Hardcoding

### Locations
1. `crates/sweet-grass-integration/src/listener.rs:652`
2. `crates/sweet-grass-integration/src/anchor.rs:597`
3. `crates/sweet-grass-integration/src/discovery.rs:500-501`
4. `crates/sweet-grass-service/src/handlers/health.rs:370`

```rust
// ❌ CURRENT (hardcoded test ports)
let test_address = std::env::var("TEST_SESSION_EVENTS_ADDR")
    .unwrap_or_else(|_| "localhost:8092".to_string());

let test_address = std::env::var("TEST_ANCHORING_ADDR")
    .unwrap_or_else(|_| "localhost:8093".to_string());

tarpc_address: Some(format!("{name}:8091")),
rest_address: Some(format!("{name}:8080")),
```

### Problem
- Test code assumes specific ports (8091-8093, 8080)
- Causes port conflicts in CI/CD pipelines
- Violates zero-knowledge principle even in tests

### Solution
```rust
// ✅ EVOLVED (OS-allocated ports)
use std::net::TcpListener;

// Helper function for tests
fn allocate_test_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("OS port allocation")
        .local_addr()
        .expect("local address")
        .port()
}

// Usage in tests:
let tarpc_port = allocate_test_port();
let rest_port = allocate_test_port();

let primal = DiscoveredPrimal {
    name: "test-primal".to_string(),
    tarpc_address: Some(format!("localhost:{tarpc_port}")),
    rest_address: Some(format!("localhost:{rest_port}")),
    // ...
};
```

### Alternative: Environment Variables Only
```rust
// ✅ EVOLVED (require env vars in tests, no defaults)
let test_address = std::env::var("TEST_SESSION_EVENTS_ADDR")
    .expect("TEST_SESSION_EVENTS_ADDR must be set for integration tests");
```

### Migration Path
1. Create `allocate_test_port()` helper in test utils
2. Update all test code to use OS-allocated ports
3. Update test documentation
4. Remove hardcoded port fallbacks

**Status**: 🟡 Not started  
**Blocker**: None  
**Effort**: 3 hours

---

## VIOLATION 3: 🟢 LOW — Vendor Name Hardcoding

### Location
`crates/sweet-grass-service/src/factory.rs:217`

```rust
// ❌ CURRENT (hardcoded vendor name in test)
#[tokio::test]
async fn test_unknown_backend_specific_message() {
    std::env::set_var("STORAGE_BACKEND", "redis");
    let result = BraidStoreFactory::from_env().await;
    assert!(result.is_err());
    
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("redis"));  // ❌ Hardcoded vendor name
    }
}
```

### Problem
- Test hardcodes "redis" as example vendor
- Should use generic "unknown_backend" or similar

### Solution
```rust
// ✅ EVOLVED (generic test case)
#[tokio::test]
async fn test_unknown_backend_error_message() {
    std::env::set_var("STORAGE_BACKEND", "unknown_backend");
    let result = BraidStoreFactory::from_env().await;
    assert!(result.is_err());
    
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("Unknown storage backend"));
        assert!(msg.contains("unknown_backend"));
        assert!(msg.contains("memory, postgres, sled"));  // List valid options
    }
}
```

### Migration Path
1. Change test to use generic "unknown_backend"
2. Update assertion to check error message structure
3. Remove vendor-specific name

**Status**: 🟢 Not started  
**Blocker**: None  
**Effort**: 15 minutes

---

## VIOLATION 4: 🟡 MEDIUM — Factory Source Primal Default

### Location
`crates/sweet-grass-factory/src/factory.rs:39`

```rust
// 🟡 ACCEPTABLE BUT SUBOPTIMAL
pub fn new(default_agent: Did) -> Self {
    Self {
        default_agent,
        source_primal: "sweetGrass".to_string(),  // Hardcoded
        niche: None,
    }
}
```

### Problem
- Default assumes primal is named "sweetGrass"
- Should accept from `SelfKnowledge`

### Solution
```rust
// ✅ EVOLVED (accept SelfKnowledge)
use sweet_grass_core::primal_info::SelfKnowledge;

impl BraidFactory {
    /// Create from self-knowledge (preferred).
    #[must_use]
    pub fn from_self_knowledge(
        default_agent: Did,
        self_knowledge: &SelfKnowledge,
    ) -> Self {
        Self {
            default_agent,
            source_primal: self_knowledge.name.clone(),
            niche: None,
        }
    }
    
    /// Create with explicit source (for testing).
    #[must_use]
    pub fn new(default_agent: Did, source_primal: impl Into<String>) -> Self {
        Self {
            default_agent,
            source_primal: source_primal.into(),
            niche: None,
        }
    }
}
```

### Migration Path
1. Add `from_self_knowledge()` constructor
2. Update service bootstrap to use it
3. Deprecate parameterless `new()` or make it require source
4. Update all call sites

**Status**: 🟡 Not started  
**Blocker**: None  
**Effort**: 1 hour

---

## COMPLIANCE VERIFICATION

### ✅ Already Compliant

These areas demonstrate **exemplary** Infant Discovery compliance:

1. **`SelfKnowledge::from_env()`** — Perfect zero-knowledge bootstrap
2. **`SongbirdDiscovery::from_env()`** — Discovers universal adapter from environment
3. **Capability-based APIs** — All integration uses capabilities, not primal names
4. **Storage backend selection** — Environment-driven, no hardcoding
5. **Port allocation** — Supports OS allocation (port 0)

### 🔍 Need Verification

1. **Documentation** — Ensure all modules explain Infant Discovery
2. **Examples** — Show capability-based patterns, not primal names
3. **Error messages** — Don't mention specific primal names in production

---

## EVOLUTION ROADMAP

### Phase 1: Critical Fixes (v0.5.0 — Feb 2026)

- [ ] Fix "rhizoCrypt" hardcoding in compression engine
- [ ] Evolve factory to use `SelfKnowledge`
- [ ] Update all examples to show capability-based discovery
- [ ] Add Infant Discovery compliance documentation

**Estimated Effort**: 1 day

### Phase 2: Test Evolution (v0.5.0 — Feb 2026)

- [ ] Remove hardcoded test ports
- [ ] Implement OS port allocation in tests
- [ ] Update test documentation
- [ ] Add test utilities for dynamic addressing

**Estimated Effort**: 0.5 days

### Phase 3: Polish (v0.5.0 — Feb 2026)

- [ ] Fix vendor name in test case
- [ ] Audit all error messages for hardcoding
- [ ] Add compliance checks to CI/CD
- [ ] Update ARCHITECTURE.md with Infant Discovery patterns

**Estimated Effort**: 0.5 days

**Total Effort**: 2 days

---

## TESTING STRATEGY

### Compliance Tests

Add automated checks for hardcoding violations:

```rust
#[test]
fn test_no_primal_name_hardcoding() {
    // Grep codebase for primal names in production code
    // Fail if found outside of:
    // - Documentation/comments
    // - Test fixtures
    // - Deprecated aliases (marked for removal)
}

#[test]
fn test_no_port_hardcoding() {
    // Verify all ports come from:
    // - Environment variables
    // - OS allocation (port 0)
    // - Configuration files
}
```

### Integration Tests

Verify Infant Discovery works end-to-end:

```rust
#[tokio::test]
async fn test_infant_bootstrap_with_zero_knowledge() {
    // Clear all environment except essentials
    std::env::remove_var("PRIMAL_NAME");
    std::env::remove_var("SONGBIRD_ADDRESS");
    
    // Bootstrap should succeed with defaults
    let self_knowledge = SelfKnowledge::from_env().unwrap();
    assert!(!self_knowledge.name.is_empty());
    assert_eq!(self_knowledge.tarpc_port, 0);  // OS-allocated
}
```

---

## SUCCESS CRITERIA

### ✅ Definition of Done

1. **Zero hardcoded primal names** in production code
2. **Zero hardcoded ports** in production code (except localhost in examples)
3. **All factories accept `SelfKnowledge`** or discovered primal info
4. **All tests use dynamic ports** or environment variables
5. **Documentation explains Infant Discovery** in all relevant modules
6. **CI/CD checks** enforce zero-hardcoding policy
7. **Phase1 primal parity** — matches BearDog, NestGate standards

---

## REFERENCES

### Related Documents
- `specs/PRIMAL_SOVEREIGNTY.md` — Core principles
- `specs/INTEGRATION_SPECIFICATION.md` — Capability-based integration
- `env.example` — Environment variable patterns
- `DEPRECATED_ALIASES_REMOVAL_PLAN.md` — Related evolution

### Phase1 Examples
- **BearDog**: Self-knowledge pattern, zero-hardcoding
- **NestGate**: Environment-driven bootstrap
- **Songbird**: Universal adapter for discovery

---

## APPENDIX: SEARCH PATTERNS

### Finding Hardcoding Violations

```bash
# Primal names (case-insensitive)
rg -i 'beardog|nestgate|rhizocrypt|loamspine|songbird|toadstool|squirrel' \
   --type rust crates/ -g '!*/tests/*' -g '!*/testing.rs'

# Port numbers
rg ':80[0-9]{2}|:30[0-9]{2}|:50[0-9]{2}' \
   --type rust crates/ -g '!*/tests/*'

# Vendor names
rg -i 'kubernetes|k8s|consul|etcd|redis|kafka' \
   --type rust crates/ -g '!*/tests/*'
```

---

**🌾 Evolution Status**: 8 violations identified, migration path clear  
**🎯 Target**: v0.5.0 (Q1 2026)  
**⏱️  Effort**: 2 days total

**Each primal knows only itself. Network effects through universal adapter.**

