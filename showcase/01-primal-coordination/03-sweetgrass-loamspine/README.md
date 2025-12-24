# 🌾 Demo: SweetGrass + LoamSpine

**Goal**: Anchor Braid commits for immutability  
**Time**: 15 minutes  
**Complexity**: Intermediate  
**Prerequisites**: LoamSpine running (or mock mode)

---

## 🎯 What This Demo Shows

1. Discover LoamSpine via capability
2. Create a Braid
3. Anchor to a spine
4. Verify anchor proof

---

## 🚀 Run the Demo

```bash
./demo-anchor.sh
```

---

## 📖 Concepts

### What is Anchoring?

Anchoring commits a Braid's hash to an immutable log (spine):
- Creates tamper-proof timestamp
- Establishes ordering
- Enables audit trails

### AnchorInfo

Information about an anchor:

```rust
pub struct AnchorInfo {
    pub braid_id: BraidId,
    pub spine_id: String,
    pub entry_hash: String,
    pub index: u64,
    pub anchored_at: Timestamp,
    pub verified: bool,
}
```

### AnchorReceipt

Receipt from an anchor operation:

```rust
pub struct AnchorReceipt {
    pub anchor: AnchorInfo,
    pub transaction_id: Option<String>,
    pub confirmations: u32,
}
```

---

## 📊 Expected Output

```
🌾 SweetGrass + LoamSpine Demo
==============================

Step 1: Discovering LoamSpine...
  Looking for capability: Anchoring
  ✅ Found LoamSpine at localhost:8093

Step 2: Creating Braid...
  Data: "Critical audit record"
  ✅ Braid created: urn:braid:xyz789

Step 3: Anchoring to spine...
  Spine ID: spine-main
  ✅ Anchor created

Anchor Receipt:
  Braid ID: urn:braid:xyz789
  Spine ID: spine-main
  Entry Hash: sha256:abc123...
  Index: 42
  Anchored At: 2025-12-23T12:00:00Z
  Confirmations: 1

Step 4: Verifying anchor...
  ✅ Anchor verified!

✅ Braid anchored for immutability!
```

---

## 🔧 Code Walkthrough

### Discovering LoamSpine

```rust
use sweet_grass_integration::anchor::{AnchorManager, LoamSpineClient};

let manager = AnchorManager::new(discovery, store, |primal| {
    Arc::new(TarpcLoamSpineClient::from_primal(primal))
}).await?;
```

### Anchoring a Braid

```rust
let receipt = manager.anchor(&braid, "spine-main").await?;

println!("Anchored at index: {}", receipt.anchor.index);
println!("Entry hash: {}", receipt.anchor.entry_hash);
```

### Verifying an Anchor

```rust
let anchor_info = manager.verify(&braid.id).await?;

if let Some(info) = anchor_info {
    if info.verified {
        println!("Anchor verified!");
    }
}
```

---

## 💡 Key Insights

### Immutability via Anchoring
Once anchored, the Braid's hash is committed to an append-only log.

### Multiple Spines
A Braid can be anchored to multiple spines for redundancy.

### Audit Trails
Anchoring creates a tamper-proof audit trail with timestamps.

---

## 🎯 Success Criteria

- [ ] Discovered LoamSpine by capability
- [ ] Anchored a Braid
- [ ] Verified the anchor
- [ ] Understood anchor proofs

---

## 📚 Next Steps

You've completed Level 1! Proceed to:

**Level 2**: `../../02-full-ecosystem/`

Experience the complete attribution pipeline!

