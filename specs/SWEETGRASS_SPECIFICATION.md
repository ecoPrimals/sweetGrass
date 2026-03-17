# SweetGrass — Semantic Provenance & Attribution Layer Specification

**Version:** 0.7.14  
**Status:** Architectural Specification  
**Author:** ecoPrimals Project  
**Date:** March 2026  
**License:** AGPL-3.0-only  

---

## Abstract

SweetGrass is the semantic provenance and attribution layer of the ecoPrimals ecosystem. It answers the fundamental question: **"What is the story of this data?"**

While RhizoCrypt captures ephemeral events and LoamSpine provides permanent storage, SweetGrass weaves meaning into data. It creates **Braids**—cryptographically signed, machine-readable provenance documents that track:
- What created this data
- Who contributed to it
- Where it came from
- How it was transformed
- Why it matters

SweetGrass transforms disconnected data into a rich, queryable knowledge graph. It provides the semantic foundation for the gAIa knowledge commons and the economic layers (sunCloud) that reward contributors.

---

## 1. Core Principles

### 1.1 The Storyteller

If RhizoCrypt is the chaotic workshop and LoamSpine is the museum, SweetGrass is the **curator's notebook**—documenting not just what exists, but its context, lineage, and significance.

SweetGrass doesn't store data; it stores **stories about data**:
- The genome sequence exists in NestGate
- The computation that processed it ran on ToadStool  
- The session that captured the process lives in RhizoCrypt
- The final result is anchored in LoamSpine
- **SweetGrass tells you how they're all connected**

### 1.2 Standards-Based Interoperability

SweetGrass is built on established W3C standards:
- **PROV-O** — Provenance Ontology (Entity, Activity, Agent)
- **JSON-LD** — Linked Data in JSON format
- **DIDs** — Decentralized Identifiers (via BearDog)

This ensures that SweetGrass metadata can be understood, verified, and exchanged with any compliant system.

### 1.3 The Attribution Imperative

SweetGrass serves a critical economic function: **tracking contributions**. Every piece of data in the ecosystem has a story of who contributed what. This attribution chain enables:
- Fair compensation via sunCloud's Radiating Attribution
- Reputation building for contributors
- Trust assessment for AI systems (gAIa)
- Legal compliance for chain-of-custody

### 1.4 Content-Addressable Linking

SweetGrass Braids are linked to data by **content hash**, not location. This means:
- Data can move without breaking provenance
- Multiple copies share the same story
- Integrity is verifiable without trust

---

## 2. Data Model

### 2.1 The Braid Structure

The fundamental object in SweetGrass is a **Braid**—a provenance record following W3C PROV-O:

```rust
/// A SweetGrass Braid (provenance record)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Braid {
    /// JSON-LD context for semantic interpretation
    #[serde(rename = "@context")]
    pub context: BraidContext,
    
    /// Unique identifier for this Braid
    #[serde(rename = "@id")]
    pub id: BraidId,
    
    /// Primary type (usually prov:Entity)
    #[serde(rename = "@type")]
    pub braid_type: BraidType,
    
    /// Hash of the data this Braid describes
    pub data_hash: ContentHash,
    
    /// MIME type of the data
    pub mime_type: String,
    
    /// Size of the data in bytes
    pub size: u64,
    
    /// How this data was generated
    pub was_generated_by: Option<Activity>,
    
    /// What entity this was derived from
    pub was_derived_from: Vec<EntityReference>,
    
    /// Who created/owns this Braid
    pub was_attributed_to: Did,
    
    /// When this Braid was created
    pub generated_at_time: Timestamp,
    
    /// Domain-specific metadata
    pub metadata: BraidMetadata,
    
    /// Custom ecoPrimals attributes
    pub ecop: EcoPrimalsAttributes,
    
    /// Cryptographic signature
    pub signature: BraidSignature,
}

/// JSON-LD context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidContext {
    /// Base context URL
    pub base: String, // "https://ecoprimals.io/contexts/sweetgrass-v1.jsonld"
    
    /// Additional context imports
    pub imports: Vec<String>,
    
    /// Inline term definitions
    pub terms: HashMap<String, TermDefinition>,
}

/// Braid identifier (URN)
pub type BraidId = String; // "urn:uuid:e8b3cda0-..."
```

### 2.2 Activity Structure

Activities represent processes that create or transform data:

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

/// Entity used as input to an activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsedEntity {
    /// Reference to the entity (Braid ID or data hash)
    pub entity: EntityReference,
    
    /// Role this entity played
    pub role: EntityRole,
    
    /// When it was used
    pub time: Option<Timestamp>,
}

/// Agent association with an activity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentAssociation {
    /// The agent (DID)
    pub agent: Did,
    
    /// Role in the activity
    pub role: AgentRole,
    
    /// Plan/protocol followed
    pub had_plan: Option<PlanReference>,
}

