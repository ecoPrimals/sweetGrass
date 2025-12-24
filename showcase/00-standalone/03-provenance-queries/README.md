# 🌾 Demo: Provenance Queries

**Goal**: Traverse the provenance graph (DAG)  
**Time**: 10 minutes  
**Complexity**: Intermediate

---

## 🎯 What This Demo Shows

1. Build provenance graph from any Braid
2. Query ancestors (sources)
3. Query descendants (derivatives)
4. Filter by activity, agent, time

---

## 🚀 Run the Demo

```bash
./demo-queries.sh
```

---

## 📖 Concepts

### Provenance Graph

A directed acyclic graph (DAG) of Braids:

```
     [Source A]     [Source B]
          \            /
           \          /
            v        v
          [Derived C]
               |
               v
          [Derived D]
```

### Query Types

| Query | Description |
|-------|-------------|
| `provenance_graph(hash)` | Full history for content |
| `derived_from(braid)` | What was used to create this? |
| `by_agent(did)` | All Braids by an agent |
| `activities_for(braid)` | What activities created this? |

### Depth Limiting

Provenance queries support depth limits to prevent traversing extremely long chains:

```rust
let graph = query_engine
    .provenance_graph(hash, Some(5))  // Max depth 5
    .await?;
```

---

## 📊 Expected Output

```
🌾 SweetGrass Provenance Query Demo
===================================

Building provenance graph for result...

Graph Structure:
  Root: sha256:7f83b165...
  Entities: 3
  Depth: 2

Traversing ancestors:
  Level 0: sha256:7f83b165... (Result)
  Level 1: sha256:abc123... (Dataset)
  Level 1: sha256:def456... (Code)

Query: All braids by Alice
  Found: 1 braid
  - urn:braid:alice-001 (Climate Research Dataset)

Query: Activities for result
  - Computation at 2025-12-23T12:00:00Z

✅ Provenance queries complete!
```

---

## 🔧 Code Walkthrough

### Building a Provenance Graph

```rust
use sweet_grass_query::QueryEngine;

let query_engine = QueryEngine::new(store);

let graph = query_engine
    .provenance_graph(
        EntityReference::by_hash(&braid.data_hash),
        Some(5),  // Max depth
    )
    .await?;

println!("Entities: {}", graph.entities.len());
println!("Depth: {}", graph.depth);
```

### Querying by Agent

```rust
let alice = Did::new("did:key:z6MkAlice");
let braids = store.by_agent(&alice).await?;

for braid in braids {
    println!("Found: {}", braid.id);
}
```

### Querying Derivations

```rust
let sources = store.derived_from(&braid.data_hash).await?;

for source in sources {
    println!("Derived from: {}", source.id);
}
```

---

## 💡 Key Insights

### DAG Not Tree
Braids can have multiple sources (many-to-one derivation).

### Depth Matters
Deep chains can be expensive. Use depth limits.

### Cycle Detection
The engine detects and handles cycles gracefully.

---

## 🎯 Success Criteria

- [ ] Built a provenance graph
- [ ] Traversed ancestors
- [ ] Queried by agent
- [ ] Understood depth limiting

---

## 📚 Next Steps

Continue to: `../04-provo-export/`

Learn how to export to W3C PROV-O standard!

