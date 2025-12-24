#!/usr/bin/env bash
#
# 🌾 SweetGrass + Nestgate: Session-Aware Braids
# Shows: Linking provenance to user sessions with attestations
# Time: ~7 minutes

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo -e "${CYAN}🏠 SweetGrass + Nestgate: Session-Aware Braids${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

sleep 1

# Step 1: The Session-Provenance Gap
echo -e "${BLUE}❓ Step 1: The Session-Provenance Gap${NC}"
echo ""
echo "Typical systems have a disconnect:"
echo ""
echo -e "${RED}   Session Management${NC}    vs    ${RED}Provenance Tracking${NC}"
echo "   (who's logged in?)          (who created data?)"
echo ""
echo "   Separate systems = gaps & inconsistencies!"
echo ""
echo -e "${YELLOW}Problems:${NC}"
echo "   • Can't prove WHO was behind an action"
echo "   • Sessions expire, but provenance persists"
echo "   • Difficult to audit user activity"
echo "   • No cryptographic link between session & data"
echo ""
echo -e "${GREEN}Solution: Session-Aware Braids${NC}"
echo "   • Nestgate manages sessions & attestations"
echo "   • SweetGrass links Braids to sessions"
echo "   • Cryptographic proof of authenticity"
echo ""

sleep 2

# Step 2: Scenario Setup
echo -e "${BLUE}📋 Step 2: Compliance Scenario${NC}"
echo ""
echo -e "${YELLOW}Context:${NC}"
echo "   Medical research lab (HIPAA compliance required)"
echo ""
echo -e "${YELLOW}Requirements:${NC}"
echo "   • Audit trail for ALL data access"
echo "   • Prove WHO accessed patient data"
echo "   • Session attestations (not just logs)"
echo "   • Non-repudiation (crypto proof)"
echo ""
echo -e "${YELLOW}Actors:${NC}"
echo "   • Dr. Smith (Researcher)"
echo "   • Nestgate (Session/Attestation service)"
echo "   • SweetGrass (Provenance tracker)"
echo ""

sleep 2

# Step 3: User Login (Nestgate)
echo -e "${BLUE}🔐 Step 3: Dr. Smith Logs In${NC}"
echo ""
echo "Dr. Smith authenticates with Nestgate..."
echo ""