/// ecoPrimals-specific activity attributes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityEcoPrimals {
    /// Compute units consumed
    pub compute_units: Option<f64>,
    
    /// Storage used (bytes)
    pub storage_bytes: Option<u64>,
    
    /// Network transfer (bytes)
    pub network_bytes: Option<u64>,
    
    /// RhizoCrypt session ID (if applicable)
    pub rhizo_session: Option<SessionId>,
    
    /// ToadStool task ID (if applicable)
    pub toadstool_task: Option<TaskId>,
    
    /// LoamSpine commit reference
    pub loam_commit: Option<LoamCommitRef>,
}
```

### 2.3 Entity Reference Types

```rust
/// Reference to a PROV entity
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntityReference {
    /// Reference by Braid ID
    ById { braid_id: BraidId },
    
    /// Reference by content hash
    ByHash { data_hash: ContentHash },
    
    /// External reference (URL)
    External { url: String, hash: Option<ContentHash> },
    
    /// Inline entity (for small/ephemeral data)
    Inline { data: InlineEntity },
}

/// Inline entity for small data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InlineEntity {
    pub content_type: String,
    pub encoding: Encoding,
    pub data: String, // Base64 or raw
    pub hash: ContentHash,
}
```

### 2.4 Agent Types

```rust
/// Agent types in PROV model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentType {
    /// Human person
    Person { 
        did: Did,
        name: Option<String>,
    },
    
    /// Software agent (AI, bot, service)
    SoftwareAgent {
        did: Did,
        software_name: String,
        version: String,
    },
    
    /// Organization
    Organization {
        did: Did,
        name: String,
        org_type: Option<String>,
    },
    
    /// Delegation (agent acting on behalf of another)
    Delegation {
        delegate: Box<AgentType>,
        on_behalf_of: Box<AgentType>,
    },
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
    
    /// Custom role
    Custom(String),
}
```

### 2.5 Activity Types

```rust
/// Standard activity types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivityType {
    // === Data Creation ===
    Creation,
    Import,
    Extraction,
    
    // === Transformation ===
    Transformation,
    Derivation,
    Aggregation,
    Filtering,
    
    // === Analysis ===
    Analysis,
    Computation,
    Simulation,
    MachineLearning,
    
    // === Scientific ===
    Experiment,
    Observation,
    Measurement,
    
    // === Collaboration ===
    Editing,
    Review,
    Approval,
    Publication,
    
    // === Gaming ===
    GameSession,
    ItemAcquisition,
    ItemTransfer,
    Achievement,
    
    // === Custom ===
    Custom { type_uri: String },
}
```

---

## 3. Architecture

### 3.1 Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     SweetGrass Service                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │    Braid    │  │   Event     │  │        Query            │ │
│  │   Factory   │  │  Listener   │  │        Engine           │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                      │               │
│         ▼                ▼                      ▼               │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                   Braid Store                              │ │
│  │         (Graph Database / Triple Store)                    │ │
│  └───────────────────────────────────────────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                  Index & Search                            │ │
│  │            (Full-text, Faceted, Temporal)                  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                            │                                    │
└────────────────────────────┼────────────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────────────────────┐
        │                    │                    │               │
        ▼                    ▼                    ▼               ▼
   ┌─────────┐         ┌──────────┐        ┌───────────┐   ┌─────────┐
   │ BearDog │         │LoamSpine │        │ ToadStool │   │  gAIa   │
   │   🐻    │         │   🦴     │        │    🍄     │   │   🌍    │
   │ Signing │         │ Commits  │        │  Events   │   │ Queries │
   └─────────┘         └──────────┘        └───────────┘   └─────────┘
```

### 3.2 Braid Factory

The Braid Factory creates and validates Braids:

```rust
/// Braid Factory API
pub trait BraidFactory {
    /// Create a new Braid for data
    async fn create_braid(
        &self,
        data_hash: ContentHash,
        mime_type: String,
        size: u64,
        activity: Option<Activity>,
        derived_from: Vec<EntityReference>,
        attributed_to: Did,
        metadata: BraidMetadata,
    ) -> Result<Braid, BraidError>;
    
    /// Create a Braid from a ToadStool activity event
    async fn create_from_activity(
        &self,
        activity_event: ActivityEvent,
        signer: &impl Signer,
    ) -> Result<Braid, BraidError>;
    
    /// Create a Braid from a RhizoCrypt dehydration
    async fn create_from_dehydration(
        &self,
        summary: DehydrationSummary,
        signer: &impl Signer,
    ) -> Result<Braid, BraidError>;
    
    /// Sign a Braid
    async fn sign_braid(
        &self,
        braid: &mut Braid,
        signer: &impl Signer,
    ) -> Result<(), BraidError>;
    
    /// Validate a Braid
    async fn validate(&self, braid: &Braid) -> Result<ValidationResult, BraidError>;
}
```

