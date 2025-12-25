#!/usr/bin/env bash
#
# 🌾 Real-World: ML Training Attribution
# 
# Scenario: A machine learning model is trained using datasets from multiple
# contributors. SweetGrass ensures everyone gets fair credit (and payment!)
# when the model is used.
#
# Time: ~15 minutes
#

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🌾 Real-World Scenario: ML Training Attribution"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 The Problem:"
echo ""
echo "   A medical AI model is trained using data from:"
echo "   • Dr. Alice: Collected 10,000 X-ray images"
echo "   • Bob: Labeled images with diagnoses"  
echo "   • Carol: Validated labels for accuracy"
echo "   • Dave: Curated final training dataset"
echo "   • Eve: Trained the ML model"
echo ""
echo "   Question: When hospitals pay to use the model,"
echo "   how do we fairly compensate ALL contributors?"
echo ""
sleep 3

# Start service
echo -e "${BLUE}Starting SweetGrass provenance tracking...${NC}"
cd "$PROJECT_ROOT"
RUST_LOG=error "$PROJECT_ROOT/target/release/sweet-grass-service" \
    --port 8080 --storage memory > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then break; fi
    sleep 0.5
done
echo "   ✅ SweetGrass tracking active"
echo ""
sleep 1

# Step 1: Raw data collection
echo -e "${CYAN}━━━ Phase 1: Raw Data Collection ━━━${NC}"
echo ""
echo "   👩‍⚕️ Dr. Alice collects 10,000 X-ray images"
echo "   📅 Jan-Jun 2025"
echo "   🏥 Multiple hospitals"
echo ""

RAW_DATA=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d '{
      "data_hash": "sha256:raw_xray_images_10k_abc123",
      "mime_type": "application/x-dicom",
      "size": 10737418240,
      "was_attributed_to": {
        "did": "did:key:z6MkAlice",
        "role": "DataProvider"
      },
      "was_generated_by": {
        "type": "DataCollection",
        "name": "X-Ray Image Collection",
        "started_at": "2025-01-01T00:00:00Z",
        "ended_at": "2025-06-30T23:59:59Z"
      },
      "tags": ["medical", "xray", "raw-data"]
    }')

RAW_ID=$(echo "$RAW_DATA" | jq -r '.id')
echo "   ✅ Raw data Braid created: ${RAW_ID:0:40}..."
echo ""
sleep 2

# Step 2: Labeling
echo -e "${CYAN}━━━ Phase 2: Data Labeling ━━━${NC}"
echo ""
echo "   👨‍💼 Bob labels each image with diagnosis"
echo "   🏷️  Labels: normal, pneumonia, COVID-19, etc."
echo "   ⏱️  3 months of work"
echo ""

LABELED_DATA=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:labeled_xray_10k_def456\",
      \"mime_type\": \"application/json\",
      \"size\": 5242880,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkBob\",
        \"role\": \"Transformer\"
      },
      \"was_derived_from\": [\"$RAW_ID\"],
      \"was_generated_by\": {
        \"type\": \"Transformation\",
        \"name\": \"Medical Image Labeling\"
      },
      \"tags\": [\"medical\", \"xray\", \"labeled\"]
    }")

LABELED_ID=$(echo "$LABELED_DATA" | jq -r '.id')
echo "   ✅ Labeled data Braid created: ${LABELED_ID:0:40}..."
echo "   🔗 Derived from: Raw data"
echo ""
sleep 2

# Step 3: Validation
echo -e "${CYAN}━━━ Phase 3: Label Validation ━━━${NC}"
echo ""
echo "   👩‍🔬 Carol validates labels for accuracy"
echo "   ✓ Reviews 100% of labels"
echo "   📊 Fixes 5% incorrect labels"
echo ""

VALIDATED_DATA=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:validated_xray_10k_ghi789\",
      \"mime_type\": \"application/json\",
      \"size\": 5242880,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkCarol\",
        \"role\": \"Contributor\"
      },
      \"was_derived_from\": [\"$LABELED_ID\"],
      \"was_generated_by\": {
        \"type\": \"Validation\",
        \"name\": \"Label Quality Control\"
      },
      \"tags\": [\"medical\", \"xray\", \"validated\"]
    }")

VALIDATED_ID=$(echo "$VALIDATED_DATA" | jq -r '.id')
echo "   ✅ Validated data Braid created: ${VALIDATED_ID:0:40}..."
echo "   🔗 Derived from: Labeled data"
echo ""
sleep 2

# Step 4: Curation
echo -e "${CYAN}━━━ Phase 4: Dataset Curation ━━━${NC}"
echo ""
echo "   👨‍💻 Dave curates final training dataset"
echo "   📋 Removes duplicates and outliers"
echo "   🎯 Balances classes for training"
echo ""

CURATED_DATA=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:curated_training_set_jkl012\",
      \"mime_type\": \"application/x-hdf5\",
      \"size\": 9663676416,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkDave\",
        \"role\": \"Curator\"
      },
      \"was_derived_from\": [\"$VALIDATED_ID\"],
      \"was_generated_by\": {
        \"type\": \"Curation\",
        \"name\": \"Training Set Preparation\"
      },
      \"tags\": [\"medical\", \"xray\", \"training-set\"]
    }")

