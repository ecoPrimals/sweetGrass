# SweetGrass — Architecture Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass is the semantic layer that grows from RhizoCrypt and LoamSpine, making their activity visible and queryable. It creates **Braids**—provenance records that track what created data, who contributed, and how it was transformed.

```
┌─────────────────────────────────────────────────────────────────┐
│                     SweetGrass Service                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │    Braid    │  │  Compression│  │        Query            │ │
│  │   Factory   │  │    Engine   │  │        Engine           │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                      │               │
│         ▼                ▼                      ▼               │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Braid Store                             │ │
│  │              (Graph Database / RDF)                        │ │
│  └───────────────────────────────────────────────────────────┘ │
│                            │                                    │
│         ┌──────────────────┼──────────────────┐                │
│         ▼                  ▼                  ▼                │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   Event     │    │  Anchor     │    │  Index &    │        │
│  │  Listener   │    │  Manager    │    │  Search     │        │
│  └──────┬──────┘    └──────┬──────┘    └─────────────┘        │
│         │                  │                                    │
└─────────┼──────────────────┼────────────────────────────────────┘
          │                  │
          ▼                  ▼
    ┌───────────┐      ┌───────────┐
    │RhizoCrypt │      │LoamSpine  │
    │   🍄      │      │   🦴      │
    │ Sessions  │      │  Commits  │
    └───────────┘      └───────────┘
```

---

## 2. Biological Model: The Grass

SweetGrass draws from both underground layers:

```
                    VISIBLE WORLD
                         │
    ┌────────────────────┼────────────────────┐
    │                    │                    │
    │   🌾 🌾 🌾 🌾 🌾 │ 🌾 🌾 🌾 🌾 🌾   │  Braids (visible)
    │   │  │  │  │  │  │  │  │  │  │  │    │  Queryable provenance
    │   │  │  │  │  │  │  │  │  │  │  │    │
════╪═══╪══╪══╪══╪══╪══╪══╪══╪══╪══╪══╪════╪══ SOIL LINE
    │   │  │  │  │  │  │  │  │  │  │  │    │
    │   │  └──┼──┴──┼──┘  │  └──┼──┴──┘    │
    │   │     │     │     │     │          │
    │   └─────┴──┬──┴─────┴─────┘          │  RhizoCrypt roots
    │            │                          │  (activity sources)
    │    ════════╪════════════════          │
    │            │                          │  LoamSpine bedrock
    │    ════════╪════════════════          │  (permanent anchors)
    │            │                          │
    └────────────┴──────────────────────────┘
```

**The grass metaphor:**
- **Roots in RhizoCrypt**: Activity events, session resolutions, vertex creation
- **Anchored in LoamSpine**: Permanent Braid commits, certificate provenance
- **Visible stalks**: Queryable Braids that applications consume

---

## 3. Core Components

### 3.1 Braid Factory

Creates and validates Braids from various sources:

```rust
/// Braid Factory component
pub struct BraidFactory {
    signer: Arc<dyn Signer>,
    beardog: Arc<dyn BearDogClient>,
    context: BraidContext,
}

impl BraidFactory {
    /// Create Braid from RhizoCrypt session
    pub async fn from_session(
        &self,
        summary: DehydrationSummary,
    ) -> Result<Vec<Braid>, BraidError>;
    
    /// Create Braid from LoamSpine entry
    pub async fn from_entry(
        &self,
        entry: &LoamEntry,
    ) -> Result<Braid, BraidError>;
    
    /// Create Braid from ToadStool activity
    pub async fn from_activity(
        &self,
        event: ActivityEvent,
    ) -> Result<Braid, BraidError>;
    
    /// Create meta-Braid (summary of Braids)
    pub async fn create_meta_braid(
        &self,
        braids: Vec<BraidId>,
        summary_type: SummaryType,
    ) -> Result<Braid, BraidError>;
}
```

### 3.2 Compression Engine

Handles the DAG → Linear compression (fungal leather model):