### 3.3 Event Listener

The Event Listener subscribes to ecosystem events and generates Braids:

```rust
/// Event Listener API
pub trait EventListener {
    /// Subscribe to ToadStool activity events
    fn subscribe_toadstool(&self, filter: ActivityFilter) -> impl Stream<Item = ActivityEvent>;
    
    /// Subscribe to RhizoCrypt session resolutions
    fn subscribe_rhizocrypt(&self, filter: SessionFilter) -> impl Stream<Item = DehydrationSummary>;
    
    /// Subscribe to LoamSpine commits
    fn subscribe_loamspine(&self, filter: EntryFilter) -> impl Stream<Item = LoamEntry>;
    
    /// Process events and generate Braids
    async fn process_events(&self) -> Result<(), ListenerError>;
}

/// Activity event from ToadStool
#[derive(Clone, Debug)]
pub struct ActivityEvent {
    pub task_id: TaskId,
    pub activity_type: ActivityType,
    pub inputs: Vec<EntityReference>,
    pub outputs: Vec<OutputEntity>,
    pub agent: Did,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub compute_units: f64,
    pub metadata: HashMap<String, Value>,
}

/// Output entity from an activity
#[derive(Clone, Debug)]
pub struct OutputEntity {
    pub data_hash: ContentHash,
    pub mime_type: String,
    pub size: u64,
    pub role: EntityRole,
}
```

### 3.4 Query Engine

The Query Engine provides rich querying over the Braid graph:

```rust
/// Query Engine API
pub trait QueryEngine {
    /// Get a Braid by ID
    async fn get_braid(&self, id: BraidId) -> Result<Option<Braid>, QueryError>;
    
    /// Get Braids for a data hash
    async fn get_braids_for_data(&self, hash: ContentHash) -> Result<Vec<Braid>, QueryError>;
    
    /// Get full provenance graph for an entity
    async fn get_provenance_graph(
        &self,
        entity: EntityReference,
        depth: Option<u32>,
    ) -> Result<ProvenanceGraph, QueryError>;
    
    /// Query Braids with filters
    async fn query(&self, query: BraidQuery) -> Result<QueryResult, QueryError>;
    
    /// GraphQL query endpoint
    async fn graphql(&self, query: String, variables: Variables) -> Result<GraphQLResponse, QueryError>;
    
    /// Get attribution chain for economic purposes
    async fn get_attribution_chain(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, QueryError>;
    
    /// Calculate contribution metrics
    async fn calculate_contributions(
        &self,
        entity: EntityReference,
    ) -> Result<ContributionMetrics, QueryError>;
}

/// Provenance graph result
#[derive(Clone, Debug)]
pub struct ProvenanceGraph {
    /// Root entity
    pub root: EntityReference,
    
    /// All entities in the graph
    pub entities: HashMap<BraidId, Braid>,
    
    /// All activities in the graph
    pub activities: HashMap<ActivityId, Activity>,
    
    /// Edges (derivation relationships)
    pub derivations: Vec<Derivation>,
    
    /// Graph depth
    pub depth: u32,
}

/// Attribution chain for economic purposes
#[derive(Clone, Debug)]
pub struct AttributionChain {
    /// Root entity
    pub entity: EntityReference,
    
    /// Contributors with their share
    pub contributors: Vec<ContributorShare>,
    
    /// Total compute used
    pub total_compute: f64,
    
    /// Total storage used
    pub total_storage: u64,
}

#[derive(Clone, Debug)]
pub struct ContributorShare {
    pub agent: Did,
    pub role: AgentRole,
    pub share: f64, // 0.0 to 1.0
    pub direct: bool, // Direct or inherited contribution
}
```

---

## 4. Storage Model

### 4.1 Braid Store

SweetGrass Braids are stored in a graph database optimized for provenance queries:

```rust
/// Braid Store trait
pub trait BraidStore: Send + Sync {
    /// Store a Braid
    async fn put(&self, braid: Braid) -> Result<(), StoreError>;
    
    /// Get a Braid by ID
    async fn get(&self, id: &BraidId) -> Result<Option<Braid>, StoreError>;
    
    /// Get Braids by data hash
    async fn get_by_hash(&self, hash: &ContentHash) -> Result<Vec<Braid>, StoreError>;
    
    /// Get Braids by agent
    async fn get_by_agent(&self, agent: &Did) -> Result<Vec<Braid>, StoreError>;
    
    /// Get derived Braids (children)
    async fn get_derived(&self, id: &BraidId) -> Result<Vec<Braid>, StoreError>;
    
    /// Get source Braids (parents)
    async fn get_sources(&self, id: &BraidId) -> Result<Vec<Braid>, StoreError>;
    
    /// Execute SPARQL query
    async fn sparql(&self, query: &str) -> Result<SparqlResult, StoreError>;
}
```

