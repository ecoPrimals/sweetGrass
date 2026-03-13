# 03: Reward Distribution (Coming in Phase 4)

**Status**: Planned for v0.9.0 (Q3 2026)  
**Integration**: Requires sunCloud implementation

---

## Overview

This showcase will demonstrate how SweetGrass attribution enables fair reward distribution through sunCloud.

---

## Planned Demos

### 1. **Attribution-Based Payments**
Calculate contributor shares and distribute payments proportionally.

### 2. **Multi-Hop Derivation Rewards**
Track attribution through derivation chains (dataset → model → application).

### 3. **Real-Time Payment Updates**
Show how new uses of data trigger automatic reward calculations.

### 4. **Historical Attribution Queries**
Query historical contribution data for audit and reporting.

### 5. **Multi-Currency Support**
Demonstrate payment distribution in various currencies/tokens.

---

## Why Not Implemented Yet?

This showcase depends on:
- ✅ SweetGrass attribution engine (complete)
- ❌ sunCloud payment infrastructure (Phase 4)
- ❌ sunCloud API integration (Phase 4)
- ❌ Payment flow implementation (Phase 4)

**Current state**: Attribution calculation works perfectly (see `../../00-local-primal/02-attribution-basics/`). Payment distribution awaits sunCloud Phase 4 implementation.

---

## What You Can Do Now

### Test Attribution (Without Payments)

```bash
cd ../../00-local-primal/02-attribution-basics
./demo-fair-credit.sh
```

This shows how attribution percentages are calculated - the same calculations sunCloud will use for reward distribution.

### See Multi-Primal Coordination

```bash
cd ../../01-primal-coordination
```

These demos show how SweetGrass integrates with other primals, preparing for sunCloud integration.

---

## Roadmap

| Milestone | Target | Status |
|-----------|--------|--------|
| Attribution engine | v0.4.0 | ✅ Complete |
| sunCloud Phase 1 | v0.9.0 | Planned |
| Payment API | v0.9.0 | Planned |
| Reward distribution showcase | v0.9.0 | Planned |

---

## Technical Design (Preview)

When implemented, reward distribution will work like this:

```rust
// 1. Query attribution for a Braid
let attribution = sweetgrass.calculate_attribution(&braid_id).await?;

// 2. sunCloud receives usage event
let usage_event = UsageEvent {
    braid_id,
    value: 100.0, // $100 payment
};

// 3. Distribute rewards proportionally
for contributor in attribution.attributions {
    let payment = usage_event.value * contributor.share;
    suncloud.pay(contributor.agent.did, payment).await?;
}

// Result:
//   Alice (DataProvider, 29%) → $29
//   Bob (Transformer, 21%)     → $21
//   Carol (Contributor, 36%)   → $36
//   Dave (Curator, 14%)        → $14
```

---

## Contact

Questions about reward distribution?
- See: `../../specs/ATTRIBUTION_GRAPH.md`
- Track: Phase 4 roadmap in `../../ROADMAP.md`

---

*This directory is intentionally minimal until sunCloud Phase 4.*  
*Attribution engine is production-ready and waiting for payment infrastructure.*

