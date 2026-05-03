# 🌾 SweetGrass Inter-Primal Integration

**"SweetGrass + ONE Other Primal"**

**Time**: ~60 minutes  
**Prerequisites**: Level 0 (local showcase) complete  
**Pattern**: Real binaries from ../bins, NO MOCKS

---

## 🎯 PURPOSE

Demonstrate SweetGrass integrating with individual primals using **REAL binaries**.

**Philosophy**: "Interactions show us gaps in our evolution"
- Real binaries reveal real integration issues
- Mocks hide problems until production
- We discover gaps NOW and evolve

---

## 🚀 QUICK START

### Automated Tour
```bash
./RUN_ME_FIRST.sh  # Runs all integrations (~60 min)
```

### Individual Integration
```bash
cd 04-sweetgrass-songbird
./demo-discovery-live.sh  # ~15 minutes
```

---

## 📋 AVAILABLE INTEGRATIONS

### ✅ 1. SweetGrass + Songbird (Discovery)
**Directory**: `04-sweetgrass-songbird/`  
**Binary**: `../../../bins/songbird-orchestrator` (20MB, real ELF)  
**Time**: 15 minutes  
**Status**: ✅ Working

**What it demonstrates**:
- Capability-based service discovery
- Register attribution services
- Query for provenance capabilities
- Runtime primal discovery
- **No hardcoded addresses**

**Run**:
```bash
cd 04-sweetgrass-songbird
./demo-discovery-live.sh
```

**Verification**:
- ✅ Starts real Songbird (PID captured)
- ✅ Verifies port listening (lsof)
- ✅ Registers SweetGrass capabilities
- ✅ Discovers services by capability
- ✅ Clean shutdown

---

### ✅ 2. SweetGrass + NestGate (Storage)
**Directory**: `02-sweetgrass-nestgate/`  
**Binary**: `../../../bins/nestgate` (3.4MB, real ELF)  
**Time**: 15 minutes  
**Status**: ✅ Working

**What it demonstrates**:
- Persistent Braid storage
- ZFS snapshot integration
- Distributed storage provenance
- Cross-primal data access

**Run**:
```bash
cd 02-sweetgrass-nestgate
./demo-storage-live.sh
```

**Verification**:
- ✅ Starts real NestGate (PID captured)
- ✅ Stores Braids in NestGate
- ✅ Retrieves from distributed storage
- ✅ Provenance tracked across primal boundary

---

### ✅ 3. SweetGrass + ToadStool (Compute)
**Directory**: `02-ml-training-provenance/`  
**Binary**: `../../../bins/toadstool-cli` (21MB, real ELF)  
**Time**: 15 minutes  
**Status**: 🟡 Partial (needs enhancement)

**What it demonstrates**:
- Compute task provenance
- ML training attribution
- GPU workload tracking
- Task execution graphs

**Run**:
```bash
cd 02-ml-training-provenance
./demo-ml-provenance.sh
```

**Note**: Currently uses ToadStool CLI. Could be enhanced to use `toadstool-byob-server` for full integration.

---

### ✅ 4. SweetGrass + Squirrel (AI Agents)
**Directory**: `05-sweetgrass-squirrel/` (to be created)  
**Binary**: `../../../bins/squirrel` (12MB, real ELF)  
**Time**: 15 minutes  
**Status**: 📋 Planned

**What it would demonstrate**:
- AI agent activity provenance
- Agent decision attribution
- Multi-agent collaboration tracking
- Agent genealogy

**Planned**:
```bash
cd 05-sweetgrass-squirrel
./demo-agent-provenance.sh
```

---

### ✅ 5. SweetGrass + BearDog (Signing)
**Directory**: `01-sweetgrass-beardog/`  
**Binary**: `../../../bins/beardog` (4.5MB, real ELF)  
**Time**: 15 minutes  
**Status**: ✅ **RESOLVED** (v0.7.28+ — `crypto.sign` delegation, BTSP Phase 3 in v0.7.29)