**Recommended backends:**
- **Apache Jena** — Full SPARQL support, RDF native
- **Neo4j** — Graph traversal performance
- **PostgreSQL + Apache AGE** — Relational with graph extension
- **Oxigraph** — Rust-native RDF store

### 4.2 Index Store

Secondary indexes for efficient querying:

```rust
/// Index Store trait
pub trait IndexStore: Send + Sync {
    /// Index by data hash
    async fn index_by_hash(&self, hash: ContentHash, braid_id: BraidId) -> Result<(), StoreError>;
    
    /// Index by agent
    async fn index_by_agent(&self, agent: Did, braid_id: BraidId) -> Result<(), StoreError>;
    
    /// Index by activity type
    async fn index_by_activity(&self, activity_type: ActivityType, braid_id: BraidId) -> Result<(), StoreError>;
    
    /// Index by time range
    async fn index_by_time(&self, timestamp: Timestamp, braid_id: BraidId) -> Result<(), StoreError>;
    
    /// Full-text index for metadata
    async fn index_text(&self, text: &str, braid_id: BraidId) -> Result<(), StoreError>;
    
    /// Search full-text index
    async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<BraidId>, StoreError>;
}
```

---

## 5. JSON-LD Context & Vocabulary

### 5.1 SweetGrass Context

```json
{
  "@context": {
    "@version": 1.1,
    "@base": "https://ecoprimals.io/",
    
    "prov": "http://www.w3.org/ns/prov#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "schema": "http://schema.org/",
    "ecop": "https://ecoprimals.io/vocab#",
    
    "id": "@id",
    "type": "@type",
    
    "dataHash": {
      "@id": "ecop:dataHash",
      "@type": "xsd:hexBinary"
    },
    "mimeType": {
      "@id": "schema:encodingFormat"
    },
    "size": {
      "@id": "schema:contentSize",
      "@type": "xsd:integer"
    },
    
    "wasGeneratedBy": {
      "@id": "prov:wasGeneratedBy",
      "@type": "@id"
    },
    "wasDerivedFrom": {
      "@id": "prov:wasDerivedFrom",
      "@type": "@id"
    },
    "wasAttributedTo": {
      "@id": "prov:wasAttributedTo",
      "@type": "@id"
    },
    "generatedAtTime": {
      "@id": "prov:generatedAtTime",
      "@type": "xsd:dateTime"
    },
    
    "used": {
      "@id": "prov:used",
      "@type": "@id"
    },
    "wasAssociatedWith": {
      "@id": "prov:wasAssociatedWith",
      "@type": "@id"
    },
    "startedAtTime": {
      "@id": "prov:startedAtTime",
      "@type": "xsd:dateTime"
    },
    "endedAtTime": {
      "@id": "prov:endedAtTime",
      "@type": "xsd:dateTime"
    },
    
    "computeUnits": {
      "@id": "ecop:computeUnits",
      "@type": "xsd:decimal"
    },
    "storageBytes": {
      "@id": "ecop:storageBytes",
      "@type": "xsd:integer"
    },
    "rhizoSession": {
      "@id": "ecop:rhizoSession",
      "@type": "@id"
    },
    "loamCommit": {
      "@id": "ecop:loamCommit",
      "@type": "@id"
    }
  }
}
```

### 5.2 Example Braid (JSON-LD)

```json
{
  "@context": "https://ecoprimals.io/contexts/sweetgrass-v1.jsonld",
  "@id": "urn:uuid:e8b3cda0-1234-5678-abcd-ef0123456789",
  "@type": "prov:Entity",
  
  "dataHash": "sha256:f2ca1bb6c7e907d06dafe4687e579fce76b37e4e93b7605022da52e6ccc26fd2",
  "mimeType": "application/vnd.alphafold-result.pdb",
  "size": 1048576,
  
  "wasGeneratedBy": {
    "@id": "urn:uuid:a1b2c3d4-5678-90ab-cdef-1234567890ab",
    "@type": "ecop:Computation",
    
    "used": [
      {
        "entity": "urn:uuid:b2c3d4e5-6789-0abc-def1-234567890abc",
        "role": "ecop:inputSequence"
      }
    ],
    
    "wasAssociatedWith": {
      "@id": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "@type": "prov:SoftwareAgent",
      "role": "ecop:computeProvider"
    },
    
    "startedAtTime": "2025-12-22T10:00:00Z",
    "endedAtTime": "2025-12-22T11:30:00Z",
    
    "ecop:computeUnits": 1.5,
    "ecop:toadstoolTask": "task-alphafold-12345",
    "ecop:rhizoSession": "session-abc123"
  },
  
  "wasAttributedTo": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "generatedAtTime": "2025-12-22T11:30:00Z",
  
  "signature": {
    "type": "Ed25519Signature2020",
    "created": "2025-12-22T11:30:01Z",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#keys-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQjecWufY46..."
  }
}
```

