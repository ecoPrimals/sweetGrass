# 🌾 Demo: PROV-O Export

**Goal**: Export to W3C standard JSON-LD format  
**Time**: 5 minutes  
**Complexity**: Beginner

---

## 🎯 What This Demo Shows

1. Convert Braids to PROV-O entities
2. Export activities as prov:Activity
3. Export agents as prov:Agent
4. Generate standard JSON-LD

---

## 🚀 Run the Demo

```bash
./demo-export.sh
```

---

## 📖 Concepts

### What is PROV-O?

PROV-O (Provenance Ontology) is a W3C standard for representing provenance information:

- **Entity** - Things (data, documents, artifacts)
- **Activity** - How things were created/modified
- **Agent** - Who was responsible

### SweetGrass Mappings

| SweetGrass | PROV-O |
|------------|--------|
| `Braid` | `prov:Entity` |
| `Activity` | `prov:Activity` |
| `Did` (Agent) | `prov:Agent` |
| `was_derived_from` | `prov:wasDerivedFrom` |
| `was_generated_by` | `prov:wasGeneratedBy` |
| `was_attributed_to` | `prov:wasAttributedTo` |

### JSON-LD Format

```json
{
  "@context": {
    "prov": "http://www.w3.org/ns/prov#",
    "xsd": "http://www.w3.org/2001/XMLSchema#"
  },
  "@graph": [
    {
      "@id": "entity:braid-abc123",
      "@type": "prov:Entity",
      "prov:wasGeneratedBy": "activity:process-456"
    }
  ]
}
```

---

## 📊 Expected Output

```
🌾 SweetGrass PROV-O Export Demo
================================

Exporting Braid to PROV-O...

PROV-O JSON-LD:
{
  "@context": {
    "prov": "http://www.w3.org/ns/prov#",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "ecop": "https://ecoprimals.org/ns#"
  },
  "@graph": [
    {
      "@id": "entity:urn:braid:abc123",
      "@type": "prov:Entity",
      "prov:wasGeneratedBy": "activity:computation-456",
      "prov:wasAttributedTo": "agent:did:key:z6MkCharlie",
      "prov:wasDerivedFrom": [
        "entity:urn:braid:dataset-001",
        "entity:urn:braid:code-002"
      ]
    }
  ]
}

✅ Export complete!
```

---

## 🔧 Code Walkthrough

### Exporting a Braid

```rust
use sweet_grass_query::QueryEngine;

let query_engine = QueryEngine::new(store);

let provo = query_engine
    .export_braid_provo(&braid.data_hash)
    .await?;

let json = serde_json::to_string_pretty(&provo)?;
println!("{}", json);
```

### Exporting Multiple Braids

```rust
let graph = query_engine
    .provenance_graph(hash, Some(5))
    .await?;

let provo = query_engine
    .export_graph_provo(&graph)
    .await?;
```

---

## 💡 Key Insights

### Interoperability
PROV-O is a W3C standard. Your exports work with any PROV-compatible system.

### Linked Data
JSON-LD enables linking across systems via URIs.

### Complete History
Export includes the full provenance chain, not just the immediate Braid.

---

## 🎯 Success Criteria

- [ ] Exported a Braid to PROV-O
- [ ] Understood the JSON-LD format
- [ ] Saw PROV-O property mappings

---

## 📚 Next Steps

Continue to: `../05-privacy-controls/`

Learn about GDPR-style data subject rights!

