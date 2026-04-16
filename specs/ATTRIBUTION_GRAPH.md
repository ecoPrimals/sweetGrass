# SweetGrass — Attribution Graph Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass provides the **attribution graph** that sunCloud walks to distribute rewards. When value is created at higher levels (Community → gAIa), attribution **radiates back down** to contributors at lower levels.

```
                    gAIa Commons
                         │
                         │ Value created at top
                         ▼
              ┌──────────────────────┐
              │   Discovery/Usage    │
              │   generates value    │
              └──────────┬───────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    Community A     Community B     Community C
         │               │               │
    ┌────┴────┐     ┌────┴────┐     ┌────┴────┐
    │         │     │         │     │         │
    ▼         ▼     ▼         ▼     ▼         ▼
  Person    Person  Person  Person  Person  Person
  
         ← ← ← Attribution radiates back down ← ← ←
```

---

## 2. Attribution Chain

### 2.1 Structure

```rust
/// Attribution chain for a data artifact
#[derive(Clone, Debug)]
pub struct AttributionChain {
    /// Root entity being attributed
    pub entity: EntityReference,
    
    /// Root Braid
    pub root_braid: BraidId,
    
    /// All contributors with their shares
    pub contributors: Vec<ContributorShare>,
    
    /// Total resources consumed
    pub resources: ResourceTotals,
    
    /// Chain depth (generations of inheritance)
    pub depth: u32,
    
    /// Calculation metadata
    pub calculation: CalculationMeta,
}

/// Individual contributor's share
#[derive(Clone, Debug)]
pub struct ContributorShare {
    /// Contributor identity
    pub agent: Did,
    
    /// Role in the contribution
    pub role: AgentRole,
    
    /// Share of attribution (0.0 - 1.0, all sum to 1.0)
    pub share: f64,
    
    /// Direct contribution (vs inherited)
    pub direct: bool,
    
    /// Inheritance depth (0 = direct, 1+ = inherited)
    pub inheritance_depth: u32,
    
    /// Source Braids for this contribution
    pub source_braids: Vec<BraidId>,
    
    /// Resource contributions
    pub resources: ResourceContribution,
}

/// Resources consumed
#[derive(Clone, Debug, Default)]
pub struct ResourceTotals {
    /// Total compute units
    pub compute_units: f64,
    
    /// Total storage bytes
    pub storage_bytes: u64,
    
    /// Total network bytes
    pub network_bytes: u64,
    
    /// Total duration (nanoseconds)
    pub duration_ns: u64,
}

/// Individual resource contribution
#[derive(Clone, Debug, Default)]
pub struct ResourceContribution {
    pub compute_units: Option<f64>,
    pub storage_bytes: Option<u64>,
    pub network_bytes: Option<u64>,
    pub data_bytes: Option<u64>,
}
```

### 2.2 Calculation

