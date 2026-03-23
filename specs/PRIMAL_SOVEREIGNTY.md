# SweetGrass — Primal Sovereignty Specification

**Version**: 1.0.0  
**Status**: Canonical  
**Last Updated**: December 2025

---

## 1. Executive Summary

SweetGrass follows **ecoPrimals primal sovereignty** principles: pure Rust, no vendor lock-in, no external tooling dependencies. This specification codifies these standards.

### Core Principles

```
❌ REJECTED                      ✅ ADOPTED
─────────────────────────────────────────────────────────
gRPC (requires protoc)           tarpc (pure Rust macros)
Protocol Buffers (Google)        serde + bincode (native)
.proto code generation           #[tarpc::service] macros
C/C++ toolchain deps             100% Rust compilation
Vendor lock-in                   Community-driven crates
```

---

## 2. Pure Rust RPC Stack

### 2.1 Protocol Hierarchy

| Layer | Technology | Purpose | Latency |
|-------|------------|---------|---------|
| **High-Performance** | tarpc + bincode | Primal-to-primal | ~50μs |
| **Universal** | Custom JSON-RPC | External clients | ~2ms |
| **Human-Friendly** | HTTP/REST + JSON | Debugging, admin | ~10ms |

### 2.2 Why Not gRPC

```rust
// ❌ gRPC requires:
// - protoc (C++ compiler)
// - protobuf (Google tooling)
// - Code generation from .proto files
// - Build system complexity

// ✅ tarpc is pure Rust:
#[tarpc::service]
pub trait SweetGrassRpc {
    async fn get_braid(id: BraidId) -> Result<Braid, ServiceError>;
    async fn attribution_chain(hash: ContentHash) -> Result<AttributionChain, ServiceError>;
    async fn compress_session(session: Session) -> Result<CompressionResult, ServiceError>;
}
// No external tooling. Rust compiler does everything.
```

### 2.3 tarpc Service Definition

```rust
use tarpc::context::Context;
use serde::{Serialize, Deserialize};

/// SweetGrass RPC Service
/// All types are serde-serializable Rust structs
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
    
    // ==================== Provenance ====================
    
    /// Get provenance graph
    async fn provenance_graph(entity: EntityReference, depth: u32) -> Result<ProvenanceGraph, ServiceError>;
    
    /// Get attribution chain
    async fn attribution_chain(hash: ContentHash) -> Result<AttributionChain, ServiceError>;
    
    /// Calculate rewards distribution
    async fn calculate_rewards(hash: ContentHash, total_value: f64) -> Result<Vec<RewardShare>, ServiceError>;
    
    // ==================== Compression ====================
    
    /// Compress session to Braids
    async fn compress_session(session: Session) -> Result<CompressionResult, ServiceError>;
    
    // ==================== Export ====================
    
    /// Export to PROV-O JSON-LD
    async fn export_provo(hash: ContentHash) -> Result<JsonLdDocument, ServiceError>;
    
    // ==================== Health ====================
    
    /// Health check
    async fn health_check() -> Result<HealthStatus, ServiceError>;
}
```

### 2.4 Server Implementation

```rust
use std::sync::Arc;
use tarpc::{server, context::Context};

/// SweetGrass tarpc server
pub struct SweetGrassServer {
    store: Arc<dyn BraidStore>,
    factory: Arc<BraidFactory>,
    query_engine: Arc<QueryEngine>,
    compression: Arc<CompressionEngine>,
}

#[tarpc::server]
impl SweetGrassRpc for SweetGrassServer {
    async fn get_braid(self, _: Context, id: BraidId) -> Result<Option<Braid>, ServiceError> {
        self.store.get(&id).await
            .map_err(|e| ServiceError::Store(e.to_string()))
    }
    
    async fn attribution_chain(self, _: Context, hash: ContentHash) -> Result<AttributionChain, ServiceError> {
        self.query_engine.attribution_chain(&hash).await
            .map_err(|e| ServiceError::Query(e.to_string()))
    }
    
    async fn compress_session(self, _: Context, session: Session) -> Result<CompressionResult, ServiceError> {
        self.compression.compress(&session)
            .map_err(|e| ServiceError::Compression(e.to_string()))
    }
    
    // ... other methods
}
```

### 2.5 Client Usage

```rust
use tarpc::{client, context};

/// Connect to SweetGrass via tarpc
pub async fn connect_tarpc(addr: &str) -> Result<SweetGrassRpcClient> {
    let transport = tarpc::serde_transport::tcp::connect(
        addr,
        tarpc::tokio_serde::formats::Bincode::default,
    ).await?;
    
    let client = SweetGrassRpcClient::new(
        client::Config::default(),
        transport,
    ).spawn();
    
    Ok(client)
}

// Usage
let client = connect_tarpc("localhost:8091").await?;
let chain = client.attribution_chain(context::current(), hash).await??;
```

---

## 3. JSON-RPC 2.0 Fallback

For non-Rust clients (Python, JavaScript, curl):

### 3.1 Request Format

```json
{
    "jsonrpc": "2.0",
    "method": "braid.get",
    "params": {
        "id": "urn:braid:sha256:abc123..."
    },
    "id": 1
}
```

