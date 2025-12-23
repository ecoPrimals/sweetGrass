# SweetGrass — Braid Compression Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass compresses high-dimensional DAG activity into lower-dimensional Braids. This process mirrors **fungal leather production**: grow the mycelium (DAG exploration), then dry and compress (dehydration to linear provenance).

```
     EXPLORATION (DAG - Full Dimensionality)
     ┌─────────────────────────────────────┐
     │  ╭──○──╮     ╭──○──╮               │
     │  │     │     │     │               │
     │  ○──○──○──○──○──○──○──○──○         │  Living mycelium
     │     │     │     │     │            │  Full branching
     │     ○     ○     ○     ○            │  Many paths explored
     └─────────────────────────────────────┘
                    │
                    │ DEHYDRATION (compress)
                    ▼
     ┌─────────────────────────────────────┐
     │  ════════════════════════════════   │  Dried & compressed
     │  Linear summary (Braid)             │  Fewer dimensions
     │  ════════════════════════════════   │  Coherent record
     └─────────────────────────────────────┘
                    │
                    │ AGGREGATION (meta-compression)
                    ▼
     ┌─────────────────────────────────────┐
     │  ══════════                         │  Meta-braids
     │  Summary of summaries               │  Higher abstractions
     └─────────────────────────────────────┘
```

---

## 2. The 0/1/Many Model

When a RhizoCrypt session resolves, it can produce **zero, one, or many** Braids:

### 2.1 Zero Braids

Session explored but produced nothing worth recording:

```rust
/// Reasons for zero Braids
pub enum DiscardReason {
    /// Session explicitly rolled back
    Rollback,
    
    /// Session had no vertices
    EmptySession,
    
    /// All branches were exploratory (no commits)
    ExploratoryOnly,
    
    /// Content below significance threshold
    BelowThreshold,
    
    /// Duplicate of existing provenance
    Duplicate,
}
```

**Examples:**
- User started editing, then cancelled
- AI explored options, none were selected
- Session was a read-only query
- All changes were local/temporary

### 2.2 One Braid

Single coherent record—**the hardest case**:

```
Session Activity                          Single Braid
┌─────────────────────┐                  ┌─────────────┐
│ ○──○──○             │                  │             │
│    │                │  ──────────────► │  Coherent   │
│    ○──○──○──○       │                  │  Summary    │
│       │             │                  │             │
│       ○             │                  └─────────────┘
└─────────────────────┘
```

This is the "war correspondent" problem: capturing ongoing complexity as a single unified truth without it becoming a "bad summary."

**Strategy: Don't force 1 prematurely.** Allow the DAG to remain until natural resolution, then compress. The pressure to create "1" too early produces bad summaries.

```rust
/// When to create a single Braid
pub fn should_be_single(session: &Session) -> bool {
    // All branches converge to single output
    session.has_single_outcome()
    
    // OR session explicitly marked as atomic
    || session.is_atomic()
    
    // OR session below split threshold
    || session.vertex_count() < SPLIT_THRESHOLD
    
    // OR user/system explicitly requests single
    || session.compression_hint() == CompressionHint::Single
}
```

### 2.3 Many Braids

Summary hierarchies—braids of braids:

```
Session Activity                          Multiple Braids
┌─────────────────────┐                  ┌─────────────┐
│ ○──○──○──○──○       │                  │  Braid A    │
│    │     │          │                  │  (branch 1) │
│    │     ○──○──○    │  ──────────────► ├─────────────┤
│    │                │                  │  Braid B    │
│    ○──○──○──○──○    │                  │  (branch 2) │
│          │          │                  ├─────────────┤
│          ○──○       │                  │  Braid C    │
└─────────────────────┘                  │  (branch 3) │
                                         ├─────────────┤
                                         │  Meta-Braid │
                                         │  (summary)  │
                                         └─────────────┘
```

**When to split:**
- Multiple independent outcomes
- Branches diverge significantly
- Different contributors on different branches
- Content exceeds coherence threshold

---

## 3. Compression Engine

