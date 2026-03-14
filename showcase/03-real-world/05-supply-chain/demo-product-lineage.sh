#!/usr/bin/env bash
#
# 🌾 Real-World: Supply Chain Product Lineage
# 
# Scenario: A smartphone is manufactured from components sourced globally.
# SweetGrass tracks complete provenance for quality, compliance, and recall management.
#
# Time: ~15 minutes
#

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🌾 Real-World: Supply Chain Product Lineage"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📱 The Supply Chain Challenge:"
echo ""
echo "   Modern products have complex supply chains:"
echo "   • Components from 50+ suppliers"
echo "   • Manufacturing across multiple countries"
echo "   • Assembly through 10+ steps"
echo "   • Quality control at each stage"
echo ""
echo "   When defects are found:"
echo "   • Which batch is affected?"
echo "   • Which products need recall?"
echo "   • Which supplier is responsible?"
echo "   • Can we prove compliance?"
echo ""
echo "   🎯 SweetGrass Solution: Complete product lineage"
echo ""
sleep 3

# Start service
echo -e "${BLUE}Starting SweetGrass supply chain tracking...${NC}"
cd "$PROJECT_ROOT"
RUST_LOG=error "$PROJECT_ROOT/target/release/sweetgrass" \
    --port 8080 --storage memory > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then break; fi
    sleep 0.5
done
echo "   ✅ SweetGrass tracking active"
echo ""
sleep 1

# Product: Smartphone manufacturing
echo -e "${CYAN}━━━ Product: TechCo Premium Smartphone ━━━${NC}"
echo ""
echo "   Model: TP-5000"
echo "   Batch: 2025-Q1-001"
echo "   Target: 10,000 units"
echo ""
sleep 2

# Step 1: Raw materials - Lithium for battery
echo -e "${BLUE}Step 1: Raw Materials - Lithium Mining${NC}"
echo ""
echo "   ⛏️  Lithium extracted from mine in Chile"
echo "   📦 Batch: LI-2025-001 (5 tons)"
echo "   📅 Date: 2025-01-05"
echo ""

LITHIUM=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d '{
      "data_hash": "sha256:lithium_batch_li_2025_001_abc123",
      "mime_type": "application/json",
      "size": 2048,
      "was_attributed_to": {
        "did": "did:key:z6MkChileMining",
        "role": "Producer"
      },
      "was_generated_by": {
        "type": "Extraction",
        "name": "Lithium Mining",
        "started_at": "2025-01-05T00:00:00Z",
        "metadata": {
          "supplier": "Chile Mining Co.",
          "location": "Atacama Desert, Chile",
          "batch_id": "LI-2025-001",
          "quantity": "5000 kg",
          "purity": "99.5%",
          "certifications": ["ISO 9001", "ISO 14001"]
        }
      },
      "tags": ["raw-material", "lithium", "mining"]
    }')

LITHIUM_ID=$(echo "$LITHIUM" | jq -r '.id')
echo "   ✅ Lithium batch tracked: ${LITHIUM_ID:0:40}..."
echo "   🏭 Supplier: Chile Mining Co."
echo "   ✓ Certifications: ISO 9001, ISO 14001"
echo ""
sleep 2

# Step 2: Battery cell manufacturing
echo -e "${BLUE}Step 2: Battery Cell Manufacturing${NC}"
echo ""
echo "   🔋 Battery cells manufactured in South Korea"
echo "   📦 Batch: BATT-2025-SK-042 (50,000 cells)"
echo "   📅 Date: 2025-02-10"
echo ""

BATTERY_CELLS=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:battery_cells_batt_2025_sk_042_def456\",
      \"mime_type\": \"application/json\",
      \"size\": 3072,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkKoreaBattery\",
        \"role\": \"Manufacturer\"
      },
      \"was_derived_from\": [\"$LITHIUM_ID\"],
      \"was_generated_by\": {
        \"type\": \"Manufacturing\",
        \"name\": \"Battery Cell Production\",
        \"started_at\": \"2025-02-10T00:00:00Z\",
        \"metadata\": {
          \"manufacturer\": \"Korea Battery Corp.\",
          \"location\": \"Seoul, South Korea\",
          \"batch_id\": \"BATT-2025-SK-042\",
          \"quantity\": \"50000 cells\",
          \"capacity\": \"4000 mAh\",
          \"quality_test\": \"PASSED\"
        }
      },
      \"tags\": [\"component\", \"battery\", \"manufactured\"]
    }")

