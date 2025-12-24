# 🌾 Demo: Privacy Controls

**Goal**: GDPR-inspired data subject rights  
**Time**: 10 minutes  
**Complexity**: Intermediate

---

## 🎯 What This Demo Shows

1. Privacy levels (Public, Private, Encrypted)
2. Retention policies
3. Data subject requests (Access, Erasure)
4. Consent tracking

---

## 🚀 Run the Demo

```bash
./demo-privacy.sh
```

---

## 📖 Concepts

### Privacy Levels

| Level | Description | Access |
|-------|-------------|--------|
| `Public` | Visible to all | Anyone |
| `Authenticated` | Requires auth | Authenticated users |
| `Private` | Restricted | Owner + explicit grants |
| `Encrypted` | Encrypted at rest | Decryption key holders |
| `AnonymizedPublic` | Anonymized version | Anyone (anonymized) |

### Retention Policies

| Policy | Description |
|--------|-------------|
| `Indefinite` | Keep forever (default) |
| `Duration(secs)` | Keep for N seconds |
| `Until(time)` | Keep until specific date |
| `UntilOrphaned` | Delete when no references |
| `LegalHold` | Cannot delete (legal reasons) |

### Data Subject Rights (GDPR-inspired)

| Right | Request Type |
|-------|--------------|
| Access | Get all data about subject |
| Rectification | Correct inaccurate data |
| Erasure | "Right to be forgotten" |
| Portability | Export in standard format |
| Objection | Opt out of processing |

---

## 📊 Expected Output

```
🌾 SweetGrass Privacy Controls Demo
===================================

Creating private Braid...

Privacy Settings:
  Level: Private
  Retention: Duration(86400 seconds)
  Consent: Obtained via ExplicitOptIn
  Granted Access: [did:key:z6MkAlice]

Access Checks:
  Owner (Alice) can access: ✅ true
  Granted (Bob) can access: ✅ true
  Other (Charlie) can access: ❌ false

Processing Restrictions:
  Analytics restricted: ✅ true
  Attribution restricted: ❌ false

Data Subject Request: Access
  Subject: did:key:z6MkAlice
  → Returns all Braids attributed to Alice

Data Subject Request: Erasure
  Subject: did:key:z6MkAlice
  Reason: ConsentWithdrawn
  → Marks Braids for deletion

✅ Privacy controls working!
```

---

## 🔧 Code Walkthrough

### Creating Private Metadata

```rust
use sweet_grass_core::privacy::{
    PrivacyLevel, PrivacyMetadata, RetentionPolicy, DurationSecs
};

let privacy = PrivacyMetadata::builder()
    .visibility(PrivacyLevel::Private)
    .retention(RetentionPolicy::Duration(DurationSecs(86400)))
    .consent_obtained(true)
    .grant_access(Did::new("did:key:z6MkBob"))
    .restrict_processing(ProcessingType::Analytics)
    .build();
```

### Checking Access

```rust
let owner = Did::new("did:key:z6MkAlice");
let requester = Did::new("did:key:z6MkBob");

if privacy.has_access(&requester, &owner) {
    println!("Access granted");
} else {
    println!("Access denied");
}
```

### Processing Restrictions

```rust
if privacy.is_processing_restricted(&ProcessingType::Analytics) {
    println!("Analytics not allowed");
}
```

### Data Subject Requests

```rust
use sweet_grass_core::privacy::DataSubjectRequest;

let request = DataSubjectRequest::Access {
    subject: Did::new("did:key:z6MkAlice"),
};

// Handle the request...
let request = DataSubjectRequest::Erasure {
    subject: Did::new("did:key:z6MkAlice"),
    braid_ids: vec![],  // All Braids
    reason: ErasureReason::ConsentWithdrawn,
};
```

---

## 💡 Key Insights

### Privacy Is Attached
Privacy metadata travels with the Braid. It's not external policy.

### Consent Is Tracked
Who consented, when, and to what version of the policy.

### Retention Is Enforced
Expired data should be automatically cleaned up.

### Rights Are Actionable
Data subject requests trigger real actions (access, delete, export).

---

## 🎯 Success Criteria

- [ ] Configured privacy levels
- [ ] Set retention policy
- [ ] Granted access to specific agents
- [ ] Restricted processing types
- [ ] Understood data subject rights

---

## 📚 Next Steps

You've completed Level 0! Proceed to:

**Level 1**: `../../01-primal-coordination/`

Learn how SweetGrass integrates with BearDog, RhizoCrypt, and LoamSpine!