```rust
/// Attribution calculator
pub struct AttributionCalculator {
    config: AttributionConfig,
    store: Arc<BraidBackend>,
}

impl AttributionCalculator {
    /// Calculate attribution chain for an entity
    pub async fn calculate(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, AttributionError> {
        // 1. Get root Braid
        let root = self.resolve_entity(&entity).await?;
        
        // 2. Build provenance graph
        let graph = self.build_provenance_graph(&root, self.config.max_depth).await?;
        
        // 3. Calculate contributions
        let mut contributions = HashMap::new();
        self.calculate_contributions(&root, &graph, &mut contributions, 0)?;
        
        // 4. Normalize shares
        let normalized = self.normalize_contributions(contributions);
        
        // 5. Calculate resource totals
        let resources = self.calculate_resources(&graph);
        
        Ok(AttributionChain {
            entity,
            root_braid: root.id.clone(),
            contributors: normalized,
            resources,
            depth: graph.stats.max_depth,
            calculation: CalculationMeta {
                algorithm: "radiating-decay".to_string(),
                config: self.config.clone(),
                timestamp: current_timestamp_nanos(),
            },
        })
    }
    
    /// Recursively calculate contributions
    fn calculate_contributions(
        &self,
        braid: &Braid,
        graph: &ProvenanceGraph,
        contributions: &mut HashMap<Did, ContributionAccumulator>,
        depth: u32,
    ) -> Result<(), AttributionError> {
        // Direct creator
        let creator = &braid.was_attributed_to;
        contributions
            .entry(creator.clone())
            .or_default()
            .add_direct(1.0, AgentRole::Creator, &braid.id);
        
        // Activity participants
        if let Some(activity) = &braid.was_generated_by {
            for assoc in &activity.was_associated_with {
                let weight = self.role_weight(&assoc.role);
                contributions
                    .entry(assoc.agent.clone())
                    .or_default()
                    .add_activity(weight, assoc.role.clone(), &activity.ecop);
            }
        }
        
        // Inherited contributions from sources
        for source_ref in &braid.was_derived_from {
            if let Some(source_id) = self.resolve_braid_id(source_ref) {
                if let Some(source) = graph.braids.get(&source_id) {
                    // Apply inheritance decay
                    let decay = self.config.inheritance_decay.powi(depth as i32 + 1);
                    
                    // Recursively calculate source contributions
                    self.calculate_contributions(source, graph, contributions, depth + 1)?;
                    
                    // Apply decay to inherited contributions
                    for (agent, acc) in contributions.iter_mut() {
                        if acc.inheritance_depth > depth {
                            acc.apply_decay(decay);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Weight multipliers for different roles
    fn role_weight(&self, role: &AgentRole) -> f64 {
        match role {
            AgentRole::Creator => 1.0,
            AgentRole::Contributor => 0.5,
            AgentRole::ComputeProvider => 0.3,
            AgentRole::StorageProvider => 0.2,
            AgentRole::DataProvider => 0.4,
            AgentRole::Validator => 0.1,
            AgentRole::Publisher => 0.1,
            AgentRole::Orchestrator => 0.15,
            AgentRole::Custom(_) => 0.2,
        }
    }
    
    /// Normalize contributions to sum to 1.0
    fn normalize_contributions(
        &self,
        contributions: HashMap<Did, ContributionAccumulator>,
    ) -> Vec<ContributorShare> {
        let total: f64 = contributions.values().map(|c| c.total()).sum();
        
        if total == 0.0 {
            return vec![];
        }
        
        contributions
            .into_iter()
            .map(|(agent, acc)| ContributorShare {
                agent,
                role: acc.primary_role(),
                share: acc.total() / total,
                direct: acc.direct_contribution > 0.0,
                inheritance_depth: acc.inheritance_depth,
                source_braids: acc.source_braids,
                resources: acc.resources,
            })
            .collect()
    }
}

/// Accumulator for contribution calculation
#[derive(Default)]
struct ContributionAccumulator {
    direct_contribution: f64,
    activity_contribution: f64,
    inherited_contribution: f64,
    inheritance_depth: u32,
    roles: HashMap<AgentRole, f64>,
    source_braids: Vec<BraidId>,
    resources: ResourceContribution,
}

impl ContributionAccumulator {
    fn add_direct(&mut self, weight: f64, role: AgentRole, braid_id: &BraidId) {
        self.direct_contribution += weight;
        *self.roles.entry(role).or_default() += weight;
        self.source_braids.push(braid_id.clone());
        self.inheritance_depth = 0;
    }
    
    fn add_activity(&mut self, weight: f64, role: AgentRole, ecop: &ActivityEcoPrimals) {
        self.activity_contribution += weight;
        *self.roles.entry(role).or_default() += weight;
        
        if let Some(cu) = ecop.compute_units {
            self.resources.compute_units = Some(
                self.resources.compute_units.unwrap_or(0.0) + cu * weight
            );
        }
    }
    
    fn apply_decay(&mut self, decay: f64) {
        self.inherited_contribution *= decay;
    }
    
    fn total(&self) -> f64 {
        self.direct_contribution + self.activity_contribution + self.inherited_contribution
    }
    
    fn primary_role(&self) -> AgentRole {
        self.roles
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(role, _)| role.clone())
            .unwrap_or(AgentRole::Contributor)
    }
}
```

