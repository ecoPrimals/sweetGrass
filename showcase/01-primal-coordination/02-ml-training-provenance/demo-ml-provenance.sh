#!/usr/bin/env bash
#
# 🌾 SweetGrass + Beardog: ML Training Provenance
# Shows: Full provenance tracking for ML training with attribution
# Time: ~8 minutes

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
BINS_DIR="$PROJECT_ROOT/../bins"
LOG_DIR="$SCRIPT_DIR/../logs"

echo ""
echo -e "${PURPLE}🐻 SweetGrass + Beardog: ML Training Provenance${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

sleep 1

# Step 1: The ML Provenance Challenge
echo -e "${BLUE}❓ Step 1: The ML Provenance Challenge${NC}"
echo ""
echo "ML training involves many actors and artifacts:"
echo ""
echo "   📊 Training Data     (who collected it?)"
echo "   🔧 Data Processing   (who cleaned/transformed it?)"
echo "   🧠 Model Training    (who ran the training?)"
echo "   ⚙️  Hyperparameters  (who chose them?)"
echo "   📈 Trained Model     (who validated it?)"
echo ""
echo -e "${YELLOW}Questions we need to answer:${NC}"
echo "   • Where did the training data come from?"
echo "   • Who contributed to the final model?"
echo "   • How much should each contributor be paid?"
echo "   • Can we reproduce this training run?"
echo ""

sleep 2

# Step 2: Scenario Setup
echo -e "${BLUE}📋 Step 2: Training Scenario${NC}"
echo ""
echo -e "${YELLOW}Team:${NC}"
echo "   • Alice: Data Collector (collected raw data)"
echo "   • Bob:   Data Engineer (cleaned & processed)"
echo "   • Carol: ML Engineer (ran training)"
echo ""
echo -e "${YELLOW}Goal:${NC}"
echo "   Train a sentiment analysis model"
echo ""
echo -e "${YELLOW}Budget:${NC}"
echo "   \$10,000 for successful model"
echo ""

sleep 2

# Step 3: Track Data Collection
echo -e "${BLUE}📊 Step 3: Alice Collects Training Data${NC}"
echo ""
echo "Alice collects 10,000 product reviews..."
echo ""

cat > "$OUTPUT_DIR/01_data_collection.json" << 'EOF'
{
  "braid_id": "braid_raw_data_001",
  "description": "Raw Product Reviews Dataset",
  "activity": {
    "activity_type": "DataCollection",
    "description": "Scraped 10,000 product reviews from public sources",
    "started_at": "2025-12-20T09:00:00Z",
    "ended_at": "2025-12-20T17:00:00Z"
  },
  "agents": [{
    "agent_id": "did:key:z6MkAlice",
    "role": "DataProvider"
  }],
  "privacy_level": "Internal",
  "metadata": {
    "record_count": 10000,
    "source": "public_reviews"
  }
}
EOF

echo -e "${GREEN}✅ Braid created: braid_raw_data_001${NC}"
echo ""
echo "   Agent: Alice (DataProvider, weight: 0.4)"
echo "   Attribution: Alice 100%"
echo ""

sleep 2

# Step 4: Track Data Processing
echo -e "${BLUE}🔧 Step 4: Bob Processes the Data${NC}"
echo ""
echo "Bob cleans and tokenizes the data..."
echo ""

cat > "$OUTPUT_DIR/02_data_processing.json" << 'EOF'
{
  "braid_id": "braid_processed_data_002",
  "description": "Cleaned & Tokenized Reviews",
  "activity": {
    "activity_type": "DataTransformation",
    "description": "Cleaned text, removed duplicates, tokenized",
    "started_at": "2025-12-21T09:00:00Z",
    "ended_at": "2025-12-21T14:00:00Z"
  },
  "agents": [{
    "agent_id": "did:key:z6MkBob",
    "role": "Transformer"
  }],
  "derived_from": ["braid_raw_data_001"],
  "privacy_level": "Internal",
  "metadata": {
    "record_count": 9500,
    "duplicates_removed": 500,
    "pipeline": "clean -> dedupe -> tokenize"
  }
}
EOF

echo -e "${GREEN}✅ Braid created: braid_processed_data_002${NC}"
echo ""
echo "   Derived from: braid_raw_data_001 (Alice's data)"
echo "   Agent: Bob (Transformer, weight: 0.3)"
echo ""
echo "   ${YELLOW}Attribution calculation:${NC}"
echo "   • Alice (DataProvider): 0.4"
echo "   • Bob (Transformer):    0.3"
echo "   • Total weight:         0.7"
echo ""
echo "   ${GREEN}Result:${NC}"
echo "   • Alice: 57.1%"
echo "   • Bob:   42.9%"
echo ""

sleep 2

# Step 5: Track Model Training (Beardog!)
echo -e "${BLUE}🧠 Step 5: Carol Trains Model (via Beardog)${NC}"
echo ""
echo "Carol submits training job to Beardog compute..."
echo ""
echo -e "${YELLOW}Training Parameters:${NC}"
echo "   • Model: BERT-base"
echo "   • Epochs: 10"
echo "   • Batch size: 32"
echo "   • Learning rate: 2e-5"
echo ""

cat > "$OUTPUT_DIR/03_ml_training.json" << 'EOF'
{
  "braid_id": "braid_trained_model_003",
  "description": "Sentiment Analysis Model v1.0",
  "activity": {
    "activity_type": "MLTraining",
    "description": "Trained BERT model for sentiment classification",
    "started_at": "2025-12-22T10:00:00Z",
    "ended_at": "2025-12-22T18:30:00Z"
  },
  "agents": [
    {
      "agent_id": "did:key:z6MkCarol",
      "role": "Creator"
    },
    {
      "agent_id": "did:primal:beardog",
      "role": "Contributor"
    }
  ],
  "derived_from": ["braid_processed_data_002"],
  "privacy_level": "Confidential",
  "metadata": {
    "model_type": "BERT-base",
    "epochs": 10,
    "final_accuracy": 0.92,
    "training_duration_hours": 8.5,
    "compute_provider": "beardog"
  }
}
EOF