---

## 6. Integration Points

### 6.1 BearDog Integration

```rust
/// BearDog client for SweetGrass
pub trait BearDogClient {
    /// Resolve DID to public key
    async fn resolve_did(&self, did: &Did) -> Result<DidDocument, BearDogError>;
    
    /// Sign a Braid
    async fn sign_braid(&self, braid: &Braid, key_id: KeyId) -> Result<BraidSignature, BearDogError>;
    
    /// Verify Braid signature
    async fn verify_braid(&self, braid: &Braid) -> Result<bool, BearDogError>;
    
    /// Get lineage proof for agent
    async fn get_lineage_proof(&self, agent: &Did) -> Result<LineageProof, BearDogError>;
}
```

### 6.2 LoamSpine Integration

```rust
/// LoamSpine client for SweetGrass
pub trait LoamSpineClient {
    /// Commit a Braid to LoamSpine
    async fn commit_braid(
        &self,
        spine: SpineId,
        braid: &Braid,
    ) -> Result<LoamCommitRef, LoamError>;
    
    /// Get Braid from LoamSpine commit
    async fn get_braid_from_commit(&self, commit: &LoamCommitRef) -> Result<Option<Braid>, LoamError>;
    
    /// Verify Braid is committed
    async fn verify_commitment(&self, braid_id: &BraidId) -> Result<Option<LoamCommitRef>, LoamError>;
}
```

### 6.3 ToadStool Integration

```rust
/// ToadStool event source for SweetGrass
pub trait ToadStoolEventSource {
    /// Subscribe to activity completion events
    fn subscribe_completions(&self) -> impl Stream<Item = ActivityEvent>;
    
    /// Get activity details
    async fn get_activity(&self, task_id: TaskId) -> Result<ActivityDetails, ToadStoolError>;
}
```

### 6.4 gAIa Integration

```rust
/// gAIa query interface for SweetGrass
pub trait GaiaQueryInterface {
    /// Query provenance for trust assessment
    async fn assess_trust(&self, entity: EntityReference) -> Result<TrustAssessment, GaiaError>;
    
    /// Get attribution for reward calculation
    async fn get_attribution(&self, entity: EntityReference) -> Result<AttributionChain, GaiaError>;
    
    /// Search knowledge graph
    async fn search(&self, query: SemanticQuery) -> Result<SearchResults, GaiaError>;
}

/// Trust assessment result
#[derive(Clone, Debug)]
pub struct TrustAssessment {
    pub entity: EntityReference,
    pub trust_score: f64, // 0.0 to 1.0
    pub factors: Vec<TrustFactor>,
    pub provenance_depth: u32,
    pub agent_reputation: HashMap<Did, f64>,
}

#[derive(Clone, Debug)]
pub struct TrustFactor {
    pub name: String,
    pub weight: f64,
    pub score: f64,
    pub evidence: Vec<BraidId>,
}
```

---

## 7. GraphQL Schema

### 7.1 Type Definitions

```graphql
type Query {
  # Get a Braid by ID
  braid(id: ID!): Braid
  
  # Get Braids for a data hash
  braidsForData(hash: String!): [Braid!]!
  
  # Get provenance graph
  provenanceGraph(entity: ID!, depth: Int): ProvenanceGraph!
  
  # Query Braids
  braids(filter: BraidFilter, pagination: Pagination): BraidConnection!
  
  # Get attribution chain
  attributionChain(entity: ID!): AttributionChain!
  
  # Search Braids
  search(query: String!, limit: Int): [Braid!]!
}

type Braid {
  id: ID!
  type: String!
  dataHash: String!
  mimeType: String!
  size: Int!
  wasGeneratedBy: Activity
  wasDerivedFrom: [Braid!]!
  derivedInto: [Braid!]!
  wasAttributedTo: Agent!
  generatedAtTime: DateTime!
  metadata: JSON
  signature: Signature!
  loamCommit: LoamCommitRef
}

type Activity {
  id: ID!
  type: ActivityType!
  used: [UsedEntity!]!
  wasAssociatedWith: [AgentAssociation!]!
  startedAtTime: DateTime!
  endedAtTime: DateTime
  computeUnits: Float
  storageBytes: Int
  rhizoSession: ID
  toadstoolTask: ID
}

type Agent {
  did: ID!
  type: AgentType!
  name: String
  braidsCreated: [Braid!]!
  activitiesPerformed: [Activity!]!
  reputation: Float
}

type ProvenanceGraph {
  root: Braid!
  entities: [Braid!]!
  activities: [Activity!]!
  edges: [ProvenanceEdge!]!
  depth: Int!
}

type AttributionChain {
  entity: Braid!
  contributors: [ContributorShare!]!
  totalCompute: Float!
  totalStorage: Int!
}

type ContributorShare {
  agent: Agent!
  role: AgentRole!
  share: Float!
  direct: Boolean!
}
```

