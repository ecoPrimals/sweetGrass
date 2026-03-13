# 🌐 Multi-Primal Workflows

**Real integration scenarios using 3-4 primals working together**

## 🎯 Purpose

Demonstrate how multiple primals coordinate to solve real-world problems. Each workflow shows:

- **Real binaries** from `../bins/` (NO MOCKS!)
- **Complete provenance** tracked in SweetGrass
- **Fair attribution** across all contributors
- **Integration gaps** that guide our evolution

## 📋 Available Workflows

### 1️⃣ **Songbird + SweetGrass + NestGate** 🐦🌾🏰
**Secure Data Pipeline with Storage**

```bash
./01-songbird-sweetgrass-nestgate.sh
```

**What it demonstrates:**
- Secure message delivery via Songbird
- Complete provenance tracking in SweetGrass
- Persistent storage in NestGate
- End-to-end audit trail

**Real-world use case:** Compliant document workflow

---

### 2️⃣ **ToadStool + SweetGrass + NestGate** 🍄🌾🏰
**Compute → Provenance → Storage**

```bash
./02-toadstool-sweetgrass-nestgate.sh
```

**What it demonstrates:**
- ML training on ToadStool compute
- Provenance of training data and models
- Model storage in NestGate
- Fair attribution for compute, data, and engineering

**Real-world use case:** ML model development pipeline

---

### 3️⃣ **Songbird + SweetGrass + Squirrel** 🐦🌾🐿️
**Messaging → Provenance → AI Agents**

```bash
./03-songbird-sweetgrass-squirrel.sh
```

**What it demonstrates:**
- Customer messages via Songbird
- AI agent analysis via Squirrel
- Complete AI decision provenance
- Transparent AI behavior tracking

**Real-world use case:** AI-augmented customer support

---

### 4️⃣ **Full Stack: Songbird + ToadStool + SweetGrass + NestGate** 🐦🍄🌾🏰
**Complete Data Science Pipeline**

```bash
./04-full-stack-data-science.sh
```

**What it demonstrates:**
- Data ingestion via Songbird
- Compute processing via ToadStool
- Provenance tracking via SweetGrass
- Result storage via NestGate
- Multi-stage attribution

**Real-world use case:** Enterprise data science workflow

---

## 🏗️ Architecture Patterns

### Pattern 1: Linear Pipeline
```
Primal A → Primal B → Primal C
```
Each primal adds value sequentially.

### Pattern 2: Hub and Spoke
```
        ┌─ Primal B
Primal A ┼─ Primal C
        └─ Primal D
```
Central coordinator with multiple services.

### Pattern 3: Mesh Network
```
Primal A ←→ Primal B
    ↕          ↕
Primal C ←→ Primal D
```
Peer-to-peer collaboration.

---

## 💡 Key Principles

### 1. **No Mocks in Showcase**
All demos use real binaries from `../bins/`. If a binary isn't available, the script gracefully degrades and documents the gap.

### 2. **Infant Discovery**
Primals discover each other at runtime based on capabilities, not hardcoded names or addresses.

### 3. **Complete Provenance**
Every interaction is tracked in SweetGrass with full attribution chains.

### 4. **Fair Attribution**
All contributors (humans, primals, AI agents) receive fair credit based on their role and contribution.

### 5. **Integration Gaps → Evolution**
When integrations fail or are incomplete, we document the gap and evolve our implementations.

---

## 🧪 Testing Integration Gaps

Each workflow script:
1. **Attempts real integration** with binaries from `../bins/`
2. **Documents what works** and what doesn't
3. **Gracefully degrades** when binaries are missing
4. **Saves detailed logs** for debugging

**Example gap discovery:**
```bash
⚠️  ToadStool BYOB server not found
→ Gap identified: Need BYOB server configuration
→ Evolution needed: Add BYOB server to bins/
```

---

## 📊 Attribution Examples

### Compute Pipeline Attribution
```
Data Collector:    25% - Provided training data
ML Engineer:       40% - Designed and trained model
ToadStool:         25% - Provided compute resources
NestGate:          10% - Persistent storage
```

### AI Support Attribution
```
Customer:          20% - Reported issue
Squirrel AI:       50% - Analyzed and responded
Songbird:          15% - Secure message delivery
SweetGrass:        15% - Provenance tracking
```

---

## 🚀 Running All Workflows

```bash
# Run all workflows in sequence
for script in *.sh; do
    echo "Running $script..."
    ./"$script"
    echo ""
done
```

---

## 📁 Output Structure

Each workflow creates an `outputs/` directory:

```
outputs/
├── compute-1735234567/
│   ├── workflow.log
│   ├── input-braid.json
│   ├── compute-braid.json
│   ├── trained-model.bin
│   └── attribution.txt
└── messaging-1735234890/
    ├── workflow.log
    ├── customer-message-braid.json
    ├── analysis-braid.json
    ├── response-braid.json
    └── attribution.txt
```

---

## 🎓 What You'll Learn

1. **How primals coordinate** without hardcoded dependencies
2. **How provenance flows** across multiple services
3. **How attribution works** in complex workflows
4. **Where integration gaps exist** in our current implementation
5. **How to build** sovereign, capability-based systems

---

## 🔗 Related Showcases

- **[00-local-primal](../../00-local-primal/)** - SweetGrass by itself
- **[01-primal-coordination](../)** - Two-primal integrations
- **Phase 1 Primals** - Mature implementations in `../../../../phase1/`

---

## 🌾 The SweetGrass Promise

**Every interaction is tracked. Every contributor is credited. Every decision is auditable.**

That's the power of provenance! 🌾
