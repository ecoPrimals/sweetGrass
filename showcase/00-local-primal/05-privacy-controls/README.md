# Level 5: Privacy Controls

**Time**: ~10 minutes  
**Prerequisites**: SweetGrass service binary  
**Philosophy**: GDPR-inspired data subject rights built into SweetGrass  

## What You'll Learn

This demo showcases SweetGrass's privacy-first approach to provenance tracking:

1. **Privacy Levels**
   - Public (unrestricted access)
   - Private (access controls)
   - Encrypted (end-to-end encryption)

2. **Retention Policies**
   - Duration-based retention
   - Legal hold policies
   - Automatic expiration

3. **Data Subject Rights** (GDPR-inspired)
   - Right to Access (export your data)
   - Right to Erasure (be forgotten)
   - Right to Portability

4. **Consent Tracking**
   - Explicit consent recording
   - Consent withdrawal
   - Audit trail

## Quick Start

```bash
./demo-privacy.sh
```

## What the Demo Does

1. Creates a Public Braid (unrestricted)
2. Creates a Private Braid with 30-day retention
3. Demonstrates access control enforcement
4. Exercises "Right to be Forgotten" (erasure)
5. Verifies data was anonymized/deleted
6. Demonstrates "Right to Access" (export)

## Real Execution

This demo uses the **real SweetGrass service binary** (no mocks).

The service is started, real HTTP API calls are made, and real privacy operations are executed.

## Key Principles Demonstrated

### Privacy by Design
```json
{
  "data_hash": "sha256:sensitive_data",
  "privacy_level": "Private",
  "retention_policy": {
    "policy_type": "Duration",
    "duration_days": 30
  }
}
```

### Right to Erasure
```bash
curl -X DELETE "$SERVICE_URL/api/v1/privacy/erase/$agent_did"
```

### Right to Access
```bash
curl "$SERVICE_URL/api/v1/privacy/export/$agent_did"
```

## Why This Matters

Traditional provenance systems treat all data equally. SweetGrass recognizes that:

- Some data is sensitive (medical, financial, personal)
- Users have rights over their data (GDPR, CCPA)
- Retention must be configurable
- Erasure must be real (not just "soft delete")

**Result**: Provenance tracking that respects human dignity and legal rights.

## Integration with Other Primals

Privacy controls apply to all SweetGrass Braids, including:
- Data stored in NestGate
- Compute jobs in ToadStool
- AI models in Squirrel

**Privacy flows through the entire ecosystem!**

## Next Steps

After completing this level, proceed to:
- **Level 6**: Storage Backends (different persistence options)
- **Level 7**: Real Verification (no-mocks validation)

## Learn More

- See `../../specs/07_PRIVACY_CONTROLS.md` for full privacy specification
- GDPR compliance details in documentation
- Privacy-preserving provenance patterns

