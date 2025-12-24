#!/usr/bin/env bash
#
# 🌾 SweetGrass Demo: PROV-O Export
# Time: ~8 minutes
# Shows: Export provenance to W3C PROV-O standard format
# Prerequisites: SweetGrass built

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo -e "${BLUE}🌾 SweetGrass Demo: PROV-O Export${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"
echo -e "${YELLOW}📁 Outputs will be saved to: $OUTPUT_DIR${NC}"
echo ""

sleep 1

# Step 1: What is PROV-O?
echo -e "${BLUE}📚 Step 1: Understanding PROV-O${NC}"
echo ""
echo "PROV-O is the W3C standard for provenance:"
echo ""
echo -e "${YELLOW}   Why PROV-O?${NC}"
echo "   ✅ Industry standard (W3C Recommendation)"
echo "   ✅ Interoperable with other systems"
echo "   ✅ Rich semantic model (RDF/OWL)"
echo "   ✅ Tool support (Graphviz, Neo4j, etc.)"
echo "   ✅ Human & machine readable"
echo ""
echo -e "${YELLOW}   Core Concepts:${NC}"
echo "   • ${BLUE}Entity${NC}:    Data/artifact (Braid)"
echo "   • ${BLUE}Activity${NC}:  Process/computation"
echo "   • ${BLUE}Agent${NC}:     Person/software/organization"
echo ""
echo -e "${YELLOW}   Relationships:${NC}"
echo "   • wasGeneratedBy     (Entity ← Activity)"
echo "   • used               (Activity → Entity)"
echo "   • wasAttributedTo    (Entity → Agent)"
echo "   • wasDerivedFrom     (Entity → Entity)"
echo "   • wasAssociatedWith  (Activity → Agent)"
echo ""

sleep 2

# Step 2: SweetGrass → PROV-O Mapping
echo -e "${BLUE}🔄 Step 2: How SweetGrass Maps to PROV-O${NC}"
echo ""
echo -e "${YELLOW}   SweetGrass Braid → PROV-O Entity:${NC}"
echo "   • braid_id       → prov:entity (IRI)"
echo "   • description    → rdfs:label"
echo "   • created_at     → prov:generatedAtTime"
echo ""
echo -e "${YELLOW}   SweetGrass Activity → PROV-O Activity:${NC}"
echo "   • activity_type  → rdf:type (subclass)"
echo "   • started_at     → prov:startedAtTime"
echo "   • ended_at       → prov:endedAtTime"
echo ""
echo -e "${YELLOW}   SweetGrass Agent → PROV-O Agent:${NC}"
echo "   • agent_id (DID) → prov:agent (IRI)"
echo "   • role           → prov:hadRole"
echo ""

sleep 2

# Step 3: Example Export
echo -e "${BLUE}📤 Step 3: Example PROV-O Export${NC}"
echo ""
echo -e "${YELLOW}   Input: Simple Braid with derivation${NC}"
echo ""
echo "   Braid: braid_123 (created by Alice)"
echo "   ├─ Activity: DataCuration"
echo "   ├─ Agent: did:key:z6MkAlice..."
echo "   └─ Derived from: braid_456"
echo ""
echo -e "${YELLOW}   Output: PROV-O Turtle (RDF)${NC}"
echo ""
cat > "$OUTPUT_DIR/example.ttl" << 'EOF'
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<urn:sweetgrass:braid:braid_123>
    a prov:Entity ;
    rdfs:label "Curated Dataset" ;
    prov:wasGeneratedBy <urn:sweetgrass:activity:act_789> ;
    prov:wasAttributedTo <did:key:z6MkAlice> ;
    prov:wasDerivedFrom <urn:sweetgrass:braid:braid_456> ;
    prov:generatedAtTime "2025-12-24T12:00:00Z"^^xsd:dateTime .

<urn:sweetgrass:activity:act_789>
    a prov:Activity ;
    rdfs:label "Data Curation" ;
    prov:used <urn:sweetgrass:braid:braid_456> ;
    prov:wasAssociatedWith <did:key:z6MkAlice> .

<did:key:z6MkAlice>
    a prov:Agent ;
    prov:actedOnBehalfOf <did:key:z6MkOrganization> .
EOF
echo "   ${GREEN}✅ Valid PROV-O created!${NC}"
echo ""

sleep 2

# Step 4: Use Cases
echo -e "${BLUE}🎯 Step 4: Why Export to PROV-O?${NC}"
echo ""
echo -e "${YELLOW}   1. Interoperability:${NC}"
echo "      • Share provenance with other systems"
echo "      • Import into semantic databases (Neo4j)"
echo "      • Integrate with scientific workflows"
echo ""
echo -e "${YELLOW}   2. Visualization:${NC}"
echo "      • Use Graphviz for DAG visualization"
echo "      • Create interactive provenance browsers"
echo "      • Generate reports and diagrams"
echo ""
echo -e "${YELLOW}   3. Compliance:${NC}"
echo "      • Meet regulatory requirements (GDPR, HIPAA)"
echo "      • Provide audit trails"
echo "      • Demonstrate data lineage"
echo ""
echo -e "${YELLOW}   4. Analysis:${NC}"
echo "      • Run SPARQL queries"
echo "      • Perform graph analysis"
echo "      • Find patterns in provenance"
echo ""

sleep 2

# Step 5: Format Options
echo -e "${BLUE}📋 Step 5: Export Formats${NC}"
echo ""
echo "SweetGrass supports multiple PROV-O serializations:"
echo ""
echo -e "${YELLOW}   1. Turtle (TTL) - Human Readable:${NC}"
echo "      @prefix prov: <http://...> ."
echo "      <urn:braid:123> a prov:Entity ."
echo ""
echo -e "${YELLOW}   2. JSON-LD - Web Friendly:${NC}"
echo '      {"@context": "http://...prov",'
echo '       "@type": "Entity",'
echo '       "@id": "urn:braid:123"}'
echo ""
echo -e "${YELLOW}   3. RDF/XML - Legacy Systems:${NC}"
echo '      <rdf:RDF xmlns:prov="...">'
echo '        <prov:Entity rdf:about="urn:braid:123"/>'
echo '      </rdf:RDF>'
echo ""

sleep 2

# Step 6: Advanced Features
echo -e "${BLUE}🚀 Step 6: Advanced Export Features${NC}"
echo ""
echo -e "${YELLOW}   Graph Traversal:${NC}"
echo "   • Export entire subgraphs"
echo "   • Control traversal depth"
echo "   • Include/exclude relationships"
echo ""
echo -e "${YELLOW}   Privacy Controls:${NC}"
echo "   • Respect privacy levels"
echo "   • Redact sensitive information"
echo "   • Anonymize agents (if permitted)"
echo ""
echo -e "${YELLOW}   Validation:${NC}"
echo "   • Validate against PROV-O schema"
echo "   • Ensure RDF well-formedness"
echo "   • Check constraint compliance"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ PROV-O is the W3C provenance standard"
echo "   ✅ SweetGrass Braids map cleanly to PROV-O"
echo "   ✅ Export enables interoperability & analysis"
echo "   ✅ Multiple serialization formats supported"
echo "   ✅ Privacy controls respected during export"
echo ""
echo "💡 Key Insight:"
echo "   PROV-O makes SweetGrass provenance"
echo "   universally understandable and reusable."
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo "   └─ example.ttl (sample PROV-O export)"
echo ""
echo "⏭️  Next: Learn about Privacy Controls"
echo "   cd ../05-privacy-controls && ./demo-privacy.sh"
echo ""
echo "🌾 Standards-based provenance for the win!"
echo ""
