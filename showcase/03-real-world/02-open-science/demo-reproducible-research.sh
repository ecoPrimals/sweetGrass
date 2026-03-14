#!/usr/bin/env bash
#
# 🌾 Real-World: Reproducible Open Science
# 
# Scenario: A research paper cites experimental data. Years later,
# another team wants to reproduce the results. SweetGrass makes this
# possible by tracking complete provenance.
#
# Time: ~12 minutes
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
echo "🌾 Real-World: Reproducible Open Science"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📚 The Reproducibility Crisis:"
echo ""
echo "   • 70% of researchers can't reproduce others' work"
echo "   • 50% can't reproduce their OWN work!"
echo "   • Missing: exact data, processing steps, parameters"
echo ""
echo "   🎯 SweetGrass Solution: Complete provenance tracking"
echo ""
sleep 3

# Start service
echo -e "${BLUE}Starting SweetGrass research tracking...${NC}"
cd "$PROJECT_ROOT"
RUST_LOG=error "$PROJECT_ROOT/target/release/sweetgrass" \
    --port 8080 --storage memory > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then break; fi
    sleep 0.5
done
echo "   ✅ SweetGrass ready"
echo ""
sleep 1

# Original research (2025)
echo -e "${CYAN}━━━ 2025: Original Research Published ━━━${NC}"
echo ""
echo "   Dr. Alice publishes paper:"
echo "   'Novel protein structure analysis reveals new cancer target'"
echo ""
echo "   Paper cites experimental data tracked in SweetGrass..."
echo ""
sleep 2

# Raw experimental data
echo "   📊 Step 1: Collecting raw experimental data"
RAW_DATA=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d '{
      "data_hash": "sha256:protein_crystallography_raw_abc123",
      "mime_type": "application/x-cif",
      "size": 2147483648,
      "was_attributed_to": {
        "did": "did:key:z6MkAlice",
        "role": "Creator"
      },
      "was_generated_by": {
        "type": "Experiment",
        "name": "X-ray Crystallography",
        "started_at": "2025-01-15T00:00:00Z",
        "ended_at": "2025-01-20T00:00:00Z",
        "metadata": {
          "instrument": "Rigaku MicroMax-007 HF",
          "wavelength": "1.5418 Angstrom",
          "temperature": "100 Kelvin",
          "exposure_time": "60 seconds"
        }
      },
      "tags": ["protein", "crystallography", "raw"]
    }')
RAW_ID=$(echo "$RAW_DATA" | jq -r '.id')
echo "      ✅ Raw data tracked"
echo ""
sleep 2

# Data processing
echo "   🔬 Step 2: Processing with specific software version"
PROCESSED=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:protein_structure_processed_def456\",
      \"mime_type\": \"application/x-pdb\",
      \"size\": 524288,
      \"was_attributed_to\": {
        \"did\": \"did:key:z6MkAlice\",
        \"role\": \"Creator\"
      },
      \"was_derived_from\": [\"$RAW_ID\"],
      \"was_generated_by\": {
        \"type\": \"Computation\",
        \"name\": \"Structure Refinement\",
        \"metadata\": {
          \"software\": \"PHENIX\",
          \"version\": \"1.21.1\",
          \"parameters\": {
            \"resolution\": \"2.1 Angstrom\",
            \"r_work\": 0.23,
            \"r_free\": 0.26
          }
        }
      },
      \"tags\": [\"protein\", \"structure\", \"refined\"]
    }")
PROCESSED_ID=$(echo "$PROCESSED" | jq -r '.id')
echo "      ✅ Processing recorded"
echo ""
sleep 2

# Analysis
echo "   📈 Step 3: Statistical analysis"
ANALYSIS=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:binding_site_analysis_ghi789\",
      \"mime_type\": \"application/json\",
      \"size\": 102400,
      \"was_attributed_to\": [
        {\"did\": \"did:key:z6MkAlice\", \"role\": \"Creator\"},
        {\"did\": \"did:key:z6MkBob\", \"role\": \"Contributor\"}
      ],
      \"was_derived_from\": [\"$PROCESSED_ID\"],
      \"was_generated_by\": {
        \"type\": \"Analysis\",
        \"name\": \"Binding Site Identification\",
        \"metadata\": {
          \"software\": \"PyMOL\",
          \"version\": \"2.5.4\",
          \"method\": \"PISA interface analysis\"
        }
      },
      \"tags\": [\"analysis\", \"binding-site\"]
    }")
ANALYSIS_ID=$(echo "$ANALYSIS" | jq -r '.id')
echo "      ✅ Analysis tracked"
echo ""
sleep 2

# Paper publication
echo "   📝 Step 4: Publishing paper"
PAPER=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "{
      \"data_hash\": \"sha256:paper_cancer_target_jkl012\",
      \"mime_type\": \"application/pdf\",
      \"size\": 8388608,
      \"was_attributed_to\": [
        {\"did\": \"did:key:z6MkAlice\", \"role\": \"Creator\"},
        {\"did\": \"did:key:z6MkBob\", \"role\": \"Contributor\"}
      ],
      \"was_derived_from\": [\"$ANALYSIS_ID\"],
      \"was_generated_by\": {
        \"type\": \"Publication\",
        \"name\": \"Research Paper\",
        \"metadata\": {
          \"title\": \"Novel protein structure analysis reveals new cancer target\",
          \"doi\": \"10.1234/example.2025.001\",
          \"journal\": \"Nature Structural Biology\",
          \"published\": \"2025-03-15\"
        }
      },
      \"tags\": [\"paper\", \"published\"]
    }")
PAPER_ID=$(echo "$PAPER" | jq -r '.id')
echo "      ✅ Paper published with complete provenance"
echo ""
echo "   🌐 Paper includes SweetGrass Braid ID in 'Data Availability' section"
echo ""
sleep 3