---

## 3. Configuration

```rust
/// Attribution calculation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttributionConfig {
    /// Maximum depth to traverse provenance graph
    pub max_depth: u32,
    
    /// Decay factor for inherited contributions (0.0 - 1.0)
    /// Each generation inherits this fraction of the previous
    pub inheritance_decay: f64,
    
    /// Role weights (override defaults)
    pub role_weights: HashMap<AgentRole, f64>,
    
    /// Minimum share threshold (below this = 0)
    pub min_share_threshold: f64,
    
    /// Include resource-based attribution
    pub include_resources: bool,
    
    /// Resource weight factors
    pub resource_weights: ResourceWeights,
}

/// Weights for resource-based attribution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceWeights {
    /// Weight for compute contribution
    pub compute: f64,
    
    /// Weight for storage contribution
    pub storage: f64,
    
    /// Weight for data contribution
    pub data: f64,
    
    /// Weight for network contribution
    pub network: f64,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            inheritance_decay: 0.7,
            role_weights: HashMap::new(),
            min_share_threshold: 0.001, // 0.1%
            include_resources: true,
            resource_weights: ResourceWeights {
                compute: 0.3,
                storage: 0.2,
                data: 0.4,
                network: 0.1,
            },
        }
    }
}
```

---

## 4. sunCloud Interface

SweetGrass provides this interface for sunCloud to query attribution:

```rust
/// sunCloud interface for attribution
// Native async fn in trait (Rust 2024 RPITIT)
pub trait SunCloudInterface: Send + Sync {
    /// Get attribution chain for an entity
    async fn get_attribution(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, AttributionError>;
    
    /// Get attribution with custom configuration
    async fn get_attribution_with_config(
        &self,
        entity: EntityReference,
        config: AttributionConfig,
    ) -> Result<AttributionChain, AttributionError>;
    
    /// Get all contributions by an agent
    async fn get_agent_contributions(
        &self,
        agent: &Did,
        time_range: Option<TimeRange>,
    ) -> Result<AgentContributions, AttributionError>;
    
    /// Record a reward distribution (for auditing)
    async fn record_distribution(
        &self,
        entity: EntityReference,
        distributions: Vec<RewardDistribution>,
    ) -> Result<DistributionReceipt, AttributionError>;
    
    /// Verify attribution calculation
    async fn verify_attribution(
        &self,
        chain: &AttributionChain,
    ) -> Result<VerificationResult, AttributionError>;
}

/// Agent's total contributions
#[derive(Clone, Debug)]
pub struct AgentContributions {
    pub agent: Did,
    pub total_contributions: u64,
    pub total_share_value: f64,
    pub by_role: HashMap<AgentRole, RoleContributions>,
    pub by_entity: Vec<EntityContribution>,
    pub resources_provided: ResourceTotals,
}

/// Contribution by role
#[derive(Clone, Debug)]
pub struct RoleContributions {
    pub role: AgentRole,
    pub count: u64,
    pub total_share: f64,
}

/// Contribution to specific entity
#[derive(Clone, Debug)]
pub struct EntityContribution {
    pub entity: EntityReference,
    pub braid_id: BraidId,
    pub share: f64,
    pub role: AgentRole,
    pub timestamp: Timestamp,
}

/// Reward distribution record
#[derive(Clone, Debug)]
pub struct RewardDistribution {
    pub agent: Did,
    pub amount: f64,
    pub currency: String,
    pub share: f64,
    pub role: AgentRole,
    pub reason: String,
}

/// Receipt for recorded distribution
#[derive(Clone, Debug)]
pub struct DistributionReceipt {
    pub entity: EntityReference,
    pub distributions: Vec<RewardDistribution>,
    pub total_distributed: f64,
    pub timestamp: Timestamp,
    pub braid_id: BraidId, // Braid recording the distribution
}
```

---

## 5. Radiating Attribution

