# SweetGrass — Data Model Specification

**Version**: 0.3.0  
**Status**: Active  
**Last Updated**: March 2026

---

## 1. Overview

SweetGrass uses W3C PROV-O as its foundation, with ecoPrimals-specific extensions. The core data model consists of:

- **Braid** — Provenance record for a data artifact
- **Activity** — Process that creates or transforms data
- **Agent** — Person, software, or organization that acts
- **Entity** — Data artifact with provenance

```
                    ┌──────────────────┐
                    │      Agent       │
                    │  (did:key:...)   │
                    └────────┬─────────┘
                             │
                    wasAssociatedWith
                             │
                             ▼
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│    Entity    │◄─────│   Activity   │◄─────│    Entity    │
│  (derived)   │ gen  │  (process)   │ used │  (source)    │
└──────────────┘      └──────────────┘      └──────────────┘
       │                                            │
       └────────────── wasDerivedFrom ──────────────┘
```

---

## 2. Braid Structure

### 2.1 Core Braid

```rust
/// A SweetGrass Braid (provenance record)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Braid {
    // ==================== JSON-LD Headers ====================
    
    /// JSON-LD context for semantic interpretation
    #[serde(rename = "@context")]
    pub context: BraidContext,
    
    /// Unique identifier (URN)
    #[serde(rename = "@id")]
    pub id: BraidId,
    
    /// Primary type
    #[serde(rename = "@type")]
    pub braid_type: BraidType,
    
    // ==================== Subject Data ====================
    
    /// Hash of the data this Braid describes
    pub data_hash: ContentHash,
    
    /// MIME type of the data
    pub mime_type: String,
    
    /// Size of the data in bytes
    pub size: u64,
    
    // ==================== Provenance ====================
    
    /// How this data was generated
    pub was_generated_by: Option<Activity>,
    
    /// What entities this was derived from
    pub was_derived_from: Vec<EntityReference>,
    
    /// Who created/owns this Braid
    pub was_attributed_to: Did,
    
    /// When this Braid was created
    pub generated_at_time: Timestamp,
    
    // ==================== Metadata ====================
    
    /// Domain-specific metadata
    pub metadata: BraidMetadata,
    
    /// ecoPrimals-specific attributes
    pub ecop: EcoPrimalsAttributes,
    
    // ==================== Witness ====================
    
    /// Primary witness (WireWitnessRef-aligned provenance event)
    pub witness: Witness,
    
    // ==================== Anchoring ====================
    
    /// LoamSpine anchor (if committed)
    pub loam_anchor: Option<LoamAnchor>,
}

/// Braid identifier (URN format) — Arc<str> newtype for O(1) clone
pub struct BraidId(Arc<str>); // "urn:braid:sha256:abc123..."

/// Content-addressed hash — Arc<str> newtype for O(1) clone
pub struct ContentHash(Arc<str>); // "sha256:abc123..."

/// Agent decentralized identifier — Arc<str> newtype
pub struct Did(Arc<str>); // "did:key:z6Mk..."

/// Activity identifier — Arc<str> newtype for O(1) clone
pub struct ActivityId(Arc<str>); // "urn:activity:uuid:..."

/// Timestamp (nanoseconds since epoch)
pub type Timestamp = u64;
```

### 2.2 Braid Types

```rust
/// Types of Braids
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BraidType {
    /// Standard entity Braid
    Entity,
    
    /// Activity Braid
    Activity,
    
    /// Agent Braid
    Agent,
    
    /// Meta-Braid (summary of other Braids)
    Collection {
        member_count: u64,
        summary_type: SummaryType,
    },
    
    /// Delegation Braid (agent acting for another)
    Delegation {
        delegate: Did,
        on_behalf_of: Did,
    },
    
    /// Slice provenance
    Slice {
        slice_mode: SliceMode,
        origin_spine: SpineId,
    },
}

/// Summary types for meta-Braids
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SummaryType {
    /// Session summary
    Session { session_id: SessionId },
    
    /// Time period summary
    Temporal { start: Timestamp, end: Timestamp },
    
    /// Activity type summary
    ActivityGroup { activity_type: String },
    
    /// Agent contribution summary
    AgentContributions { agent: Did },
    
    /// Custom grouping
    Custom { criteria: String },
}
```

### 2.3 JSON-LD Context