### 7.2 Example Queries

```graphql
# Get full provenance for a data hash
query GetProvenance($hash: String!) {
  braidsForData(hash: $hash) {
    id
    generatedAtTime
    wasGeneratedBy {
      type
      startedAtTime
      endedAtTime
      computeUnits
      wasAssociatedWith {
        agent { did, name }
        role
      }
    }
    wasDerivedFrom {
      id
      dataHash
      wasAttributedTo { did }
    }
  }
}

# Get attribution chain for economic reward
query GetAttribution($entity: ID!) {
  attributionChain(entity: $entity) {
    contributors {
      agent { did, name }
      role
      share
      direct
    }
    totalCompute
  }
}

# Search for Braids by keyword
query SearchBraids($query: String!) {
  search(query: $query, limit: 10) {
    id
    dataHash
    mimeType
    wasAttributedTo { did, name }
    generatedAtTime
  }
}
```

---

## 8. API Specification

### 8.1 tarpc Service Definition (Primary)

Per `PRIMAL_SOVEREIGNTY.md`: **pure Rust, no gRPC, no protobuf, no vendor lock-in.**

```rust
#[tarpc::service]
pub trait SweetGrassRpc {
    // Braid operations
    async fn create_braid(data: Vec<u8>, mime_type: String, metadata: Option<String>) -> Result<Braid, RpcError>;
    async fn get_braid(id: BraidId) -> Result<Option<Braid>, RpcError>;
    async fn get_braid_by_hash(hash: ContentHash) -> Result<Option<Braid>, RpcError>;
    async fn query_braids(filter: QueryFilter) -> Result<Vec<Braid>, RpcError>;
    async fn delete_braid(id: BraidId) -> Result<bool, RpcError>;

    // Provenance
    async fn provenance_graph(hash: ContentHash, max_depth: Option<usize>) -> Result<ProvenanceGraph, RpcError>;
    async fn export_provo(hash: ContentHash) -> Result<JsonLdDocument, RpcError>;

    // Attribution
    async fn attribution_chain(hash: ContentHash) -> Result<AttributionChain, RpcError>;

    // Anchoring (delegates to persistence capability)
    async fn anchor_braid(braid_id: BraidId) -> Result<AnchorResult, RpcError>;
    async fn verify_anchor(braid_id: BraidId) -> Result<VerifyResult, RpcError>;

    // Health & capability
    async fn health() -> Result<HealthStatus, RpcError>;
}
```

### 8.1.1 JSON-RPC 2.0 (Required IPC)

Per wateringHole `UNIVERSAL_IPC_STANDARD_V3`, JSON-RPC 2.0 over Unix domain sockets
is the required baseline protocol. Method names follow `{domain}.{operation}` per
`SEMANTIC_METHOD_NAMING_STANDARD.md`.

| Domain         | Operations                                                        |
|----------------|-------------------------------------------------------------------|
| `braid`        | create, get, get\_by\_hash, query, delete, commit                 |
| `anchoring`    | anchor, verify                                                    |
| `provenance`   | graph, export\_provo, export\_graph\_provo                        |
| `attribution`  | chain, calculate\_rewards, top\_contributors                      |
| `compression`  | compress\_session, create\_meta\_braid                            |
| `contribution` | record, record\_session, record\_dehydration                      |
| `health`       | check                                                             |
| `capability`   | list                                                              |

### 8.2 REST API

```yaml
openapi: 3.0.0
info:
  title: SweetGrass API
  version: 1.0.0

paths:
  /braids:
    post:
      summary: Create a new Braid
    get:
      summary: Query Braids

  /braids/{braid_id}:
    get:
      summary: Get Braid by ID

  /braids/by-hash/{data_hash}:
    get:
      summary: Get Braids for a data hash

  /braids/{braid_id}/provenance:
    get:
      summary: Get provenance graph
      parameters:
        - name: depth
          in: query
          schema:
            type: integer

  /braids/{braid_id}/attribution:
    get:
      summary: Get attribution chain

  /search:
    get:
      summary: Search Braids
      parameters:
        - name: q
          in: query
          required: true
          schema:
            type: string

  /graphql:
    post:
      summary: GraphQL endpoint
```

