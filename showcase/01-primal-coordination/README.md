# 🌾 Level 1: Primal Coordination

**Goal**: See SweetGrass coordinating with other ecoPrimals  
**Prerequisites**: Level 0 completed, primals available  
**Time**: 45 minutes  
**Complexity**: Intermediate

---

## 🎯 What You'll Learn

- Sign Braids with BearDog
- Compress RhizoCrypt sessions to Braids
- Anchor commits with LoamSpine
- Capability-based discovery (no hardcoding!)
- tarpc integration (pure Rust RPC)

---

## 📁 Demos

### 1. SweetGrass + BearDog (15 min)
**Directory**: `01-sweetgrass-beardog/`

Cryptographically sign Braids with DID-based identities.

```bash
cd 01-sweetgrass-beardog
./demo-signed-braid.sh
```

**What you'll see**:
- Create a Braid
- Discover BearDog via capability
- Sign with Ed25519
- Verify W3C Data Integrity proof

**Key Integration**:
```
SweetGrass                    BearDog
    │                            │
    ├── Create Braid ────────────┤
    │                            │
    ├── Discover(Signing) ───────┤
    │   ← TarpcBearDogClient ────┤
    │                            │
    ├── Sign(braid) ─────────────┤
    │   ← BraidSignature ────────┤
    │                            │
    └── Store signed Braid ──────┘
```

---

### 2. SweetGrass + RhizoCrypt (15 min)
**Directory**: `02-sweetgrass-rhizocrypt/`

Compress edit sessions into Braids.

```bash
cd 02-sweetgrass-rhizocrypt
./demo-session-compression.sh
```

**What you'll see**:
- Subscribe to RhizoCrypt events
- Receive session commits
- Compress to Braids (0/1/Many model)
- Track session provenance

**Compression Model**:
```
Session Outcome → Braid Count
───────────────────────────────
Empty/Rollback  → 0 (discard)
Single Commit   → 1 (single Braid)
Branched DAG    → N (multiple Braids)
```

**Key Integration**:
```
RhizoCrypt                   SweetGrass
    │                            │
    ├── Session Started ─────────┤
    │                            │
    ├── Vertices Added ──────────┤
    │                            │
    ├── Session Committed ───────┤
    │   ← Listen via tarpc ──────┤
    │                            │
    └── Compress to Braid(s) ────┘
```

---

### 3. SweetGrass + LoamSpine (15 min)
**Directory**: `03-sweetgrass-loamspine/`

Anchor Braid commits for immutability.

```bash
cd 03-sweetgrass-loamspine
./demo-anchor.sh
```

**What you'll see**:
- Create a Braid
- Discover LoamSpine via capability
- Anchor to a spine
- Verify anchor proof

**Key Integration**:
```
SweetGrass                   LoamSpine
    │                            │
    ├── Create Braid ────────────┤
    │                            │
    ├── Discover(Anchoring) ─────┤
    │   ← TarpcLoamSpineClient ──┤
    │                            │
    ├── Anchor(braid, spine) ────┤
    │   ← AnchorReceipt ─────────┤
    │                            │
    └── Store anchor proof ──────┘
```

---

## 🔧 Capability-Based Discovery

SweetGrass discovers primals at runtime via capabilities:

```rust
use sweet_grass_integration::{LocalDiscovery, Capability};

let discovery = LocalDiscovery::new();

// Find a primal that can sign
let signer = discovery.find_one(&Capability::Signing).await?;
println!("Found signer: {} at {}", signer.name, signer.address);

// Find a primal that can anchor
let anchor = discovery.find_one(&Capability::Anchoring).await?;
println!("Found anchor: {} at {}", anchor.name, anchor.address);
```

**No hardcoded addresses!** Primals are discovered by capability.

---

## 🔗 tarpc Integration

SweetGrass uses pure Rust RPC (no gRPC/protobuf):

```rust
use sweet_grass_integration::signer::{TarpcBearDogClient, create_beardog_client_async};

// Create client from discovered primal
let client = create_beardog_client_async(&primal.tarpc_address).await?;

// Sign a braid
let signature = client.sign(braid_bytes).await?;
```

**Benefits**:
- Pure Rust (no C dependencies)
- Type-safe (Rust types on wire)
- Fast (zero-copy where possible)
- Primal Sovereignty compliant

---

## 📊 Expected Output

### Signed Braid
```
🌾 Signing Braid with BearDog...

Braid ID: urn:braid:abc123...
Discovering signing capability...
  ✅ Found BearDog at localhost:8091

Signing...
  ✅ Signature created
  
Signature Details:
  Type: Ed25519Signature2020
  Signer: did:key:z6MkBearDog...
  Proof: eyJhbGciOiJFZ...

✅ Signed Braid stored!
```

### Session Compression
```
🌾 Compressing RhizoCrypt session...

Subscribing to session events...
  ✅ Connected to RhizoCrypt

Received: SessionCommitted
  Session ID: session-456
  Vertices: 12
  Branches: 1

Compressing (Single model)...
  ✅ Created 1 Braid

Braid: urn:braid:def789...
  Derived from: 12 vertices
  Attribution: 3 contributors
```

---

## 🛠️ Configuration

### Discovery Configuration
```toml
[discovery]
# Where to look for primals
method = "local"  # or "songbird", "dns"

# Fallback addresses (if discovery fails)
[fallback]
beardog = "localhost:8091"
rhizocrypt = "localhost:8092"
loamspine = "localhost:8093"
```

### tarpc Configuration
```toml
[tarpc]
# Connection timeout
connect_timeout_secs = 10

# Request timeout
request_timeout_secs = 30
```

---

## 💡 Key Insights

### Capability-Based > Address-Based
Instead of hardcoding `localhost:8091`, discover by capability.
This enables:
- Dynamic primal deployment
- Failover to alternatives
- Multi-machine meshes

### tarpc > gRPC
Pure Rust RPC means:
- No protobuf compilation
- No C/C++ dependencies
- Type safety across wire
- Primal Sovereignty

### Signatures Are Proofs
BearDog signatures provide:
- Authorship proof (who)
- Integrity proof (unchanged)
- Non-repudiation (can't deny)

---

## 🎯 Success Criteria

Level 1 is complete when you can:

- [ ] Sign a Braid with BearDog
- [ ] Compress a session from RhizoCrypt
- [ ] Anchor a commit with LoamSpine
- [ ] Use capability-based discovery
- [ ] Understand tarpc integration

---

## 📚 Next Steps

After Level 1, proceed to:

1. **Level 2**: `../02-full-ecosystem/README.md`
   - Complete attribution pipeline
   - Multi-primal provenance
   - Reward distribution

2. **Experiment**:
   - Run with live primals
   - Test failover scenarios
   - Measure RPC latency

---

**Ready?** Start with `01-sweetgrass-beardog/demo-signed-braid.sh`!

🌾 **Coordinate the primals!** 🌾