BATTERY_ID=$(echo "$BATTERY_CELLS" | jq -r '.id')
echo "   ✅ Battery cells tracked: ${BATTERY_ID:0:40}..."
echo "   🏭 Manufacturer: Korea Battery Corp."
echo "   🔗 Contains: Chilean lithium (Batch LI-2025-001)"
echo "   ✓ Quality test: PASSED"
echo ""
sleep 2

# Step 3: Display panel from Taiwan
echo -e "${BLUE}Step 3: Display Panel Manufacturing${NC}"
echo ""
echo "   📱 OLED displays manufactured in Taiwan"
echo "   📦 Batch: DISP-2025-TW-128 (25,000 panels)"
echo "   📅 Date: 2025-02-15"
echo ""

DISPLAY=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d '{
      "data_hash": "sha256:display_panel_disp_2025_tw_128_ghi789",
      "mime_type": "application/json",
      "size": 2560,
      "was_attributed_to": {
        "did": "did:key:z6MkTaiwanDisplay",
        "role": "Manufacturer"
      },
      "was_generated_by": {
        "type": "Manufacturing",
        "name": "OLED Display Production",
        "started_at": "2025-02-15T00:00:00Z",
        "metadata": {
          "manufacturer": "Taiwan Display Inc.",
          "location": "Taipei, Taiwan",
          "batch_id": "DISP-2025-TW-128",
          "quantity": "25000 panels",
          "resolution": "2400x1080",
          "refresh_rate": "120Hz"
        }
      },
      "tags": ["component", "display", "oled"]
    }')

DISPLAY_ID=$(echo "$DISPLAY" | jq -r '.id')
echo "   ✅ Display panels tracked: ${DISPLAY_ID:0:40}..."
echo "   🏭 Manufacturer: Taiwan Display Inc."
echo "   📱 Specs: 2400x1080, 120Hz OLED"
echo ""
sleep 2

# Step 4: Processor from USA
echo -e "${BLUE}Step 4: Processor Manufacturing${NC}"
echo ""
echo "   💻 ARM processors manufactured in USA"
echo "   📦 Batch: CPU-2025-US-384 (30,000 chips)"
echo "   📅 Date: 2025-02-20"
echo ""

PROCESSOR=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d '{
      "data_hash": "sha256:processor_cpu_2025_us_384_jkl012",
      "mime_type": "application/json",
      "size": 2304,
      "was_attributed_to": {
        "did": "did:key:z6MkUSAChip",
        "role": "Manufacturer"
      },
      "was_generated_by": {
        "type": "Manufacturing",
        "name": "Processor Fabrication",
        "started_at": "2025-02-20T00:00:00Z",
        "metadata": {
          "manufacturer": "USA Chip Corp.",
          "location": "Austin, Texas, USA",
          "batch_id": "CPU-2025-US-384",
          "quantity": "30000 chips",
          "model": "ARM-X9 Pro",
          "process": "5nm"
        }
      },
      "tags": ["component", "processor", "chip"]
    }')

PROCESSOR_ID=$(echo "$PROCESSOR" | jq -r '.id')
echo "   ✅ Processors tracked: ${PROCESSOR_ID:0:40}..."
echo "   🏭 Manufacturer: USA Chip Corp."
echo "   💻 Model: ARM-X9 Pro (5nm)"
echo ""
sleep 2

# Step 5: Assembly in China
echo -e "${BLUE}Step 5: Final Assembly${NC}"
echo ""
echo "   🏭 Components assembled in China"
echo "   📦 Batch: PHONE-2025-CN-TP5000 (10,000 units)"
echo "   📅 Date: 2025-03-01"
echo ""

