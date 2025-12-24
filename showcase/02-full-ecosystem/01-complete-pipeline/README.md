# 🌾 Demo: Complete Attribution Pipeline

**Goal**: Track data from creation through all transformations  
**Time**: 20 minutes  
**Complexity**: Advanced  
**Prerequisites**: All primals available

---

## 🎯 What This Demo Shows

1. Multi-contributor data creation
2. Derivation chain tracking
3. Cross-primal coordination
4. Complete attribution calculation
5. PROV-O export of full history

---

## 🚀 Run the Demo

```bash
./demo-full-pipeline.sh
```

---

## 📖 The Story

### Scene 1: Data Creation
**Alice** creates a climate research dataset:
- Signs with BearDog
- Records in SweetGrass
- Stores in NestGate (optional)

### Scene 2: Analysis
**Bob** runs analysis on Alice's data:
- RhizoCrypt captures session
- SweetGrass compresses to Braid
- Links derivation to Alice's Braid

### Scene 3: Visualization
**Charlie** creates visualization from Bob's analysis:
- Signs with BearDog
- Anchors with LoamSpine
- Links derivation to Bob's Braid

### Scene 4: Attribution
Query: "Who contributed to Charlie's visualization?"
- SweetGrass calculates attribution chain
- Alice: 49%, Bob: 21%, Charlie: 30%

---

## 📊 Expected Output

```
🌾 Complete Attribution Pipeline Demo
=====================================

═══════════════════════════════════════
Scene 1: Alice Creates Dataset
═══════════════════════════════════════

Creating dataset...
  Data: Climate measurements
  Agent: did:key:z6MkAlice
  ✅ Braid: urn:braid:alice-001

Signing with BearDog...
  ✅ Signature: Ed25519Signature2020

═══════════════════════════════════════
Scene 2: Bob Runs Analysis
═══════════════════════════════════════

Starting RhizoCrypt session...
  Session: analysis-session-001
  Vertices: 5
  ✅ Session committed

Compressing session...
  Derived from: urn:braid:alice-001
  ✅ Braid: urn:braid:bob-001

═══════════════════════════════════════
Scene 3: Charlie Creates Visualization
═══════════════════════════════════════

Creating visualization...
  Derived from: urn:braid:bob-001
  Agent: did:key:z6MkCharlie
  ✅ Braid: urn:braid:charlie-001

Anchoring with LoamSpine...
  Spine: spine-main
  Index: 42
  ✅ Anchored

═══════════════════════════════════════
Scene 4: Attribution Calculation
═══════════════════════════════════════

Query: Attribution for Charlie's visualization

Provenance Graph:
  [Alice: Dataset] ──┐
                     ├──> [Bob: Analysis] ──> [Charlie: Viz]
  [Bob: Code] ───────┘

Attribution Chain:
  Alice:   49.0% (Creator → Derived → Derived)
  Bob:     21.0% (Creator → Derived)
  Charlie: 30.0% (Creator)

Reward Distribution ($1000 total):
  Alice:   $490.00
  Bob:     $210.00
  Charlie: $300.00

═══════════════════════════════════════
PROV-O Export
═══════════════════════════════════════

{
  "@context": { "prov": "http://www.w3.org/ns/prov#" },
  "@graph": [
    { "@id": "entity:alice-001", "@type": "prov:Entity" },
    { "@id": "entity:bob-001", "prov:wasDerivedFrom": "entity:alice-001" },
    { "@id": "entity:charlie-001", "prov:wasDerivedFrom": "entity:bob-001" }
  ]
}

═══════════════════════════════════════
✅ Pipeline Complete!
═══════════════════════════════════════

Summary:
  Braids created: 3
  Contributors: 3
  Total attribution: 100%
  Anchored: 1
  PROV-O entities: 3
```

---

## 🔧 Architecture

```
┌─────────────────────────────────────────────────────┐
│                COMPLETE PIPELINE                     │
├─────────────────────────────────────────────────────┤
│                                                      │
│  Alice ──┬── BearDog (sign) ──┬── SweetGrass       │
│          │                    │     │               │
│          └── NestGate (store) │     │               │
│                               │     ↓               │
│  Bob ────── RhizoCrypt ───────┴── Compress         │
│             (session)              │               │
│                                    ↓               │
│  Charlie ─── BearDog (sign) ───── Derived          │
│          │                         │               │
│          └── LoamSpine (anchor) ───┘               │
│                                    │               │
│                                    ↓               │
│                            ATTRIBUTION              │
│                        Alice 49% Bob 21%           │
│                           Charlie 30%              │
│                                                      │
└─────────────────────────────────────────────────────┘
```

---

## 💡 Key Insights

### Every Step Is Recorded
SweetGrass captures provenance at each transformation.

### Attribution Flows Automatically
Derivation links enable automatic attribution calculation.

### Standards-Based Export
PROV-O export enables interoperability with other systems.

---

## 🎯 Success Criteria

- [ ] Tracked complete data lifecycle
- [ ] Computed multi-contributor attribution
- [ ] Exported cross-primal provenance
- [ ] Calculated reward distribution

---

## 📚 Next Steps

Continue to: `../02-multi-primal-provenance/`

Learn how provenance works across primal boundaries!