### 3.2 Response Format

```json
{
    "jsonrpc": "2.0",
    "result": {
        "id": "urn:braid:sha256:abc123...",
        "data_hash": "sha256:abc123...",
        "mime_type": "application/json",
        "was_attributed_to": "did:key:z6Mk..."
    },
    "id": 1
}
```

### 3.3 Python Client

```python
import requests

class SweetGrassClient:
    def __init__(self, url="http://localhost:8080/jsonrpc"):
        self.url = url
        self.id = 0
    
    def _call(self, method, params):
        self.id += 1
        response = requests.post(self.url, json={
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.id
        })
        return response.json()["result"]
    
    def get_braid(self, braid_id):
        return self._call("braid.get", {"id": braid_id})
    
    def attribution_chain(self, content_hash):
        return self._call("attribution.chain", {"hash": content_hash})
```

---

## 4. HTTP/REST for Human Access

Keep REST for debugging and admin:

```
GET  /api/v1/braids              List Braids
GET  /api/v1/braids/{id}         Get Braid by ID
GET  /api/v1/braids/hash/{hash}  Get by content hash
GET  /api/v1/provenance/{hash}   Get provenance graph
GET  /api/v1/attribution/{hash}  Get attribution chain
GET  /health                     Health check
```

---

## 5. Serialization

### 5.1 Binary (tarpc)

```rust
// Use bincode for high-performance binary serialization
tarpc::serde_transport::tcp::connect(
    addr,
    tarpc::tokio_serde::formats::Bincode::default,
)
```

### 5.2 JSON (REST/JSON-RPC)

```rust
// Use serde_json for human-readable formats
#[derive(Serialize, Deserialize)]
pub struct Braid {
    pub id: BraidId,
    pub data_hash: ContentHash,
    // All types derive Serialize/Deserialize
}
```

---

## 6. Transport Fallback

```
┌─────────────────────────────────────────────────────────┐
│                    SweetGrass Client                     │
└────────────────────────┬────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
    │ tarpc   │    │JSON-RPC │    │  REST   │
    │ Binary  │    │  JSON   │    │  HTTP   │
    │ ~50μs   │    │  ~2ms   │    │  ~10ms  │
    └────┬────┘    └────┬────┘    └────┬────┘
         │               │               │
         │    Primary    │   Fallback    │   Debug
         └───────────────┴───────────────┘
```

### 6.1 Auto-Fallback

```rust
pub struct SweetGrassMultiClient {
    tarpc: Option<SweetGrassRpcClient>,
    jsonrpc_url: String,
    rest_url: String,
}

impl SweetGrassMultiClient {
    pub async fn get_braid(&self, id: &BraidId) -> Result<Braid> {
        // Try tarpc first (fastest)
        if let Some(tarpc) = &self.tarpc {
            if let Ok(result) = tarpc.get_braid(context::current(), id.clone()).await {
                return result.map_err(Into::into);
            }
        }
        
        // Fall back to JSON-RPC
        if let Ok(result) = self.jsonrpc_get_braid(id).await {
            return Ok(result);
        }
        
        // Last resort: REST
        self.rest_get_braid(id).await
    }
}
```

---

## 7. Dependencies

### 7.1 Approved Crates

```toml
[dependencies]
# High-performance RPC (pure Rust)
tarpc = { version = "0.37", features = ["full"] }

# Serialization (native Rust)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# HTTP fallback
axum = "0.7"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Async runtime
tokio = { version = "1.40", features = ["full"] }

# No gRPC, no protobuf, no C dependencies
```

### 7.2 Forbidden Dependencies

```toml
# ❌ NEVER add these:
# tonic (gRPC - requires protobuf)
# prost (protobuf - Google tooling)
# protobuf (Google)
# grpc (C++ deps)
```

---

## 8. Inter-Primal Communication

All ecoPrimals use tarpc for primal-to-primal:

```
┌─────────────┐     tarpc      ┌─────────────┐
│ SweetGrass  │◄──────────────►│  RhizoCrypt │
│   🌾        │    bincode     │     🍄      │
└─────────────┘                └─────────────┘
      │                              │
      │ tarpc                  tarpc │
      │                              │
┌─────▼─────┐                  ┌─────▼─────┐
│ LoamSpine │                  │  BearDog  │
│    🦴     │                  │    🐻     │
└───────────┘                  └───────────┘
```

---

## 9. Performance Targets

| Protocol | Latency | Throughput |
|----------|---------|------------|
| tarpc (bincode) | < 100μs | 100K+ req/s |
| JSON-RPC | < 5ms | 20K req/s |
| REST | < 20ms | 10K req/s |

---

## 10. Summary

**SweetGrass is a pure Rust primal:**

- ✅ `tarpc` for high-performance RPC
- ✅ `serde` for serialization
- ✅ `bincode` for binary format
- ✅ No `.proto` files
- ✅ No `protoc` compiler
- ✅ No C/C++ dependencies
- ✅ No vendor lock-in
- ✅ 100% Rust toolchain

**The Rust compiler is our code generator.**

---

*SweetGrass: Pure Rust, primal sovereignty.*