**What it demonstrates**:
- Braid signing with Ed25519 (Tower-delegated via `CryptoDelegate`)
- DID resolution (`Did::from_public_key_bytes`)
- Cryptographic integrity (Tower-tier witnesses)
- Digital signatures via BearDog `crypto.sign` over UDS

**Integration path**: BearDog provides `crypto.sign` over UDS JSON-RPC.
`CryptoDelegate::resolve()` discovers the socket at startup. Braids carry
Tower-tier Ed25519 witnesses when BearDog is reachable, with graceful
fallback to unsigned when unavailable.

**Workaround**:
Current demos use BearDog CLI directly (shell commands), not as a service.

---

## 🔍 REAL BINARY VERIFICATION

### How to Verify NO MOCKS

Each integration demo includes verification steps:

```bash
# 1. Binary exists and is real ELF
file ../../../bins/songbird-orchestrator
# Output: ELF 64-bit LSB pie executable...

# 2. Process created with PID
ps aux | grep songbird-orchestrator

# 3. Port actually listening
lsof -i :8000

# 4. Real HTTP responses
curl http://localhost:8000/health

# 5. Logs generated
tail -f logs/songbird.log
```

**All demos save verification logs** to prove real execution.

---

## 📊 INTEGRATION STATUS MATRIX

| Primal | Binary | Size | Status | Demo | Verification |
|--------|--------|------|--------|------|--------------|
| **Songbird** | songbird-orchestrator | 20MB | ✅ Working | discovery-live | ✅ Verified |
| **NestGate** | nestgate | 3.4MB | ✅ Working | storage-live | ✅ Verified |
| **ToadStool** | toadstool-cli | 21MB | 🟡 Partial | ml-provenance | 🟡 Can enhance |
| **Squirrel** | squirrel | 12MB | 📋 Planned | - | - |
| **BearDog** | beardog | 4.5MB | ✅ Working | signed-braid | ✅ UDS crypto.sign |

**Legend**:
- ✅ Working - Fully functional with real binary
- 🟡 Partial - Works but could be enhanced
- 📋 Planned - Binary available, demo not created
- ✅ Resolved - Previously blocked, resolved in v0.7.28

---

## 🎓 LEARNING OBJECTIVES

After completing this showcase, you should understand:

- [ ] **Capability-based discovery** (Songbird integration)
- [ ] **Cross-primal storage** (NestGate integration)
- [ ] **Compute provenance** (ToadStool integration)
- [ ] **Real binary integration** (no mocks anywhere)
- [ ] **Gap discovery process** (BearDog limitation found)
- [ ] **Service lifecycle** (start, verify, use, shutdown)

---

## 🔧 TROUBLESHOOTING

### "Binary not found"
```bash
# Check if binaries exist
ls -lah ../../../bins/

# Should see:
# songbird-orchestrator, nestgate, toadstool-cli, beardog, squirrel
```

### "Port already in use"
```bash
# Check what's using the port
lsof -i :8000

# Kill if needed
kill $(lsof -t -i:8000)
```

### "Demo script not executable"
```bash
chmod +x */demo-*.sh
chmod +x RUN_ME_FIRST.sh
```

### "Want to see integration gaps"
```bash
# Integration gaps resolved — see wateringHole/handoffs/ for history
ls ../../wateringHole/handoffs/SWEETGRASS_*
```

---

## 💡 KEY INSIGHTS

### Why Real Binaries Matter

**With Mocks** ❌:
```bash
# Fake function that always returns success
mock_songbird() {
  echo '{"status": "ok"}'
}
```
**Problems**:
- ❌ Hides API mismatches
- ❌ Hides integration bugs
- ❌ Hides performance issues
- ❌ Delays discovery until production

**With Real Binaries** ✅:
```bash
# Actual binary from ../bins
../../../bins/songbird-orchestrator --port 8000 &
PID=$!
```
**Benefits**:
- ✅ Discovers API gaps immediately (BearDog!)
- ✅ Tests real integration
- ✅ Validates performance
- ✅ Evolves NOW, not in production

---

### Gap Discovery Philosophy

**"Interactions show us gaps in our evolution"**

This showcase **intentionally** uses real binaries to find integration issues:

