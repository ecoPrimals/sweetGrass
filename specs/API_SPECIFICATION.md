# SweetGrass — API Specification

**Version**: 1.0.0  
**Status**: Canonical  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass exposes three API interfaces following **pure Rust primal sovereignty**:

| Interface | Protocol | Use Case | Latency |
|-----------|----------|----------|---------|
| **tarpc** | Binary (bincode) | Primal-to-primal, high-performance | ~50μs |
| **JSON-RPC** | JSON over HTTP | External clients (Python, JS) | ~2ms |
| **REST** | JSON over HTTP | Human debugging, admin tools | ~10ms |

> ⚠️ **No gRPC/protobuf** — We use pure Rust tarpc instead. See [PRIMAL_SOVEREIGNTY.md](./PRIMAL_SOVEREIGNTY.md).

---

## 2. tarpc API (Primary)

### 2.1 Service Definition

```rust
use tarpc::context::Context;
use serde::{Serialize, Deserialize};

/// SweetGrass RPC Service - Pure Rust, no protobuf
#[tarpc::service]
pub trait SweetGrassRpc {
    // ==================== Braid Operations ====================
    
    /// Create a new Braid
    async fn create_braid(request: CreateBraidRequest) -> Result<Braid, ServiceError>;
    
    /// Get Braid by ID
    async fn get_braid(id: BraidId) -> Result<Option<Braid>, ServiceError>;
    
    /// Get Braid by content hash
    async fn get_braid_by_hash(hash: ContentHash) -> Result<Option<Braid>, ServiceError>;
    
    /// Query Braids with filter
    async fn query_braids(filter: QueryFilter, order: QueryOrder) -> Result<QueryResult, ServiceError>;
    
    /// Delete Braid
    async fn delete_braid(id: BraidId) -> Result<bool, ServiceError>;
    
    // ==================== Provenance ====================
    
    /// Get provenance graph for an entity
    async fn provenance_graph(
        entity: EntityReference,
        max_depth: u32,
        include_activities: bool,
    ) -> Result<ProvenanceGraph, ServiceError>;
    
    /// Get attribution chain
    async fn attribution_chain(
        hash: ContentHash,
        config: AttributionConfig,
    ) -> Result<AttributionChain, ServiceError>;
    
    /// Get top contributors for an entity
    async fn top_contributors(
        hash: ContentHash,
        limit: u32,
    ) -> Result<Vec<ContributorShare>, ServiceError>;
    
    /// Calculate reward distribution
    async fn calculate_rewards(
        hash: ContentHash,
        total_value: f64,
    ) -> Result<Vec<RewardShare>, ServiceError>;
    
    // ==================== Agent Queries ====================
    
    /// Get agent's contributions
    async fn agent_contributions(
        agent: Did,
        time_range: Option<TimeRange>,
    ) -> Result<AgentContributions, ServiceError>;
    
    /// Get Braids by agent
    async fn braids_by_agent(agent: Did) -> Result<Vec<Braid>, ServiceError>;
    
    // ==================== Compression ====================
    
    /// Compress RhizoCrypt session to Braids
    async fn compress_session(session: Session) -> Result<CompressionResult, ServiceError>;
    
    /// Create meta-Braid (summary)
    async fn create_meta_braid(
        braid_ids: Vec<BraidId>,
        summary_type: SummaryType,
    ) -> Result<Braid, ServiceError>;
    
    // ==================== Anchoring ====================
    
    /// Anchor Braid to LoamSpine
    async fn anchor_braid(
        braid_id: BraidId,
        spine_id: String,
    ) -> Result<LoamAnchor, ServiceError>;
    
    /// Verify Braid anchor
    async fn verify_anchor(braid_id: BraidId) -> Result<AnchorVerification, ServiceError>;
    
    // ==================== Export ====================
    
    /// Export to PROV-O JSON-LD
    async fn export_provo(hash: ContentHash) -> Result<JsonLdDocument, ServiceError>;
    
    /// Export provenance graph to PROV-O
    async fn export_graph_provo(
        entity: EntityReference,
        depth: u32,
    ) -> Result<JsonLdDocument, ServiceError>;
    
    // ==================== Health ====================
    
    /// Health check
    async fn health_check() -> Result<HealthStatus, ServiceError>;
    
    /// Get service status
    async fn status() -> Result<ServiceStatus, ServiceError>;
}
```

### 2.2 Request/Response Types