### 5.1 The Model

Attribution flows **downward** through the hierarchy:

```
Level 0: gAIa Commons (root)
    │
    │  Entity used/discovered at gAIa level
    │  Value generated = V
    │
    ▼
Level 1: Community Spine
    │
    │  Community contributed this entity
    │  Community share = S₁
    │
    │  ┌─────────────────────────┐
    │  │ Member shares within    │
    │  │ community determined by │
    │  │ internal attribution    │
    │  └─────────────────────────┘
    │
    ▼
Level 2: Personal Spines
    │
    │  Individuals contributed to community
    │  Individual share = S₁ × decay × personal_share
    │
    ▼
Level 3: Source Data
    
    Original data providers get inherited share
    Source share = S₁ × decay² × source_share
```

### 5.2 Implementation

```rust
/// Radiating attribution calculator
pub struct RadiatingAttribution {
    calculator: Arc<AttributionCalculator>,
    spine_hierarchy: Arc<dyn SpineHierarchy>,
}

impl RadiatingAttribution {
    /// Calculate radiating attribution from a top-level entity
    pub async fn radiate(
        &self,
        entity: EntityReference,
        value: f64,
    ) -> Result<RadiationResult, AttributionError> {
        // 1. Get basic attribution chain
        let chain = self.calculator.calculate(entity.clone()).await?;
        
        // 2. Group by spine level
        let by_level = self.group_by_spine_level(&chain)?;
        
        // 3. Calculate shares at each level
        let mut distributions = Vec::new();
        let mut remaining = value;
        
        for (level, contributors) in by_level.iter().enumerate() {
            let level_share = self.level_share(level, value);
            
            for contributor in contributors {
                let amount = level_share * contributor.share;
                if amount >= self.calculator.config.min_share_threshold * value {
                    distributions.push(RadiatedShare {
                        agent: contributor.agent.clone(),
                        amount,
                        level: level as u32,
                        source_share: contributor.share,
                    });
                    remaining -= amount;
                }
            }
        }
        
        // 4. Remainder goes to gAIa commons
        if remaining > 0.0 {
            distributions.push(RadiatedShare {
                agent: self.gaia_did(),
                amount: remaining,
                level: 0,
                source_share: remaining / value,
            });
        }
        
        Ok(RadiationResult {
            entity,
            total_value: value,
            distributions,
        })
    }
    
    /// Calculate share for each level
    fn level_share(&self, level: usize, total: f64) -> f64 {
        // Level 0 (gAIa): 10%
        // Level 1 (Community): 30%
        // Level 2 (Personal): 40%
        // Level 3+ (Sources): 20% (decaying)
        match level {
            0 => total * 0.10,
            1 => total * 0.30,
            2 => total * 0.40,
            _ => total * 0.20 * (0.7_f64).powi(level as i32 - 3),
        }
    }
}

/// Result of radiating attribution
#[derive(Clone, Debug)]
pub struct RadiationResult {
    pub entity: EntityReference,
    pub total_value: f64,
    pub distributions: Vec<RadiatedShare>,
}

/// Individual share in radiation
#[derive(Clone, Debug)]
pub struct RadiatedShare {
    pub agent: Did,
    pub amount: f64,
    pub level: u32,
    pub source_share: f64,
}
```

---

## 6. Query API

```rust
/// Attribution query interface
// Native async fn in trait (Rust 2024 RPITIT)
pub trait AttributionQuery: Send + Sync {
    /// Get attribution for entity
    async fn attribution(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, QueryError>;
    
    /// Get attribution with depth limit
    async fn attribution_depth(
        &self,
        entity: EntityReference,
        max_depth: u32,
    ) -> Result<AttributionChain, QueryError>;
    
    /// Get top contributors for entity
    async fn top_contributors(
        &self,
        entity: EntityReference,
        limit: usize,
    ) -> Result<Vec<ContributorShare>, QueryError>;
    
    /// Get contribution history for agent
    async fn agent_history(
        &self,
        agent: &Did,
        pagination: Pagination,
    ) -> Result<Page<EntityContribution>, QueryError>;
    
    /// Search by contributor
    async fn search_by_contributor(
        &self,
        agent: &Did,
        filter: ContributionFilter,
    ) -> Result<Vec<EntityReference>, QueryError>;
}
```