### 3.1 Core Interface

```rust
/// Compression Engine
pub struct CompressionEngine {
    config: CompressionConfig,
    analyzer: SessionAnalyzer,
    factory: Arc<BraidFactory>,
}

impl CompressionEngine {
    /// Compress a session to Braids
    pub async fn compress(
        &self,
        session: &Session,
        summary: &DehydrationSummary,
    ) -> Result<CompressionResult, CompressionError> {
        // 1. Analyze session structure
        let analysis = self.analyzer.analyze(session)?;
        
        // 2. Determine strategy
        let strategy = self.select_strategy(&analysis);
        
        // 3. Execute compression
        match strategy {
            CompressionStrategy::Discard(reason) => {
                Ok(CompressionResult::None { reason })
            }
            CompressionStrategy::Single => {
                let braid = self.compress_single(session, summary).await?;
                Ok(CompressionResult::Single(braid))
            }
            CompressionStrategy::Split(branches) => {
                let braids = self.compress_branches(session, branches).await?;
                let summary = self.create_meta_braid(&braids).await?;
                Ok(CompressionResult::Multiple { braids, summary: Some(summary) })
            }
            CompressionStrategy::Hierarchical(levels) => {
                let result = self.compress_hierarchical(session, levels).await?;
                Ok(result)
            }
        }
    }
}
```

### 3.2 Session Analysis

```rust
/// Session Analyzer
pub struct SessionAnalyzer {
    config: AnalysisConfig,
}

impl SessionAnalyzer {
    /// Analyze session structure for compression
    pub fn analyze(&self, session: &Session) -> Result<SessionAnalysis, AnalysisError> {
        let graph = session.build_graph();
        
        Ok(SessionAnalysis {
            // Structure metrics
            vertex_count: graph.vertex_count(),
            branch_count: self.count_branches(&graph),
            max_depth: self.find_max_depth(&graph),
            convergence: self.measure_convergence(&graph),
            
            // Content metrics
            unique_outputs: self.find_unique_outputs(&graph),
            contributors: self.find_contributors(&graph),
            activity_types: self.classify_activities(&graph),
            
            // Coherence metrics
            semantic_coherence: self.measure_coherence(&graph),
            temporal_span: self.measure_temporal_span(&graph),
            
            // Outcome
            outcome: session.outcome(),
        })
    }
    
    /// Count distinct branches
    fn count_branches(&self, graph: &SessionGraph) -> usize {
        // Find vertices with multiple children
        graph.vertices()
            .filter(|v| v.children().len() > 1)
            .count()
    }
    
    /// Measure how much branches reconverge
    fn measure_convergence(&self, graph: &SessionGraph) -> f64 {
        let tips = graph.tips().count();
        let branches = self.count_branches(graph);
        
        if branches == 0 {
            1.0 // Fully linear
        } else {
            tips as f64 / branches as f64
        }
    }
    
    /// Measure semantic coherence
    fn measure_coherence(&self, graph: &SessionGraph) -> f64 {
        // Heuristic: more shared ancestry = more coherent
        // Implementation: measure average path overlap
        
        let tips: Vec<_> = graph.tips().collect();
        if tips.len() <= 1 {
            return 1.0;
        }
        
        let mut total_overlap = 0.0;
        let mut comparisons = 0;
        
        for i in 0..tips.len() {
            for j in (i+1)..tips.len() {
                let path_i = graph.ancestors(tips[i]);
                let path_j = graph.ancestors(tips[j]);
                let overlap = self.path_overlap(&path_i, &path_j);
                total_overlap += overlap;
                comparisons += 1;
            }
        }
        
        total_overlap / comparisons as f64
    }
}

/// Session analysis result
#[derive(Clone, Debug)]
pub struct SessionAnalysis {
    pub vertex_count: usize,
    pub branch_count: usize,
    pub max_depth: usize,
    pub convergence: f64,
    pub unique_outputs: Vec<ContentHash>,
    pub contributors: HashSet<Did>,
    pub activity_types: HashMap<ActivityType, usize>,
    pub semantic_coherence: f64,
    pub temporal_span: Duration,
    pub outcome: SessionOutcome,
}
```

