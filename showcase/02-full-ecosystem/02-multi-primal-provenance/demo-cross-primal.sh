#!/usr/bin/env bash
#
# 🌾 Multi-Primal Provenance Demo
#
# This demo shows provenance tracking across primal boundaries.
#

set -euo pipefail

echo ""
echo "🌾 Multi-Primal Provenance Demo"
echo "==============================="
echo ""

echo "📝 Cross-Primal Provenance Overview"
echo ""
echo "When data moves between primals, SweetGrass"
echo "maintains a unified provenance record."
echo ""

echo "Example Flow:"
echo ""
echo "  ┌────────────┐"
echo "  │  Squirrel  │  AI processing"
echo "  │    (AI)    │  → creates output"
echo "  └─────┬──────┘"
echo "        │ SweetGrass records"
echo "        ↓"
echo "  ┌────────────┐"
echo "  │ ToadStool  │  Heavy compute"
echo "  │ (Compute)  │  → derived result"
echo "  └─────┬──────┘"
echo "        │ SweetGrass records"
echo "        ↓"
echo "  ┌────────────┐"
echo "  │  NestGate  │  Persistent storage"
echo "  │ (Storage)  │  → stored data"
echo "  └─────┬──────┘"
echo "        │ SweetGrass records"
echo "        ↓"
echo "  ┌────────────┐"
echo "  │ Songbird   │  Coordinates mesh"
echo "  │  (Mesh)    │  → activity record"
echo "  └────────────┘"
echo ""

echo "Unified Query:"
echo "  provenance_graph(final_output)"
echo ""
echo "  → Returns Braids from ALL primals"
echo "  → Attribution calculated across boundaries"
echo "  → PROV-O export includes all steps"
echo ""

echo "Example Attribution:"
echo "  Squirrel AI:    35%"
echo "  ToadStool GPU:  25%"
echo "  Original Data:  40%"
echo ""

echo "🌾 Multi-Primal Provenance Demo Complete!"
echo ""
echo "Key Takeaways:"
echo "  - SweetGrass is the unified provenance layer"
echo "  - Provenance spans all primals"
echo "  - Attribution works across boundaries"
echo ""