```rust
// All types are serde-serializable Rust structs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBraidRequest {
    pub data_hash: ContentHash,
    pub mime_type: String,
    pub size: u64,
    pub attributed_to: Did,
    pub activity: Option<Activity>,
    pub derived_from: Vec<EntityReference>,
    pub metadata: Option<BraidMetadata>,
    pub ecop: Option<EcoPrimalsAttributes>,
    pub anchor: bool,
    pub anchor_spine: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryFilter {
    pub data_hash: Option<ContentHash>,
    pub attributed_to: Option<Did>,
    pub braid_type: Option<BraidType>,
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    pub mime_type: Option<String>,
    pub tag: Option<String>,
    pub source_primal: Option<String>,
    pub niche: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueryOrder {
    NewestFirst,
    OldestFirst,
    LargestFirst,
    SmallestFirst,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub braids: Vec<Braid>,
    pub total_count: usize,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttributionConfig {
    pub max_depth: u32,
    pub decay_factor: f64,
    pub min_share: f64,
    pub include_resources: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RewardShare {
    pub agent: Did,
    pub share: f64,
    pub amount: f64,
    pub role: AgentRole,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Timestamp,
    pub end: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorVerification {
    pub anchored: bool,
    pub anchor: Option<LoamAnchor>,
    pub verified: bool,
    pub verification_time: Option<Timestamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub store_status: String,
    pub braid_count: usize,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub healthy: bool,
    pub uptime_seconds: u64,
    pub braid_count: usize,
    pub store_type: String,
    pub version: String,
}
```

### 2.3 Error Types

```rust
#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Store error: {0}")]
    Store(String),
    
    #[error("Query error: {0}")]
    Query(String),
    
    #[error("Compression error: {0}")]
    Compression(String),
    
    #[error("Anchor error: {0}")]
    Anchor(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 2.4 Server Implementation

```rust
use std::sync::Arc;
use tarpc::{server, context::Context};

pub struct SweetGrassServer {
    store: Arc<dyn BraidStore>,
    factory: Arc<BraidFactory>,
    query: Arc<QueryEngine>,
    compression: Arc<CompressionEngine>,
}

#[tarpc::server]
impl SweetGrassRpc for SweetGrassServer {
    async fn get_braid(self, _: Context, id: BraidId) -> Result<Option<Braid>, ServiceError> {
        self.store.get(&id).await
            .map_err(|e| ServiceError::Store(e.to_string()))
    }
    
    async fn attribution_chain(
        self,
        _: Context,
        hash: ContentHash,
        config: AttributionConfig,
    ) -> Result<AttributionChain, ServiceError> {
        self.query.attribution_chain_with_config(&hash, config).await
            .map_err(|e| ServiceError::Query(e.to_string()))
    }
    
    async fn compress_session(
        self,
        _: Context,
        session: Session,
    ) -> Result<CompressionResult, ServiceError> {
        self.compression.compress(&session)
            .map_err(|e| ServiceError::Compression(e.to_string()))
    }
    
    async fn health_check(self, _: Context) -> Result<HealthStatus, ServiceError> {
        let count = self.store.count(&QueryFilter::default()).await
            .map_err(|e| ServiceError::Store(e.to_string()))?;
        
        Ok(HealthStatus {
            status: "UP".to_string(),
            store_status: "ok".to_string(),
            braid_count: count,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
    
    // ... implement remaining methods
}
```

### 2.5 Server Startup

```rust
use tarpc::serde_transport::tcp;

pub async fn start_tarpc_server(
    addr: &str,
    server: SweetGrassServer,
) -> Result<()> {
    let listener = tcp::listen(addr, tarpc::tokio_serde::formats::Bincode::default).await?;
    
    tracing::info!("🌾 SweetGrass tarpc server listening on {}", addr);
    
    listener
        .filter_map(|r| async { r.ok() })
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = server.clone();
            channel.execute(server.serve())
        })
        .buffer_unordered(100)
        .for_each(|_| async {})
        .await;
    
    Ok(())
}
```

### 2.6 Client Usage

```rust
use tarpc::{client, context};

pub struct SweetGrassClient {
    client: SweetGrassRpcClient,
}

impl SweetGrassClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let transport = tarpc::serde_transport::tcp::connect(
            addr,
            tarpc::tokio_serde::formats::Bincode::default,
        ).await?;
        
        let client = SweetGrassRpcClient::new(
            client::Config::default(),
            transport,
        ).spawn();
        
        Ok(Self { client })
    }
    
    pub async fn get_braid(&self, id: BraidId) -> Result<Option<Braid>> {
        self.client.get_braid(context::current(), id).await?
            .map_err(Into::into)
    }
    
    pub async fn attribution_chain(&self, hash: ContentHash) -> Result<AttributionChain> {
        self.client.attribution_chain(
            context::current(),
            hash,
            AttributionConfig::default(),
        ).await?.map_err(Into::into)
    }
}

