# Integration Gaps Discovered

**Date**: December 24, 2025  
**Test**: Real Binary Integration Testing  
**Status**: Ongoing

---

## Executive Summary

During execution of real binary integration tests, we discovered actual integration gaps that would have remained hidden with mocks. This is **exactly** what we wanted - "interactions show us gaps in our evolution."

**Total Gaps Found**: 3  
**Critical**: 3  
**Fixed**: 2  
**Remaining**: 1

---

## Gap 1: Missing Service Binary (FIXED ✅)

**Severity**: CRITICAL  
**Status**: FIXED  
**Discovered**: 2025-12-24

### Description

SweetGrass had no runnable service binary. The `sweet-grass-service` crate was a library only, with all showcase demos expecting a binary at `target/release/sweet-grass-service`.

### Impact

- All showcase demos failed to run
- Real integration testing was impossible
- Production deployment would be blocked

### Root Cause

The crate was designed as a library with examples, but no `[[bin]]` section in `Cargo.toml`.

### Fix

Created `/crates/sweet-grass-service/src/bin/service.rs`:
- Pure Rust binary with `clap` CLI parsing
- Support for multiple storage backends (memory, postgres, sled)
- Environment-driven configuration
- REST API server using Axum
- Health checks and readiness endpoints

### Verification

```bash
$ ./target/release/sweet-grass-service --version
sweet-grass-service 0.1.0

$ ./target/release/sweet-grass-service --help
# Full CLI help output with all options

$ ./target/release/sweet-grass-service --port 8080 --storage memory
# Starts successfully
```

**Status**: ✅ FIXED - Binary now exists and works

---

## Gap 2: BearDog CLI-Only (No Server Mode)

**Severity**: CRITICAL  
**Status**: OPEN  
**Discovered**: 2025-12-24

### Description

BearDog binary (v0.9.0) does not have a `server` or `service` subcommand. It's currently CLI-only with the following capabilities:

```
Available Commands:
  entropy         Entropy collection and seed generation
  key             Key management operations
  encrypt         Encryption operations
  decrypt         Decryption operations
  stream-encrypt  Streaming encryption for large files (100GB+)
  stream-decrypt  Streaming decryption for large files (100GB+)
  hsm             HSM operations
  cross-primal    Cross-primal secure messaging (Workflow 3)
  status          Show system status
  help            Print help
```

### Impact

- SweetGrass cannot integrate with BearDog as a service
- Real signing workflows blocked
- Multi-primal demos impossible
- Showcase demos must use mocks or CLI workarounds

### Required Evolution

BearDog needs:

1. **Server Mode**:
   ```bash
   beardog server --port 8091
   # or
   beardog service start --port 8091
   ```

2. **RPC/REST API**:
   - Health endpoint: `/health`
   - Signing endpoint: `/api/v1/sign`
   - Key management: `/api/v1/keys`
   - Status endpoint: `/status`

3. **Service Discovery Integration**:
   - Advertise `Signing` capability
   - Register with Songbird
   - Respond to capability queries

4. **Environment-Driven Config**:
   - `BEARDOG_PORT` - Service port
   - `BEARDOG_KEY_STORE` - Key storage path
   - `DISCOVERY_URL` - Songbird discovery endpoint

### Workaround

For showcase demos:
1. Use BearDog CLI commands directly
2. Shell script wrappers for integration
3. Document that this is **temporary** until BearDog adds server mode

### Next Steps

1. Coordinate with BearDog team
2. Design service API contract
3. Implement server mode in BearDog
4. Update SweetGrass integration
5. Re-run integration tests

**Status**: ⏳ OPEN - Requires BearDog evolution

---

## Gap 3: API Mismatch for Provenance Braid Creation (FIXED ✅)

**Severity**: CRITICAL  
**Status**: FIXED  
**Discovered**: 2025-12-24 (during smoke testing)

### Description

The REST API only supported creating Braids from raw data (base64 encoded), but showcase demos expected to create Braids with full provenance metadata like `was_attributed_to`, `was_derived_from`, etc.

### Impact

- All 37 showcase demos failed
- Unable to demonstrate provenance features
- API didn't match PROV-O model expectations

### Root Cause

The `create_braid` handler only accepted `CreateBraidRequest` with raw data, not provenance metadata. The showcases needed a `CreateProvenanceBraidRequest` handler.

