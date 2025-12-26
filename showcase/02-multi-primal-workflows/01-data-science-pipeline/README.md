# 🌾 Multi-Primal Workflow: Full-Stack Data Science

**REVOLUTIONARY: 4 Primals Working Together**

## The Vision

SweetGrass isn't just a standalone service. It's the **provenance glue** that binds the entire ecoPrimals ecosystem together.

This demo shows SweetGrass orchestrating **4 real primals** in a complete, trustworthy data science pipeline.

---

## The Pipeline

```
Medical Researcher
       ↓
   NestGate (secure storage)
       ↓
Data Scientist
       ↓
   ToadStool (compute training)
       ↓
   SweetGrass (provenance tracking)
       ↓
   Doctor (uses model)
       ↓
   Squirrel (AI inference)
       ↓
Complete Attribution
```

Every step tracked. Everyone credited fairly.

---

## What This Demo Shows

### 1. Real Multi-Primal Integration
- ✅ NestGate for secure encrypted storage
- ✅ ToadStool for distributed compute
- ✅ SweetGrass for complete provenance
- ✅ Squirrel for AI attribution
- ✅ NO MOCKS - all real binaries from `../../../bins/`

### 2. Complete Workflow
- **Data Ingestion**: Medical dataset → NestGate (encrypted)
- **Training Job**: Data scientist submits job → ToadStool
- **Model Creation**: Trained model → NestGate (encrypted)
- **Inference Request**: Doctor requests diagnosis → Squirrel
- **Result + Attribution**: Complete provenance chain

### 3. Fair Credit for Everyone
- Medical Researcher: 30% (provided training data)
- Data Scientist: 25% (designed model)
- ToadStool: 15% (compute execution)
- NestGate: 10% (secure storage)
- Squirrel: 10% (AI inference)
- Doctor: 10% (clinical interpretation)

### 4. Revolutionary Trust
- Complete audit trail
- No black boxes
- Full transparency
- Regulatory compliance

---

## How to Run

```bash
./demo-full-stack.sh
```

**Time:** ~15 minutes

**Prerequisites:**
- All primal binaries in `../../../bins/`:
  - `nestgate`
  - `toadstool-byob-server`
  - `squirrel`
- SweetGrass built (`cargo build --release`)

---

## What You'll See

### Phase 1: Service Startup
All 4 primals start:
- NestGate (secure storage)
- ToadStool (compute)
- SweetGrass (provenance)
- Squirrel (AI attribution)

### Phase 2: Data Ingestion
Medical researcher uploads encrypted training dataset:
- Dataset metadata Braid created
- Data encrypted and stored in NestGate
- Provenance recorded in SweetGrass

### Phase 3: ML Training
Data scientist submits training job:
- Training job Braid created
- ToadStool executes compute
- Progress tracked in real-time

### Phase 4: Model Storage
Trained model created:
- Model Braid created
- Model encrypted and stored in NestGate
- Complete lineage from data → training → model

### Phase 5: AI Inference
Doctor uses model for diagnosis:
- Inference request Braid created
- Squirrel executes AI inference
- Result Braid created with diagnosis

### Phase 6: Fair Attribution
Complete credit chain calculated:
- 6 contributors identified
- Fair shares allocated
- Revolutionary transparency

---

## Real-World Impact

### Trustworthy AI
**Problem**: AI is a black box. No one knows where data came from, how models were trained, who contributed what.

**Solution**: Complete provenance from data → training → inference → result. Full transparency. Complete trust.

### Fair Compensation
**Problem**: Data providers get nothing. Infrastructure providers get nothing. Only platform owners benefit.

**Solution**: Fair attribution for everyone. Data providers paid. Infrastructure credited. Everyone benefits fairly.

### Regulatory Compliance
**Problem**: EU AI Act requires transparency. HIPAA requires audit trails. Current AI can't comply.

**Solution**: Complete provenance out of the box. Every step tracked. Full audit trail. Compliance built-in.

### Federated Trust
**Problem**: Organizations can't trust each other's AI. No way to verify claims. No shared provenance.

