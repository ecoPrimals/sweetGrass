# SweetGrass — Specifications Index

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## Overview

SweetGrass is the **semantic provenance and attribution layer** of the ecoPrimals ecosystem. It grows from both RhizoCrypt (the living fungal network) and LoamSpine (the permanent geological record), making their activity visible and queryable.

```
           ☀️ VISIBLE WORLD (Applications, gAIa, sunCloud)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       🌾 SWEETGRASS — Semantic layer above ground
           Braids, Attribution, Provenance Graphs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                      SOIL LINE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
       🍄 RHIZOCRYPT — Active fungal network (ephemeral)
───────────────────────────────────────────────────────────
       🦴 LOAMSPINE — Deep geological record (permanent)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Document Map

```
sweetGrass/specs/
├── 00_SPECIFICATIONS_INDEX.md     ← You are here
├── SWEETGRASS_SPECIFICATION.md    ← Master specification
├── ARCHITECTURE.md                ← System architecture
├── DATA_MODEL.md                  ← Braid & Entity structures
├── BRAID_COMPRESSION.md           ← 0/1/Many model, summaries
├── NICHE_PATTERNS.md              ← Configurable semantic patterns
├── ATTRIBUTION_GRAPH.md           ← Provenance for sunCloud
├── API_SPECIFICATION.md           ← gRPC & REST APIs
└── INTEGRATION_SPECIFICATION.md   ← Primal integrations
```

---

## Reading Order

### 1. Conceptual Foundation
| Document | Purpose |
|----------|---------|
| [SWEETGRASS_SPECIFICATION.md](./SWEETGRASS_SPECIFICATION.md) | Master spec: principles, data model, full API |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System components and data flow |

### 2. Core Concepts
| Document | Purpose |
|----------|---------|
| [DATA_MODEL.md](./DATA_MODEL.md) | Braid, Activity, Agent, Entity structures |
| [BRAID_COMPRESSION.md](./BRAID_COMPRESSION.md) | How DAGs compress to Braids (0/1/many) |

### 3. Ecosystem Integration
| Document | Purpose |
|----------|---------|
| [NICHE_PATTERNS.md](./NICHE_PATTERNS.md) | How SweetGrass configures for biomeOS niches |
| [ATTRIBUTION_GRAPH.md](./ATTRIBUTION_GRAPH.md) | Provenance graphs for sunCloud attribution |
| [INTEGRATION_SPECIFICATION.md](./INTEGRATION_SPECIFICATION.md) | RhizoCrypt, LoamSpine, BearDog integrations |

### 4. Implementation
| Document | Purpose |
|----------|---------|
| [API_SPECIFICATION.md](./API_SPECIFICATION.md) | gRPC protobuf, REST OpenAPI, GraphQL |

---

## Quick Reference

### What SweetGrass Does

| Function | Description |
|----------|-------------|
| **Provenance** | Tracks what created data, who contributed, where it came from |
| **Attribution** | Calculates contributor shares for sunCloud rewards |
| **Semantic Linking** | Connects data across RhizoCrypt sessions and LoamSpine spines |
| **Query Engine** | GraphQL and SPARQL for provenance graph traversal |

### Core Data Structures

| Structure | Purpose |
|-----------|---------|
| **Braid** | Provenance record following W3C PROV-O |
| **Activity** | Process that creates or transforms data |
| **Agent** | Person, software, or organization that acts |
| **Entity** | Data artifact with provenance |

### Braid Cardinality

| Count | Meaning |
|-------|---------|
| **0** | Session explored but discarded |
| **1** | Single coherent record (hardest case) |
| **Many** | Summary hierarchies, braids of braids |

### Standards

| Standard | Usage |
|----------|-------|
| **W3C PROV-O** | Provenance ontology (Entity, Activity, Agent) |
| **JSON-LD** | Linked data serialization |
| **DIDs** | Decentralized identifiers (via BearDog) |
| **Schema.org** | Common vocabulary terms |

---

## Key Concepts

### The Fungal Leather Model

SweetGrass uses a biological metaphor for how provenance is created:

1. **Growth** (RhizoCrypt): DAG exploration, full dimensionality, many branches
2. **Dehydration**: Compress to linear summary (Braid)
3. **Aggregation**: Summaries of summaries, meta-braids

This matches the fungal leather process: grow the mycelium, then dry and compress into fewer dimensions.

### Primals as Infrastructure Legos

SweetGrass is not a fixed architecture but a **configurable semantic capability**. Its behavior depends on how it's organized with other primals in a biomeOS niche:

- **Distributed Science niche**: Deep attribution chains, permanent Braids
- **Gaming niche**: Item provenance, lightweight activity tracking
- **Real-time niche**: Streaming provenance, ephemeral Braids

### Radiating Attribution

When value is created at higher levels (Community → gAIa), SweetGrass provides the provenance graph that sunCloud walks to distribute rewards back down to contributors.

---

## Dependencies

### Required Primals

| Primal | Dependency Type |
|--------|-----------------|
| **BearDog** | Required: DID resolution, signing |
| **LoamSpine** | Required: Permanent Braid anchoring |
| **RhizoCrypt** | Required: Session activity source |

### Optional Primals

| Primal | Integration |
|--------|-------------|
| **ToadStool** | Activity events from compute tasks |
| **NestGate** | Large payload references |
| **Songbird** | Service discovery |
| **Squirrel** | AI agent provenance |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.2.0 | Dec 2025 | Added compression model, niche patterns |
| 0.1.0 | Dec 2025 | Initial specification |

---

*SweetGrass: Weaving the stories that give data its meaning.*