### 3.3 Strategy Selection

```rust
/// Compression strategies
pub enum CompressionStrategy {
    /// Produce no Braids
    Discard(DiscardReason),
    
    /// Produce single Braid
    Single,
    
    /// Split into multiple Braids by branch
    Split(Vec<BranchSpec>),
    
    /// Hierarchical compression with meta-levels
    Hierarchical(Vec<CompressionLevel>),
}

impl CompressionEngine {
    /// Select compression strategy based on analysis
    fn select_strategy(&self, analysis: &SessionAnalysis) -> CompressionStrategy {
        // Check for discard conditions
        if analysis.outcome == SessionOutcome::Rollback {
            return CompressionStrategy::Discard(DiscardReason::Rollback);
        }
        
        if analysis.vertex_count == 0 {
            return CompressionStrategy::Discard(DiscardReason::EmptySession);
        }
        
        if analysis.unique_outputs.is_empty() {
            return CompressionStrategy::Discard(DiscardReason::ExploratoryOnly);
        }
        
        // Check for single Braid
        if analysis.vertex_count < self.config.split_threshold
            && analysis.semantic_coherence > self.config.coherence_threshold
            && analysis.unique_outputs.len() == 1
        {
            return CompressionStrategy::Single;
        }
        
        // Check for split
        if analysis.branch_count > 0 && analysis.convergence < 0.5 {
            let branches = self.identify_branches(analysis);
            return CompressionStrategy::Split(branches);
        }
        
        // Default to hierarchical for complex sessions
        if analysis.vertex_count > self.config.hierarchical_threshold {
            let levels = self.plan_hierarchy(analysis);
            return CompressionStrategy::Hierarchical(levels);
        }
        
        // Fall back to single
        CompressionStrategy::Single
    }
}
```

---

## 4. Meta-Braids (Summaries of Summaries)

### 4.1 Meta-Braid Structure

```rust
/// Meta-Braid: summary of other Braids
#[derive(Clone, Debug)]
pub struct MetaBraid {
    /// The meta-Braid itself
    pub braid: Braid,
    
    /// Braids this summarizes
    pub summarizes: Vec<BraidId>,
    
    /// Summary level (1 = first summary, 2 = summary of summaries, etc.)
    pub level: u32,
    
    /// Aggregated statistics
    pub stats: AggregatedStats,
}

/// Aggregated statistics across summarized Braids
#[derive(Clone, Debug, Default)]
pub struct AggregatedStats {
    pub total_braids: u64,
    pub total_activities: u64,
    pub total_compute_units: f64,
    pub total_storage_bytes: u64,
    pub unique_agents: HashSet<Did>,
    pub activity_distribution: HashMap<ActivityType, u64>,
    pub time_range: Option<(Timestamp, Timestamp)>,
}

impl CompressionEngine {
    /// Create meta-Braid from multiple Braids
    pub async fn create_meta_braid(
        &self,
        braids: &[Braid],
    ) -> Result<Braid, CompressionError> {
        // Aggregate statistics
        let stats = self.aggregate_stats(braids);
        
        // Create summary content
        let summary = BraidSummary {
            summarizes: braids.iter().map(|b| b.id.clone()).collect(),
            level: 1,
            stats: stats.clone(),
        };
        
        // Create the meta-Braid
        let braid = self.factory.create(BraidSpec {
            braid_type: BraidType::Collection {
                member_count: braids.len() as u64,
                summary_type: SummaryType::Session {
                    session_id: braids[0].ecop.rhizo_session.clone().unwrap_or_default(),
                },
            },
            data_hash: self.hash_summary(&summary),
            mime_type: "application/vnd.sweetgrass.summary+json".to_string(),
            was_derived_from: braids.iter()
                .map(|b| EntityReference::ById { braid_id: b.id.clone() })
                .collect(),
            metadata: BraidMetadata {
                custom: serde_json::to_value(&summary)?,
                ..Default::default()
            },
            ..Default::default()
        }).await?;
        
        Ok(braid)
    }
}
```