---

## 9. Economic Integration (sunCloud)

### 9.1 Contribution Calculation

SweetGrass provides the data for sunCloud's Radiating Attribution:

```rust
/// Calculate contributions for reward distribution
pub async fn calculate_contributions(
    &self,
    entity: EntityReference,
) -> Result<ContributionMetrics, SweetGrassError> {
    // Get full provenance graph
    let graph = self.get_provenance_graph(&entity, None).await?;
    
    // Calculate contribution shares
    let mut contributions = HashMap::new();
    
    for braid in graph.entities.values() {
        // Direct creator
        let creator = &braid.was_attributed_to;
        contributions.entry(creator.clone())
            .or_insert(ContributionAccumulator::default())
            .add_direct_contribution(1.0);
        
        // Activity participants
        if let Some(activity) = &braid.was_generated_by {
            for assoc in &activity.was_associated_with {
                let share = role_weight(&assoc.role);
                contributions.entry(assoc.agent.clone())
                    .or_insert(ContributionAccumulator::default())
                    .add_activity_contribution(share, activity.ecop.compute_units);
            }
        }
        
        // Inherited contributions from sources
        for source in &braid.was_derived_from {
            let source_contributions = self.get_contributions(&source).await?;
            for (agent, share) in source_contributions {
                contributions.entry(agent)
                    .or_insert(ContributionAccumulator::default())
                    .add_inherited_contribution(share * INHERITANCE_DECAY);
            }
        }
    }
    
    // Normalize to sum to 1.0
    let total: f64 = contributions.values().map(|c| c.total()).sum();
    let normalized: Vec<ContributorShare> = contributions
        .into_iter()
        .map(|(agent, acc)| ContributorShare {
            agent,
            share: acc.total() / total,
            direct: acc.direct > 0.0,
            role: acc.primary_role(),
        })
        .collect();
    
    Ok(ContributionMetrics {
        entity,
        contributors: normalized,
        total_compute: graph.total_compute(),
        total_storage: graph.total_storage(),
    })
}

/// Weight multipliers for different roles
fn role_weight(role: &AgentRole) -> f64 {
    match role {
        AgentRole::Creator => 1.0,
        AgentRole::Contributor => 0.5,
        AgentRole::ComputeProvider => 0.3,
        AgentRole::StorageProvider => 0.2,
        AgentRole::DataProvider => 0.4,
        AgentRole::Validator => 0.1,
        AgentRole::Publisher => 0.1,
        AgentRole::Custom(_) => 0.2,
    }
}

const INHERITANCE_DECAY: f64 = 0.7; // Each generation inherits 70%
```

### 9.2 sunCloud Interface

```rust
/// sunCloud economic interface
pub trait SunCloudInterface {
    /// Get contribution metrics for reward distribution
    async fn get_contributions(&self, entity: EntityReference) -> Result<ContributionMetrics, Error>;
    
    /// Record reward distribution
    async fn record_distribution(
        &self,
        entity: EntityReference,
        distributions: Vec<RewardDistribution>,
    ) -> Result<DistributionReceipt, Error>;
    
    /// Get total contributions by agent
    async fn get_agent_contributions(&self, agent: Did) -> Result<AgentContributions, Error>;
}

#[derive(Clone, Debug)]
pub struct RewardDistribution {
    pub agent: Did,
    pub amount: Decimal,
    pub currency: Currency,
    pub share: f64,
    pub reason: DistributionReason,
}
```

---

## 10. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Braid creation latency | < 50ms | Including signing |
| Braid lookup by ID | < 5ms | Indexed |
| Braids by hash | < 10ms | Indexed |
| Provenance graph (depth 5) | < 100ms | Graph traversal |
| Attribution calculation | < 200ms | Full chain |
| GraphQL query | < 50ms | Simple queries |
| Full-text search | < 100ms | 10 results |
| Event processing | > 1000/sec | From ToadStool |

---

## 11. Security Considerations

### 11.1 Signature Verification

- All Braids must be signed
- Signatures verified on retrieval
- Invalid signatures rejected

### 11.2 Access Control

- Braids can be public or private
- Private Braids require BearDog authorization
- Query results filtered by access

### 11.3 Data Integrity

- Content hashes verified on linking
- Circular dependencies detected and rejected
- Temporal consistency enforced

### 11.4 Privacy

- Metadata can be encrypted
- Selective disclosure supported
- Zero-knowledge proofs (future)

---

## 12. Implementation Roadmap

### Phase 1: Core Engine — COMPLETE (v0.1.0–v0.5.0)
- [x] Braid data structures (W3C PROV-O aligned)
- [x] JSON-LD context and serialization
- [x] In-memory Braid store
- [x] PostgreSQL, Sled, and Redb storage backends
- [x] Content hashing (SHA-256)