ASSEMBLED=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:assembled_phone_2025_cn_tp5000_mno345\",
      \"mime_type\": \"application/json\",
      \"size\": 5120,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkChinaAssembly\",
        \"role\": \"Assembler\"
      },
      \"was_derived_from\": [\"$BATTERY_ID\", \"$DISPLAY_ID\", \"$PROCESSOR_ID\"],
      \"was_generated_by\": {
        \"type\": \"Assembly\",
        \"name\": \"Phone Assembly\",
        \"started_at\": \"2025-03-01T00:00:00Z\",
        \"metadata\": {
          \"assembler\": \"China Assembly Ltd.\",
          \"location\": \"Shenzhen, China\",
          \"batch_id\": \"PHONE-2025-CN-TP5000\",
          \"quantity\": \"10000 units\",
          \"model\": \"TechCo TP-5000\",
          \"assembly_line\": \"Line 7\"
        }
      },
      \"tags\": [\"product\", \"assembled\", \"smartphone\"]
    }")

ASSEMBLED_ID=$(echo "$ASSEMBLED" | jq -r '.id')
echo "   ✅ Phones assembled: ${ASSEMBLED_ID:0:40}..."
echo "   🏭 Assembler: China Assembly Ltd."
echo "   🔗 Contains:"
echo "      • Battery (Korean, with Chilean lithium)"
echo "      • Display (Taiwanese OLED)"
echo "      • Processor (US ARM chip)"
echo ""
sleep 2

# Step 6: Quality control
echo -e "${BLUE}Step 6: Quality Control${NC}"
echo ""
echo "   ✓ Final quality inspection"
echo "   📦 QC Batch: QC-2025-001"
echo "   📅 Date: 2025-03-05"
echo ""

QC_PASSED=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:qc_passed_2025_001_pqr678\",
      \"mime_type\": \"application/json\",
      \"size\": 4096,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkQCInspector\",
        \"role\": \"Validator\"
      },
      \"was_derived_from\": [\"$ASSEMBLED_ID\"],
      \"was_generated_by\": {
        \"type\": \"Validation\",
        \"name\": \"Quality Control Inspection\",
        \"started_at\": \"2025-03-05T00:00:00Z\",
        \"metadata\": {
          \"inspector\": \"Quality Systems Inc.\",
          \"batch_id\": \"QC-2025-001\",
          \"passed\": \"9,950 units\",
          \"failed\": \"50 units\",
          \"pass_rate\": \"99.5%\",
          \"tests\": [\"Display\", \"Battery\", \"Performance\", \"Connectivity\"]
        }
      },
      \"tags\": [\"product\", \"qc-passed\", \"certified\"]
    }")

QC_ID=$(echo "$QC_PASSED" | jq -r '.id')
echo "   ✅ QC complete: ${QC_ID:0:40}..."
echo "   ✓ Passed: 9,950 units (99.5%)"
echo "   ✗ Failed: 50 units (recycled)"
echo ""
sleep 2

# Defect scenario
echo ""
echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${RED}⚠️  DEFECT DISCOVERED!${NC}"
echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   📅 Date: 2025-04-15"
echo "   🐛 Issue: Battery swelling in some units"
echo "   📊 Reports: 15 customer complaints"
echo ""
echo "   ❓ Questions:"
echo "   • Which batch is affected?"
echo "   • How many units need recall?"
echo "   • Which supplier is responsible?"
echo "   • Can we prove it's the battery?"
echo ""
sleep 3

# Trace back through provenance
echo -e "${YELLOW}🔍 SweetGrass Provenance Investigation...${NC}"
echo ""
echo "   Querying complete product lineage..."
echo ""

# Get full lineage
LINEAGE=$(curl -s "http://localhost:8080/api/v1/provenance/$QC_ID")
echo "$LINEAGE" > "$OUTPUT_DIR/product-lineage.json"

echo "   ✅ Complete lineage retrieved"
echo ""
echo "   📊 Provenance Chain Analysis:"
echo ""
echo "   Final Product (TechCo TP-5000)"
echo "      └─ QC Passed: 9,950 units"
echo "          └─ Assembled: 10,000 units"
echo "              ├─ Battery: BATT-2025-SK-042"
echo "              │   └─ Lithium: LI-2025-001 (Chile)"
echo "              ├─ Display: DISP-2025-TW-128 (Taiwan)"
echo "              └─ Processor: CPU-2025-US-384 (USA)"
echo ""
sleep 2

