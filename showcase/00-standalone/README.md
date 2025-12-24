# 🌾 Level 0: Standalone SweetGrass

**Goal**: Understand SweetGrass as a complete attribution platform  
**Prerequisites**: SweetGrass built (`cargo build`)  
**Time**: 40 minutes  
**Complexity**: Beginner-Intermediate

---

## 🎯 What You'll Learn

- Braid creation and management
- Attribution calculation with role weights
- Provenance graph traversal
- W3C PROV-O export
- Privacy controls and data subject rights

---

## 📁 Demos

### 1. Braid Basics (5 min) ⭐ START HERE
**Directory**: `01-braid-basics/`

Learn the fundamentals of Braids - cryptographically signed provenance records.

```bash
cd 01-braid-basics
./demo-create-braid.sh
```

**What you'll see**:
- Create a Braid from raw data
- Query Braids by ID and hash
- Create derived Braids
- View Braid metadata

**Key Concepts**:
- `BraidId` - Unique identifier (URN format)
- `ContentHash` - SHA-256 of data content
- `was_derived_from` - Links to source Braids
- `was_attributed_to` - Primary contributor (DID)

---

### 2. Attribution Engine (10 min)
**Directory**: `02-attribution-engine/`

Calculate fair attribution across contribution chains.

```bash
cd 02-attribution-engine
./demo-attribution.sh
```

**What you'll see**:
- Role-based weights (Creator: 1.0, Contributor: 0.5, etc.)
- Attribution propagation through derivations
- Time decay calculations
- Final reward proportions

**Example Attribution Chain**:
```
Document Created by Alice (Creator: 100%)
    ↓ Bob adds analysis (Contributor)
Processed Version (Alice: 70%, Bob: 30%)
    ↓ Charlie creates visualization (Creator)
Final Output (Alice: 49%, Bob: 21%, Charlie: 30%)
```

**Key Concepts**:
- `AgentRole` - 12 roles with default weights
- `AttributionCalculator` - Computes shares
- Decay factor - Reduces attribution over time
- Derivation depth - Limits chain traversal

---

### 3. Provenance Queries (10 min)
**Directory**: `03-provenance-queries/`

Traverse the provenance graph (DAG) to understand data history.

```bash
cd 03-provenance-queries
./demo-queries.sh
```

**What you'll see**:
- Build provenance graph from any Braid
- Walk ancestors (sources)
- Walk descendants (derivatives)
- Filter by activity type, agent, time range

**Query Types**:
- `provenance(hash)` - Full history for content
- `derived_from(braid)` - What was used to create this?
- `by_agent(did)` - All Braids by an agent
- `activities_for(braid)` - What happened?

**Key Concepts**:
- `ProvenanceGraph` - DAG of provenance relationships
- `QueryEngine` - Unified query interface
- Depth limiting - Prevent infinite traversal
- Cycle detection - Handle loops gracefully

---

### 4. PROV-O Export (5 min)
**Directory**: `04-provo-export/`

Export to W3C standard JSON-LD format.

```bash
cd 04-provo-export
./demo-export.sh
```

**What you'll see**:
- Convert Braids to PROV-O entities
- Activities as prov:Activity
- Agents as prov:Agent
- Standard JSON-LD context

**Example Output**:
```json
{
  "@context": {
    "prov": "http://www.w3.org/ns/prov#",
    "xsd": "http://www.w3.org/2001/XMLSchema#"
  },
  "@graph": [
    {
      "@id": "entity:braid-abc123",
      "@type": "prov:Entity",
      "prov:wasGeneratedBy": "activity:process-456",
      "prov:wasAttributedTo": "agent:did:key:z6Mk..."
    }
  ]
}
```

**Key Concepts**:
- `ProvoExporter` - Converts to standard format
- JSON-LD - Linked data format
- Interoperability - Share with other PROV systems

---

### 5. Privacy Controls (10 min)
**Directory**: `05-privacy-controls/`

