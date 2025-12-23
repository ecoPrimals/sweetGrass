# 🌾 SweetGrass — What's Next

**Last Updated**: December 22, 2025

---

## 🎯 Implementation Roadmap

### Phase 1: Braid Structure (Weeks 5-6)

**Goal**: Implement PROV-O compatible provenance records.

> **Note**: SweetGrass development starts after core RhizoCrypt/LoamSpine work.

#### Week 5: Core Types
- [ ] Implement `BraidId` type
- [ ] Implement `EntityReference` for linking
- [ ] Implement core `Braid` structure (PROV-O Entity):
  ```rust
  pub struct Braid {
      pub id: BraidId,
      pub data_hash: ContentHash,
      pub mime_type: String,
      pub size: u64,
      pub was_generated_by: Option<Activity>,
      pub was_derived_from: Vec<EntityReference>,
      pub was_attributed_to: Vec<Attribution>,
      pub signature: Signature,
  }
  ```
- [ ] Implement `Activity` structure (PROV-O Activity):
  ```rust
  pub struct Activity {
      pub id: ActivityId,
      pub activity_type: ActivityType,
      pub started_at: Timestamp,
      pub ended_at: Option<Timestamp>,
      pub was_associated_with: Vec<AgentRef>,
      pub used: Vec<EntityReference>,
  }
  ```
- [ ] Add Braid serialization (JSON-LD compatible)
- [ ] Unit tests for Braid creation

#### Week 6: Attribution Types
- [ ] Implement `Attribution` structure:
  ```rust
  pub struct Attribution {
      pub agent: Did,
      pub role: AgentRole,
      pub compute_units: Option<f64>,
      pub derived_from: Vec<ContentHash>,
      pub custom: HashMap<String, Value>,
  }
  ```
- [ ] Implement `AgentRole` enum:
  - `Creator` — Original author
  - `Contributor` — Added value
  - `Curator` — Organized/validated
  - `Publisher` — Made available
  - `Transformer` — Modified/derived
- [ ] Add role weighting for attribution calculation
- [ ] Unit tests for attribution

---

### Phase 2: Provenance Graph (Weeks 7-8)

**Goal**: Implement DAG traversal and queries.

#### Week 7: Graph Traversal
- [ ] Implement `ProvenanceGraph` structure:
  ```rust
  pub struct ProvenanceGraph {
      pub root: EntityReference,
      pub entities: HashMap<EntityReference, Braid>,
      pub activities: HashMap<ActivityId, Activity>,
      pub depth: u32,
  }
  ```
- [ ] Implement RhizoCrypt DAG walker
- [ ] Implement derivation chain following
- [ ] Add depth limiting for queries
- [ ] Unit tests for traversal

#### Week 8: Attribution Calculation
- [ ] Implement `AttributionChain` structure:
  ```rust
  pub struct AttributionChain {
      pub entity: EntityReference,
      pub contributors: Vec<ContributorShare>,
      pub total_compute: f64,
  }
  
  pub struct ContributorShare {
      pub agent: Did,
      pub share: f64,  // 0.0 to 1.0
      pub role: AgentRole,
      pub direct: bool,
  }
  ```
- [ ] Implement share calculation algorithm
- [ ] Add role-based weighting
- [ ] Add inheritance through derivation
- [ ] Unit tests for attribution math

---

### Phase 3: LoamSpine Integration (Weeks 9-10)

**Goal**: Listen to commits and create Braids.

#### Week 9: Event Listener
- [ ] Implement `LoamSpineListener` trait
- [ ] Subscribe to commit events
- [ ] Create Braids from `SessionCommit` entries
- [ ] Create Braids from `CertificateMint` entries
- [ ] Store Braids (in-memory initially)

#### Week 10: Braid Storage
- [ ] Implement `BraidStore` trait
- [ ] Add content-hash indexing
- [ ] Add agent indexing (for queries)
- [ ] Add timestamp indexing
- [ ] Integration tests with LoamSpine

---

### Phase 4: Query Engine (Weeks 11-12)

**Goal**: Provide rich attribution queries.

#### Week 11: Query Interface
- [ ] Implement `SweetGrassQueries` trait:
  ```rust
  pub trait SweetGrassQueries {
      async fn get_braid(&self, id: BraidId) -> Result<Option<Braid>>;
      async fn provenance_graph(&self, entity: EntityReference, depth: u32) -> Result<ProvenanceGraph>;
      async fn attribution_chain(&self, entity: EntityReference) -> Result<AttributionChain>;
      async fn contributions_by_agent(&self, agent: Did) -> Result<Vec<ContributionSummary>>;
  }
  ```
- [ ] Implement basic queries
- [ ] Add caching for common queries
- [ ] Unit tests for queries

#### Week 12: PROV-O Export
- [ ] Implement JSON-LD export:
  ```rust
  impl ProvenanceGraph {
      pub fn to_prov_o(&self) -> serde_json::Value;
  }
  ```
- [ ] Add W3C PROV-O vocabulary
- [ ] Add RDF/Turtle export (optional)
- [ ] Validation against PROV-O schema
- [ ] Integration tests

---

### Phase 5: sunCloud Integration (Weeks 13-14)

**Goal**: Power economic distribution.

#### Week 13: Attribution API
- [ ] Implement `SunCloudAttributionSource` trait:
  ```rust
  pub trait SunCloudAttributionSource {
      async fn get_attribution_chain(&self, entity: EntityReference) -> Result<AttributionChain>;
      async fn calculate_shares(&self, entity: EntityReference, total_value: Decimal) -> Result<Vec<RewardShare>>;
  }
  ```
- [ ] Add reward calculation
- [ ] Add proportional distribution
- [ ] Unit tests for economics

#### Week 14: Integration & Testing
- [ ] End-to-end tests: RhizoCrypt → LoamSpine → SweetGrass → sunCloud
- [ ] Performance benchmarking
- [ ] Chaos testing
- [ ] Documentation completion
- [ ] Showcase demos

---

## 📊 Success Metrics

| Metric | Target |
|--------|--------|
| Braid creation | < 50ms |
| Provenance query (depth 5) | < 100ms |
| Attribution calculation | < 200ms |
| PROV-O export | < 500ms |
| Test coverage | > 80% |

---

## 🔗 Dependencies

### External
- `serde_json` — JSON-LD serialization
- `tokio` — Async runtime

### Gen 1 Primals
- **BearDog** — Braid signing (Week 5)
- **Songbird** — Service discovery (Week 13)

### Phase 2 Siblings
- **RhizoCrypt** — DAG traversal (Week 7)
- **LoamSpine** — Commit events (Week 9)

### Downstream
- **sunCloud** — Attribution queries (Week 13)

---

## 📚 Reference Documents

- [specs/SWEETGRASS_SPECIFICATION.md](./specs/SWEETGRASS_SPECIFICATION.md) — Full specification
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — Provenance ontology
- [../ARCHITECTURE.md](../ARCHITECTURE.md) — Unified architecture
- [../INTEGRATION_OVERVIEW.md](../INTEGRATION_OVERVIEW.md) — Data flows

---

*SweetGrass: Every piece of data has a story.*

