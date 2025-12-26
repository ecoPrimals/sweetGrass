# 🌾🐿️ SweetGrass + Squirrel Integration

**"AI Agent Provenance & Attribution"**

**Time**: ~15 minutes  
**Binary**: `../../../../bins/squirrel` (12MB, real ELF)  
**Status**: ✅ Ready to implement

---

## 🎯 PURPOSE

Demonstrate how SweetGrass tracks AI agent decisions, multi-agent collaboration, and calculates fair attribution for AI-generated content.

**Philosophy**: "Every AI decision has a story. Every contributor deserves credit."

---

## 🚀 WHAT YOU'LL SEE

### 1. AI Agent Decision Tracking
- Agent creates content
- Decision provenance captured
- Training data attribution
- Model lineage tracked

### 2. Multi-Agent Collaboration
- Multiple agents work together
- Each agent's contribution tracked
- Fair credit distribution
- Collaboration graph

### 3. Agent Genealogy
- Agent creation provenance
- Training data sources
- Model evolution
- Capability inheritance

---

## 📋 DEMOS

### `demo-ai-agent-provenance.sh`
**Time**: 10 minutes  
**What it shows**:
- Start real Squirrel service
- Agent generates content
- Track decision provenance
- Calculate attribution
- Export to PROV-O

### `demo-multi-agent-collaboration.sh` (planned)
**Time**: 10 minutes  
**What it shows**:
- Multiple Squirrel agents
- Collaborative task
- Attribution across agents
- Conflict resolution tracking

---

## 🔍 INTEGRATION PATTERNS

### Pattern 1: Single Agent Provenance

```rust
// Track AI agent decision with full provenance

// 1. Create Braid for agent's training data
let training_data_braid = factory.from_data(
    &training_corpus,
    "application/json",
    Some("did:key:data_curator"),
)?;

// 2. Create Braid for trained model
let model_braid = factory.derive_from(
    &training_data_braid,
    &model_hash,
    DerivationType::MLTraining,
)?;

// 3. Agent generates content
let agent = SquirrelClient::discover().await?;
let content = agent.generate("Write a poem").await?;

// 4. Create Braid for AI-generated content
let content_braid = factory
    .from_data(&content.text, "text/plain", None)?
    .with_attribution(Attribution::creator("did:agent:squirrel_v1"))
    .with_attribution(Attribution::contributor("did:key:data_curator"))
    .with_derivation(&model_braid.id, DerivationType::Generation)
    .build()?;

// Now we have complete provenance:
// training_data → model → agent_decision → content
```

### Pattern 2: Multi-Agent Attribution

```rust
// Track collaboration between multiple AI agents

let agents = vec![
    SquirrelClient::connect("agent_researcher").await?,
    SquirrelClient::connect("agent_writer").await?,
    SquirrelClient::connect("agent_editor").await?,
];

// Research phase
let research = agents[0].research("quantum computing").await?;
let research_braid = factory.from_agent_output(&research)?;

// Writing phase (derived from research)
let draft = agents[1].write_from(&research).await?;
let draft_braid = factory
    .derive_from(&research_braid, &draft.hash, DerivationType::Transformation)?
    .with_attribution(Attribution::contributor("did:agent:researcher"))
    .with_attribution(Attribution::creator("did:agent:writer"))
    .build()?;

// Editing phase (derived from draft)
let final_doc = agents[2].edit(&draft).await?;
let final_braid = factory
    .derive_from(&draft_braid, &final_doc.hash, DerivationType::Revision)?
    .with_attribution(Attribution::contributor("did:agent:researcher"))
    .with_attribution(Attribution::contributor("did:agent:writer"))
    .with_attribution(Attribution::creator("did:agent:editor"))
    .build()?;

// Calculate fair attribution
let attribution = store.calculate_attribution(&final_braid.id).await?;
// Researcher: 25% (data provider)
// Writer: 50% (primary creator)
// Editor: 25% (refiner)
```

---

## 💡 KEY INSIGHTS

### Why AI Agent Provenance Matters

**Problem**: AI-generated content lacks attribution
- Who trained the model?
- What data was used?
- Who deserves credit?
- How to verify authenticity?

**Solution**: SweetGrass + Squirrel
- ✅ Complete agent genealogy
- ✅ Training data attribution
- ✅ Decision provenance
- ✅ Fair credit distribution

### Real-World Value

**For Content Creators**:
- Prove AI assistance vs plagiarism
- Track training data sources
- Fair compensation for data providers
- Transparent AI usage

**For AI Developers**:
- Model lineage tracking
- Training data provenance
- Performance attribution
- Reproducible results

**For Organizations**:
- AI audit trails
- Compliance (EU AI Act)
- IP protection
- Quality assurance

---

## 🎓 LEARNING OBJECTIVES

After this demo, you'll understand:

- [ ] How to track AI agent decisions
- [ ] Multi-agent collaboration provenance
- [ ] Fair attribution for AI-generated content
- [ ] Agent genealogy and lineage
- [ ] Training data attribution
- [ ] Real Squirrel integration (no mocks!)

---

## 🔧 PREREQUISITES

```bash
# Verify Squirrel binary exists
ls -lh ../../../../bins/squirrel

# Should see: 12MB ELF executable
```

---

## 🚀 QUICK START

```bash
# Run the main demo
./demo-ai-agent-provenance.sh

# Or test integration
./demo-ai-agent-integration-test.sh
```

---

## 📊 EXPECTED RESULTS

```
✅ Squirrel service started (PID: XXXXX)
✅ Agent generated content
✅ Provenance Braid created
✅ Attribution calculated
✅ Training data credited
✅ PROV-O export created
```

---

## 🌟 SUCCESS CRITERIA

Integration is successful when:

- [ ] Real Squirrel binary starts and responds
- [ ] Agent decisions tracked in SweetGrass
- [ ] Attribution includes training data sources
- [ ] Multi-agent collaboration works
- [ ] PROV-O export validates
- [ ] NO MOCKS used anywhere

---

## ⏭️ WHAT'S NEXT

After Squirrel integration:

**Multi-Primal Workflows**:
```bash
cd ../07-multi-primal-workflows
# See Squirrel + NestGate + SweetGrass working together
```

**Federation**:
```bash
cd ../../02-federation
# Multi-tower AI agent provenance
```

---

🌾🐿️ **Fair attribution for AI agents!** 🌾🐿️

*Following patterns from mature primals: Real binaries, NO MOCKS*
