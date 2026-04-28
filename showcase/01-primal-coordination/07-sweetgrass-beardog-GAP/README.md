# 🌾🐻🐕 SweetGrass + BearDog: Signing Gap Analysis

> **HISTORICAL** — December 2025 gap analysis. As of v0.7.28 (April 2026),
> `braid.create` delegates signing to BearDog `crypto.sign` over UDS JSON-RPC.
> Braids carry Tower-tier Ed25519 witnesses. Retained as fossil record.

**Status:** RESOLVED (v0.7.28 — BearDog `crypto.sign` delegation)

---

## Executive Summary

BearDog is the **cryptographic signing and DID resolution** primal. SweetGrass creates Braids (provenance documents) that **should be cryptographically signed** to prove authenticity and prevent tampering.

**Current State:** SweetGrass Braids are **NOT cryptographically signed**.

**Impact:** Medium-High (authenticity, tamper-proof guarantees missing)

**Effort:** Medium (signature integration, verification, DID resolution)

---

## The Problem

### What BearDog Provides
- **Cryptographic signing** of documents/messages
- **DID (Decentralized Identifier)** resolution
- **Signature verification**
- **Key management** (secure, non-custodial)

### What SweetGrass Currently Lacks

1. **No Signature Field**
   - Braids have no `signature` field
   - No `proof` object per W3C PROV-O standard
   - Cannot verify Braid authenticity

2. **No BearDog Integration**
   - No calls to BearDog for signing
   - No signature verification on retrieval
   - No DID resolution for agents

3. **Trust Gap**
   - Anyone can claim to be any agent
   - Braids can be tampered with
   - No cryptographic proof of authorship

---

## Current Architecture (Without BearDog)

```
User → SweetGrass API
         ↓
   Create Braid
         ↓
   Store in DB
         ↓
   Return Braid (UNSIGNED ❌)
```

**Missing:** Cryptographic proof that the Braid was created by the claimed agent.

---

## Target Architecture (With BearDog)

```
User → SweetGrass API
         ↓
   Create Braid
         ↓
   Call BearDog for signature
         ↓
   Attach signature + proof
         ↓
   Store SIGNED Braid
         ↓
   Verify signature on retrieval
         ↓
   Return VERIFIED Braid ✅
```

**Result:** Cryptographic proof of authenticity for every Braid.

---

## What Needs to Be Built

### 1. Signature Field in Braid

Add W3C PROV-O compliant `proof` object:

```rust
pub struct Braid {
    // ... existing fields ...
    
    /// Cryptographic proof (signature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<CryptographicProof>,
}

pub struct CryptographicProof {
    /// Proof type (e.g., "Ed25519Signature2020")
    pub proof_type: String,
    
    /// When the signature was created
    pub created: DateTime<Utc>,
    
    /// Verification method (DID#key)
    pub verification_method: String,
    
    /// Signature value (base64)
    pub signature_value: String,
}
```

### 2. BearDog Client Integration

Create a client for BearDog service:

```rust
pub struct BearDogClient {
    endpoint: String,
    client: reqwest::Client,
}

impl BearDogClient {
    /// Sign a Braid using BearDog
    pub async fn sign_braid(
        &self,
        braid: &Braid,
        signer_did: &str,
    ) -> Result<CryptographicProof> {
        // Call BearDog signing endpoint
        // Return proof object
    }
    
    /// Verify a Braid signature
    pub async fn verify_braid(
        &self,
        braid: &Braid,
    ) -> Result<bool> {
        // Call BearDog verification endpoint
        // Return true if valid
    }
    
    /// Resolve a DID to its public key
    pub async fn resolve_did(
        &self,
        did: &str,
    ) -> Result<PublicKey> {
        // Call BearDog DID resolution
        // Return public key
    }
}
```

### 3. Signature Integration Points

**On Braid Creation:**
```rust
// sweet-grass-core/src/factory.rs
pub async fn create_braid(
    &self,
    request: BraidCreationRequest,
) -> Result<Braid> {
    // 1. Create unsigned Braid
    let mut braid = self.build_braid(request)?;
    
    // 2. Call BearDog for signature
    if let Some(beardog) = &self.beardog_client {
        let proof = beardog.sign_braid(&braid, &request.agent_did).await?;
        braid.proof = Some(proof);
    }
    
    // 3. Store signed Braid
    self.store.save(braid).await
}
```

**On Braid Retrieval:**
```rust
// sweet-grass-query/src/engine.rs
pub async fn get_braid(&self, id: &str) -> Result<Braid> {
    // 1. Retrieve Braid
    let braid = self.store.get(id).await?;
    
    // 2. Verify signature if present
    if let Some(proof) = &braid.proof {
        if let Some(beardog) = &self.beardog_client {
            let valid = beardog.verify_braid(&braid).await?;
            if !valid {
                return Err(Error::InvalidSignature);
            }
        }
    }
    
    // 3. Return verified Braid
    Ok(braid)
}
```

### 4. Configuration Updates

Add BearDog endpoint to config:

