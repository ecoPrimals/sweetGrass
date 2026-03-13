# 🌐 Federation Showcase: Two-Tower Mesh

**Demonstrating multi-instance SweetGrass federation with cross-tower provenance**

## 🎯 Purpose

Show how multiple SweetGrass instances (towers) can federate to create a distributed provenance network. This demonstrates:

- **Tower-to-tower communication** using tarpc
- **Cross-tower provenance queries** 
- **Distributed attribution** across towers
- **Mesh topology** (peer-to-peer, not hub-and-spoke)
- **Capability-based discovery** (no hardcoded tower addresses)

## 🏗️ Architecture

```
┌─────────────────┐         ┌─────────────────┐
│  Tower Alpha    │◄───────►│  Tower Beta     │
│  (Port 8200)    │  tarpc  │  (Port 8201)    │
│                 │         │                 │
│  • Local Braids │         │  • Local Braids │
│  • Query Engine │         │  • Query Engine │
│  • Federation   │         │  • Federation   │
└─────────────────┘         └─────────────────┘
        │                           │
        └───────────┬───────────────┘
                    │
            ┌───────▼────────┐
            │  Federated     │
            │  Provenance    │
            │  Graph         │
            └────────────────┘
```

## 📋 Demonstrations

### 1️⃣ **Basic Federation** (`01-basic-federation.sh`)

**What it shows:**
- Start two SweetGrass towers
- Register each tower with the other
- Create Braids on each tower
- Query local and remote Braids

**Time:** ~5 minutes

```bash
./01-basic-federation.sh
```

---

### 2️⃣ **Cross-Tower Derivation** (`02-cross-tower-derivation.sh`)

**What it shows:**
- Create a Braid on Tower Alpha
- Derive a new Braid on Tower Beta from Alpha's Braid
- Query the complete provenance graph across towers
- Calculate attribution spanning both towers

**Time:** ~8 minutes

```bash
./02-cross-tower-derivation.sh
```

---

### 3️⃣ **Distributed Collaboration** (`03-distributed-collaboration.sh`)

**What it shows:**
- Multi-agent collaboration across towers
- Data flows between towers (Alpha → Beta → Alpha)
- Complex provenance graphs with cross-tower references
- Fair attribution across distributed contributors

**Time:** ~12 minutes

```bash
./03-distributed-collaboration.sh
```

---

## 🔑 Key Principles

### 1. **Peer-to-Peer Mesh**
No central coordinator. Each tower is equal. Towers discover each other through capability announcements.

### 2. **Infant Discovery**
Towers don't hardcode peer addresses. They discover peers at runtime through:
- Environment variables
- Service discovery protocols
- Capability registries

### 3. **Sovereign Data**
Each tower owns its Braids. Cross-tower queries are requests, not assumptions.

### 4. **Transparent Federation**
Users can query as if it's a single system, but the federation is explicit in the provenance.

### 5. **Fair Attribution**
Contributors on different towers receive fair credit based on their actual contributions.

---

## 🧪 Testing Scenarios

### Scenario 1: Research Collaboration
```
Tower Alpha (University A) → Data collection
Tower Beta (University B)  → Analysis
Result: Joint publication with fair attribution
```

### Scenario 2: Supply Chain
```
Tower Alpha (Manufacturer) → Product creation
Tower Beta (Distributor)   → Logistics
Result: Complete product provenance
```

### Scenario 3: AI Training
```
Tower Alpha (Data Provider) → Training data
Tower Beta (ML Lab)         → Model training
Result: Traceable AI lineage
```

---

## 📊 Federation Metrics

Each demo tracks:
- **Cross-tower latency**: Time for remote Braid queries
- **Federation overhead**: Extra data for cross-tower references
- **Attribution accuracy**: Correct credit distribution
- **Graph completeness**: All provenance links preserved

---

## 🚀 Running All Federation Demos

```bash
# Run all federation demos in sequence
for script in *.sh; do
    echo "Running $script..."
    ./"$script"
    echo ""
done
```

---

## 📁 Output Structure

```
outputs/
├── basic-federation-1735234567/
│   ├── tower-alpha.log
│   ├── tower-beta.log
│   ├── alpha-braids.json
│   ├── beta-braids.json
│   └── federation-status.json
├── cross-tower-derivation-1735234890/
│   ├── tower-alpha.log
│   ├── tower-beta.log
│   ├── source-braid.json
│   ├── derived-braid.json
│   ├── provenance-graph.json
│   └── attribution.txt
└── distributed-collaboration-1735235123/
    ├── tower-alpha.log
    ├── tower-beta.log
    ├── workflow.json
    ├── provenance-graph.json
    └── attribution.txt
```

---

## 🎓 What You'll Learn

1. **How towers federate** without hardcoded dependencies
2. **How provenance spans** multiple towers
3. **How attribution works** in distributed systems
4. **How to query** federated provenance graphs
5. **How to build** sovereign, federated systems

---

## 🔗 Related Showcases

- **[00-local-primal](../00-local-primal/)** - Single tower capabilities
- **[01-primal-coordination](../01-primal-coordination/)** - Multi-primal integration
- **Songbird Federation** - See `../../../../phase1/songbird/showcase/federation/` for mature federation patterns

---

## 🌾 Federation Promise

**Every tower is sovereign. Every Braid is owned. Every query is transparent. Every contributor is credited fairly.**

That's the power of federated provenance! 🌾

---

## 🛠️ Technical Details

### Tower Communication
- **Protocol**: tarpc (pure Rust RPC)
- **Serialization**: bincode (fast, compact)
- **Discovery**: Capability-based (no hardcoding)
- **Security**: DID-based authentication (future)

### Cross-Tower References
```json
{
  "derivations": [{
    "from_entity": "urn:braid:sha256:abc123...",
    "from_tower": "did:primal:sweetgrass:alpha",
    "derivation_type": "Revision"
  }]
}
```

### Federation API
```rust
// Tower registration
tower.register_peer("did:primal:sweetgrass:beta", "http://localhost:8201");

// Cross-tower query
let braid = tower.query_federated("urn:braid:...", recursive: true).await?;

// Attribution across towers
let chain = tower.calculate_attribution_federated(braid_id).await?;
```

---

## 📈 Maturity Path

This showcase follows the **Songbird federation model**:

1. ✅ **Phase 1**: Basic two-tower mesh (this showcase)
2. 🔄 **Phase 2**: Multi-tower mesh (3-5 towers)
3. 🔮 **Phase 3**: Dynamic discovery and routing
4. 🔮 **Phase 4**: Byzantine fault tolerance
5. 🔮 **Phase 5**: Global provenance network

We're implementing Phase 1, learning from Songbird's successful multi-tower deployments.

---

## 🌾 The Federation Vision

Imagine a world where:
- Universities share research provenance across institutions
- Supply chains track products across companies
- AI models have complete lineage across organizations
- Every contributor gets fair credit, regardless of location

**That's the power of federated SweetGrass! 🌾**