```rust
/// JSON-LD context for semantic interpretation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidContext {
    /// Base context URL
    #[serde(rename = "@base")]
    pub base: String,
    
    /// Version
    #[serde(rename = "@version")]
    pub version: f32,
    
    /// Vocabulary imports
    #[serde(rename = "@vocab")]
    pub vocab: Option<String>,
    
    /// Additional context URLs
    #[serde(flatten)]
    pub imports: HashMap<String, String>,
}

impl Default for BraidContext {
    fn default() -> Self {
        let mut imports = HashMap::new();
        imports.insert("prov".to_string(), "http://www.w3.org/ns/prov#".to_string());
        imports.insert("xsd".to_string(), "http://www.w3.org/2001/XMLSchema#".to_string());
        imports.insert("schema".to_string(), "http://schema.org/".to_string());
        imports.insert("ecop".to_string(), "https://ecoprimals.io/vocab#".to_string());
        
        Self {
            base: "https://ecoprimals.io/".to_string(),
            version: 1.1,
            vocab: None,
            imports,
        }
    }
}
```

---

## 3. Activity Structure

### 3.1 Core Activity

```rust
/// A PROV-O Activity (process/action)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    /// Activity identifier
    #[serde(rename = "@id")]
    pub id: ActivityId,
    
    /// Activity type
    #[serde(rename = "@type")]
    pub activity_type: ActivityType,
    
    /// Inputs used by this activity
    pub used: Vec<UsedEntity>,
    
    /// Agent(s) who performed the activity
    pub was_associated_with: Vec<AgentAssociation>,
    
    /// When the activity started
    pub started_at_time: Timestamp,
    
    /// When the activity ended
    pub ended_at_time: Option<Timestamp>,
    
    /// Activity metadata
    pub metadata: ActivityMetadata,
    
    /// ecoPrimals-specific attributes
    pub ecop: ActivityEcoPrimals,
}

// ActivityId defined in Section 2.1 as Arc<str> newtype
```

### 3.2 Activity Types

```rust
/// Standard activity types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivityType {
    // === Creation ===
    Creation,
    Import,
    Extraction,
    Generation,
    
    // === Transformation ===
    Transformation,
    Derivation,
    Aggregation,
    Filtering,
    Merge,
    Split,
    
    // === Analysis ===
    Analysis,
    Computation,
    Simulation,
    MachineLearning,
    Inference,
    
    // === Scientific ===
    Experiment,
    Observation,
    Measurement,
    Validation,
    
    // === Collaboration ===
    Editing,
    Review,
    Approval,
    Publication,
    
    // === RhizoCrypt-specific ===
    SessionStart,
    SessionCommit,
    SessionRollback,
    SliceCheckout,
    SliceReturn,
    
    // === LoamSpine-specific ===
    CertificateMint,
    CertificateTransfer,
    CertificateLoan,
    CertificateReturn,
    
    // === Custom ===
    Custom { type_uri: String },
}
```

### 3.3 Used Entity

```rust
/// Entity used as input to an activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsedEntity {
    /// Reference to the entity
    pub entity: EntityReference,
    
    /// Role this entity played
    pub role: EntityRole,
    
    /// When it was used
    pub time: Option<Timestamp>,
    
    /// How much was used (for partial consumption)
    pub extent: Option<UsageExtent>,
}

/// Role an entity plays in an activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityRole {
    Input,
    Template,
    Configuration,
    Reference,
    Training,
    Validation,
    Custom(String),
}

/// Extent of entity usage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UsageExtent {
    Full,
    Partial { fraction: f64 },
    Bytes { start: u64, end: u64 },
    Subset { description: String },
}
```

### 3.4 ecoPrimals Activity Attributes

```rust
/// ecoPrimals-specific activity attributes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActivityEcoPrimals {
    /// Compute units consumed
    pub compute_units: Option<f64>,
    
    /// Storage used (bytes)
    pub storage_bytes: Option<u64>,
    
    /// Network transfer (bytes)
    pub network_bytes: Option<u64>,
    
    /// Duration (nanoseconds)
    pub duration_ns: Option<u64>,
    
    /// RhizoCrypt session ID
    pub rhizo_session: Option<SessionId>,
    
    /// RhizoCrypt vertex hashes (if tracking individual vertices)
    pub rhizo_vertices: Option<Vec<VertexHash>>,
    
    /// ToadStool task ID
    pub toadstool_task: Option<TaskId>,
    
    /// LoamSpine entry reference
    pub loam_entry: Option<LoamEntryRef>,
    
    /// Niche context
    pub niche: Option<NicheId>,
}
```

---

## 4. Agent Structure

### 4.1 Core Agent

```rust
/// An agent (person, software, organization)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// Agent identifier (DID)
    #[serde(rename = "@id")]
    pub id: Did,
    
    /// Agent type
    #[serde(rename = "@type")]
    pub agent_type: AgentType,
    
    /// Display name
    pub name: Option<String>,
    
    /// Additional attributes
    pub attributes: AgentAttributes,
}

/// Agent types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentType {
    /// Human person
    Person,
    
    /// Software agent (AI, bot, service)
    SoftwareAgent {
        software_name: String,
        version: String,
    },
    
    /// Organization
    Organization {
        org_type: Option<String>,
    },
    
    /// Hardware device
    Device {
        device_type: String,
    },
}
```

