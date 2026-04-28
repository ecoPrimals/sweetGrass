# 🐻 BearDog Integration Gap — Honest Assessment

> **HISTORICAL** — December 2025 gap analysis. As of v0.7.28 (April 2026),
> sweetGrass delegates signing to BearDog `crypto.sign` over UDS JSON-RPC.
> Architecture mismatch fully resolved. Retained as fossil record.

**Date**: December 28, 2025  
**Status**: RESOLVED (v0.7.28 — UDS JSON-RPC `crypto.sign` delegation)

---

## 📊 SUMMARY

**Gap**: BearDog and SweetGrass use **different RPC architectures**

| Component | RPC Type | Port | Protocol |
|-----------|----------|------|----------|
| **SweetGrass** | tarpc (TCP+bincode) | 8088 | `SigningRpc` trait |
| **BearDog** | HTTP REST API | 9000 | JSON over HTTP |

**Impact**: Direct tarpc integration not possible with current BearDog binary  
**Severity**: MEDIUM (workaround exists)  
**Priority**: Document honestly, provide path forward

---

## 🔍 WHAT WE DISCOVERED

### SweetGrass Expects

SweetGrass integration layer (`sweet-grass-integration`) expects a **tarpc signing service**:

```rust
#[tarpc::service]
pub trait SigningRpc {
    async fn sign_braid(braid_bytes: Vec<u8>) -> Result<Vec<u8>, String>;
    async fn verify_braid(braid_bytes: Vec<u8>) -> Result<bool, String>;
    async fn current_did() -> Result<String, String>;
    async fn resolve_did(did: String) -> Result<Option<String>, String>;
    async fn health() -> Result<bool, String>;
}
```

**Connection**: TCP with bincode serialization  
**Discovery**: Via `Capability::Signing`

### BearDog Provides

BearDog has a **unified HTTP REST API** (`unified_api_server.rs`):

```rust
let config = BearDogApiServerConfig {
    bind_addr: "127.0.0.1:9000".parse().unwrap(),
    enable_cors: true,
    version: env!("CARGO_PKG_VERSION").to_string(),
};

let server = BearDogApiServer::new(config, btsp_provider).await?;
```

**Protocol**: HTTP REST with JSON  
**Port**: 9000 (default)  
**Capabilities**: Genesis, BTSP (Beardog Tunnel Secured Protocol)

---

## 💡 WHY THE MISMATCH?

### Design Philosophy Differences

**SweetGrass Philosophy**:
- Pure Rust tarpc for all inter-primal RPC
- No gRPC/protobuf (Primal Sovereignty)
- Binary serialization (bincode) for performance
- Type-safe RPC at compile time

**BearDog Philosophy**:
- HTTP REST for universal compatibility
- Works with any language/client
- JSON for human readability
- CORS support for web apps

**Both are valid!** Different primals evolved different patterns.

---

## 🎯 INTEGRATION PATHS

### Path A: HTTP REST Adapter (RECOMMENDED) ✅

**Create**: `HttpSigningClient` in `sweet-grass-integration`

```rust
pub struct HttpSigningClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl SigningClient for HttpSigningClient {
    async fn sign(&self, braid: &Braid) -> Result<BraidSignature> {
        let response = self.http_client
            .post(&format!("{}/sign", self.base_url))
            .json(&braid)
            .send()
            .await?;
        Ok(response.json().await?)
    }
    
    // ... other methods
}
```

**Pros**:
- Works with current BearDog
- No changes to BearDog needed
- HTTP clients are well-tested
- Can add retry logic easily

**Cons**:
- Requires `reqwest` dependency
- JSON serialization overhead
- Not as performant as tarpc

**Time**: 2-3 hours to implement

---

### Path B: BearDog tarpc Service (FUTURE)

**Create**: New `beardog-tarpc-service` crate in BearDog

```rust
// In BearDog repository
use sweet_grass_integration::signer::SigningRpc;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:9001".parse().unwrap();
    let server = tarpc::server::BaseChannel::with_defaults(transport);
    // Implement SigningRpc trait...
    server.execute(SigningRpcServer::new()).await;
}
```

**Pros**:
- Unified tarpc architecture
- Better performance
- Type safety
- Consistent with other primals

**Cons**:
- Requires BearDog changes
- Need to coordinate with BearDog team
- Maintenance burden

**Time**: 6-8 hours + BearDog team coordination

---

### Path C: Mock-Based Testing (CURRENT) ✅

