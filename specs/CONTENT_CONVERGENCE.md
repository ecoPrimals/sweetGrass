# SweetGrass — Content Convergence Specification

**Version**: 0.1.0
**Status**: Proposed
**Last Updated**: March 16, 2026
**Depends On**: [DATA_MODEL.md](./DATA_MODEL.md), [ARCHITECTURE.md](./ARCHITECTURE.md)

---

## 1. Motivation

SweetGrass indexes Braids by their `ContentHash` for O(1) retrieval. The current
in-memory index uses a `HashMap<ContentHash, BraidId>` — a 1:1 mapping where
the last write wins. When two independent agents produce Braids with identical
content hashes (same data, different provenance paths), the earlier index entry
is silently overwritten.

This "collision" is not a bug in the hash function — it is **provenance convergence**:
independent paths arriving at the same content. The convergence itself carries
semantic meaning that the current lossy index discards.

The derivation index (`HashMap<ContentHash, HashSet<BraidId>>`) already preserves
1:many relationships for derivation sources. The content hash index should evolve
to match.

### The Cross-Hatched Letter Analogy

Historically, writers overlaid text by rotating the page 90 degrees and writing
across existing lines — "cross-hatching" — because paper was scarce. Both layers
of information persisted. The constraint (limited paper) produced a richer artifact.

Content convergence is the digital equivalent: when the "paper" (a `ContentHash`)
is shared by multiple provenance paths, the layered arrivals carry more
information than either alone. The convergence point becomes a natural site
for provenance intersection analysis.

### Connection to rhizoCrypt and loamSpine

- **rhizoCrypt** resolves intersections by growing more DAG — branching is the
  native response to convergence in the ephemeral network.
- **loamSpine** resolves intersections through linear spine chaining — the
  permanent record appends rather than branches.
- **sweetGrass** sits above both and must represent both linear and branching
  convergence patterns in its provenance index.

The evolution from "collision-lossy" to "collision-preserving" indexing aligns
sweetGrass with both substrates without privileging either.

---

## 2. Current State

### 2.1 Content Hash Index (Lossy)

```rust
// crates/sweet-grass-store/src/memory/indexes.rs
pub(super) struct Indexes {
    /// content hash → Braid ID (1:1, last-write-wins)
    pub hash: RwLock<HashMap<ContentHash, BraidId>>,
    // ...
}
```

When `Indexes::add()` is called with a Braid whose `data_hash` already exists
in the index, the previous `BraidId` mapping is silently replaced.

### 2.2 Derivation Index (Preserving)

```rust
    /// derivation source hash → Braid IDs (1:many)
    pub derivation: RwLock<HashMap<ContentHash, HashSet<BraidId>>>,
```

This index already handles the 1:many case correctly — multiple Braids can
derive from the same source content.

### 2.3 What Is Lost

When two agents independently produce Braids with the same `ContentHash`:

| Preserved | Lost |
|-----------|------|
| Latest Braid's ID | Earlier Braid's ID |
| Latest agent's DID | Convergence timestamp delta |
| Content data | Independent derivation paths |
| — | Agent agreement signal |

---

## 3. Proposed Evolution

### 3.1 ContentConvergence

Replace the 1:1 content hash mapping with a convergence-preserving structure:

```rust
/// What lives at a content hash when multiple provenance paths converge.
///
/// Analogous to the "sub-hash" concept: instead of discarding collisions,
/// we capture what data lies at the convergence point.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentConvergence {
    /// The first Braid indexed for this content hash.
    pub primary: BraidId,

    /// Subsequent arrivals at the same content hash.
    /// Empty when only one Braid maps to this hash (the common case).
    pub convergent: Vec<ConvergentArrival>,
}

/// A provenance path that arrived at content already indexed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConvergentArrival {
    /// The Braid that converged on existing content.
    pub braid_id: BraidId,

    /// The agent who produced this convergent Braid.
    pub agent: Did,

    /// When this convergent arrival was indexed.
    pub arrived_at: Timestamp,

    /// The derivation path this agent followed to reach the same content.
    pub derivation_path: Vec<EntityReference>,
}
```

### 3.2 Evolved Index

```rust
pub(super) struct Indexes {
    /// content hash → convergence record (1:1+N)
    pub hash: RwLock<HashMap<ContentHash, ContentConvergence>>,
    // ... other indexes unchanged
}
```

### 3.3 Insertion Semantics

```rust
impl Indexes {
    pub fn add(&self, braid: &Braid) {
        let mut hash_idx = self.hash.write();
        match hash_idx.entry(braid.data_hash.clone()) {
            Entry::Vacant(e) => {
                e.insert(ContentConvergence {
                    primary: braid.id.clone(),
                    convergent: Vec::new(),
                });
            }
            Entry::Occupied(mut e) => {
                e.get_mut().convergent.push(ConvergentArrival {
                    braid_id: braid.id.clone(),
                    agent: braid.was_attributed_to.clone(),
                    arrived_at: braid.generated_at_time,
                    derivation_path: braid.was_derived_from.clone(),
                });
            }
        }
        // ... other indexes
    }
}
```

### 3.4 Query Semantics

| Query | Current Behavior | Evolved Behavior |
|-------|-----------------|------------------|
| `get_by_hash(h)` | Returns `Option<BraidId>` (latest) | Returns `Option<&ContentConvergence>` |
| `get_by_hash(h).primary` | N/A | First Braid indexed for this hash |
| `get_by_hash(h).convergent` | N/A | All subsequent arrivals |
| `convergence_count(h)` | N/A | `1 + convergent.len()` |
| `convergence_agents(h)` | N/A | Set of all agents who converged |

---

## 4. Data Science Opportunities