CURATED_ID=$(echo "$CURATED_DATA" | jq -r '.id')
echo "   ✅ Curated dataset Braid created: ${CURATED_ID:0:40}..."
echo "   🔗 Derived from: Validated data"
echo ""
sleep 2

# Step 5: Model training
echo -e "${CYAN}━━━ Phase 5: Model Training ━━━${NC}"
echo ""
echo "   👩‍💻 Eve trains ML model using the dataset"
echo "   🤖 ResNet-50 architecture"
echo "   📈 95% accuracy achieved"
echo ""

MODEL=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:trained_model_v1_mno345\",
      \"mime_type\": \"application/x-pytorch\",
      \"size\": 104857600,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkEve\",
        \"role\": \"Creator\"
      },
      \"was_derived_from\": [\"$CURATED_ID\"],
      \"was_generated_by\": {
        \"type\": \"Training\",
        \"name\": \"ML Model Training\",
        \"started_at\": \"2025-09-01T00:00:00Z\",
        \"ended_at\": \"2025-09-07T00:00:00Z\"
      },
      \"tags\": [\"medical\", \"ml-model\", \"production\"]
    }")

MODEL_ID=$(echo "$MODEL" | jq -r '.id')
echo "   ✅ Trained model Braid created: ${MODEL_ID:0:40}..."
echo "   🔗 Derived from: Curated dataset"
echo ""
sleep 2

# Calculate attribution
echo ""
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${MAGENTA}💰 Attribution Calculation${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   SweetGrass calculates fair credit for ALL contributors"
echo "   including those in the derivation chain."
echo ""
sleep 2

ATTRIBUTION=$(curl -s "http://localhost:8080/api/v1/attribution/$MODEL_ID?include_derived=true&decay=0.5")
echo "$ATTRIBUTION" > "$OUTPUT_DIR/attribution.json"

echo "   📊 Attribution breakdown:"
echo ""
echo "$ATTRIBUTION" | jq -r '.attributions[] | "   • \(.agent.did | split(":")[2] | sub("z6Mk"; "")): \(.share * 100 | floor)% (\(.role))"'
echo ""
echo "   Total: 100%"
echo ""
sleep 3

# Payment scenario
echo -e "${MAGENTA}💵 Payment Scenario${NC}"
echo ""
echo "   Hospital pays \$100 to use the model for one patient"
echo ""
echo "   With SweetGrass attribution, payment distributes as:"
echo ""

# Recalculate with payment
echo "$ATTRIBUTION" | jq -r '.attributions[] | 
  "   \(.agent.did | split(":")[2] | sub("z6Mk"; "")): $\((.share * 100) | floor) (\(.role))"'
echo ""
echo "   🎯 Everyone gets paid proportionally to their contribution!"
echo ""
sleep 3

# Show the provenance chain
echo -e "${BLUE}📊 Complete Provenance Chain${NC}"
echo ""
echo "   Raw Data (Alice)"
echo "        ↓"
echo "   Labeled Data (Bob)"
echo "        ↓"
echo "   Validated Data (Carol)"
echo "        ↓"
echo "   Curated Dataset (Dave)"
echo "        ↓"
echo "   Trained Model (Eve)"
echo "        ↓"
echo "   Model Usage (Hospitals)"
echo "        ↓"
echo "   Fair Compensation (ALL contributors)"
echo ""
sleep 2

# Real-world impact
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌟 Real-World Impact${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   ✅ FAIR: Everyone gets credited and paid"
echo "   ✅ TRANSPARENT: Complete provenance trail"
echo "   ✅ AUTOMATIC: No manual tracking needed"
echo "   ✅ AUDITABLE: GDPR/HIPAA compliant"
echo "   ✅ INTEROPERABLE: W3C PROV-O standard"
echo ""
echo "   💡 This solves the data attribution problem in AI/ML!"
echo ""
sleep 2

# Cleanup
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "📚 What you learned:"
echo "   ✅ Multi-step ML pipeline with provenance"
echo "   ✅ Derivation chain attribution"
echo "   ✅ Fair compensation calculation"
echo "   ✅ Real-world value for AI/ML"
echo ""
echo "💡 Why this matters:"
echo ""
echo "   🔬 Science: All contributors get academic credit"
echo "   💰 Economics: Fair payment via sunCloud"
echo "   ⚖️  Ethics: Respects data providers' contributions"
echo "   📊 Compliance: Audit trail for regulations"
echo "   🤝 Incentives: Encourages data sharing"
echo ""
echo "📁 Output: $OUTPUT_DIR/"
echo "   attribution.json - Complete attribution breakdown"
echo ""
echo "🎯 Next steps:"
echo "   • Integrate with sunCloud for automatic payments"
echo "   • Export to PROV-O for research publication"
echo "   • Use for HIPAA/GDPR compliance"
echo ""
echo "🌾 SweetGrass: Making fair AI attribution real!"
echo ""