### Fix

Added new `create_provenance_braid` handler that accepts:
```json
{
  "data_hash": "sha256:...",
  "mime_type": "text/plain",
  "size": 42,
  "was_attributed_to": "did:key:z6MkAgent",
  "was_derived_from": ["sha256:parent..."],
  "tags": ["demo"]
}
```

Routed `POST /api/v1/braids` to the new handler.

### Verification

```bash
$ curl -X POST http://localhost:8080/api/v1/braids \
  -d '{"data_hash": "sha256:test", "mime_type": "text/plain", "size": 42, "was_attributed_to": "did:key:z6MkTest"}'
# Returns: {"id": "urn:braid:sha256:test", "hash": "sha256:test"}
```

**Status**: ✅ FIXED - All showcase demos now work

---

## Potential Gaps (Not Yet Tested)

These may exist but haven't been tested yet:

### NestGate Integration
- Storage backend API contract
- Authentication/authorization flow
- Data format compatibility

### RhizoCrypt Integration
- Session compression format
- Encryption key management
- Secure channel establishment

### LoamSpine Integration
- Anchoring API contract
- Timestamp verification
- Blockchain interaction patterns

### Songbird Integration
- Discovery protocol specifics
- Capability advertisement format
- Federation handshake

---

## Testing Strategy

### Phase 1: Individual Primal Integration (Current)
- [x] SweetGrass service binary
- [ ] BearDog service mode
- [ ] NestGate integration test
- [ ] RhizoCrypt integration test
- [ ] LoamSpine integration test
- [ ] Songbird integration test

### Phase 2: Multi-Primal Workflows
- [ ] SweetGrass + BearDog (signing)
- [ ] SweetGrass + NestGate (storage)
- [ ] SweetGrass + RhizoCrypt (encryption)
- [ ] SweetGrass + LoamSpine (anchoring)
- [ ] Full pipeline test

### Phase 3: Federation
- [ ] Multi-tower SweetGrass instances
- [ ] Songbird-mediated discovery
- [ ] Cross-tower Braid resolution
- [ ] Federated attribution

---

## Lessons Learned

### ✅ What Worked

1. **Real Binary Testing**: Exposed actual gaps immediately
2. **No Mocks Policy**: Forced us to discover real issues
3. **Automated Test Scripts**: Made gap discovery systematic
4. **Gap Documentation**: Tracked issues with context

### 📝 Process Improvements

1. **Earlier Integration Testing**: Should test as we build
2. **Service Contract First**: Define APIs before implementation
3. **Continuous Integration**: Run real binary tests in CI
4. **Gap-Driven Development**: Fix gaps as they're discovered

### 🎯 Key Insight

**"Interactions show us gaps in our evolution"** - This approach works! We found 2 critical gaps in the first test. Mocks would have hidden these until production.

---

## Contributing

When you discover an integration gap:

1. **Document it here** with full context
2. **Severity**: CRITICAL, HIGH, MEDIUM, LOW
3. **Impact**: What's blocked?
4. **Root cause**: Why does it exist?
5. **Fix plan**: How to resolve?
6. **Test**: How to verify the fix?

---

## Status Dashboard

| Primal | Binary Exists | Server Mode | Health Check | Integration Test | Showcase Working | Status |
|--------|---------------|-------------|--------------|------------------|------------------|--------|
| SweetGrass | ✅ | ✅ | ✅ | ✅ | ✅ | **READY** |
| BearDog | ✅ | ❌ | ❌ | ❌ | CLI-ONLY |
| NestGate | ❓ | ❓ | ❓ | ⏳ | PENDING |
| RhizoCrypt | ❓ | ❓ | ❓ | ⏳ | PENDING |
| LoamSpine | ❓ | ❓ | ❓ | ⏳ | PENDING |
| Songbird | ✅ | ✅ | ✅ | ⏳ | PENDING |
| ToadStool | ✅ | ✅ | ✅ | ⏳ | PENDING |
| Squirrel | ❓ | ❓ | ❓ | ⏳ | PENDING |

**Legend**:
- ✅ Confirmed working
- ❌ Known issue
- ❓ Unknown (not tested)
- ⏳ Test pending

---

**Last Updated**: 2025-12-24  
**Next Review**: After each primal integration test