# Fast forward to 2028
echo ""
echo -e "${MAGENTA}━━━ 2028: Another Team Wants to Reproduce ━━━${NC}"
echo ""
echo "   👨‍🔬 Dr. Carol's team finds the paper interesting"
echo "   🔬 They want to reproduce the results"
echo "   ❓ But where is the EXACT data? EXACT parameters?"
echo ""
sleep 2

# Query provenance
echo "   🔍 Looking up SweetGrass Braid ID from paper..."
echo ""
FULL_PROVENANCE=$(curl -s "http://localhost:8080/api/v1/braids/$PAPER_ID/lineage")
echo "$FULL_PROVENANCE" > "$OUTPUT_DIR/full-provenance.json"
echo "      ✅ Complete provenance chain retrieved!"
echo ""
sleep 2

# Show lineage
echo "   📊 Complete Lineage (4 generations):"
echo ""
echo "      Raw Data (2025-01-15)"
echo "      ├─ Instrument: Rigaku MicroMax-007 HF"
echo "      ├─ Wavelength: 1.5418 Å"
echo "      └─ Temperature: 100 K"
echo "           ↓"
echo "      Processed Structure (PHENIX v1.21.1)"
echo "      ├─ Resolution: 2.1 Å"
echo "      ├─ R-work: 0.23"
echo "      └─ R-free: 0.26"
echo "           ↓"
echo "      Analysis (PyMOL v2.5.4)"
echo "      └─ Method: PISA interface analysis"
echo "           ↓"
echo "      Published Paper (DOI: 10.1234/example.2025.001)"
echo ""
sleep 3

# Export for reproduction
echo "   💾 Exporting for reproduction..."
echo ""
PROVO_EXPORT=$(curl -s "http://localhost:8080/api/v1/provenance/$PAPER_ID")
echo "$PROVO_EXPORT" > "$OUTPUT_DIR/paper-provenance.jsonld"
echo "      ✅ PROV-O export complete"
echo ""
echo "   Now Dr. Carol can:"
echo "   • Download exact raw data (via content hash)"
echo "   • Use exact software versions"
echo "   • Apply exact parameters"
echo "   • Reproduce results perfectly!"
echo ""
sleep 3

# Reproduction success
echo -e "${GREEN}━━━ Reproduction Successful! ━━━${NC}"
echo ""
echo "   ✅ Dr. Carol reproduced results with 99.9% accuracy"
echo "   ✅ Differences traceable to minor equipment variation"
echo "   ✅ Original findings validated"
echo ""
sleep 2

# Academic credit
echo -e "${BLUE}📚 Academic Credit Distribution${NC}"
echo ""
ATTRIBUTION=$(curl -s "http://localhost:8080/api/v1/attribution/$PAPER_ID?include_derived=true")
echo "$ATTRIBUTION" > "$OUTPUT_DIR/attribution.json"

echo "   When citing the reproduced work, credit goes to:"
echo ""
echo "$ATTRIBUTION" | jq -r '.attributions[] | "   • \(.agent.did | split(":")[2] | sub("z6Mk"; "")): \(.role)"'
echo ""
echo "   🎯 Fair credit for ALL contributors!"
echo ""
sleep 2

# Real-world benefits
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌟 Real-World Benefits${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "   🔬 REPRODUCIBILITY:"
echo "      • Exact data preservation"
echo "      • Complete method documentation"
echo "      • Software version tracking"
echo ""
echo "   📜 COMPLIANCE:"
echo "      • NIH/NSF data sharing requirements"
echo "      • Journal reproducibility mandates"
echo "      • FAIR data principles (Findable, Accessible,"
echo "        Interoperable, Reusable)"
echo ""
echo "   🎓 ACADEMIC CREDIT:"
echo "      • All contributors properly credited"
echo "      • Derivative work attribution"
echo "      • Citation tracking"
echo ""
echo "   ⚡ EFFICIENCY:"
echo "      • No manual documentation"
echo "      • Automated provenance tracking"
echo "      • Instant lineage queries"
echo ""
sleep 3

# Funding impact
echo -e "${MAGENTA}💰 Funding Impact${NC}"
echo ""
echo "   Many funding agencies now REQUIRE:"
echo "   ✅ Data management plans"
echo "   ✅ Reproducibility protocols"
echo "   ✅ Open data sharing"
echo ""
echo "   SweetGrass satisfies ALL requirements automatically!"
echo ""
sleep 2

# Cleanup
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "📚 What you learned:"
echo "   ✅ Complete research provenance tracking"
echo "   ✅ Reproduction with exact parameters"
echo "   ✅ PROV-O export for interoperability"
echo "   ✅ Fair academic credit distribution"
echo "   ✅ Compliance with funding requirements"
echo ""
echo "💡 Why this solves the reproducibility crisis:"
echo ""
echo "   🔍 FINDABLE: Content-addressable data"
echo "   🔓 ACCESSIBLE: Standard export formats"
echo "   🔗 INTEROPERABLE: W3C PROV-O compliant"
echo "   ♻️  REUSABLE: Complete metadata preservation"
echo ""
echo "📁 Output: $OUTPUT_DIR/"
echo "   full-provenance.json - Complete lineage"
echo "   paper-provenance.jsonld - PROV-O export"
echo "   attribution.json - Credit distribution"
echo ""
echo "🎯 Integration points:"
echo "   • Jupyter notebooks: Auto-track experiments"
echo "   • Zenodo/Figshare: Export on publish"
echo "   • ORCID: Link to researcher identities"
echo "   • Protocols.io: Protocol provenance"
echo ""
echo "🌾 SweetGrass: Making science reproducible by default!"
echo ""

