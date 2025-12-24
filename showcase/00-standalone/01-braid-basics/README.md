# 🌾 Demo: Braid Basics

**Goal**: Learn the fundamentals of Braids  
**Time**: 5 minutes  
**Complexity**: Beginner

---

## 🎯 What This Demo Shows

1. Create a Braid from raw data
2. Query a Braid by ID
3. Query a Braid by content hash
4. Create a derived Braid
5. View the derivation chain

---

## 🚀 Run the Demo

```bash
./demo-create-braid.sh
```

Or run the Rust example directly:

```bash
cargo run --example braid_basics --package sweet-grass-service
```

---

## 📖 Concepts

### What is a Braid?

A Braid is a cryptographically signed provenance record following the W3C PROV-O ontology:

```rust
pub struct Braid {
    pub id: BraidId,              // Unique identifier (URN)
    pub data_hash: ContentHash,   // SHA-256 of content
    pub mime_type: String,        // Content type
    pub size: u64,                // Size in bytes
    pub was_generated_by: Option<Activity>,  // How created
    pub was_derived_from: Vec<EntityReference>,  // Sources
    pub was_attributed_to: Did,   // Primary contributor
    pub metadata: BraidMetadata,  // Additional info
    pub signature: BraidSignature,  // Cryptographic proof
}
```

### BraidId

A unique identifier in URN format:
```
urn:braid:550e8400-e29b-41d4-a716-446655440000
```

### ContentHash

SHA-256 hash of the data content:
```
sha256:7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069
```

### EntityReference

How a Braid references other entities:

| Type | Description |
|------|-------------|
| `ById` | Reference by BraidId |
| `ByHash` | Reference by ContentHash |
| `ByLoam` | Reference to LoamSpine entry |
| `External` | External URI reference |
| `Inline` | Inline data |

---

## 📊 Expected Output

```
🌾 SweetGrass Braid Basics Demo
================================

Step 1: Creating a Braid from data...

  Data: "Hello, SweetGrass!"
  MIME: text/plain
  Agent: did:key:z6MkDemoAgent...

  ✅ Braid Created:
     ID: urn:braid:abc123...
     Hash: sha256:7f83b165...
     Size: 18 bytes

Step 2: Querying by ID...

  ✅ Found Braid:
     Creator: did:key:z6MkDemoAgent...
     Created: 2025-12-23T12:00:00Z

Step 3: Querying by Hash...

  ✅ Found same Braid by content hash

Step 4: Creating a derived Braid...

  Derived from: urn:braid:abc123...
  New content: "Hello, SweetGrass! (processed)"

  ✅ Derived Braid Created:
     ID: urn:braid:def456...
     Sources: 1 (urn:braid:abc123...)

Step 5: Viewing derivation chain...

  [Source] urn:braid:abc123...
      ↓
  [Derived] urn:braid:def456...

================================
🌾 Demo Complete!
```

---

## 🔧 Code Walkthrough

### Creating a Braid

```rust
use sweet_grass_factory::BraidFactory;
use sweet_grass_core::agent::Did;

let agent = Did::new("did:key:z6MkDemoAgent");
let factory = BraidFactory::new(agent);

let braid = factory.from_data(
    b"Hello, SweetGrass!",
    "text/plain",
    None,  // No derivations
)?;

println!("Created: {}", braid.id);
```

### Storing a Braid

```rust
use sweet_grass_store::MemoryStore;

let store = MemoryStore::new();
store.put(&braid).await?;
```

### Querying by ID

```rust
let retrieved = store.get(&braid.id).await?;
if let Some(b) = retrieved {
    println!("Found: {}", b.id);
}
```

### Creating a Derived Braid

```rust
use sweet_grass_core::entity::EntityReference;

let source_ref = EntityReference::ById(braid.id.clone());
let derived = factory.from_data(
    b"Hello, SweetGrass! (processed)",
    "text/plain",
    Some(vec![source_ref]),
)?;

println!("Derived from: {:?}", derived.was_derived_from);
```

---

## 💡 Tips

### Braids Are Immutable
You cannot modify a Braid after creation. Create a new derived Braid instead.

### Hashes Are Content-Addressed
Two Braids with identical content will have the same `data_hash`.

### DIDs Are Agent Identifiers
Use Decentralized Identifiers (DIDs) for agents. Format: `did:key:z6Mk...`

---

## 🎯 Success Criteria

This demo is complete when you:

- [ ] Created a Braid from data
- [ ] Retrieved a Braid by ID
- [ ] Retrieved a Braid by hash
- [ ] Created a derived Braid
- [ ] Understood the derivation chain

---

## 📚 Next Steps

Continue to: `../02-attribution-engine/`

Learn how attribution flows through derivation chains!

