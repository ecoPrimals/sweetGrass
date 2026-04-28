# 🌾 SweetGrass Showcase Index

**Last Updated**: April 28, 2026  
**Philosophy**: "Interactions show us gaps in our evolution" - Real binaries, no mocks  
**Inspiration**: Following NestGate (local-first), Songbird (federation), ToadStool (compute)

---

## 🎯 Quick Start

```bash
# Fastest path (automated tour, 30 min)
cd 00-local-primal && ./RUN_ME_FIRST.sh

# Or pick your learning path below
```

---

## 📚 Showcase Structure

### **Level 0: Local Primal** → "SweetGrass BY ITSELF is Amazing"
**Directory**: `00-local-primal/`  
**Time**: 50 minutes  
**Prerequisites**: None - Start here!

Demonstrate SweetGrass's value **independently** before showing ecosystem integration.

**What you'll learn**:
- Create and query Braids
- Calculate fair attribution
- Traverse provenance graphs
- Export to W3C PROV-O
- Configure privacy controls
- Use multiple storage backends

**Demos**:
1. **Hello Attribution** (5 min) - First Braid, content hashing
2. **Fair Credit** (10 min) - Role weights, attribution chains
3. **Provenance Queries** (10 min) - Graph traversal, filtering
4. **PROV-O Standard** (5 min) - W3C JSON-LD export
5. **Privacy Controls** (10 min) - GDPR-inspired data rights
6. **Storage Backends** (10 min) - Memory, redb (recommended), PostgreSQL

**Status**: ✅ Complete  
**Start**: `cd 00-local-primal && ./RUN_ME_FIRST.sh`

---

### **Level 1: Inter-Primal** → "SweetGrass + ONE Other Primal"
**Directory**: `01-primal-coordination/`  
**Time**: 60 minutes  
**Prerequisites**: Level 0 complete

Demonstrate SweetGrass integrating with individual primals using **real binaries from ../bins**.

**Real Integration Demos**:

#### 1. SweetGrass + Songbird (Discovery)
**Directory**: `01-sweetgrass-songbird/`  
**Binary**: `../../../bins/songbird-orchestrator`  
**Time**: 15 minutes

- Capability-based service discovery
- Register attribution services
- Query for provenance capabilities
- **No mocks** - real Songbird instance

#### 2. SweetGrass + NestGate (Storage)
**Directory**: `02-sweetgrass-nestgate/`  
**Binary**: `../../../bins/nestgate`  
**Time**: 15 minutes

- Store Braids in NestGate
- ZFS snapshot integration
- Distributed storage provenance
- **No mocks** - real NestGate instance

#### 3. SweetGrass + ToadStool (Compute)
**Directory**: `03-sweetgrass-toadstool/`  
**Binary**: `../../../bins/toadstool-cli`  
**Time**: 15 minutes

- Compute task provenance
- GPU workload attribution
- Task execution graphs
- **No mocks** - real ToadStool instance

#### 4. SweetGrass + BearDog (Signing)
**Directory**: `01-sweetgrass-beardog/`  
**Binary**: `../../../bins/beardog`  
**Time**: 15 minutes  
**Status**: ✅ **RESOLVED** (v0.7.28 — UDS JSON-RPC `crypto.sign` delegation)

- Braid signing with Ed25519 (Tower-delegated via `CryptoDelegate`)
- DID resolution (`Did::from_public_key_bytes`)
- Cryptographic integrity (Tower-tier witnesses)
- **Gap resolved**: BearDog server mode + BTSP Phase 2 + `crypto.sign` delegation

**Start**: `cd 01-primal-coordination/01-sweetgrass-beardog && ./demo-signed-braid-live.sh`

---

### **Level 2: Federation** → "Multi-Tower SweetGrass Mesh"
**Directory**: `02-federation/` (planned)  
**Time**: 45 minutes  
**Prerequisites**: Level 0 + Level 1

