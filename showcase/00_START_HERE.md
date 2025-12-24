# 🌾 SweetGrass Showcase — Start Here

**The Attribution Layer for ecoPrimals**

**Version**: 0.2.2 | **Status**: Production Ready | **Tests**: 419 passing

---

## 🎯 What Is This?

This showcase demonstrates **SweetGrass** - the provenance and attribution layer that tells the story of every piece of data:

✅ **Braids** - Cryptographic provenance records (W3C PROV-O)  
✅ **Attribution** - Fair credit for all contributors  
✅ **Provenance** - Complete data history traversal  
✅ **Privacy** - GDPR-style data subject rights  
✅ **Integration** - Seamless coordination with other primals

---

## 🚀 Quick Start (5 Minutes)

### Option A: Just Show Me Something!
```bash
# Run the quick demo
./scripts/quick-demo.sh
```

### Option B: I Want the Full Picture
Read this file, then follow the [Progressive Learning Path](#-progressive-learning-path) below.

### Option C: I'm Ready to Deep Dive
```bash
cd 00-standalone
cat README.md
```

---

## 📚 Showcase Structure

Three progressive levels:

### Level 0: Standalone SweetGrass (40 min) ⭐ START HERE
**Goal**: Understand SweetGrass as a complete attribution platform

**What you'll see**:
- Braid creation and querying
- Attribution calculation
- Provenance graph traversal
- PROV-O export
- Privacy controls

**Path**: `00-standalone/`

---

### Level 1: Primal Coordination (45 min)
**Goal**: See SweetGrass coordinating with other primals

**What you'll see**:
- SweetGrass + BearDog (Signed Braids)
- SweetGrass + RhizoCrypt (Session Compression)
- SweetGrass + LoamSpine (Commit Anchoring)

**Path**: `01-primal-coordination/`

---

### Level 2: Full Ecosystem (60+ min) 🌟
**Goal**: Complete attribution pipeline with rewards

**What you'll see**:
- Complete data lifecycle tracking
- Multi-primal provenance
- Cross-tower coordination
- Reward distribution (planned)

**Path**: `02-full-ecosystem/`

---

## 🎓 Progressive Learning Path

### For New Users (2 hours)
```
Step 1: Level 0 (40 min) → Understand Braids and Attribution
Step 2: Level 1 (45 min) → See primal coordination
Step 3: Level 2 (30 min) → Experience complete ecosystem
```

**Start**: `00-standalone/01-braid-basics/`

---

### For Developers (60 min)
```
Step 1: Level 0 demos 1-2 (15 min) → Braids, Attribution
Step 2: Level 1 demo 1 (15 min) → BearDog signing
Step 3: Level 2 demo 1 (30 min) → Complete pipeline
```

**Start**: `00-standalone/01-braid-basics/`

---

### For Data Scientists (45 min)
```
Step 1: Level 0 demo 2 (15 min) → Attribution Engine
Step 2: Level 0 demo 4 (10 min) → PROV-O Export
Step 3: Level 2 demo 1 (20 min) → Multi-contributor chains
```

**Start**: `00-standalone/02-attribution-engine/`

---

## 🌟 Featured Demos

### 1. Braid Basics (Level 0) 📦
**Why it matters**: Every piece of data becomes traceable

```bash
cd 00-standalone/01-braid-basics
./demo-create-braid.sh
```

---

### 2. Attribution Engine (Level 0) ⚖️
**Why it matters**: Fair credit for all contributors

```bash
cd 00-standalone/02-attribution-engine
./demo-attribution.sh
```

**Example**:
```
Original Data (Alice: 100%)
    ↓ derived
Processed (Alice: 70%, Bob: 30%)
    ↓ derived
Final (Alice: 49%, Bob: 21%, Charlie: 30%)
```

---

### 3. Complete Pipeline (Level 2) 🌊
**Why it matters**: The full story, end to end

```bash
cd 02-full-ecosystem/01-complete-pipeline
./demo-full-pipeline.sh
```

---

## 🛠️ Prerequisites

### Minimum (Level 0)
- ✅ SweetGrass built (`cargo build`)
- That's it! Level 0 works standalone.

### Recommended (Level 1+)
- ✅ BearDog running (for signing)
- ✅ RhizoCrypt running (for sessions)
- ✅ LoamSpine running (for anchoring)

### Optional (Level 2)
- ✅ Songbird (for multi-tower)
- ✅ sunCloud (for rewards)

### Verification
```bash
./utils/setup.sh   # Check prerequisites
./utils/verify.sh  # Quick verification
```

---

## 💡 Tips for Success

1. **Start Simple**: Begin with Level 0 even if experienced
2. **Run in Order**: Demos build on each other
3. **Read the READMEs**: Each level has detailed explanations
4. **Experiment**: All demos are safe to modify

---

## 🎯 Success Criteria

### After Level 0
- [ ] I can create and query Braids
- [ ] I understand attribution calculation
- [ ] I can traverse provenance graphs
- [ ] I know how to export to PROV-O

### After Level 1
- [ ] I can sign Braids with BearDog
- [ ] I understand session compression
- [ ] I can anchor commits with LoamSpine

### After Level 2
- [ ] I can track complete data lifecycles
- [ ] I understand multi-contributor attribution
- [ ] I'm ready for production deployment

---

## 🚀 Ready? Choose Your Path:

### Fastest: 5-Minute Demo
```bash
./scripts/quick-demo.sh
```

### Recommended: Level 0 Complete
```bash
cd 00-standalone
cat README.md
```

### Comprehensive: Full Showcase
```bash
cat README.md
```

---

## 🌟 Why SweetGrass?

**Every piece of data has a story. Every contributor deserves credit.**

- 🌾 **Provenance**: Track where data came from
- ⚖️ **Attribution**: Fair credit for all contributors
- 🔒 **Privacy**: GDPR-style data subject rights
- 🔗 **Integration**: Works with all ecoPrimals
- 📜 **Standards**: W3C PROV-O compatible

---

## 📚 Additional Resources

### In This Showcase
- `README.md` - Complete navigation
- `00-standalone/` - Standalone demos
- `01-primal-coordination/` - Integration demos
- `02-full-ecosystem/` - Production demos

### In Repository Root
- `../STATUS.md` - Project status
- `../ROADMAP.md` - Future development
- `../specs/` - Technical specifications

### Ecosystem Showcases
- `../../../phase1/songBird/showcase/` - Service mesh
- `../../../phase1/squirrel/showcase/` - AI orchestration
- `../../../phase1/bearDog/examples/` - Sovereign security

---

## 🎉 Let's Begin!

**Choose your starting point**:

1. **Quick Demo** → `./scripts/quick-demo.sh`
2. **Level 0** → `cd 00-standalone`
3. **Full Navigation** → `cat README.md`

---

**Status**: ✅ Ready to demonstrate attribution and provenance  
**Next**: Choose your learning path above

🌾 **Every piece of data has a story. Let's tell it!** 🌾

---

*SweetGrass v0.2.2 - ecoPrimals Phase 2*  
*December 23, 2025*