**Gaps Discovered So Far**:
1. ✅ **SweetGrass service binary missing** (FIXED in Phase 2)
2. ✅ **API mismatch for provenance creation** (FIXED in Phase 2)
3. ✅ **BearDog server mode missing** (RESOLVED v0.7.28 — `crypto.sign` delegation; BTSP Phase 3 AEAD in v0.7.29)

**Each gap makes us better!**

---

## 📚 PATTERN EXAMPLES

### Starting Real Service

```bash
#!/usr/bin/env bash
set -euo pipefail

# Start real binary from ../bins
BINARY="../../../bins/songbird-orchestrator"
PORT=8000

# Verify it's real
file "$BINARY" | grep -q "ELF" || exit 1

# Start in background
"$BINARY" --port $PORT > logs/songbird.log 2>&1 &
PID=$!

# Save PID
echo $PID > pids/songbird.pid

# Wait for startup
sleep 3

# Verify running
if ! ps -p $PID > /dev/null; then
  echo "Failed to start"
  exit 1
fi

# Verify port
if ! lsof -i :$PORT | grep -q LISTEN; then
  echo "Port not listening"
  kill $PID
  exit 1
fi

# Now use it
curl http://localhost:$PORT/health

# Clean shutdown
kill $PID
wait $PID
```

### Gap Discovery Logging

```bash
# If something doesn't work, log it
if ! curl -s http://localhost:$PORT/api/endpoint; then
  cat >> gaps/discovered-$(date +%Y%m%d).md << EOF
## Gap: Endpoint Missing

**Date**: $(date)
**Service**: Songbird
**Issue**: /api/endpoint returned 404
**Expected**: 200 OK with JSON response
**Impact**: Cannot complete integration

**Next Steps**:
1. Check Songbird API docs
2. Coordinate with Songbird team
3. Update integration code

EOF
fi
```

---

## ⏭️ WHAT'S NEXT?

### After Inter-Primal Integration:

**Option A**: **Federation** (Recommended next)
```bash
cd ../02-federation
```
- Multi-tower SweetGrass mesh
- Cross-tower provenance queries
- Distributed attribution
- **Time**: ~45 minutes

**Option B**: **Full Ecosystem**
```bash
cd ../02-full-ecosystem
```
- All primals working together
- Complete ML pipeline with attribution
- Multi-primal provenance
- **Time**: ~60 minutes

**Option C**: **Real-World Value**
```bash
cd ../03-real-world
```
- $40M+ demonstrated value
- Concrete business cases
- ROI calculations
- **Time**: ~90 minutes

---

## 🌟 SUCCESS CRITERIA

Inter-primal integration is complete when you can:

- [ ] Start real Songbird and discover SweetGrass
- [ ] Store Braids in real NestGate
- [ ] Track compute provenance in ToadStool
- [ ] Understand why real > mocks
- [ ] Document gaps discovered
- [ ] Clean shutdown all services

---

## 📝 NOTES

### Available Binaries

All binaries in `../../../bins/` are:
- ✅ Real ELF executables (not scripts)
- ✅ Built from phase1 primals
- ✅ Production-ready
- ✅ Executable permissions set

### No Mocks Anywhere

This showcase contains:
- ✅ **ZERO** mock functions
- ✅ **ZERO** fake responses
- ✅ **ZERO** simulated services

Everything uses real binaries or documents why not possible.

### Gap Documentation

Integration gaps tracked in:
- `wateringHole/handoffs/SWEETGRASS_*` - Handoff history with gap resolutions
- `gaps/*.md` - Per-demo gap logs (if generated by demo runs)
- `outputs/*/INTEGRATION_GAPS.md` - Per-run discoveries (if generated by demo runs)

---

**Ready to see real integration?**

```bash
./RUN_ME_FIRST.sh
```

Or pick an integration:

```bash
cd 04-sweetgrass-songbird && ./demo-discovery-live.sh
```

---

🌾 **Real binaries, real integration, real evolution!** 🌾

*Following patterns from:*
- *🎵 Songbird: Real execution verification*
- *🍄 ToadStool: Compute integration mastery*
- *🏰 NestGate: Cross-primal storage*
