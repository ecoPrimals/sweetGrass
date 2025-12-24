#!/usr/bin/env bash
#
# 🌾 SweetGrass Demo: Privacy Controls
# Time: ~12 minutes
# Shows: GDPR-inspired privacy and consent management
# Prerequisites: SweetGrass built

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo -e "${PURPLE}🌾 SweetGrass Demo: Privacy Controls${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"
echo -e "${YELLOW}📁 Outputs will be saved to: $OUTPUT_DIR${NC}"
echo ""

sleep 1

# Step 1: Privacy Philosophy
echo -e "${BLUE}🛡️  Step 1: SweetGrass Privacy Philosophy${NC}"
echo ""
echo "Privacy isn't a feature—it's a FUNDAMENTAL RIGHT."
echo ""
echo -e "${YELLOW}   Core Principles (GDPR-inspired):${NC}"
echo "   1. ${PURPLE}Data Minimization${NC}    - Collect only what's needed"
echo "   2. ${PURPLE}Purpose Limitation${NC}   - Use data only as consented"
echo "   3. ${PURPLE}Storage Limitation${NC}   - Delete after retention period"
echo "   4. ${PURPLE}Transparency${NC}         - Users see ALL data & provenance"
echo "   5. ${PURPLE}User Control${NC}         - Data subjects have rights"
echo ""
echo -e "${YELLOW}   Data Subject Rights:${NC}"
echo "   • ${BLUE}Right to Access${NC}        - See all your data"
echo "   • ${BLUE}Right to Rectification${NC} - Correct inaccurate data"
echo "   • ${BLUE}Right to Erasure${NC}       - Delete data ('right to be forgotten')"
echo "   • ${BLUE}Right to Portability${NC}   - Export data in usable format"
echo "   • ${BLUE}Right to Object${NC}        - Opt-out of processing"
echo ""

sleep 2

# Step 2: Privacy Levels
echo -e "${BLUE}📊 Step 2: Privacy Levels${NC}"
echo ""
echo "Each Braid has a privacy level:"
echo ""
echo -e "${GREEN}   Public (0):${NC}"
echo "   • Visible to everyone"
echo "   • No restrictions on access"
echo "   • Example: Published research paper"
echo ""
echo -e "${YELLOW}   Internal (1):${NC}"
echo "   • Visible within organization"
echo "   • Access controlled by policy"
echo "   • Example: Company internal report"
echo ""
echo -e "${PURPLE}   Confidential (2):${NC}"
echo "   • Restricted to specific agents"
echo "   • Explicit access grants required"
echo "   • Example: Financial records"
echo ""
echo -e "${RED}   Secret (3):${NC}"
echo "   • Highest sensitivity"
echo "   • Minimal access, audit all views"
echo "   • Example: Personal health data"
echo ""

sleep 2

# Step 3: Consent Management
echo -e "${BLUE}✅ Step 3: Consent Management${NC}"
echo ""
echo -e "${YELLOW}   Purpose-Based Consent:${NC}"
echo ""
echo "   Alice consents to:"
echo "   • ${GREEN}Analytics${NC}          ✅ (expires: 2026-12-31)"
echo "   • ${GREEN}Research${NC}           ✅ (expires: never)"
echo "   • ${RED}Marketing${NC}          ❌ (not consented)"
echo ""
echo "   ${BLUE}Result:${NC}"
echo "   • Analytics queries work until 2026-12-31"
echo "   • Research queries always work"
echo "   • Marketing queries are BLOCKED"
echo ""
echo "   ${GREEN}✅ Purpose limitation enforced automatically!${NC}"
echo ""

sleep 2

# Step 4: Retention Policies
echo -e "${BLUE}📅 Step 4: Retention Policies${NC}"
echo ""
echo "Data shouldn't live forever without reason."
echo ""
echo -e "${YELLOW}   Example Policy: User Interaction Logs${NC}"
echo ""
echo "   Retention: 90 days"
echo "   Created:   2025-09-24"
echo "   Expires:   2025-12-24 ${RED}← TODAY!${NC}"
echo ""
echo "   ${YELLOW}Action: Automatic deletion triggered${NC}"
echo ""
echo "   ${GREEN}✅ Data minimization enforced!${NC}"
echo ""
echo -e "${YELLOW}   Exception: Legal Hold${NC}"
echo "   • Ongoing investigation"
echo "   • Retention extended automatically"
echo "   • Documented in provenance"
echo ""

sleep 2