### 4.2 Hierarchical Compression

```rust
/// Hierarchical compression for very large sessions
pub struct HierarchicalCompressor {
    engine: Arc<CompressionEngine>,
    config: HierarchicalConfig,
}

impl HierarchicalCompressor {
    /// Compress with multiple summary levels
    pub async fn compress(
        &self,
        session: &Session,
        levels: Vec<CompressionLevel>,
    ) -> Result<CompressionResult, CompressionError> {
        let mut current_braids = Vec::new();
        let mut all_braids = Vec::new();
        
        // Level 0: Leaf Braids from session segments
        let segments = self.segment_session(session, &levels[0]);
        for segment in segments {
            let braid = self.engine.compress_single(&segment.session, &segment.summary).await?;
            current_braids.push(braid);
        }
        all_braids.extend(current_braids.clone());
        
        // Subsequent levels: Meta-Braids
        for level in &levels[1..] {
            let groups = self.group_braids(&current_braids, level);
            let mut next_level = Vec::new();
            
            for group in groups {
                let meta = self.engine.create_meta_braid(&group).await?;
                next_level.push(meta);
            }
            
            all_braids.extend(next_level.clone());
            current_braids = next_level;
        }
        
        // Top-level summary
        let root_summary = if current_braids.len() > 1 {
            Some(self.engine.create_meta_braid(&current_braids).await?)
        } else {
            current_braids.pop()
        };
        
        Ok(CompressionResult::Multiple {
            braids: all_braids,
            summary: root_summary,
        })
    }
}

/// Compression level specification
#[derive(Clone, Debug)]
pub struct CompressionLevel {
    /// Level number (0 = leaf, higher = more summary)
    pub level: u32,
    
    /// How to segment/group at this level
    pub grouping: GroupingStrategy,
    
    /// Maximum items per group
    pub max_group_size: usize,
}

/// How to group Braids at each level
#[derive(Clone, Debug)]
pub enum GroupingStrategy {
    /// Group by time window
    Temporal { window: Duration },
    
    /// Group by activity type
    ActivityType,
    
    /// Group by contributor
    Contributor,
    
    /// Group by branch in DAG
    Branch,
    
    /// Fixed size groups
    FixedSize { size: usize },
}
```

---

## 5. Configuration

```rust
/// Compression configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum vertices for compression (below = single or none)
    pub min_vertices: usize,
    
    /// Threshold for splitting into multiple Braids
    pub split_threshold: usize,
    
    /// Threshold for hierarchical compression
    pub hierarchical_threshold: usize,
    
    /// Coherence threshold for single Braid (0.0 - 1.0)
    pub coherence_threshold: f64,
    
    /// Maximum Braids per session
    pub max_braids_per_session: usize,
    
    /// Enable meta-Braid generation
    pub generate_summaries: bool,
    
    /// Maximum summary depth
    pub max_summary_depth: u32,
    
    /// Compression hints behavior
    pub honor_hints: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_vertices: 1,
            split_threshold: 100,
            hierarchical_threshold: 1000,
            coherence_threshold: 0.7,
            max_braids_per_session: 100,
            generate_summaries: true,
            max_summary_depth: 3,
            honor_hints: true,
        }
    }
}
```

---

## 6. Compression Hints

Sessions can provide hints to guide compression:

```rust
/// Compression hints from session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompressionHint {
    /// Force single Braid
    Single,
    
    /// Allow any compression
    Auto,
    
    /// Split by specified criteria
    Split { criteria: SplitCriteria },
    
    /// No Braid needed (ephemeral session)
    Ephemeral,
    
    /// Treat as atomic unit
    Atomic,
    
    /// Important: prioritize preservation
    Important { retention: RetentionPolicy },
}

/// Split criteria from hints
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SplitCriteria {
    /// Split at these vertex hashes
    AtVertices(Vec<VertexHash>),
    
    /// Split by contributor
    ByContributor,
    
    /// Split by activity type
    ByActivityType,
    
    /// Split by time intervals
    ByTime { interval: Duration },
}
```