```rust
/// Compression Engine component
pub struct CompressionEngine {
    store: Arc<BraidBackend>,
    config: CompressionConfig,
}

impl CompressionEngine {
    /// Compress a DAG session to Braids
    /// Returns 0, 1, or many Braids based on content
    pub async fn compress_session(
        &self,
        session: &Session,
        mode: CompressionMode,
    ) -> Result<CompressionResult, CompressionError>;
    
    /// Create summary Braid from multiple Braids
    pub async fn summarize(
        &self,
        braids: Vec<&Braid>,
        depth: u32,
    ) -> Result<Braid, CompressionError>;
    
    /// Determine optimal compression strategy
    pub fn analyze_session(
        &self,
        session: &Session,
    ) -> CompressionStrategy;
}

/// Compression result (0, 1, or many)
pub enum CompressionResult {
    /// No Braids produced (session discarded)
    None { reason: DiscardReason },
    
    /// Single coherent Braid
    Single(Braid),
    
    /// Multiple Braids with hierarchy
    Multiple {
        braids: Vec<Braid>,
        summary: Option<Braid>,
    },
}

/// Why a session produced no Braids
pub enum DiscardReason {
    Rollback,
    EmptySession,
    ExploratoryOnly,
    BelowThreshold,
}
```

### 3.3 Query Engine

Provides rich querying over the Braid graph:

```rust
/// Query Engine component
pub struct QueryEngine {
    store: Arc<BraidBackend>,
    index: Arc<dyn IndexStore>,
}

impl QueryEngine {
    /// Get Braid by ID
    pub async fn get(&self, id: &BraidId) -> Result<Option<Braid>, QueryError>;
    
    /// Get Braids for a data hash
    pub async fn by_hash(&self, hash: ContentHash) -> Result<Vec<Braid>, QueryError>;
    
    /// Get full provenance graph
    pub async fn provenance_graph(
        &self,
        entity: EntityReference,
        depth: Option<u32>,
    ) -> Result<ProvenanceGraph, QueryError>;
    
    /// Calculate attribution chain
    pub async fn attribution_chain(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, QueryError>;
    
    /// Execute GraphQL query
    pub async fn graphql(
        &self,
        query: String,
        variables: Variables,
    ) -> Result<GraphQLResponse, QueryError>;
    
    /// Execute SPARQL query
    pub async fn sparql(&self, query: &str) -> Result<SparqlResult, QueryError>;
}
```

### 3.4 Event Listener

Subscribes to ecosystem events:

```rust
/// Event Listener component
pub struct EventListener {
    rhizo_client: Arc<dyn RhizoCryptClient>,
    loam_client: Arc<dyn LoamSpineClient>,
    toadstool_client: Option<Arc<dyn ToadStoolClient>>,
    factory: Arc<BraidFactory>,
    store: Arc<BraidBackend>,
}

impl EventListener {
    /// Start listening for events
    pub async fn start(&self) -> Result<(), ListenerError>;
    
    /// Subscribe to RhizoCrypt session resolutions
    fn subscribe_sessions(&self) -> impl Stream<Item = DehydrationSummary>;
    
    /// Subscribe to LoamSpine commits
    fn subscribe_commits(&self) -> impl Stream<Item = LoamEntry>;
    
    /// Subscribe to ToadStool activities (if available)
    fn subscribe_activities(&self) -> impl Stream<Item = ActivityEvent>;
    
    /// Process an event and create Braids
    async fn process_event(&self, event: EcosystemEvent) -> Result<(), ListenerError>;
}
```

### 3.5 Anchor Manager

Manages Braid anchoring to LoamSpine:

```rust
/// Anchor Manager component
pub struct AnchorManager {
    loam_client: Arc<dyn LoamSpineClient>,
    store: Arc<BraidBackend>,
}

impl AnchorManager {
    /// Anchor a Braid to LoamSpine
    pub async fn anchor(
        &self,
        braid: &Braid,
        spine_id: SpineId,
    ) -> Result<AnchorReceipt, AnchorError>;
    
    /// Verify Braid is anchored
    pub async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>, AnchorError>;
    
    /// Get all anchors for a Braid
    pub async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>, AnchorError>;
}

/// Anchor information
#[derive(Clone, Debug)]
pub struct AnchorInfo {
    pub braid_id: BraidId,
    pub spine_id: SpineId,
    pub entry_hash: EntryHash,
    pub index: u64,
    pub timestamp: u64,
    pub verified: bool,
}
```

---

## 4. Data Flow

### 4.1 Session Resolution Flow

