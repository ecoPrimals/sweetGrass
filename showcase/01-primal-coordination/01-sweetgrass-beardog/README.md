# 🌾 Demo: SweetGrass + BearDog

**Goal**: Cryptographically sign Braids with DID-based identities  
**Time**: 15 minutes  
**Complexity**: Intermediate  
**Prerequisites**: BearDog running (or mock mode)

---

## 🎯 What This Demo Shows

1. Discover BearDog via capability
2. Create a Braid
3. Sign with Ed25519
4. Verify W3C Data Integrity proof

---

## 🚀 Run the Demo

```bash
./demo-signed-braid.sh
```

---

## 📖 Concepts

### Capability-Based Discovery

SweetGrass finds BearDog by capability, not hardcoded address:

```rust
let discovery = LocalDiscovery::new();
let signer = discovery.find_one(&Capability::Signing).await?;
// signer.address = "localhost:8091" (discovered at runtime)
```

### BraidSignature

W3C Data Integrity compatible signature:

```rust
pub struct BraidSignature {
    pub sig_type: SignatureType,     // Ed25519Signature2020
    pub created: Timestamp,
    pub verification_method: String, // did:key:z6Mk...#keys-1
    pub proof_purpose: ProofPurpose, // AssertionMethod
    pub proof_value: String,         // Base64-encoded signature
}
```

### tarpc Integration

Pure Rust RPC to BearDog:

```rust
use sweet_grass_integration::signer::TarpcBearDogClient;

let client = TarpcBearDogClient::connect("localhost:8091").await?;
let signature = client.sign(braid_bytes).await?;
```

---

## 📊 Expected Output

```
🌾 SweetGrass + BearDog Demo
============================

Step 1: Discovering BearDog...
  Looking for capability: Signing
  ✅ Found BearDog at localhost:8091

Step 2: Creating Braid...
  Data: "Important research data"
  Agent: did:key:z6MkAlice...
  ✅ Braid created: urn:braid:abc123

Step 3: Signing with BearDog...
  Connecting via tarpc...
  ✅ Signature created

Signature Details:
  Type: Ed25519Signature2020
  Created: 2025-12-23T12:00:00Z
  Verification Method: did:key:z6MkBearDog...#keys-1
  Proof Purpose: assertionMethod
  Proof Value: eyJhbGciOiJFZERTQSIsInR5cCI6...

Step 4: Verifying signature...
  ✅ Signature valid!

✅ Signed Braid ready!
```

---

## 🔧 Code Walkthrough

### Discovering BearDog

```rust
use sweet_grass_integration::discovery::{LocalDiscovery, PrimalDiscovery};
use sweet_grass_core::config::Capability;

let discovery = LocalDiscovery::new();
let primal = discovery.find_one(&Capability::Signing).await?;

println!("Found {} at {}", primal.name, primal.address);
```

### Signing a Braid

```rust
use sweet_grass_integration::signer::{DiscoverySigner, Signer};

let signer = DiscoverySigner::new(discovery)?;

let braid = factory.from_data(b"data", "text/plain", None)?;
let signed_braid = signer.sign(&braid).await?;

println!("Signature: {:?}", signed_braid.signature);
```

### Verifying

```rust
let is_valid = signer.verify(&signed_braid).await?;
assert!(is_valid);
```

---

## 💡 Key Insights

### No Hardcoded Addresses
BearDog's address is discovered at runtime via capability lookup.

### Pure Rust RPC
tarpc provides type-safe RPC without gRPC/protobuf dependencies.

### W3C Data Integrity
Signatures follow the W3C Data Integrity standard for interoperability.

---

## 🎯 Success Criteria

- [ ] Discovered BearDog by capability
- [ ] Signed a Braid
- [ ] Verified the signature
- [ ] Understood tarpc integration

---

## 📚 Next Steps

Continue to: `../02-sweetgrass-rhizocrypt/`

Learn how to compress RhizoCrypt sessions to Braids!