# Step 5: Right to Access
echo -e "${BLUE}🔍 Step 5: Right to Access (Data Subject Request)${NC}"
echo ""
echo -e "${YELLOW}   User Request: Show me ALL my data${NC}"
echo ""
echo "   Query: data_subject_request(did:key:z6MkAlice)"
echo ""
echo "   Results:"
echo "   ┌─ Profile Data (1 record)"
echo "   │  └─ Name, email, created_at"
echo "   ├─ Activity Logs (47 records)"
echo "   │  └─ Last 90 days of interactions"
echo "   ├─ Generated Content (12 records)"
echo "   │  └─ Documents, comments, uploads"
echo "   └─ Consents (3 records)"
echo "      └─ Analytics, Research, Marketing"
echo ""
echo "   ${GREEN}✅ Complete transparency!${NC}"
echo ""

sleep 2

# Step 6: Right to Erasure
echo -e "${BLUE}🗑️  Step 6: Right to Erasure ('Right to be Forgotten')${NC}"
echo ""
echo -e "${YELLOW}   User Request: Delete all my data${NC}"
echo ""
echo "   Processing..."
echo "   ├─ ${GREEN}✅${NC} Profile deleted"
echo "   ├─ ${GREEN}✅${NC} Activity logs deleted (47 records)"
echo "   ├─ ${GREEN}✅${NC} Consents revoked (3 records)"
echo "   ├─ ${YELLOW}⚠️${NC}  Generated content: 12 records found"
echo "   │  ├─ 8 records: Anonymized (collaborative work)"
echo "   │  └─ 4 records: Deleted (sole author)"
echo "   └─ ${GREEN}✅${NC} Provenance updated (all deletions recorded)"
echo ""
echo "   ${GREEN}✅ User data erased, provenance intact!${NC}"
echo ""

sleep 2

# Step 7: Right to Portability
echo -e "${BLUE}📦 Step 7: Right to Portability${NC}"
echo ""
echo -e "${YELLOW}   User Request: Export my data${NC}"
echo ""
echo "   Export Format: JSON + PROV-O Turtle"
echo ""
cat > "$OUTPUT_DIR/export.json" << 'EOF'
{
  "data_subject": "did:key:z6MkAlice",
  "exported_at": "2025-12-24T12:00:00Z",
  "profile": {
    "name": "Alice",
    "email": "alice@example.com",
    "created_at": "2024-01-15T09:30:00Z"
  },
  "braids": [
    {
      "braid_id": "braid_123",
      "description": "Research Dataset",
      "created_at": "2025-06-01T14:00:00Z"
    }
  ],
  "consents": [
    {
      "purpose": "Analytics",
      "granted_at": "2025-01-01T00:00:00Z",
      "expires_at": "2026-12-31T23:59:59Z"
    }
  ]
}
EOF
echo "   ${GREEN}✅ Exported to: $OUTPUT_DIR/export.json${NC}"
echo ""
echo "   Machine-readable format for migration!"
echo ""

sleep 2

# Step 8: Provenance of Privacy Operations
echo -e "${BLUE}📜 Step 8: Privacy Operations ARE Provenance${NC}"
echo ""
echo "Every privacy action is recorded in the provenance graph:"
echo ""
echo -e "${YELLOW}   Example: Deletion Request${NC}"
echo ""
echo "   Activity: DataErasure"
echo "   ├─ Agent: did:key:z6MkAlice"
echo "   ├─ Timestamp: 2025-12-24T12:00:00Z"
echo "   ├─ Records Affected: 47"
echo "   └─ Reason: User request (GDPR Art. 17)"
echo ""
echo "   ${GREEN}✅ Audit trail for compliance!${NC}"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ SweetGrass embeds GDPR-inspired privacy"
echo "   ✅ Privacy levels control access"
echo "   ✅ Consent determines purpose-based use"
echo "   ✅ Retention policies enforce data minimization"
echo "   ✅ Data subject rights are built-in (Access, Erasure, Portability)"
echo "   ✅ ALL privacy operations are recorded in provenance"
echo ""
echo "💡 Key Insight:"
echo "   Privacy and provenance are TWO SIDES OF THE SAME COIN."
echo "   Good provenance enables privacy. Privacy requires provenance."
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo "   └─ export.json (sample data portability export)"
echo ""
echo "🎉 You've completed all standalone demos!"
echo ""
echo "⏭️  Next: See SweetGrass working with other primals"
echo "   cd ../../01-primal-coordination && ./RUN_ME_FIRST.sh"
echo ""
echo "🌾 Privacy is a human right, not a feature!"
echo ""