echo "Training in progress..."
echo ""
echo "   [████████████████████] 100% - Epoch 10/10"
echo ""
echo -e "${GREEN}✅ Training complete!${NC}"
echo ""
echo "   Accuracy: 92%"
echo "   Model: braid_trained_model_003"
echo ""

sleep 2

# Step 6: Calculate Attribution
echo -e "${BLUE}💰 Step 6: Calculate Fair Attribution${NC}"
echo ""
echo "The model is successful! Time to pay \$10,000."
echo ""
echo -e "${YELLOW}Attribution Chain:${NC}"
echo ""
echo "   Level 1: Alice's raw data"
echo "   └─ Alice (DataProvider): 0.4"
echo ""
echo "   Level 2: Bob's processed data"
echo "   ├─ Alice (inherited):    0.4"
echo "   └─ Bob (Transformer):    0.3"
echo "   └─ Total:                0.7"
echo ""
echo "   Level 3: Carol's trained model"
echo "   ├─ Alice (inherited):    0.4"
echo "   ├─ Bob (inherited):      0.3"
echo "   ├─ Carol (Creator):      1.0"
echo "   └─ Beardog (Contributor): 0.5"
echo "   └─ Total:                2.2"
echo ""
echo -e "${YELLOW}Final Attribution:${NC}"
echo ""

cat > "$OUTPUT_DIR/04_attribution.txt" << 'EOF'
Attribution Breakdown:
----------------------
Alice (DataProvider):  0.4 / 2.2 = 18.2%  →  $1,820
Bob (Transformer):     0.3 / 2.2 = 13.6%  →  $1,360
Carol (Creator):       1.0 / 2.2 = 45.5%  →  $4,550
Beardog (Contributor): 0.5 / 2.2 = 22.7%  →  $2,270

Total: 100% → $10,000
EOF

cat "$OUTPUT_DIR/04_attribution.txt"
echo ""
echo -e "${GREEN}✅ Fair compensation calculated!${NC}"
echo ""

sleep 2

# Step 7: Provenance Query
echo -e "${BLUE}🔍 Step 7: Query the Provenance Graph${NC}"
echo ""
echo "Let's trace the model back to its sources..."
echo ""
echo -e "${YELLOW}Query: provenance_ancestors(braid_trained_model_003)${NC}"
echo ""
echo "   Result:"
echo "   braid_trained_model_003 (Sentiment Model)"
echo "   └─ braid_processed_data_002 (Cleaned Data)"
echo "      └─ braid_raw_data_001 (Raw Reviews)"
echo ""
echo -e "${GREEN}✅ Complete lineage captured!${NC}"
echo ""

sleep 2

# Step 8: PROV-O Export for Reproducibility
echo -e "${BLUE}📤 Step 8: Export for Reproducibility${NC}"
echo ""
echo "Export the full provenance to PROV-O for documentation..."
echo ""

cat > "$OUTPUT_DIR/05_provenance_export.ttl" << 'EOF'
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<urn:sweetgrass:braid:braid_trained_model_003>
    a prov:Entity ;
    rdfs:label "Sentiment Analysis Model v1.0" ;
    prov:wasGeneratedBy <urn:sweetgrass:activity:ml_training> ;
    prov:wasAttributedTo <did:key:z6MkCarol> ;
    prov:wasDerivedFrom <urn:sweetgrass:braid:braid_processed_data_002> .

<urn:sweetgrass:braid:braid_processed_data_002>
    a prov:Entity ;
    rdfs:label "Cleaned & Tokenized Reviews" ;
    prov:wasAttributedTo <did:key:z6MkBob> ;
    prov:wasDerivedFrom <urn:sweetgrass:braid:braid_raw_data_001> .

<urn:sweetgrass:braid:braid_raw_data_001>
    a prov:Entity ;
    rdfs:label "Raw Product Reviews Dataset" ;
    prov:wasAttributedTo <did:key:z6MkAlice> .

<urn:sweetgrass:activity:ml_training>
    a prov:Activity ;
    rdfs:label "BERT Model Training" ;
    prov:used <urn:sweetgrass:braid:braid_processed_data_002> ;
    prov:wasAssociatedWith <did:key:z6MkCarol> ;
    prov:wasAssociatedWith <did:primal:beardog> .
EOF

echo -e "${GREEN}✅ PROV-O export created!${NC}"
echo ""
echo "   Now anyone can:"
echo "   • Understand the full training process"
echo "   • Reproduce the training run"
echo "   • Verify attribution calculations"
echo "   • Audit for compliance"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ ML training involves many contributors"
echo "   ✅ SweetGrass tracks EVERY step of the pipeline"
echo "   ✅ Attribution flows through derivation chains"
echo "   ✅ Fair compensation calculated automatically"
echo "   ✅ Beardog compute gets credited as contributor"
echo "   ✅ Full provenance exportable for reproducibility"
echo ""
echo "💡 Key Insight:"
echo "   When ML training provenance is tracked properly,"
echo "   EVERYONE gets fair credit for their contribution."
echo "   Data collectors, engineers, trainers, AND compute!"
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo "   ├─ 01_data_collection.json"
echo "   ├─ 02_data_processing.json"
echo "   ├─ 03_ml_training.json"
echo "   ├─ 04_attribution.txt"
echo "   └─ 05_provenance_export.ttl"
echo ""
echo "🌾 Fair attribution for all contributors!"
echo ""