### 6.1 GraphQL Schema

```graphql
type Query {
  # Get attribution chain for entity
  attribution(entity: ID!, maxDepth: Int): AttributionChain!
  
  # Get top contributors
  topContributors(entity: ID!, limit: Int): [ContributorShare!]!
  
  # Get agent's contributions
  agentContributions(agent: ID!, timeRange: TimeRange): AgentContributions!
}

type AttributionChain {
  entity: Entity!
  rootBraid: Braid!
  contributors: [ContributorShare!]!
  resources: ResourceTotals!
  depth: Int!
}

type ContributorShare {
  agent: Agent!
  role: AgentRole!
  share: Float!
  direct: Boolean!
  inheritanceDepth: Int!
  sourceBraids: [Braid!]!
  resources: ResourceContribution
}

type AgentContributions {
  agent: Agent!
  totalContributions: Int!
  totalShareValue: Float!
  byRole: [RoleContributions!]!
  byEntity: [EntityContribution!]!
  resourcesProvided: ResourceTotals!
}

type ResourceTotals {
  computeUnits: Float!
  storageBytes: Int!
  networkBytes: Int!
  durationNs: Int!
}
```

---

## 7. Verification

Attribution calculations can be verified:

```rust
/// Attribution verification
pub struct AttributionVerifier {
    store: Arc<BraidBackend>,
}

impl AttributionVerifier {
    /// Verify an attribution chain
    pub async fn verify(
        &self,
        chain: &AttributionChain,
    ) -> Result<VerificationResult, VerificationError> {
        let mut issues = Vec::new();
        
        // 1. Verify root Braid exists and matches
        let root = self.store.get(&chain.root_braid).await?
            .ok_or(VerificationError::RootNotFound)?;
        
        // 2. Verify shares sum to 1.0
        let total: f64 = chain.contributors.iter().map(|c| c.share).sum();
        if (total - 1.0).abs() > 0.001 {
            issues.push(VerificationIssue::SharesNotNormalized { total });
        }
        
        // 3. Verify each contributor appears in provenance graph
        for contributor in &chain.contributors {
            let found = self.verify_contributor_in_graph(&root, &contributor.agent).await?;
            if !found {
                issues.push(VerificationIssue::ContributorNotInGraph {
                    agent: contributor.agent.clone(),
                });
            }
        }
        
        // 4. Verify source Braids exist
        for contributor in &chain.contributors {
            for braid_id in &contributor.source_braids {
                if self.store.get(braid_id).await?.is_none() {
                    issues.push(VerificationIssue::SourceBraidNotFound {
                        braid_id: braid_id.clone(),
                    });
                }
            }
        }
        
        Ok(VerificationResult {
            valid: issues.is_empty(),
            issues,
            verified_at: current_timestamp_nanos(),
        })
    }
}

/// Verification result
#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub valid: bool,
    pub issues: Vec<VerificationIssue>,
    pub verified_at: Timestamp,
}

/// Verification issues
#[derive(Clone, Debug)]
pub enum VerificationIssue {
    SharesNotNormalized { total: f64 },
    ContributorNotInGraph { agent: Did },
    SourceBraidNotFound { braid_id: BraidId },
    SignatureInvalid { braid_id: BraidId },
    DepthExceeded { expected: u32, actual: u32 },
}
```

---

## 8. References

- [SWEETGRASS_SPECIFICATION.md](./SWEETGRASS_SPECIFICATION.md) — Master specification
- [DATA_MODEL.md](./DATA_MODEL.md) — Braid structure
- [sunCloud Economics](../../whitePaper/economics/) — Economic model
- [LoamSpine Spines](../../loamSpine/specs/DATA_MODEL.md) — Spine hierarchy

---

*SweetGrass: Tracing value back to its sources.*

