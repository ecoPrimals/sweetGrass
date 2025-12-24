# 🌾 Demo: Multi-Primal Provenance

**Goal**: Track provenance across primal boundaries  
**Time**: 20 minutes  
**Complexity**: Advanced  
**Prerequisites**: Multiple primals available

---

## 🎯 What This Demo Shows

1. Data moving between primals
2. Unified provenance across boundaries
3. Cross-primal attribution
4. Complete ecosystem export

---

## 🚀 Run the Demo

```bash
./demo-cross-primal.sh
```

---

## 📖 Concepts

### The Challenge

When data flows through multiple primals:
- Squirrel processes with AI
- ToadStool runs heavy compute
- NestGate stores results
- Songbird coordinates

**Who keeps track of everything?**

### SweetGrass: The Unified Layer

SweetGrass sits across all primals as the provenance backbone:

```
┌─────────────────────────────────────────┐
│           SweetGrass Layer              │
│  (Provenance + Attribution)             │
├─────────────────────────────────────────┤
│                                          │
│  Squirrel ─→ ToadStool ─→ NestGate     │
│     │            │            │         │
│     └────────────┴────────────┘         │
│              ↓                          │
│         Songbird (coordinates)          │
│                                          │
└─────────────────────────────────────────┘
```

### Cross-Primal EntityReference

Braids can reference entities from any primal:

```rust
EntityReference::ByLoam {
    spine_id: "toadstool-results",
    entry_hash: "sha256:abc...",
}

EntityReference::External {
    uri: "squirrel://model/gpt-4o/inference/123",
}
```

---

## 📊 Expected Output

```
🌾 Multi-Primal Provenance Demo
===============================

Flow: AI → Compute → Storage

Step 1: Squirrel AI Processing
  Input: User query
  Model: Local LLM
  → Braid: urn:braid:squirrel-001
  Attribution: User 30%, AI 70%

Step 2: ToadStool GPU Compute
  Input: AI output + dataset
  Derived from: urn:braid:squirrel-001
  → Braid: urn:braid:toadstool-001
  Attribution: User 21%, AI 49%, GPU Worker 30%

Step 3: NestGate Storage
  Stores: Final result
  Derived from: urn:braid:toadstool-001
  → Braid: urn:braid:nestgate-001

Unified Query: provenance_graph(nestgate-001)
  → Returns 3 Braids across 3 primals
  → Full attribution chain
  → Complete PROV-O export

Attribution Summary:
  Original User:  21%
  Squirrel AI:    49%
  ToadStool GPU:  30%

✅ Cross-primal provenance complete!
```

---

## 🔧 Architecture

```
┌─────────────────────────────────────────────────────┐
│                 DATA FLOW                            │
├─────────────────────────────────────────────────────┤
│                                                      │
│  User Query                                          │
│      │                                               │
│      ↓                                               │
│  ┌──────────┐                                       │
│  │ Squirrel │  AI inference                         │
│  │   (AI)   │  → creates Braid (squirrel-001)      │
│  └────┬─────┘                                       │
│       │ SweetGrass: records provenance              │
│       ↓                                             │
│  ┌──────────┐                                       │
│  │ToadStool │  GPU compute                          │
│  │(Compute) │  → derives Braid (toadstool-001)     │
│  └────┬─────┘                                       │
│       │ SweetGrass: links derivation                │
│       ↓                                             │
│  ┌──────────┐                                       │
│  │ NestGate │  Persistent storage                   │
│  │(Storage) │  → stores with Braid (nestgate-001)  │
│  └────┬─────┘                                       │
│       │ SweetGrass: records storage                 │
│       ↓                                             │
│  ┌──────────┐                                       │
│  │SweetGrass│  Query: "Who contributed?"            │
│  │ (Query)  │  → Alice 21%, AI 49%, GPU 30%        │
│  └──────────┘                                       │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 💡 Key Insights

### Unified View
Query any Braid and get its complete history across all primals.

### Attribution Spans Boundaries
Contributors from different primals are all credited fairly.

### Standards-Based
PROV-O export includes references to all primals.

---

## 🎯 Success Criteria

- [ ] Understood cross-primal data flow
- [ ] Queried unified provenance
- [ ] Calculated cross-primal attribution
- [ ] Exported complete PROV-O

---

## 📚 Next Steps

Continue to: `../03-reward-distribution/`

Learn how attribution enables fair reward distribution!

(Note: Reward distribution requires sunCloud integration, planned for v0.3.0)