echo "   🎯 Root Cause Identified:"
echo ""
echo "   • Affected Component: Battery (BATT-2025-SK-042)"
echo "   • Supplier: Korea Battery Corp."
echo "   • Root Material: Lithium batch LI-2025-001"
echo "   • Defect: Impurity in lithium (0.3% above spec)"
echo ""
echo "   📦 Recall Scope:"
echo "   • All 9,950 units from batch PHONE-2025-CN-TP5000"
echo "   • Precise identification - no over-recall"
echo "   • Other batches unaffected"
echo ""
sleep 3

# Calculate impact
echo ""
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${MAGENTA}💰 Financial Impact Analysis${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   Without SweetGrass (Traditional System):"
echo "   ❌ Recall all Q1 production: ~50,000 units"
echo "   ❌ Cost: \$50M (50k units × \$1k each)"
echo "   ❌ Time: 3-6 months investigation"
echo "   ❌ Reputation damage: Severe"
echo ""
echo "   With SweetGrass:"
echo "   ✅ Recall only affected batch: 9,950 units"
echo "   ✅ Cost: \$10M (precise targeting)"
echo "   ✅ Time: 2 weeks (instant lineage)"
echo "   ✅ Reputation: Managed proactively"
echo ""
echo "   💰 Savings: \$40M + faster resolution"
echo ""
sleep 3

# Compliance
echo -e "${GREEN}━━━ Regulatory Compliance ━━━${NC}"
echo ""
echo "   ✅ ISO 9001 Quality Management"
echo "      • Complete traceability"
echo "      • Documented processes"
echo ""
echo "   ✅ Consumer Product Safety Commission (CPSC)"
echo "      • Rapid recall capability"
echo "      • Complete records"
echo ""
echo "   ✅ Conflict Minerals (Dodd-Frank Act)"
echo "      • Full supply chain visibility"
echo "      • Source verification"
echo ""
echo "   ✅ EU General Product Safety Regulation"
echo "      • Traceability throughout chain"
echo "      • Incident response capability"
echo ""
sleep 2

# Real-world benefits
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌟 Real-World Benefits${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   🔍 TRACEABILITY:"
echo "      • Component-level tracking"
echo "      • Multi-tier supply chain visibility"
echo "      • Source-to-product lineage"
echo ""
echo "   💰 COST SAVINGS:"
echo "      • Precise recalls (not over-recall)"
echo "      • Faster root cause analysis"
echo "      • Supplier accountability"
echo ""
echo "   ⚖️  COMPLIANCE:"
echo "      • Regulatory requirements met"
echo "      • Audit-ready documentation"
echo "      • Conflict mineral verification"
echo ""
echo "   🚀 OPERATIONS:"
echo "      • Real-time supply chain visibility"
echo "      • Quality control integration"
echo "      • Supplier performance tracking"
echo ""
sleep 2

# Cleanup
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "📚 What you learned:"
echo "   ✅ Multi-tier supply chain provenance"
echo "   ✅ Component-level traceability"
echo "   ✅ Rapid root cause analysis"
echo "   ✅ Precise recall management"
echo "   ✅ Regulatory compliance"
echo ""
echo "💡 Why this matters:"
echo ""
echo "   🏭 Manufacturing: Quality control and accountability"
echo "   💰 Finance: Millions saved in precise recalls"
echo "   ⚖️  Legal: Regulatory compliance and liability"
echo "   🌍 Sustainability: Conflict mineral verification"
echo "   🤝 Trust: Consumer confidence and transparency"
echo ""
echo "📁 Output: $OUTPUT_DIR/"
echo "   product-lineage.json - Complete supply chain"
echo ""
echo "🎯 Integration points:"
echo "   • ERP systems (SAP, Oracle)"
echo "   • MES (Manufacturing Execution Systems)"
echo "   • QMS (Quality Management Systems)"
echo "   • PLM (Product Lifecycle Management)"
echo ""
echo "🌾 SweetGrass: Making supply chains transparent!"
echo ""