```
RhizoCrypt                    SweetGrass                    LoamSpine
    │                              │                              │
    │  Session resolves            │                              │
    ├─────────────────────────────►│                              │
    │  DehydrationSummary          │                              │
    │                              │                              │
    │                         Compression Engine                  │
    │                         analyzes session                    │
    │                              │                              │
    │                         ┌────┴────┐                         │
    │                         │ 0, 1, N │                         │
    │                         └────┬────┘                         │
    │                              │                              │
    │                         Braid Factory                       │
    │                         creates Braids                      │
    │                              │                              │
    │                         Anchor Manager                      │
    │                         ─────────────────────────────────►  │
    │                         commits to spine                    │
    │                              │                              │
    │                         Store Braids                        │
    │                              │                              │
```

### 4.2 Query Flow

```
Application                   SweetGrass                    BraidStore
    │                              │                              │
    │  GraphQL Query               │                              │
    ├─────────────────────────────►│                              │
    │                              │                              │
    │                         Query Engine                        │
    │                         parses query                        │
    │                              │                              │
    │                              ├─────────────────────────────►│
    │                              │  Graph traversal             │
    │                              │◄─────────────────────────────┤
    │                              │  Braids                      │
    │                              │                              │
    │                         Build response                      │
    │                              │                              │
    │◄─────────────────────────────┤                              │
    │  GraphQL Response            │                              │
    │                              │                              │
```

---

## 5. Crate Organization

```
sweetGrass/
├── Cargo.toml                    # Workspace manifest
├── deny.toml                     # cargo-deny (AGPL, no gRPC/protobuf)
├── crates/
│   ├── sweet-grass-core/         # Core types and traits
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── braid/            # Braid structure (types, builder, tests)
│   │       ├── activity.rs       # Activity structure
│   │       ├── agent.rs          # Agent types (Did, roles)
│   │       ├── entity.rs         # Entity references
│   │       ├── config.rs         # SweetGrassConfig
│   │       ├── contribution.rs   # Contribution tracking
│   │       ├── dehydration.rs    # Braid dehydration
│   │       ├── hash.rs           # Content hashing (sha256)
│   │       ├── primal.rs         # Primal/Capability types
│   │       ├── primal_info.rs    # SelfKnowledge
│   │       ├── privacy.rs        # Consent, redaction
│   │       ├── scyborg.rs        # scyBorg attribution (ORC/CC-BY-SA/AGPL)
│   │       └── error.rs          # Error types
│   │
│   ├── sweet-grass-store/        # Store trait + MemoryStore
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs         # BraidStore trait
│   │       ├── error.rs
│   │       └── memory/           # In-memory backend (mod, indexes, filter)
│   │
│   ├── sweet-grass-store-postgres/ # PostgreSQL backend
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── migrations.rs
│   │       ├── error.rs
│   │       └── store/            # BraidStore impl
│   │
│   ├── sweet-grass-store-redb/   # redb embedded backend (recommended)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       └── store/            # BraidStore impl
│   │
│   ├── sweet-grass-store-sled/   # Sled backend (legacy, feature-gated)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       └── store/            # BraidStore impl
│   │
│   ├── sweet-grass-factory/      # Braid creation + attribution engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── factory/          # BraidFactory (mod, contribution, tests)
│   │       └── attribution/      # AttributionCalculator (mod, chain, tests)
│   │
│   ├── sweet-grass-compression/  # 0/1/Many session compression
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs         # Compression logic
│   │       ├── analyzer.rs       # Session analysis
│   │       ├── strategy.rs       # Strategy selection
│   │       ├── session.rs        # Session types
│   │       └── error.rs
│   │
│   ├── sweet-grass-query/        # Graph traversal + PROV-O export
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provo.rs          # W3C PROV-O JSON-LD export
│   │       ├── traversal.rs      # Graph traversal
│   │       ├── error.rs
│   │       └── engine/           # Query engine (mod, tests)
│   │
│   ├── sweet-grass-integration/  # Primal discovery + capability clients
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── anchor.rs         # Anchoring client (LoamSpine)
│   │       ├── error.rs
│   │       ├── testing.rs        # Test helpers
│   │       ├── discovery/        # Capability-based registry discovery
│   │       ├── listener/         # Session events client
│   │       └── signer/           # Signing client (BearDog)
│   │
│   └── sweet-grass-service/      # UniBin server (tarpc + JSON-RPC + REST + UDS)
│       └── src/
│           ├── lib.rs
│           ├── bin/service.rs    # UniBin entry point (sweetgrass binary)
│           ├── bootstrap.rs      # Infant discovery bootstrap
│           ├── factory.rs        # BraidStoreFactory
│           ├── rpc.rs            # tarpc service trait
│           ├── router.rs         # Axum router (REST + JSON-RPC)
│           ├── state.rs          # AppState (shared across all transports)
│           ├── uds.rs            # Unix domain socket JSON-RPC
│           ├── error.rs
│           ├── handlers/         # REST + JSON-RPC handlers
│           └── server/           # tarpc server impl
│
├── fuzz/                         # Fuzz targets (libfuzzer)
│   └── fuzz_targets/
│       ├── fuzz_braid_deserialize.rs
│       ├── fuzz_attribution.rs
│       └── fuzz_query_filter.rs
│
├── docs/guides/                  # Development guides
└── specs/                        # Technical specifications
```