---

## 7. The War Correspondent Problem

Creating a single coherent record of ongoing complexity is the hardest case. Here's how SweetGrass approaches it:

### 7.1 Strategies for "1"

```rust
impl CompressionEngine {
    /// Compress complex session to single Braid
    async fn compress_single(
        &self,
        session: &Session,
        summary: &DehydrationSummary,
    ) -> Result<Braid, CompressionError> {
        // Strategy 1: Focus on outcomes, not process
        let outcomes = session.outcomes();
        
        // Strategy 2: Preserve key decision points
        let decisions = self.find_decision_points(session);
        
        // Strategy 3: Compress branches to "alternatives considered"
        let alternatives = self.summarize_alternatives(session);
        
        // Strategy 4: Maintain attribution even when compressing
        let contributors = self.extract_all_contributors(session);
        
        // Build the single Braid
        let activity = Activity {
            activity_type: ActivityType::SessionCommit,
            used: session.inputs()
                .map(|v| UsedEntity {
                    entity: EntityReference::ByHash { 
                        data_hash: v.data_hash.clone(),
                        mime_type: v.mime_type.clone(),
                    },
                    role: EntityRole::Input,
                    time: Some(v.timestamp),
                    extent: None,
                })
                .collect(),
            was_associated_with: contributors.iter()
                .map(|(did, role)| AgentAssociation {
                    agent: did.clone(),
                    role: role.clone(),
                    on_behalf_of: None,
                    had_plan: None,
                })
                .collect(),
            started_at_time: session.started_at,
            ended_at_time: Some(session.ended_at),
            metadata: ActivityMetadata {
                decision_points: Some(decisions),
                alternatives_considered: Some(alternatives),
                ..Default::default()
            },
            ecop: ActivityEcoPrimals {
                rhizo_session: Some(session.id.clone()),
                compute_units: Some(summary.compute_units),
                ..Default::default()
            },
        };
        
        self.factory.create(BraidSpec {
            braid_type: BraidType::Entity,
            data_hash: outcomes[0].data_hash.clone(),
            was_generated_by: Some(activity),
            was_derived_from: session.inputs()
                .map(|v| EntityReference::ByHash {
                    data_hash: v.data_hash.clone(),
                    mime_type: v.mime_type.clone(),
                })
                .collect(),
            ecop: EcoPrimalsAttributes {
                compression: Some(CompressionMeta {
                    vertex_count: session.vertex_count() as u64,
                    branch_count: session.branch_count() as u64,
                    ratio: 1.0 / session.vertex_count() as f64,
                    summarizes: vec![],
                }),
                ..Default::default()
            },
            ..Default::default()
        }).await
    }
}
```

### 7.2 When Not to Force "1"

Sometimes the right answer is to wait:

```rust
/// Should we delay compression?
pub fn should_delay_compression(session: &Session) -> Option<DelayReason> {
    // Active conflict resolution in progress
    if session.has_unresolved_conflicts() {
        return Some(DelayReason::UnresolvedConflicts);
    }
    
    // Waiting for external input
    if session.has_pending_inputs() {
        return Some(DelayReason::PendingInputs);
    }
    
    // Multiple active branches with no convergence signal
    if session.branch_count() > 1 && !session.has_convergence_hint() {
        return Some(DelayReason::NoConvergence);
    }
    
    None
}

pub enum DelayReason {
    UnresolvedConflicts,
    PendingInputs,
    NoConvergence,
    ExplicitHold,
}
```

---

## 8. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [DATA_MODEL.md](./DATA_MODEL.md) — Braid structure
- [RhizoCrypt Dehydration](../../rhizoCrypt/specs/DEHYDRATION_PROTOCOL.md)
- [LoamSpine Commits](../../loamSpine/specs/DATA_MODEL.md)

---

*SweetGrass: Compressing chaos into coherent stories.*