// Usage
let client = SweetGrassClient::connect("localhost:8091").await?;
let braid = client.get_braid(braid_id).await?;
let chain = client.attribution_chain(hash).await?;
```

---

## 3. JSON-RPC 2.0 API

For non-Rust clients (Python, JavaScript, curl).

### 3.1 Endpoint

```
POST /jsonrpc
Content-Type: application/json
```

### 3.2 Methods

| Method | Parameters | Returns |
|--------|------------|---------|
| `braid.create` | CreateBraidRequest | Braid |
| `braid.get` | { id: string } | Braid |
| `braid.get_by_hash` | { hash: string } | Braid |
| `braid.query` | { filter, order } | QueryResult |
| `braid.delete` | { id: string } | bool |
| `braid.commit` | { id: string } | CommitResult |
| `provenance.graph` | { entity, depth } | ProvenanceGraph |
| `attribution.chain` | { hash, config } | AttributionChain |
| `attribution.top_contributors` | { hash, limit } | ContributorShare[] |
| `compression.compress_session` | Session | CompressionResult |
| `provenance.export_provo` | { hash } | JsonLdDocument |
| `contribution.record` | ContributionRecord | Braid |
| `contribution.record_dehydration` | DehydrationSummary | Braid |
| `anchoring.anchor` | { id: string } | AnchorResult |
| `anchoring.verify` | { id: string } | VerifyResult |
| `anchoring.get_anchors` | { id: string } | Anchor[] |
| `health.check` | {} | HealthStatus |

### 3.3 Request Examples

```bash
# Get Braid
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "braid.get",
    "params": { "id": "urn:braid:sha256:abc123" },
    "id": 1
  }'

# Get attribution chain
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "attribution.chain",
    "params": {
      "hash": "sha256:abc123",
      "config": { "max_depth": 10, "decay_factor": 0.7 }
    },
    "id": 2
  }'

# Calculate rewards
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "sweetgrass.calculateRewards",
    "params": { "hash": "sha256:abc123", "value": 1000.0 },
    "id": 3
  }'
```

### 3.4 Response Examples

```json
{
  "jsonrpc": "2.0",
  "result": {
    "id": "urn:braid:sha256:abc123",
    "data_hash": "sha256:abc123",
    "mime_type": "application/json",
    "size": 1024,
    "was_attributed_to": "did:key:z6Mk...",
    "generated_at_time": 1703260800000000000
  },
  "id": 1
}
```

### 3.5 Error Response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Not found: Braid urn:braid:sha256:xyz"
  },
  "id": 1
}
```

---

## 4. REST API

For human debugging and admin tools.

### 4.1 Endpoints

```yaml
# Braids
GET    /api/v1/braids                    List/query Braids
POST   /api/v1/braids                    Create Braid
GET    /api/v1/braids/{id}               Get by ID
DELETE /api/v1/braids/{id}               Delete Braid
GET    /api/v1/braids/hash/{hash}        Get by content hash

# Provenance
GET    /api/v1/provenance/{hash}         Get provenance graph
GET    /api/v1/provenance/{hash}/prov-o  Export as PROV-O

# Attribution
GET    /api/v1/attribution/{hash}        Get attribution chain
POST   /api/v1/attribution/{hash}/rewards Calculate rewards

# Agents
GET    /api/v1/agents/{did}/braids       Get agent's Braids
GET    /api/v1/agents/{did}/contributions Get contributions

# Compression
POST   /api/v1/compress                  Compress session

# Health
GET    /health                           Full health check
GET    /live                             Liveness probe
GET    /ready                            Readiness probe
```

### 4.2 Query Parameters

```yaml
# GET /api/v1/braids
agent: string          # Filter by DID
tag: string            # Filter by tag
mime_type: string      # Filter by MIME type
type: string           # Filter by Braid type
created_after: int     # Timestamp filter
created_before: int    # Timestamp filter
order: string          # newest_first, oldest_first, etc.
limit: int             # Pagination limit
offset: int            # Pagination offset
```

### 4.3 Request Examples

```bash
# List Braids by agent
curl "http://localhost:8080/api/v1/braids?agent=did:key:z6Mk...&limit=10"

# Get Braid by ID
curl "http://localhost:8080/api/v1/braids/urn:braid:sha256:abc123"

# Get provenance graph
curl "http://localhost:8080/api/v1/provenance/sha256:abc123?depth=5"

# Get attribution chain
curl "http://localhost:8080/api/v1/attribution/sha256:abc123"

# Calculate rewards
curl -X POST "http://localhost:8080/api/v1/attribution/sha256:abc123/rewards" \
  -H "Content-Type: application/json" \
  -d '{"total_value": 1000.0}'
```

---

## 5. Port Assignments

| Protocol | Default Port | Environment Variable |
|----------|--------------|---------------------|
| tarpc | 8091 | `SWEETGRASS_TARPC_PORT` |
| HTTP (REST + JSON-RPC) | 8080 | `SWEETGRASS_HTTP_PORT` |

---

## 6. References

- [PRIMAL_SOVEREIGNTY.md](./PRIMAL_SOVEREIGNTY.md) — Pure Rust principles
- [DATA_MODEL.md](./DATA_MODEL.md) — Data structures
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture

---

*SweetGrass: Pure Rust APIs for provenance and attribution.*
