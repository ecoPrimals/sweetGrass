# 🌾 Level 2: Full Ecosystem

**Goal**: Complete attribution pipeline with rewards  
**Prerequisites**: Level 0 & 1 completed, all primals available  
**Time**: 60+ minutes  
**Complexity**: Advanced-Expert

---

## 🎯 What You'll Learn

- Complete data lifecycle provenance
- Multi-contributor attribution chains
- Cross-primal coordination via Songbird
- Proportional reward distribution (sunCloud)
- Production-ready deployment patterns

---

## 📁 Demos

### 1. Complete Pipeline (20 min)
**Directory**: `01-complete-pipeline/`

Track data from creation through all transformations.

```bash
cd 01-complete-pipeline
./demo-full-pipeline.sh
```

**What you'll see**:
- Alice creates original data
- Bob processes it (derivation)
- Charlie creates visualization (derivation)
- Complete provenance exported
- Attribution calculated for all

**Pipeline Flow**:
```
Alice: Creates Dataset
    ↓ BearDog signs
    ↓ SweetGrass records
    
Bob: Runs Analysis
    ↓ RhizoCrypt captures session
    ↓ SweetGrass compresses
    ↓ Links to Alice's Braid
    
Charlie: Creates Visualization
    ↓ LoamSpine anchors
    ↓ SweetGrass records
    ↓ Links to Bob's Braid
    
Query: "Who contributed to Charlie's viz?"
    → Alice: 49%
    → Bob: 21%
    → Charlie: 30%
```

---

### 2. Multi-Primal Provenance (20 min)
**Directory**: `02-multi-primal-provenance/`

Track provenance across primal boundaries.

```bash
cd 02-multi-primal-provenance
./demo-cross-primal.sh
```

**What you'll see**:
- Data moves between primals
- Provenance follows across boundaries
- Attribution computed correctly
- PROV-O export includes all steps

**Cross-Primal Tracking**:
```
Squirrel (AI)           → SweetGrass (Braid: AI Output)
ToadStool (Compute)     → SweetGrass (Braid: Compute Result)
NestGate (Storage)      → SweetGrass (Braid: Stored Data)
Songbird (Orchestration) → SweetGrass (Braid: Coordination Record)
```

---

### 3. Reward Distribution (Planned)
**Directory**: `03-reward-distribution/`

Proportional rewards based on attribution.

```bash
cd 03-reward-distribution
./demo-rewards.sh  # Coming with sunCloud integration
```

**What you'll see**:
- Query attribution for an output
- Calculate reward proportions
- Distribute to contributors
- Audit trail in SweetGrass

**Example**:
```
Output: Charlie's Visualization
Reward Pool: 100 tokens

Distribution:
  Alice (Creator): 49 tokens
  Bob (Processor): 21 tokens
  Charlie (Viz): 30 tokens
```

---