---

## 6. Configuration

```rust
/// SweetGrass configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweetGrassConfig {
    /// Common primal configuration
    #[serde(flatten)]
    pub common: CommonConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Compression configuration
    pub compression: CompressionConfig,
    
    /// Query configuration
    pub query: QueryConfig,
    
    /// Listener configuration
    pub listener: ListenerConfig,
    
    /// Anchor configuration
    pub anchor: AnchorConfig,
}

/// Storage configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    pub connection_string: Option<String>,
    pub path: Option<PathBuf>,
}

/// Compression configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum vertices for single Braid
    pub single_braid_threshold: usize,
    
    /// Maximum vertices before splitting
    pub split_threshold: usize,
    
    /// Enable summary generation
    pub generate_summaries: bool,
    
    /// Summary depth limit
    pub summary_depth: u32,
}

/// Listener configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListenerConfig {
    /// Enable RhizoCrypt listener
    pub rhizocrypt: bool,
    
    /// Enable LoamSpine listener
    pub loamspine: bool,
    
    /// Enable ToadStool listener
    pub toadstool: bool,
    
    /// Buffer size for event processing
    pub buffer_size: usize,
}
```

---

## 7. Threading Model

```
┌─────────────────────────────────────────────────────────────────┐
│                     SweetGrass Runtime                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Tokio Runtime                             ││
│  ├─────────────────────────────────────────────────────────────┤│
│  │                                                              ││
│  │  ┌────────────┐  ┌────────────┐  ┌────────────────────────┐ ││
│  │  │ Event      │  │ Query      │  │ Compression            │ ││
│  │  │ Listener   │  │ Handler    │  │ Workers                │ ││
│  │  │ Tasks      │  │ Tasks      │  │                        │ ││
│  │  └─────┬──────┘  └─────┬──────┘  └──────────┬─────────────┘ ││
│  │        │               │                     │               ││
│  │        ▼               ▼                     ▼               ││
│  │  ┌───────────────────────────────────────────────────────┐  ││
│  │  │                  Work Queue (mpsc)                    │  ││
│  │  └───────────────────────────────────────────────────────┘  ││
│  │                            │                                 ││
│  │                            ▼                                 ││
│  │  ┌───────────────────────────────────────────────────────┐  ││
│  │  │              Store Connection Pool                    │  ││
│  │  │                (deadpool / bb8)                       │  ││
│  │  └───────────────────────────────────────────────────────┘  ││
│  │                                                              ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Braid creation | < 50ms | Including signing |
| Braid lookup | < 5ms | By ID, indexed |
| Provenance graph (depth 5) | < 100ms | Graph traversal |
| Attribution calculation | < 200ms | Full chain |
| GraphQL query | < 50ms | Simple queries |
| Event throughput | > 1000/sec | From listeners |
| Compression latency | < 100ms | Per session |

---

## 9. References

- [SWEETGRASS_SPECIFICATION.md](./SWEETGRASS_SPECIFICATION.md) — Master specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [BRAID_COMPRESSION.md](./BRAID_COMPRESSION.md) — Compression model
- [RhizoCrypt Specification](../../rhizoCrypt/specs/)
- [LoamSpine Specification](../../loamSpine/specs/)

---

*SweetGrass: Weaving the stories that give data its meaning.*