### Phase 2: Event Processing — COMPLETE (v0.5.0–v0.6.0)
- [x] Session event listener (capability-based, mock isolation)
- [x] Dehydration handler (session compression)
- [x] Automatic Braid generation from sessions
- [x] Anchoring client (capability-based, mock isolation)

### Phase 3: Query Engine — COMPLETE (v0.6.0–v0.7.0)
- [x] Provenance graph traversal
- [x] Attribution chain calculation
- [x] PROV-O JSON-LD export
- [ ] Full-text search (planned v0.9.0+)

### Phase 4: Economic Integration — PARTIAL (v0.7.0–v0.7.20)
- [x] Contribution calculation
- [x] Attribution normalization and radiating attribution
- [ ] sunCloud interface (planned v0.9.0)
- [ ] Reward tracking (planned v0.9.0)

### Phase 5: Protocol & Deployment — COMPLETE (v0.7.0–v0.7.20)
- [x] tarpc primary RPC (pure Rust, no gRPC)
- [x] JSON-RPC 2.0 with `{domain}.{operation}` naming
- [x] REST fallback API
- [x] UniBin binary (`sweetgrass server|status|capabilities|socket|--version`)
- [x] `capability.list` for runtime discovery
- [x] ecoBin compliance (pure Rust, no C deps)
- [x] TOML config with XDG hierarchy

### Phase 6: Hardening — IN PROGRESS (v0.7.9–v0.7.20)
- [x] `#![forbid(unsafe_code)]` on all crates
- [x] Zero `unwrap`/`expect`/`panic!` in production
- [x] SPDX headers and AGPL-3.0-only licensing
- [x] Criterion benchmarks (7 groups)
- [x] Chaos and fault-injection tests
- [x] Fuzz targets (3) and property tests (proptest)
- [ ] Privacy features (module exists, integration pending)
- [ ] Security audit

### Planned: v0.8.0+ — Real Deployment
- [ ] Connect to deployed primals (BearDog signing, LoamSpine anchoring)
- [ ] Multi-primal integration tests over real sockets
- [ ] Protocol negotiation (tarpc-preferred, JSON-RPC fallback)
- [ ] Zero-copy evolution (hot-path clone reduction)

### Planned: v0.9.0 — sunCloud
- [ ] sunCloud attribution API
- [ ] Real-time attribution streaming
- [ ] Payment flow integration

### Planned: v1.0.0 — GA
- [ ] API versioning
- [ ] Full PROV-O compliance
- [ ] Distributed provenance across primals
- [ ] Kubernetes manifests and deploy graphs

---

## 13. References

- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — Provenance Ontology
- [JSON-LD](https://json-ld.org/) — Linked Data in JSON
- [W3C DIDs](https://www.w3.org/TR/did-core/) — Decentralized Identifiers
- [Schema.org](https://schema.org/) — Shared vocabulary
- rhizoCrypt Specification — Ephemeral DAG (see `ecoPrimals/phase2/rhizoCrypt/`)
- LoamSpine Specification — Permanent storage (see `ecoPrimals/phase2/loamSpine/`)
- BearDog Specification — Identity and signing (see `ecoPrimals/phase1/beardog/`)

---

## Appendix A: Example Workflow (Scientific Discovery)

```
1. Researcher imports genomic dataset
   → SweetGrass creates Braid:
     - dataHash: sha256:abc123...
     - wasAttributedTo: did:key:researcher
     - metadata: { source: "NCBI", accession: "NC_000001" }

2. ToadStool runs AlphaFold prediction
   → SweetGrass creates Braid:
     - dataHash: sha256:def456... (predicted structure)
     - wasGeneratedBy: {
         type: "Computation",
         used: [{ entity: "sha256:abc123...", role: "input" }],
         wasAssociatedWith: [
           { agent: did:key:researcher, role: "Creator" },
           { agent: did:key:compute-node, role: "ComputeProvider" }
         ],
         computeUnits: 1.5
       }
     - wasDerivedFrom: [Braid for input sequence]

3. Structure committed to LoamSpine
   → Braid updated with loamCommit reference
   → Permanent, verifiable provenance

4. Years later: Discovery leads to drug target
   → sunCloud calculates attribution:
     - Researcher: 40% (Creator)
     - Compute provider: 20% (ComputeProvider)
     - NCBI: 15% (DataProvider, inherited)
     - AlphaFold team: 10% (SoftwareAgent citation)
     - gAIa commons: 15% (reversion of unreachable)

5. Revenue distributed automatically
   → All contributors rewarded proportionally
   → Permanent record in LoamSpine
```

---

*SweetGrass: Weaving the stories that give data its meaning.*