**Solution**: SweetGrass provides cross-organizational trust. Provenance travels with data. Trust without centralized authority.

---

## Technical Architecture

### Service Coordination

```
┌─────────────┐
│  NestGate   │  Secure encrypted storage
└──────┬──────┘
       │
       ↓
┌─────────────┐
│  ToadStool  │  Distributed compute
└──────┬──────┘
       │
       ↓
┌─────────────┐
│ SweetGrass  │  Provenance tracking (ORCHESTRATOR)
└──────┬──────┘
       │
       ↓
┌─────────────┐
│  Squirrel   │  AI attribution
└─────────────┘
```

### Provenance Chain

Each Braid links to previous Braids:

```
Dataset Braid
    ↓ derived_from
Training Job Braid
    ↓ derived_from
Trained Model Braid
    ↓ derived_from
Inference Request Braid
    ↓ derived_from
Diagnosis Result Braid
```

Query the final Braid → get complete lineage back to original data.

### Attribution Metadata

Each Braid tracks:
- `was_attributed_to`: Who created/contributed
- `derived_from`: Dependencies (data, models, jobs)
- `activity`: What happened (ingestion, training, inference)
- `used`: Resources consumed
- `generated_by`: Which primal executed

Complete attribution chain queryable at any time.

---

## Use Cases

### Medical AI
- Patient data encrypted (NestGate)
- Model trained securely (ToadStool)
- Complete provenance (SweetGrass)
- Fair attribution (Squirrel)
- HIPAA compliant

### Financial AI
- Trading data secured (NestGate)
- Models auditable (SweetGrass)
- Compute verifiable (ToadStool)
- Attribution transparent (Squirrel)
- Regulatory compliant

### Scientific Research
- Research data preserved (NestGate)
- Compute reproducible (ToadStool)
- Provenance published (SweetGrass)
- Credit fair (Squirrel)
- Open science enabled

---

## The Revolutionary Difference

### Current AI (Unfair, Opaque)

```
Data scraped → Black box training → Opaque model → Unattributed result
     ❌              ❌                  ❌               ❌
 No credit      No audit trail     No trust        No fairness
```

### ecoPrimals AI (Fair, Transparent)

```
Data tracked → Provenance training → Auditable model → Attributed result
     ✅              ✅                    ✅                  ✅
 Fair credit    Complete audit        Full trust      Fair fairness
```

---

## Why This Matters

### For Data Providers
- **Get paid** for contributions
- **Control** how data is used
- **Credit** when AI succeeds

### For Model Builders
- **Recognition** for work
- **IP protection** built-in
- **Fair compensation**

### For Infrastructure Providers
- **Credit** for compute/storage
- **Fair share** of value
- **Transparent pricing**

### For AI Users
- **Trust** through transparency
- **Compliance** built-in
- **Attribution** clear

### For Society
- **Fair AI** for everyone
- **Trust** through provenance
- **Transparency** by design

---

## Next Steps

1. **Run the demo** to see 4 primals working together
2. **Review outputs** in `outputs/multi-primal-*/`
3. **Explore** how this enables trustworthy AI for your use case
4. **Build** your own multi-primal workflows

---

## Comparison: Single vs Multi-Primal

### Single Primal (Limited)
```
SweetGrass alone:
- Tracks provenance ✅
- But: No secure storage
- But: No compute execution
- But: No AI attribution
```

### Multi-Primal (Complete)
```
NestGate + ToadStool + SweetGrass + Squirrel:
- Secure storage ✅
- Distributed compute ✅
- Complete provenance ✅
- Fair AI attribution ✅
- REVOLUTIONARY trustworthy AI ✅
```

---

## The Future of AI

This isn't incremental improvement. This is **revolutionary change**.

When AI has:
- Complete provenance (SweetGrass)
- Secure storage (NestGate)
- Verifiable compute (ToadStool)
- Fair attribution (Squirrel)

We unlock:
- **Trustworthy AI** (transparency)
- **Fair AI** (attribution)
- **Compliant AI** (audit trails)
- **Federated AI** (cross-org trust)

🌾 **This is the future. This is ecoPrimals.** 🌾

