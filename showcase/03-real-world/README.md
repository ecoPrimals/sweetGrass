# 🌍 Real-World Scenarios

**Purpose**: Demonstrate concrete value of SweetGrass in production use cases

---

## Available Scenarios

### 1. ✅ ML Training Attribution

**Directory**: `01-ml-training-attribution/`  
**Time**: 15 minutes  
**Status**: Complete

**Scenario**: A medical AI model trained using datasets from 5 contributors. SweetGrass ensures fair credit and payment when the model is used.

**Run**:
```bash
cd 01-ml-training-attribution
./demo-fair-ml-credit.sh
```

**Learn**:
- Multi-step ML pipeline provenance
- Derivation chain attribution
- Fair compensation calculation
- Real-world AI/ML value

---

### 2. ✅ Open Science

**Directory**: `02-open-science/`  
**Time**: 12 minutes  
**Status**: Complete

**Scenario**: Research paper with experimental data. Years later, another team perfectly reproduces results using complete provenance trail.

**Run**:
```bash
cd 02-open-science
./demo-reproducible-research.sh
```

**Learn**:
- Complete research provenance
- Reproducibility with exact parameters
- FAIR principles (Findable, Accessible, Interoperable, Reusable)
- Compliance with funding requirements

---

### 3. ✅ Content Creation Royalties

**Directory**: `03-content-royalties/`  
**Time**: 12 minutes  
**Status**: Complete

**Scenario**: A song goes through composition, production, vocals, remix, and sampling. SweetGrass ensures fair royalties for ALL contributors.

**Run**:
```bash
cd 03-content-royalties
./demo-fair-music-royalties.sh
```

**Learn**:
- Multi-generation music provenance
- Automatic royalty calculation
- Fair compensation across derivation chains
- Music industry integration

---

### 4. ✅ HIPAA Compliance

**Directory**: `04-hipaa-compliance/`  
**Time**: 12 minutes  
**Status**: Complete

**Scenario**: Medical record with complete audit trail. Patient data flows through 5 healthcare roles with full HIPAA compliance.

**Run**:
```bash
cd 04-hipaa-compliance
./demo-audit-trail.sh
```

**Learn**:
- Complete HIPAA-compliant audit trail
- Immutable medical record provenance
- Instant auditor report generation
- Regulatory compliance automation

---

### 5. ✅ Supply Chain

**Directory**: `05-supply-chain/`  
**Time**: 15 minutes  
**Status**: Complete

**Scenario**: Smartphone manufacturing from global supply chain. Defect discovered - SweetGrass enables precise recall saving $40M.

**Run**:
```bash
cd 05-supply-chain
./demo-product-lineage.sh
```

**Learn**:
- Multi-tier supply chain provenance
- Component-level traceability
- Rapid root cause analysis
- Precise recall management (vs. over-recall)

---

## Progress

**Complete**: 5/5 (100%) ✅
- ✅ ML Training Attribution
- ✅ Open Science
- ✅ Content Royalties
- ✅ HIPAA Compliance
- ✅ Supply Chain

---

## Why Real-World Scenarios?

These demos show:
- **Concrete value** - Not abstract concepts
- **Production use cases** - Real problems solved
- **ROI demonstration** - Business justification
- **Integration patterns** - How to use SweetGrass

---

## Run All Scenarios

```bash
for dir in 0*/; do
    if [ -x "$dir/demo-*.sh" ]; then
        cd "$dir"
        ./demo-*.sh
        cd ..
    fi
done
```

---

## Creating New Scenarios

When adding new scenarios:
1. Create numbered directory (`02-scenario-name/`)
2. Write `demo-*.sh` script
3. Include narrative explanation
4. Show measurable impact
5. Update this README

---

🌾 **Real-world value through real-world demos**