Demonstrate multi-tower SweetGrass federation (inspired by Songbird's success).

**Planned Demos**:
1. **Two-Tower Attribution** - Cross-tower Braid queries
2. **Federated Queries** - Distributed graph traversal
3. **Attribution Sync** - Keep attribution consistent across towers

**Status**: 📋 Planned (following Songbird's `02-federation/` pattern)

---

### **Level 3: Full Ecosystem** → "All Primals Together"
**Directory**: `02-full-ecosystem/`  
**Time**: 60 minutes  
**Prerequisites**: Level 0 + Level 1

Demonstrate SweetGrass in complete multi-primal workflows.

**Demos**:
1. **Complete Pipeline** (20 min) - ML training with full provenance
2. **Multi-Primal Provenance** (20 min) - Attribution across primals
3. **Cross-Primal Federation** (20 min) - Distributed ecosystem queries

**Status**: 🟡 Partial (needs real binary integration)  
**Start**: `cd 02-full-ecosystem/01-complete-pipeline && ./demo-full-pipeline-live.sh`

---

### **Level 4: Real-World** → "Concrete Value Demonstrations"
**Directory**: `03-real-world/`  
**Time**: 90 minutes  
**Prerequisites**: Level 0 (can be explored independently)

Demonstrate **measurable real-world value** with concrete scenarios.

**Value Demonstrations**:

| Demo | Value | Time |
|------|-------|------|
| **ML Training Attribution** | $100k/month fair distribution | 15 min |
| **Open Science** | 3-year reproducibility guarantee | 15 min |
| **Content Royalties** | 5-contributor auto-distribution | 20 min |
| **HIPAA Compliance** | Weeks → minutes for audit reports | 20 min |
| **Supply Chain** | **$40M saved** in precise recall | 20 min |

**Status**: ✅ Complete (narrative demos)  
**Start**: `cd 03-real-world/05-supply-chain && ./demo-product-lineage.sh`

---

## 🎓 Recommended Learning Paths

### **Path A: Beginner** (90 minutes)
For first-time users wanting to understand SweetGrass fundamentals:

```bash
1. Level 0: Local Primal (50 min)
   → cd 00-local-primal && ./RUN_ME_FIRST.sh

2. Level 4: One Real-World Demo (20 min)
   → cd 03-real-world/05-supply-chain && ./demo-product-lineage.sh

3. Level 1: One Integration (15 min)
   → cd 01-primal-coordination/04-sweetgrass-songbird && ./demo-discovery-live.sh
```

**After this**: You understand SweetGrass value and basics

---

### **Path B: Developer** (2 hours)
For developers wanting to integrate SweetGrass:

```bash
1. Level 0: Local Primal (50 min)
   → Full tour of API and features

2. Level 1: All Integrations (60 min)
   → Learn integration patterns with real binaries

3. Level 3: One Ecosystem Demo (20 min)
   → See multi-primal coordination
```

**After this**: You can integrate SweetGrass into your project

---

### **Path C: Architect** (3 hours)
For architects evaluating SweetGrass for production:

```bash
1. Level 0: Local Primal (50 min)
2. Level 1: All Integrations (60 min)
3. Level 2: Federation (45 min) - when available
4. Level 4: All Real-World Demos (90 min)
```

**After this**: You can design production SweetGrass deployments

---

### **Path D: Stakeholder** (30 minutes)
For stakeholders wanting to see value quickly:

```bash
1. Level 4: Real-World Demos (30 min)
   → See $40M+ demonstrated value
   → Understand ROI and business case
```

**After this**: You understand business value

---

## 📊 Showcase Status Matrix

| Level | Directory | Demos | Status | Real Binaries | Gaps |
|-------|-----------|-------|--------|---------------|------|
| **0** | `00-local-primal/` | 6 | ✅ Complete | N/A (local) | None |
| **1** | `01-primal-coordination/` | 4 | 🟡 3/4 working | Yes (../bins) | BearDog server mode |
| **2** | `02-federation/` | 3 | 📋 Planned | Yes (planned) | Not started |
| **3** | `02-full-ecosystem/` | 3 | 🟡 Partial | Partial | Needs real binaries |
| **4** | `03-real-world/` | 5 | ✅ Complete | N/A (narrative) | None |

**Legend**:
- ✅ Complete and tested
- 🟡 Partial / needs work
- 📋 Planned / not started
- N/A - Not applicable

---

## 🔍 Real vs Mock Philosophy

### Our Commitment: **NO MOCKS**

Following NestGate's **LIVE_DEMO_VERIFICATION_NO_MOCKS_DEC_22_2025.md**:

**✅ What we do**:
- Use real binaries from `../bins`
- Verify processes with `ps`, `lsof`, `curl`
- Capture real interactions and responses
- Document gaps discovered in real testing
- Clean shutdown and verification

**❌ What we don't do**:
- Mock services
- Fake responses
- Simulated binaries
- Hardcoded test data (except for narratives)

**Why?**: "Interactions show us gaps in our evolution"
- Real binaries reveal real integration issues
- Mocks hide problems until production
- We want to discover gaps NOW and evolve

**Evidence**: 3 real integration gaps found and resolved through showcase testing

---

## 🛠️ Running the Showcases

### Prerequisites
```bash
# Build SweetGrass
cd /path/to/sweetGrass
cargo build --release

# Verify bins available
ls -la ../bins/
# Should see: songbird-orchestrator, nestgate, toadstool-cli, etc.
```

### Quick Demo (5 minutes)
```bash
cd showcase
./scripts/quick-demo.sh  # Curated 5-minute highlights
```

### Full Tour (3 hours)
```bash
cd showcase/00-local-primal
./RUN_ME_FIRST.sh  # Automated Level 0 (50 min)

cd ../01-primal-coordination
./RUN_ME_FIRST.sh  # Automated Level 1 (60 min)

# Continue through levels...
```

### Individual Demo
```bash
cd showcase/03-real-world/05-supply-chain
./demo-product-lineage.sh  # Single demo
```

---

## 📖 Documentation

### Showcase-Specific Docs
- **This file**: `00_SHOWCASE_INDEX.md` - Master index
- **Main README**: `README.md` - Overview and philosophy
- **Specs**: `../specs/` — Technical specifications
- **Handoffs**: See `wateringHole/handoffs/` for integration gap resolution history

### Level READMEs
- **Level 0**: `00-local-primal/README.md`
- **Level 1**: `01-primal-coordination/README.md`
- **Level 3**: `02-full-ecosystem/README.md`
- **Level 4**: `03-real-world/README.md`

### Individual Demo READMEs
Each demo directory has its own `README.md` with detailed instructions.

---

## 🎯 Success Criteria

### Showcase is successful when:
- [ ] New users can complete Level 0 in < 60 minutes
- [ ] Developers can integrate after Level 0 + Level 1
- [ ] All real binary integrations work (Level 1)
- [ ] Federation demos work (Level 2)
- [ ] Real-world value is clearly demonstrated (Level 4)
- [ ] No mocks anywhere in integration tests
- [ ] All gaps are documented and tracked

---

## 🏆 Showcase Quality Standards

Following phase1 primal patterns:

### From NestGate 🏰
- ✅ Local-first approach
- ✅ Progressive complexity (Level 1-6)
- ✅ Automated tour scripts
- ✅ "BY ITSELF is amazing" messaging
- ✅ Clear time estimates
- ✅ Success criteria per level

### From Songbird 🎵
- ✅ Real execution verification
- ✅ Multi-tower federation demos
- ✅ Capability-based discovery examples
- ✅ Clean startup/shutdown
- ✅ Gap discovery documentation

### From ToadStool 🍄
- ✅ Compute integration demos
- ✅ Task provenance examples
- ✅ Inter-primal patterns
- ✅ Performance benchmarks
- ✅ Real workload demonstrations

---

## 📞 Support

### Issues?
- Check individual demo READMEs
- Review `wateringHole/handoffs/` for integration gap history
- See troubleshooting sections in level READMEs

### Want to Contribute?
- Add new real-world scenarios
- Improve existing demos
- Add federation examples
- Document discovered gaps

---

## 🗺️ Showcase Evolution

### Completed ✅
- Level 0: Local Primal (6 demos)
- Level 4: Real-World (5 demos)
- Level 1: Partial (3/4 integrations)

### In Progress 🟡
- Level 1: Complete all 4 integrations (waiting on BearDog server mode)
- Level 3: Real binary integration
- Gap discovery and documentation

### Planned 📋
- Level 2: Federation showcase
- Cross-tower benchmarks
- Advanced federation patterns
- sunCloud integration demos

---

**Last Updated**: March 16, 2026  
**Maintainer**: SweetGrass Team  
**Philosophy**: "Interactions show us gaps in our evolution"

🌾 **Ready to explore?** Start with Level 0!

```bash
cd 00-local-primal && ./RUN_ME_FIRST.sh
```

🌾 **Let's tell some data stories!** 🌾

