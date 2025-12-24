# 🌾 Demo: Attribution Engine

**Goal**: Learn how attribution flows through derivation chains  
**Time**: 10 minutes  
**Complexity**: Beginner

---

## 🎯 What This Demo Shows

1. Role-based attribution weights
2. Derivation chain propagation
3. Multi-contributor attribution
4. Reward calculation

---

## 🚀 Run the Demo

```bash
./demo-attribution.sh
```

---

## 📖 Concepts

### Role Weights

Different roles have different attribution weights:

| Role | Default Weight | Description |
|------|----------------|-------------|
| `Creator` | 1.0 | Original creator |
| `Contributor` | 0.5 | Added to existing work |
| `DataProvider` | 0.4 | Supplied data |
| `Transformer` | 0.3 | Modified/transformed |
| `Curator` | 0.2 | Organized/selected |
| `Orchestrator` | 0.15 | Coordinated work |
| `Publisher` | 0.1 | Published/distributed |
| `Reviewer` | 0.1 | Reviewed/validated |

### Attribution Chain

When Bob derives from Alice's work:
- Alice gets partial credit in Bob's derivative
- Credit flows proportionally through the chain

```
Alice: Creates Dataset (100%)
    ↓ Bob derives (adds 30%)
Bob's Version: Alice 70%, Bob 30%
    ↓ Charlie derives (adds 30%)
Charlie's Version: Alice 49%, Bob 21%, Charlie 30%
```

### Time Decay

Optional decay reduces attribution over time:
- Recent contributions weighted higher
- Configurable decay rate

---

## 📊 Expected Output

```
🌾 SweetGrass Attribution Engine Demo
=====================================

Creating attribution chain...

Step 1: Alice creates original data
  Role: Creator (weight: 1.0)
  Attribution: Alice 100%

Step 2: Bob processes data (derives from Alice)
  Role: Contributor (weight: 0.5)
  Attribution:
    Alice: 70% (propagated from source)
    Bob: 30% (new contribution)

Step 3: Charlie creates visualization (derives from Bob)
  Role: Creator (weight: 1.0)
  Attribution:
    Alice: 49% (70% × 70%)
    Bob: 21% (30% × 70%)
    Charlie: 30% (new contribution)

Reward Distribution ($1000 total):
  Alice: $490.00
  Bob: $210.00
  Charlie: $300.00

✅ Attribution calculated fairly!
```

---

## 🔧 Code Walkthrough

### Creating an Attribution Calculator

```rust
use sweet_grass_factory::AttributionCalculator;

let calculator = AttributionCalculator::default();
```

### Calculating Attribution

```rust
let attribution = query_engine
    .attribution_chain(&result_braid.data_hash)
    .await?;

for contributor in &attribution.contributors {
    println!("{}: {:.1}%", contributor.agent, contributor.share * 100.0);
}
```

### Customizing Weights

```rust
use sweet_grass_factory::AttributionConfig;

let config = AttributionConfig::builder()
    .creator_weight(1.0)
    .contributor_weight(0.5)
    .decay_rate(0.1)
    .build();

let calculator = AttributionCalculator::with_config(config);
```

---

## 💡 Key Insights

### Attribution Is Automatic
When you create a derived Braid with `was_derived_from`, SweetGrass automatically tracks the attribution chain.

### Shares Always Sum to 100%
Total attribution shares always equal 100%, distributed proportionally among contributors.

### Roles Determine Initial Weight
A Creator gets full credit, while a Reviewer gets less. The weights are configurable.

---

## 🎯 Success Criteria

- [ ] Understood role-based weights
- [ ] Created a derivation chain
- [ ] Calculated attribution shares
- [ ] Computed reward distribution

---

## 📚 Next Steps

Continue to: `../03-provenance-queries/`

Learn how to traverse the provenance graph!