### 4.1 Convergence as Signal

When independent agents produce the same content hash, this is a strong signal:

- **Reproducibility**: Two independent computations produced identical results
- **Consensus**: Multiple agents agree on a data artifact
- **Redundancy**: The same content exists via multiple provenance paths

### 4.2 Controlled Hash Table Sizing

By modifying hash table sizes and hash functions, convergence rates change.
This is analogous to the sub-hashing technique: intentionally adjusting the
hash space to control collision granularity.

| Technique | Effect | Use Case |
|-----------|--------|----------|
| Narrower hash (truncated) | More convergence | Similarity clustering |
| Wider hash (full SHA-256) | Less convergence | Exact content identity |
| Domain-specific hash | Semantic convergence | Cross-session pattern discovery |
| Locality-sensitive hash | Approximate convergence | Near-duplicate detection |

### 4.3 Linear ↔ Branching Duality

SweetGrass already embodies both paradigms:

| Paradigm | Structure | Analog |
|----------|-----------|--------|
| **Linear** | `IndexMap` (temporal order), LoamSpine anchors | Spine, append-only log |
| **Branching** | `was_derived_from: Vec<EntityReference>` (DAG) | rhizoCrypt DAG, fungal mycelium |
| **Convergence** | `ContentConvergence` (proposed) | Cross-hatched letter, rhizo intersection |

The convergence layer is where linear and branching coexist: the linear timeline
of arrivals intersects with the branching graph of derivations. This mirrors
biological systems where linear growth (hyphae extension) and branching
(anastomosis) produce emergent network topology.

---

## 5. Storage Backend Implications

### 5.1 Memory Store

Direct evolution of `Indexes` as described in Section 3.

### 5.2 PostgreSQL

```sql
-- New table for convergence tracking
CREATE TABLE IF NOT EXISTS content_convergence (
    content_hash TEXT NOT NULL,
    braid_id TEXT NOT NULL,
    agent_did TEXT NOT NULL,
    arrived_at BIGINT NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    derivation_path JSONB,
    PRIMARY KEY (content_hash, braid_id)
);

CREATE INDEX idx_convergence_hash ON content_convergence (content_hash);
CREATE INDEX idx_convergence_primary ON content_convergence (content_hash)
    WHERE is_primary = TRUE;
```

### 5.3 redb

```rust
const CONVERGENCE_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("content_convergence");
```

Key: `ContentHash`, Value: bincode-serialized `ContentConvergence`.

---

## 6. API Surface

### 6.1 New JSON-RPC Methods

| Method | Description |
|--------|-------------|
| `convergence.query` | Query convergence records by content hash |
| `convergence.agents` | List agents who converged on a content hash |
| `convergence.timeline` | Temporal view of convergent arrivals |
| `convergence.statistics` | Aggregate convergence metrics across store |

### 6.2 Extended Existing Methods

| Method | Extension |
|--------|-----------|
| `braid.get_by_hash` | Returns convergence metadata alongside primary Braid |
| `provenance.graph` | Convergence edges in graph export |
| `provenance.export_provo` | Convergent arrivals as PROV-O alternateOf relations |

---

## 7. Relationship to Ecosystem Experiments

This specification defines sweetGrass's internal architecture. The broader
ecosystem experiment — exploring hash-table-sizing techniques as data science
tools — is coordinated via:

- **ISSUE-013** in `wateringHole/SPRING_EVOLUTION_ISSUES.md`
- **Experiment guide** in `wateringHole/CONTENT_CONVERGENCE_EXPERIMENT_GUIDE.md`

Springs that can contribute:

| Spring | Contribution |
|--------|-------------|
| **wetSpring** | Molecular fingerprint convergence (same molecule, different basis sets) |
| **groundSpring** | Geological measurement convergence (overlapping sensor grids) |
| **hotSpring** | Plasma parameter convergence (independent simulation paths) |
| **airSpring** | Agricultural measurement convergence (multi-sensor ET₀) |
| **neuralSpring** | Model output convergence (ensemble agreement) |
| **ludoSpring** | Game state convergence (independent agent strategies) |

---

## 8. Implementation Phases

### Phase 1: Core Types (sweetGrass)
- Define `ContentConvergence` and `ConvergentArrival` in `sweet-grass-core`
- Evolve `MemoryStore` indexes
- Add `convergence.query` JSON-RPC method
- Unit and integration tests

### Phase 2: Storage Backends
- PostgreSQL `content_convergence` table and migrations
- redb convergence table
- Query filter extensions

### Phase 3: PROV-O Integration
- Convergent arrivals as `prov:alternateOf` relations
- Graph export includes convergence edges
- JSON-LD context extended with convergence vocabulary

### Phase 4: Ecosystem Experiments
- Springs produce convergent content via independent paths
- sweetGrass collects convergence statistics
- Analysis of convergence patterns as reproducibility signals

---

## 9. References

- [DATA_MODEL.md](./DATA_MODEL.md) — Core Braid and Entity structures
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [BRAID_COMPRESSION.md](./BRAID_COMPRESSION.md) — Session compression (related DAG patterns)
- `wateringHole/CONTENT_CONVERGENCE_EXPERIMENT_GUIDE.md` — Spring experiment guide
- `wateringHole/SPRING_EVOLUTION_ISSUES.md` ISSUE-013 — Ecosystem coordination
- `rhizoCrypt/specs/CONTENT_INDEX_EXPERIMENT.md` — Related LSH experiment (ISSUE-012)
- W3C PROV-O `prov:alternateOf` — [https://www.w3.org/TR/prov-o/#alternateOf](https://www.w3.org/TR/prov-o/#alternateOf)

---

*When independent paths converge on the same content, the convergence itself is provenance.*