### 4.2 Agent Association

```rust
/// Agent's association with an activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentAssociation {
    /// The agent
    pub agent: Did,
    
    /// Role in the activity
    pub role: AgentRole,
    
    /// Acting on behalf of another agent
    pub on_behalf_of: Option<Did>,
    
    /// Plan/protocol followed
    pub had_plan: Option<PlanReference>,
}

/// Roles agents can play
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentRole {
    /// Primary creator/author
    Creator,
    
    /// Contributor (partial contribution)
    Contributor,
    
    /// Publisher/distributor
    Publisher,
    
    /// Validator/reviewer
    Validator,
    
    /// Data source provider
    DataProvider,
    
    /// Compute resource provider
    ComputeProvider,
    
    /// Storage resource provider
    StorageProvider,
    
    /// Orchestrator/coordinator
    Orchestrator,
    
    /// Custom role
    Custom(String),
}
```

---

## 5. Entity Reference

### 5.1 Reference Types

```rust
/// Reference to a PROV entity
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntityReference {
    /// Reference by Braid ID
    ById { braid_id: BraidId },
    
    /// Reference by content hash
    ByHash { 
        data_hash: ContentHash,
        mime_type: Option<String>,
    },
    
    /// Reference by LoamSpine location
    ByLoamEntry {
        spine_id: SpineId,
        entry_hash: EntryHash,
    },
    
    /// External reference (URL)
    External { 
        url: String, 
        hash: Option<ContentHash>,
    },
    
    /// Inline entity (for small data)
    Inline(InlineEntity),
}

/// Inline entity for small data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InlineEntity {
    /// Content type
    pub content_type: String,
    
    /// Encoding
    pub encoding: Encoding,
    
    /// Data (Base64 or raw)
    pub data: String,
    
    /// Hash for verification
    pub hash: ContentHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Encoding {
    Base64,
    Utf8,
    Hex,
}
```

---

## 6. ecoPrimals Attributes

### 6.1 Braid Attributes

```rust
/// ecoPrimals-specific Braid attributes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EcoPrimalsAttributes {
    /// Source primal
    pub source_primal: Option<PrimalId>,
    
    /// Niche context
    pub niche: Option<NicheId>,
    
    /// RhizoCrypt session reference
    pub rhizo_session: Option<SessionId>,
    
    /// LoamSpine commit reference
    pub loam_commit: Option<LoamCommitRef>,
    
    /// Certificate reference (if Braid describes a certificate)
    pub certificate: Option<CertificateId>,
    
    /// Slice reference (if Braid tracks slice provenance)
    pub slice: Option<SliceRef>,
    
    /// Compression metadata
    pub compression: Option<CompressionMeta>,
    
    /// Attribution hints for sunCloud
    pub attribution_hints: Option<AttributionHints>,
}

/// Slice reference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceRef {
    pub slice_id: SliceId,
    pub mode: SliceMode,
    pub origin_spine: SpineId,
    pub origin_entry: EntryHash,
    pub checkout_time: Timestamp,
    pub return_time: Option<Timestamp>,
}

/// Compression metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionMeta {
    /// Original vertex count
    pub vertex_count: u64,
    
    /// Branches explored
    pub branch_count: u64,
    
    /// Compression ratio
    pub ratio: f64,
    
    /// Parent Braids (if this is a summary)
    pub summarizes: Vec<BraidId>,
}
```

### 6.2 Attribution Hints

```rust
/// Hints for sunCloud attribution calculation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AttributionHints {
    /// Explicit contribution weights (if known)
    pub explicit_weights: Option<HashMap<Did, f64>>,
    
    /// Compute contribution (if measurable)
    pub compute_contribution: Option<f64>,
    
    /// Data contribution size
    pub data_contribution_bytes: Option<u64>,
    
    /// Inherit attribution from sources
    pub inherit_from_sources: bool,
    
    /// Decay factor for inheritance
    pub inheritance_decay: Option<f64>,
}
```

---

## 7. Witness (`WireWitnessRef`)

The primary witness on a Braid follows the ecosystem `WireWitnessRef` vocabulary,
a self-describing provenance event that supersedes the former W3C LD-Proof
`BraidSignature` pattern.