## 🏗️ Complete Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ECOSYSTEM OVERVIEW                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │ Squirrel │    │ ToadStool│    │ NestGate │              │
│  │   (AI)   │    │ (Compute)│    │ (Storage)│              │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘              │
│       │               │               │                     │
│       └───────────────┼───────────────┘                     │
│                       ↓                                     │
│              ┌────────────────┐                             │
│              │   SweetGrass   │ ← Provenance for ALL       │
│              │  (Attribution) │                             │
│              └───────┬────────┘                             │
│                      │                                      │
│       ┌──────────────┼──────────────┐                      │
│       ↓              ↓              ↓                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│  │ BearDog  │  │RhizoCrypt│  │LoamSpine │                 │
│  │(Signing) │  │(Sessions)│  │(Anchoring│                 │
│  └──────────┘  └──────────┘  └──────────┘                 │
│                                                              │
│              ┌────────────────┐                             │
│              │    Songbird    │ ← Orchestrates all         │
│              │  (Mesh Coord)  │                             │
│              └────────────────┘                             │
│                                                              │
│              ┌────────────────┐                             │
│              │    sunCloud    │ ← Rewards (planned)        │
│              │   (Rewards)    │                             │
│              └────────────────┘                             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🌊 Data Flow

### Creation Phase
```
1. Alice creates data
2. Squirrel or ToadStool processes
3. BearDog signs the result
4. SweetGrass creates Braid
5. NestGate stores (optional)
```

### Derivation Phase
```
1. Bob retrieves Alice's data
2. Processes/transforms it
3. RhizoCrypt captures session
4. SweetGrass compresses to Braid
5. Links to Alice's Braid
```

### Query Phase
```
1. Query: "Who contributed to X?"
2. SweetGrass traverses provenance graph
3. Calculates attribution weights
4. Returns contributor shares
5. Exports to PROV-O if needed
```

### Reward Phase (Future)
```
1. Reward event for output
2. SweetGrass provides attribution
3. sunCloud calculates distribution
4. Tokens distributed to contributors
5. SweetGrass records distribution Braid
```

---

## 📊 Expected Output

### Complete Pipeline
```
🌾 Running Complete Attribution Pipeline...

Step 1: Alice creates dataset
  ✅ Braid: urn:braid:alice-001
  Signed by: did:key:z6MkAlice...

Step 2: Bob runs analysis
  ✅ Session captured: session-bob-001
  ✅ Compressed to Braid: urn:braid:bob-001
  Derived from: urn:braid:alice-001

Step 3: Charlie creates visualization
  ✅ Braid: urn:braid:charlie-001
  Anchored to: spine-main
  Derived from: urn:braid:bob-001

Attribution for Charlie's visualization:
  Alice: 49% (Creator)
  Bob: 21% (Processor)
  Charlie: 30% (Visualizer)

✅ Pipeline complete!
```

---

## 🛠️ Configuration

### Complete Stack
```toml
[sweetgrass]
store = "postgres"  # Production: PostgreSQL
discovery = "songbird"  # Multi-tower

[integration]
beardog = "capability:signing"
rhizocrypt = "capability:session_streaming"
loamspine = "capability:anchoring"

[rewards]
enabled = false  # Enable with sunCloud
distribution = "proportional"
```

### Multi-Tower
```toml
[federation]
enabled = true
coordinator = "songbird"

[discovery]
method = "songbird"
fallback = "mdns"
```

---

## 💡 Key Insights

### SweetGrass Is Central
Every primal that produces data should record provenance with SweetGrass.
This enables:
- Complete audit trails
- Fair attribution
- Proportional rewards

### Attribution Flows Through Derivations
When Bob derives from Alice, Alice gets partial credit.
The chain can be arbitrarily deep.

### Privacy Travels With Data
Privacy settings on Braids are enforced at query time.
Private data stays private across the ecosystem.

### Rewards Require Attribution
sunCloud uses SweetGrass attribution to distribute rewards.
No attribution = no fair rewards.

---

## 🎯 Success Criteria

Level 2 is complete when you can:

- [ ] Track complete data lifecycle
- [ ] Compute multi-contributor attribution
- [ ] Export cross-primal provenance
- [ ] Understand reward distribution model
- [ ] Deploy production pipeline

---

## 📚 Production Deployment

### Prerequisites
1. PostgreSQL for persistent storage
2. All primals running (BearDog, RhizoCrypt, LoamSpine)
3. Songbird for coordination
4. Proper DID infrastructure

### Deployment Steps
1. Configure PostgreSQL store
2. Set up capability discovery
3. Deploy SweetGrass service
4. Register with Songbird
5. Test end-to-end pipeline

### Monitoring
- Health: `/health`, `/live`, `/ready`
- Metrics: tarpc stats, query latencies
- Logs: Structured JSON logging

---

**The complete ecosystem demo showcases SweetGrass as the provenance backbone of ecoPrimals!**

🌾 **Every piece of data has a story. Every contributor gets credit.** 🌾