```toml
[service]
port = 8080

[beardog]
enabled = true
endpoint = "http://localhost:7070"  # Discovered at runtime
timeout_ms = 5000
```

### 5. Infant Discovery Integration

**SweetGrass should NOT hardcode BearDog's location.**

Use capability-based discovery:

```rust
// At startup
let signing_capability = discover_capability("signing").await?;
let beardog_endpoint = signing_capability.endpoint;
let beardog_client = BearDogClient::new(beardog_endpoint);
```

---

## Why This Matters

### 1. **Authenticity**
Without signatures, anyone can claim to be anyone. With BearDog:
- Cryptographic proof of authorship
- Tamper-evident Braids
- Trust without centralized authority

### 2. **Compliance**
Many regulations require:
- Non-repudiation (can't deny creating a document)
- Audit trails (provenance must be trustworthy)
- Data integrity (detect tampering)

### 3. **Federation**
When SweetGrass instances share Braids:
- Need to verify they came from claimed source
- Prevent malicious Braid injection
- Trust across organizational boundaries

### 4. **Legal Standing**
Signed Braids can be:
- Evidence in disputes
- Proof of compliance
- Legally binding records

---

## Implementation Phases

### Phase 1: Core Integration (2 days)
- [ ] Add `proof` field to `Braid` struct
- [ ] Create `BearDogClient`
- [ ] Implement signing on creation
- [ ] Implement verification on retrieval

### Phase 2: Discovery (1 day)
- [ ] Remove hardcoded BearDog endpoint
- [ ] Implement capability-based discovery
- [ ] Add fallback to unsigned mode if BearDog unavailable

### Phase 3: Advanced Features (2 days)
- [ ] Batch signing (multiple Braids at once)
- [ ] Signature caching (avoid re-verification)
- [ ] DID resolution caching
- [ ] Signature rotation support

### Phase 4: Testing (1 day)
- [ ] Unit tests for signing/verification
- [ ] Integration tests with real BearDog binary
- [ ] E2E tests for full flow
- [ ] Chaos tests (BearDog unavailable, slow, etc.)

**Total Effort:** ~6 days

---

## Demo Script (Once Implemented)

**File:** `demo-cryptographic-signing.sh`

**Scenario:**
1. Start BearDog and SweetGrass
2. Create a signed Braid (calls BearDog)
3. Verify the signature
4. Attempt to tamper with the Braid (verification fails)
5. Show DID resolution
6. Demonstrate multi-party signatures

**Impact:** Shows cryptographic trust in action. No mocks.

---

## Comparison with Phase 1 Primals

### NestGate + BearDog
- NestGate encrypts data
- BearDog signs encrypted data
- **Pattern:** Storage + Signing = Secure Storage

### Songbird + BearDog
- Songbird discovers services
- BearDog signs service advertisements
- **Pattern:** Discovery + Signing = Trusted Discovery

### SweetGrass + BearDog
- SweetGrass tracks provenance
- BearDog signs provenance records
- **Pattern:** Provenance + Signing = Trusted Provenance

---

## Risk Assessment

### If We Don't Integrate BearDog

**High Risk Scenarios:**
- Malicious actors forge provenance
- Disputes over authorship
- Regulatory non-compliance
- Federation trust breakdown

**Medium Risk Scenarios:**
- Internal tampering undetected
- Audit trail integrity questioned
- Loss of legal standing

**Low Risk Scenarios:**
- Single-tenant deployments (trust implicit)
- Read-only provenance (no writes)

### Recommendation

**Integrate BearDog for production deployments.**

For demos/testing, unsigned mode is acceptable with clear disclaimers.

---

## Current Workaround

**For demos without BearDog:**

Add disclaimer to showcase scripts:

```bash
echo "⚠️  DEMO MODE: Braids are NOT cryptographically signed"
echo "    In production, integrate BearDog for signatures"
```

This is honest about current limitations while showing the architecture.

---

## Summary

| Aspect | Status | Impact |
|--------|--------|--------|
| Signing Integration | ✅ Shipped (v0.7.28) | Resolved |
| DID Resolution | ✅ `Did::from_public_key_bytes` | Resolved |
| Verification | ✅ Tower-tier witnesses | Resolved |
| W3C PROV-O Proof | ✅ `Witness::from_tower_ed25519` | Resolved |
| Infant Discovery | ✅ `CryptoDelegate::resolve()` | Resolved |
| BearDog Binary | ✅ Available | N/A |

> All phases complete as of v0.7.28. See `CHANGELOG.md` for implementation details.

---

## Conclusion

> **RESOLVED** — As of v0.7.28 (April 2026), BearDog integration is complete.
> `braid.create` and `anchoring.anchor` delegate signing to BearDog `crypto.sign`
> over UDS JSON-RPC. Braids carry Tower-tier Ed25519 witnesses. The gap analysis
> above is retained as a fossil record of the original design intent.

🌾🐻🐕 **Signed Provenance = Trusted Provenance** 🐻🐕🌾