**Status**: Already implemented in `sweet-grass-integration`

```rust
#[cfg(test)]
use sweet_grass_integration::testing::MockSigningClient;

let client = MockSigningClient::new();
let signed = client.sign(&braid).await?;
```

**Pros**:
- Works now for testing
- No external dependencies
- Fast test execution
- Isolated testing

**Cons**:
- Not for production
- Doesn't test real integration
- Doesn't exercise BearDog

**Usage**: Unit/integration tests only

---

## 📝 HONEST DOCUMENTATION

### What Works ✅

1. **Mock-based testing**: All SweetGrass tests pass
2. **BearDog CLI**: Works standalone (key management, encryption, etc.)
3. **BearDog HTTP API**: Works standalone (REST endpoints)
4. **Integration design**: Correct capability-based architecture

### What Doesn't Work ⚠️

1. **tarpc integration**: Architecture mismatch (tarpc vs HTTP)
2. **Live signing demo**: Can't connect SweetGrass to BearDog service
3. **Discovery integration**: BearDog doesn't advertise `Capability::Signing` via tarpc

### Path Forward 🚀

**Recommended**: Implement **Path A** (HTTP REST Adapter)

**Why**:
- Works with current BearDog
- 2-3 hours implementation time
- Unblocks showcase
- Can be optimized later

**Then**: Demonstrate in showcase with **honest note** about adapter

---

## 🌾 SHOWCASE UPDATE

### Updated Demo: `demo-signed-braid-conceptual.sh`

```bash
#!/bin/bash
# SweetGrass + BearDog Integration — Conceptual Demo
# 
# ⚠️ NOTE: This demo shows the integration DESIGN, not live execution.
# BearDog uses HTTP REST while SweetGrass expects tarpc.
# See BEARDOG_INTEGRATION_GAP.md for details.

echo "🌾 SweetGrass + BearDog Integration Demo"
echo "========================================="
echo ""
echo "⚠️  ARCHITECTURE NOTE:"
echo "   BearDog: HTTP REST API (port 9000)"
echo "   SweetGrass: tarpc RPC (expected)"
echo "   Status: HTTP adapter needed (see gap report)"
echo ""
echo "📋 What This Demo Shows:"
echo "   1. BearDog capabilities (CLI usage)"
echo "   2. Integration design (capability-based)"
echo "   3. Path forward (HTTP adapter)"
echo ""

# Show BearDog capabilities
./beardog --version
./beardog hsm discover
./beardog cross-primal --help

echo ""
echo "🔧 Integration Path Forward:"
echo "   Option A: HTTP REST adapter (2-3 hours)"
echo "   Option B: BearDog tarpc service (6-8 hours + coordination)"
echo "   Current: Mock-based testing (works now)"
echo ""
echo "📖 See: BEARDOG_INTEGRATION_GAP.md for full analysis"
```

**Key**: Be honest, show design, explain gap, provide path forward

---

## 🎯 RECOMMENDATION

**For this session**:
1. ✅ Document gap honestly (this file)
2. ✅ Update showcase demo to show design + gap
3. ✅ Mark as "conceptual" not "working"
4. ✅ Move to LoamSpine integration (both use tarpc!)

**For next session**:
- Implement HTTP REST adapter (Path A)
- Test with real BearDog HTTP server
- Update demo to "working"
- Show in showcase with adapter note

**For future**:
- Discuss tarpc service with BearDog team
- Unified RPC architecture across primals
- Performance benchmarks (tarpc vs HTTP)

---

## 💭 LESSONS LEARNED

### Good Things

1. **Honest gap discovery** through showcase building
2. **Both architectures are valid** (tarpc vs HTTP)
3. **Multiple integration paths** exist
4. **Mock testing works** for development

### Evolution Opportunities

1. **Ecosystem RPC consistency** discussion needed
2. **Adapter patterns** should be documented
3. **Protocol negotiation** for future primals
4. **Performance benchmarks** to guide decisions

---

## 🏆 CONFIDENCE LEVEL

**Current Integration**: ⚠️ **CONCEPTUAL**  
**Path Forward**: ✅ **CLEAR**  
**Time to Working**: 2-3 hours (HTTP adapter)  
**Showcase Status**: Document gap honestly, show design

---

**Status**: Gap documented honestly ✅  
**Next**: Move to LoamSpine (tarpc-compatible) ✅  
**Philosophy**: "Showcases reveal truth and opportunity"

🌾 **Every gap discovered is a learning opportunity!** 🐻