cat > "$OUTPUT_DIR/01_session_created.json" << 'EOF'
{
  "session_id": "sess_abc123xyz",
  "user_did": "did:key:z6MkDrSmith",
  "created_at": "2025-12-24T09:00:00Z",
  "expires_at": "2025-12-24T17:00:00Z",
  "ip_address": "10.0.1.42",
  "user_agent": "Mozilla/5.0 (Research Workstation)",
  "mfa_verified": true,
  "session_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
EOF

echo -e "${GREEN}✅ Session created: sess_abc123xyz${NC}"
echo ""
echo "   User: did:key:z6MkDrSmith"
echo "   MFA: ✅ Verified"
echo "   Expires: 17:00:00 (8 hours)"
echo ""

sleep 2

# Step 4: Create Session-Linked Braid
echo -e "${BLUE}📊 Step 4: Dr. Smith Accesses Patient Data${NC}"
echo ""
echo "Dr. Smith queries patient cohort for study..."
echo ""
echo "SweetGrass creates a Braid linked to the session:"
echo ""

cat > "$OUTPUT_DIR/02_session_linked_braid.json" << 'EOF'
{
  "braid_id": "braid_patient_cohort_001",
  "description": "Patient Cohort for Diabetes Study",
  "activity": {
    "activity_type": "DataAccess",
    "description": "Retrieved anonymized patient records",
    "started_at": "2025-12-24T09:15:00Z",
    "ended_at": "2025-12-24T09:15:03Z"
  },
  "agents": [{
    "agent_id": "did:key:z6MkDrSmith",
    "role": "DataProvider"
  }],
  "session_info": {
    "session_id": "sess_abc123xyz",
    "attestation_required": true
  },
  "privacy_level": "Secret",
  "metadata": {
    "record_count": 500,
    "anonymized": true,
    "study_id": "DIABETES_2025_001",
    "access_reason": "Research"
  }
}
EOF

echo -e "${GREEN}✅ Braid created: braid_patient_cohort_001${NC}"
echo ""
echo "   Linked to: sess_abc123xyz"
echo "   Privacy: Secret (highest level)"
echo "   Attestation: Required"
echo ""

sleep 2

# Step 5: Nestgate Attestation
echo -e "${BLUE}✍️  Step 5: Nestgate Issues Attestation${NC}"
echo ""
echo "Nestgate cryptographically signs the session activity..."
echo ""

cat > "$OUTPUT_DIR/03_session_attestation.json" << 'EOF'
{
  "attestation_id": "attest_789def",
  "session_id": "sess_abc123xyz",
  "braid_id": "braid_patient_cohort_001",
  "attested_at": "2025-12-24T09:15:03Z",
  "claims": {
    "user_identity": "did:key:z6MkDrSmith",
    "action": "DataAccess",
    "resource": "PatientCohort",
    "mfa_verified": true,
    "ip_address": "10.0.1.42"
  },
  "signature": {
    "algorithm": "EdDSA",
    "public_key": "z6MkNestgate...",
    "signature": "5f8a9b3c..."
  }
}
EOF

echo -e "${GREEN}✅ Attestation created: attest_789def${NC}"
echo ""
echo -e "${YELLOW}What this proves:${NC}"
echo "   • Dr. Smith was authenticated (MFA verified)"
echo "   • Action happened during valid session"
echo "   • Session was from known workstation (IP)"
echo "   • Cryptographically signed by Nestgate"
echo ""
echo -e "${GREEN}Result: Non-repudiable audit trail!${NC}"
echo ""

sleep 2

# Step 6: Create Derived Data
echo -e "${BLUE}📈 Step 6: Dr. Smith Creates Analysis${NC}"
echo ""
echo "Dr. Smith runs statistical analysis on the cohort..."
echo ""

cat > "$OUTPUT_DIR/04_analysis_braid.json" << 'EOF'
{
  "braid_id": "braid_analysis_002",
  "description": "Statistical Analysis - Diabetes Correlations",
  "activity": {
    "activity_type": "DataAnalysis",
    "description": "Computed correlations for HbA1c vs lifestyle factors",
    "started_at": "2025-12-24T10:00:00Z",
    "ended_at": "2025-12-24T11:30:00Z"
  },
  "agents": [{
    "agent_id": "did:key:z6MkDrSmith",
    "role": "Creator"
  }],
  "derived_from": ["braid_patient_cohort_001"],
  "session_info": {
    "session_id": "sess_abc123xyz",
    "attestation_id": "attest_789def"
  },
  "privacy_level": "Internal",
  "metadata": {
    "methodology": "Pearson correlation",
    "p_value": 0.003,
    "significant": true
  }
}
EOF

echo -e "${GREEN}✅ Braid created: braid_analysis_002${NC}"
echo ""
echo "   Derived from: braid_patient_cohort_001"
echo "   Session: sess_abc123xyz (same session)"
echo "   Attestation: attest_789def (inherited)"
echo ""
echo "   ${YELLOW}Provenance chain:${NC}"
echo "   Patient Data → Statistical Analysis"
echo "   └─ Both linked to same session!"
echo ""

sleep 2

# Step 7: Audit Query
echo -e "${BLUE}🔍 Step 7: Compliance Audit Query${NC}"
echo ""
echo "HIPAA auditor asks: 'Show me all of Dr. Smith's"
echo "                     patient data access yesterday.'"
echo ""
echo -e "${YELLOW}Query: braids_by_agent_and_date(${NC}"
echo "         agent: did:key:z6MkDrSmith,"
echo "         date:  2025-12-24"
echo "       )"
echo ""
echo "   Results:"
echo "   ┌─ braid_patient_cohort_001"
echo "   │  ├─ Time: 09:15:00"
echo "   │  ├─ Session: sess_abc123xyz"
echo "   │  ├─ Attestation: attest_789def ✅"
echo "   │  ├─ IP: 10.0.1.42"
echo "   │  └─ MFA: Verified ✅"
echo "   └─ braid_analysis_002"
echo "      ├─ Time: 10:00:00 - 11:30:00"
echo "      ├─ Derived from: braid_patient_cohort_001"
echo "      ├─ Session: sess_abc123xyz"
echo "      └─ Attestation: attest_789def ✅"
echo ""
echo -e "${GREEN}✅ Complete audit trail with proof!${NC}"
echo ""

sleep 2

# Step 8: Session Expiry & Data Persistence
echo -e "${BLUE}⏰ Step 8: Session Expires, Provenance Persists${NC}"
echo ""
echo "17:00:00 - Dr. Smith's session expires..."
echo ""
echo -e "${RED}   Session sess_abc123xyz: EXPIRED${NC}"
echo ""
echo "But the provenance and attestations remain:"
echo ""
echo -e "${GREEN}   ✅ braid_patient_cohort_001 still exists${NC}"
echo -e "${GREEN}   ✅ braid_analysis_002 still exists${NC}"
echo -e "${GREEN}   ✅ Attestations still valid${NC}"
echo -e "${GREEN}   ✅ Audit trail intact${NC}"
echo ""
echo "   ${YELLOW}Key Point:${NC}"
echo "   Sessions are temporary, provenance is forever."
echo "   The attestation PROVES the session was valid"
echo "   at the time of data access."
echo ""

sleep 2

# Step 9: Benefits
echo -e "${BLUE}💡 Step 9: Session-Aware Braids Benefits${NC}"
echo ""
echo -e "${YELLOW}1. Compliance:${NC}"
echo "   • HIPAA/GDPR audit trails"
echo "   • Non-repudiable proof of actions"
echo "   • Cryptographic attestations"
echo ""
echo -e "${YELLOW}2. Security:${NC}"
echo "   • Link actions to authenticated sessions"
echo "   • Detect unauthorized access"
echo "   • Track session lifecycle"
echo ""
echo -e "${YELLOW}3. Transparency:${NC}"
echo "   • Users see their own activity"
echo "   • Clear attribution for all actions"
echo "   • Temporal ordering preserved"
echo ""
echo -e "${YELLOW}4. Integration:${NC}"
echo "   • Nestgate handles auth complexity"
echo "   • SweetGrass handles provenance complexity"
echo "   • Together: complete solution"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Sessions and provenance should be linked"
echo "   ✅ Nestgate provides cryptographic attestations"
echo "   ✅ SweetGrass links Braids to sessions"
echo "   ✅ Non-repudiable audit trails for compliance"
echo "   ✅ Sessions expire, provenance persists"
echo "   ✅ Perfect for HIPAA/GDPR requirements"
echo ""
echo "💡 Key Insight:"
echo "   Session-aware provenance isn't just better logging—"
echo "   it's CRYPTOGRAPHIC PROOF of who did what and when."
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo "   ├─ 01_session_created.json"
echo "   ├─ 02_session_linked_braid.json"
echo "   ├─ 03_session_attestation.json"
echo "   └─ 04_analysis_braid.json"
echo ""
echo "🌾 Trust through attestation!"
echo ""

