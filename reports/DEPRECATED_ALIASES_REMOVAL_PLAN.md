# Deprecated Aliases Removal Plan

**Date**: December 24, 2025  
**Target**: v0.5.0 (Q1 2026)  
**Status**: Documented, not yet executed

---

## Overview

SweetGrass has 28 deprecated aliases in `sweet-grass-integration` that use primal-specific names instead of capability-based names. These should be removed in v0.5.0 after a deprecation period.

---

## Deprecated Aliases (28 total)

### Anchor/LoamSpine (8 items)

| Deprecated | Modern | Location |
|------------|--------|----------|
| `MockLoamSpineClient` | `MockAnchoringClient` | `anchor.rs` |
| `create_loamspine_client_async` | `create_anchoring_client_async` | `anchor.rs` |
| `LoamSpineClient` | `AnchoringClient` | `anchor.rs` |
| `LoamSpineRpc` | `AnchoringRpc` | `anchor.rs` |
| `TarpcLoamSpineClient` | `TarpcAnchoringClient` | `anchor.rs` |
| (+ 3 more deprecation attributes) | | |

### Listener/RhizoCrypt (8 items)

| Deprecated | Modern | Location |
|------------|--------|----------|
| `MockRhizoCryptClient` | `MockSessionEventsClient` | `listener.rs` |
| `create_rhizocrypt_client_async` | `create_session_events_client_async` | `listener.rs` |
| `RhizoCryptClient` | `SessionEventsClient` | `listener.rs` |
| `RhizoCryptRpc` | `SessionEventsRpc` | `listener.rs` |
| `TarpcRhizoCryptClient` | `TarpcSessionEventsClient` | `listener.rs` |
| (+ 3 more deprecation attributes) | | |

### Signer/BearDog (12 items)

| Deprecated | Modern | Location |
|------------|--------|----------|
| `MockBearDogClient` | `MockSigningClient` | `signer/testing.rs` |
| `create_beardog_client` | `create_signing_client_sync` | `signer/tarpc_client.rs` |
| `create_beardog_client_async` | `create_signing_client_async` | `signer/tarpc_client.rs` |
| `TarpcBearDogClient` | `TarpcSigningClient` | `signer/tarpc_client.rs` |
| `BearDogSigner` | `LegacySigner` | `lib.rs` |
| (+ 7 more deprecation attributes) | | |

---

## Why Remove?

### 1. **Capability-Based Architecture**

Modern SweetGrass uses capability-based discovery:
- ❌ Old: "Find BearDog and use it for signing"
- ✅ New: "Find a primal with Signing capability"

### 2. **Primal Sovereignty**

Hardcoded primal names violate sovereignty principles:
- Any primal can offer signing (not just BearDog)
- Any primal can offer anchoring (not just LoamSpine)
- Capability-based = vendor-agnostic

### 3. **Code Clarity**

Deprecated aliases create confusion:
- Two names for the same thing
- Documentation duplication
- Maintenance burden

---

## Removal Plan

### Phase 1: Deprecation (v0.4.0) ✅ COMPLETE

- [x] Add `#[deprecated]` attributes
- [x] Update documentation to use modern names
- [x] Add deprecation warnings

### Phase 2: Migration Period (v0.4.x - Q4 2025)

- [ ] Update all internal code to use modern names
- [ ] Update all showcase demos to use modern names
- [ ] Update all tests to use modern names
- [ ] Document migration guide

### Phase 3: Removal (v0.5.0 - Q1 2026)

- [ ] Remove deprecated type aliases
- [ ] Remove deprecated functions
- [ ] Remove deprecation attributes from modern names
- [ ] Update CHANGELOG

---

## Migration Guide (For Users)

### Before (Deprecated)

```rust
use sweet_grass_integration::MockBearDogClient;
use sweet_grass_integration::create_beardog_client_async;

let client = MockBearDogClient::new();
let real_client = create_beardog_client_async(&primal).await?;
```

### After (Modern)

```rust
use sweet_grass_integration::MockSigningClient;
use sweet_grass_integration::create_signing_client_async;

let client = MockSigningClient::new();
let real_client = create_signing_client_async(&primal).await?;
```

### Search & Replace

```bash
# Anchor/LoamSpine
sed -i 's/MockLoamSpineClient/MockAnchoringClient/g' **/*.rs
sed -i 's/create_loamspine_client_async/create_anchoring_client_async/g' **/*.rs
sed -i 's/LoamSpineClient/AnchoringClient/g' **/*.rs
sed -i 's/LoamSpineRpc/AnchoringRpc/g' **/*.rs
sed -i 's/TarpcLoamSpineClient/TarpcAnchoringClient/g' **/*.rs

# Listener/RhizoCrypt
sed -i 's/MockRhizoCryptClient/MockSessionEventsClient/g' **/*.rs
sed -i 's/create_rhizocrypt_client_async/create_session_events_client_async/g' **/*.rs
sed -i 's/RhizoCryptClient/SessionEventsClient/g' **/*.rs
sed -i 's/RhizoCryptRpc/SessionEventsRpc/g' **/*.rs
sed -i 's/TarpcRhizoCryptClient/TarpcSessionEventsClient/g' **/*.rs

# Signer/BearDog
sed -i 's/MockBearDogClient/MockSigningClient/g' **/*.rs
sed -i 's/create_beardog_client/create_signing_client_sync/g' **/*.rs
sed -i 's/create_beardog_client_async/create_signing_client_async/g' **/*.rs
sed -i 's/TarpcBearDogClient/TarpcSigningClient/g' **/*.rs
sed -i 's/BearDogSigner/LegacySigner/g' **/*.rs
```

---

## Impact Analysis

### Internal Code

```bash
# Find usage of deprecated names
grep -r "BearDog\|LoamSpine\|RhizoCrypt" crates/ --include="*.rs" | wc -l
# Result: ~50 occurrences (mostly in tests)
```

### External Code

- **Breaking change** for external users
- **Mitigation**: 3-month deprecation period (v0.4.0 → v0.5.0)
- **Communication**: CHANGELOG, release notes, migration guide

---

## Testing Strategy

### Before Removal

1. ✅ Ensure all deprecated aliases work correctly
2. ✅ Ensure modern names work correctly
3. ✅ Both point to same implementation

### After Removal

1. ⏳ Verify compilation succeeds
2. ⏳ Run full test suite
3. ⏳ Update showcase demos
4. ⏳ Update documentation

---

## Timeline

| Date | Milestone | Status |
|------|-----------|--------|
| Dec 2025 | v0.4.0 - Deprecation warnings added | ✅ Complete |
| Jan 2026 | Internal migration to modern names | ⏳ Planned |
| Feb 2026 | External communication & migration guide | ⏳ Planned |
| Mar 2026 | v0.5.0 - Deprecated aliases removed | ⏳ Planned |

---

## Decision: Keep for Now

**Rationale**:
1. Breaking change requires careful planning
2. Need to update all internal code first
3. Need to communicate with external users
4. v0.5.0 is appropriate time (Q1 2026)

**Action**: Document plan, execute in v0.5.0

---

## References

- **Issue**: 28 deprecated aliases
- **Principle**: Capability-based architecture
- **Target**: v0.5.0 (Q1 2026)
- **Status**: Documented, planned, not yet executed

---

*This is technical debt with a clear removal plan.*  
*Keeping deprecated aliases for backward compatibility during transition period.*