GDPR-inspired data subject rights.

```bash
cd 05-privacy-controls
./demo-privacy.sh
```

**What you'll see**:
- Privacy levels (Public, Private, Encrypted)
- Retention policies (Duration, LegalHold)
- Data subject requests (Access, Erasure)
- Consent tracking

**Privacy Levels**:
| Level | Description |
|-------|-------------|
| `Public` | Visible to all |
| `Authenticated` | Requires auth |
| `Private` | Owner + explicit grants |
| `Encrypted` | Requires decryption key |
| `AnonymizedPublic` | Anonymized version public |

**Data Subject Rights**:
- **Access** - Get all data about a subject
- **Rectification** - Correct inaccurate data
- **Erasure** - "Right to be forgotten"
- **Portability** - Export in standard format
- **Objection** - Opt out of processing

**Key Concepts**:
- `PrivacyMetadata` - Attached to Braids
- `RetentionPolicy` - When to delete
- `ConsentDetails` - How consent obtained
- `PrivacyLevel` - Visibility control

---

## 🚀 Quick Start

### Run All Level 0 Demos
```bash
# From showcase/00-standalone/
for dir in */; do
  echo "=== Running $dir ==="
  cd "$dir" && ./demo-*.sh && cd ..
done
```

### Run Specific Demo
```bash
cd 01-braid-basics
./demo-create-braid.sh
```

---

## 📊 Expected Output

### Braid Creation
```
🌾 Creating Braid from data...

Braid Created:
  ID: urn:braid:abc123-def456-...
  Hash: sha256:7f83b1657ff...
  Size: 1024 bytes
  Type: text/plain
  Creator: did:key:z6MkAlice...

✅ Braid stored successfully!
```

### Attribution Calculation
```
📊 Calculating attribution chain...

Source Braid (Alice):
  - Role: Creator (weight: 1.0)
  - Share: 100%

Derived Braid (Alice + Bob):
  - Alice: 70% (Creator → Derived)
  - Bob: 30% (Contributor)

Final Braid (Alice + Bob + Charlie):
  - Alice: 49%
  - Bob: 21%
  - Charlie: 30%

✅ Attribution calculated!
```

---

## 🛠️ Configuration

### Default Agent (for demos)
```rust
let agent = Did::new("did:key:z6MkDemoAgent");
```

### Default Store (in-memory)
```rust
let store = MemoryStore::new();
```

### Attribution Weights
```rust
AgentRole::Creator => 1.0
AgentRole::Contributor => 0.5
AgentRole::DataProvider => 0.4
AgentRole::Transformer => 0.3
AgentRole::Curator => 0.2
AgentRole::Publisher => 0.1
```

---

## 💡 Key Insights

### Braids Are Immutable
Once created, a Braid cannot be modified. Create a new derived Braid instead.

### Attribution Is Proportional
Attribution flows through the derivation chain. Contributors to sources get credit in derivatives.

### Privacy Is Attached
Privacy metadata travels with the Braid. Controls are enforced at query time.

### PROV-O Is Standard
Export to PROV-O for interoperability with other provenance systems.

---

## 🎯 Success Criteria

Level 0 is complete when you can:

- [ ] Create a Braid from raw data
- [ ] Create a derived Braid
- [ ] Calculate attribution for a chain
- [ ] Query provenance history
- [ ] Export to PROV-O JSON-LD
- [ ] Configure privacy settings

---

## 📚 Next Steps

After Level 0, proceed to:

1. **Level 1**: `../01-primal-coordination/README.md`
   - Sign Braids with BearDog
   - Compress sessions from RhizoCrypt
   - Anchor commits with LoamSpine

2. **Experiment**:
   - Create deeper derivation chains
   - Try different role weights
   - Test privacy access controls

---

**Ready?** Start with `01-braid-basics/demo-create-braid.sh`!

🌾 **Let's tell some data stories!** 🌾