```rust
/// Self-describing provenance witness (WireWitnessRef vocabulary).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    pub agent: Did,
    pub kind: String,           // "signature", "hash", "checkpoint", "marker", "timestamp"
    pub evidence: String,       // opaque payload (base64 sig bytes, hash, etc.)
    pub witnessed_at: Timestamp,
    pub encoding: String,       // "base64", "hex", "utf8", "none"
    pub algorithm: Option<String>,  // "ed25519", "ecdsa-p256", etc.
    pub tier: Option<String>,       // "local", "gateway", "anchor", "external", "open"
    pub context: Option<String>,
}

impl Witness {
    pub fn from_ed25519(agent: &Did, signature_bytes: &[u8]) -> Self {
        Self {
            agent: agent.clone(),
            kind: "signature".to_string(),
            evidence: base64::encode(signature_bytes),
            witnessed_at: current_timestamp_nanos(),
            encoding: "base64".to_string(),
            algorithm: Some("ed25519".to_string()),
            tier: Some("local".to_string()),
            context: None,
        }
    }
}
```

---

## 8. LoamSpine Anchor

```rust
/// LoamSpine anchor information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamAnchor {
    /// Spine where anchored
    pub spine_id: SpineId,
    
    /// Entry hash
    pub entry_hash: EntryHash,
    
    /// Entry index
    pub index: u64,
    
    /// When anchored
    pub anchored_at: Timestamp,
    
    /// Verified on chain
    pub verified: bool,
}
```

---

## 9. Provenance Graph

```rust
/// Provenance graph result from queries
#[derive(Clone, Debug)]
pub struct ProvenanceGraph {
    /// Root entity
    pub root: BraidId,
    
    /// All Braids in the graph
    pub braids: HashMap<BraidId, Braid>,
    
    /// All activities in the graph
    pub activities: HashMap<ActivityId, Activity>,
    
    /// Edges (derivation relationships)
    pub edges: Vec<ProvenanceEdge>,
    
    /// Graph depth traversed
    pub depth: u32,
    
    /// Statistics
    pub stats: GraphStats,
}

/// Edge in provenance graph
#[derive(Clone, Debug)]
pub struct ProvenanceEdge {
    pub from: BraidId,
    pub to: BraidId,
    pub edge_type: EdgeType,
    pub activity: Option<ActivityId>,
}

#[derive(Clone, Debug)]
pub enum EdgeType {
    WasDerivedFrom,
    WasGeneratedBy,
    WasAttributedTo,
    Used,
    WasAssociatedWith,
    ActedOnBehalfOf,
}

/// Graph statistics
#[derive(Clone, Debug, Default)]
pub struct GraphStats {
    pub braid_count: usize,
    pub activity_count: usize,
    pub agent_count: usize,
    pub edge_count: usize,
    pub max_depth: u32,
}
```

---

## 10. Serialization

### 10.1 JSON-LD Example

```json
{
  "@context": {
    "@version": 1.1,
    "@base": "https://ecoprimals.io/",
    "prov": "http://www.w3.org/ns/prov#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "schema": "http://schema.org/",
    "ecop": "https://ecoprimals.io/vocab#"
  },
  "@id": "urn:braid:sha256:abc123...",
  "@type": "prov:Entity",
  
  "dataHash": "sha256:abc123...",
  "mimeType": "application/json",
  "size": 1024,
  
  "wasGeneratedBy": {
    "@id": "urn:activity:uuid:def456...",
    "@type": "ecop:Computation",
    "used": [
      {
        "entity": { "dataHash": "sha256:input123..." },
        "role": "ecop:Input"
      }
    ],
    "wasAssociatedWith": [
      {
        "agent": "did:key:z6Mk...",
        "role": "ecop:Creator"
      }
    ],
    "startedAtTime": "2025-12-22T10:00:00Z",
    "endedAtTime": "2025-12-22T10:05:00Z",
    "ecop:computeUnits": 0.5,
    "ecop:rhizoSession": "session-abc123"
  },
  
  "wasDerivedFrom": [
    { "braidId": "urn:braid:sha256:source..." }
  ],
  
  "wasAttributedTo": "did:key:z6Mk...",
  "generatedAtTime": "2025-12-22T10:05:00Z",
  
  "ecop": {
    "sourcePrimal": "rhizocrypt",
    "loamCommit": {
      "spineId": "spine-123",
      "entryHash": "sha256:entry...",
      "index": 42
    }
  },
  
  "witness": {
    "agent": "did:key:z6Mk...",
    "kind": "signature",
    "evidence": "z3FXQjecWufY46...",
    "witnessed_at": 1734861901000000000,
    "encoding": "base64",
    "algorithm": "ed25519",
    "tier": "local"
  }
}
```

---

## 11. References

- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — Provenance Ontology
- [JSON-LD](https://json-ld.org/) — Linked Data in JSON
- [W3C Data Integrity](https://w3c.github.io/vc-data-integrity/) — Signatures
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [BRAID_COMPRESSION.md](./BRAID_COMPRESSION.md) — Compression model
- [CONTENT_CONVERGENCE.md](./CONTENT_CONVERGENCE.md) — Hash convergence and provenance intersection

---

*SweetGrass: Weaving the stories that give data its meaning.*

