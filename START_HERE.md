# 🌾 SweetGrass — Start Here

Welcome to SweetGrass! This guide will get you up to speed quickly.

---

## 📖 What is SweetGrass?

SweetGrass is the **Attribution Layer** for Phase 2 of ecoPrimals. It's the "storyteller" — weaving meaning into data by tracking provenance and attribution. SweetGrass answers: **"What is the story of this data?"**

**Key insight**: SweetGrass doesn't store data; it stores stories *about* data — who created it, who contributed, how it was transformed.

---

## 🚀 Quick Start

### 1. Build the Project

```bash
cd /path/to/ecoPrimals/phase2/sweetGrass
cargo build
```

### 2. Run Tests

```bash
cargo test
```

### 3. Explore the Code

```bash
# Main entry point
cat crates/sweet-grass-core/src/lib.rs

# Configuration
cat crates/sweet-grass-core/src/config.rs

# Error types
cat crates/sweet-grass-core/src/error.rs
```

---

## 🏗️ Architecture Overview

```
SweetGrass
    │
    ├── Braids (provenance records)
    │   ├── Entity (what data)
    │   │   ├── Content hash
    │   │   ├── MIME type
    │   │   └── Size
    │   │
    │   ├── Activity (how created)
    │   │   ├── Activity type
    │   │   ├── Timestamps
    │   │   └── Used entities
    │   │
    │   └── Attribution (who contributed)
    │       ├── Agent (DID)
    │       ├── Role
    │       └── Contribution weight
    │
    ├── Queries (attribution engine)
    │   ├── Provenance graph
    │   ├── Attribution chain
    │   └── Contributor shares
    │
    └── Export (interoperability)
        ├── JSON-LD
        └── W3C PROV-O
```

---

## 📚 Key Concepts

### 1. Braids
Cryptographically signed provenance records (PROV-O compatible):
- **Entity** — The data being described
- **Activity** — How it was generated
- **Attribution** — Who contributed

### 2. PROV-O Model
W3C standard for provenance:
```
Entity (data) ←─ wasGeneratedBy ─→ Activity
                                      │
                                      ↓
                              wasAssociatedWith
                                      │
                                      ↓
                                   Agent
```

### 3. Attribution Chains
Track contributions through derivations:
```
Original Data (Creator: Alice, 100%)
    ↓ derived
Processed Data (Alice: 70%, Bob: 30%)
    ↓ derived
Final Output (Alice: 50%, Bob: 20%, Charlie: 30%)
```

### 4. Radiating Attribution
Powers sunCloud economics:
- Walk the attribution chain
- Calculate contributor shares
- Distribute value proportionally

---

## 📂 Project Structure

```
sweetGrass/
├── Cargo.toml           # Workspace manifest
├── README.md            # Overview
├── STATUS.md            # Current status
├── WHATS_NEXT.md        # Roadmap
├── START_HERE.md        # This file
│
├── crates/
│   └── sweet-grass-core/    # Core library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # Entry + traits
│           ├── config.rs    # Configuration
│           └── error.rs     # Error types
│
├── specs/
│   └── SWEETGRASS_SPECIFICATION.md  # Full spec (~900 lines)
│
└── showcase/            # Demo applications (coming soon)
```

---

## 🔗 Integration Points

### Depends On (Gen 1)
| Primal | Purpose |
|--------|---------|
| **BearDog** | Braid signing, DID resolution |
| **Songbird** | Service discovery |

### Phase 2 Siblings
| Primal | Relationship |
|--------|--------------|
| **RhizoCrypt** | Traverses DAG for provenance |
| **LoamSpine** | Listens to commit events |

### Downstream Systems
| System | Relationship |
|--------|--------------|
| **sunCloud** | Queries for attribution shares |
| **gAIa** | Trust assessment from provenance |

---

## 🎯 Current Status

| Aspect | Status |
|--------|--------|
| **Scaffolding** | ✅ Complete |
| **Build** | ✅ Passing |
| **Braid Types** | ⬜ Not started |
| **Query Engine** | ⬜ Not started |
| **PROV-O Export** | ⬜ Not started |

See [STATUS.md](./STATUS.md) for detailed status.

---

## 📝 Next Steps for Contributors

### Immediate (Week 5)
1. Implement `Braid` struct (PROV-O Entity)
2. Implement `Activity` struct
3. Implement `Attribution` struct
4. Add Braid signing

### Short Term (Weeks 6-8)
1. Implement provenance graph traversal
2. Implement attribution calculation
3. Implement PROV-O JSON-LD export

See [WHATS_NEXT.md](./WHATS_NEXT.md) for full roadmap.

---

## 📖 Further Reading

| Document | Description |
|----------|-------------|
| [specs/SWEETGRASS_SPECIFICATION.md](./specs/SWEETGRASS_SPECIFICATION.md) | Complete technical specification |
| [W3C PROV-O](https://www.w3.org/TR/prov-o/) | Provenance Ontology standard |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Unified Phase 2 architecture |
| [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) | Cross-primal data flows |
| [../sourDough/CONVENTIONS.md](../sourDough/CONVENTIONS.md) | Coding conventions |

---

## 💡 The Storyteller Analogy

From the specification:

> If RhizoCrypt is the chaotic workshop and LoamSpine is the museum, SweetGrass is the **curator's notebook** — documenting not just what exists, but its context, lineage, and significance.
>
> SweetGrass doesn't store data; it stores **stories about data**:
> - The genome sequence exists in NestGate
> - The computation that processed it ran on ToadStool
> - The session that captured the process lives in RhizoCrypt
> - The final result is anchored in LoamSpine
> - **SweetGrass tells you how they're all connected**

---

## 💡 Attribution Use Cases

### Scientific Discovery
```
Scientist submits protein folding job
    ↓
ToadStool processes (1.5 compute units)
    ↓
RhizoCrypt captures session
    ↓
LoamSpine commits result
    ↓
SweetGrass creates Braid:
  - Entity: Output structure
  - Activity: AlphaFold processing
  - Attribution:
    - Scientist (creator, 40%)
    - Compute provider (transformer, 30%)
    - Training data source (derived, 30%)
```

### Years Later
```
Discovery leads to drug target
    ↓
sunCloud queries SweetGrass
    ↓
Attribution chain calculated
    ↓
Rewards distributed to all contributors
```

---

## ❓ Questions?

- Check [STATUS.md](./STATUS.md) for current state
- Check [WHATS_NEXT.md](./WHATS_NEXT.md) for roadmap
- Read the [specification](./specs/SWEETGRASS_SPECIFICATION.md) for deep details

---

*SweetGrass: Every piece of data has a story.*

