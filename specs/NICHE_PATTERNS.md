# SweetGrass — Niche Patterns Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass is not a fixed architecture but a **configurable semantic capability**. Its behavior adapts based on how it's organized with other primals in a biomeOS niche.

Primals are **infrastructure legos**—building blocks that can be combined in different ways. biomeOS orchestrates these combinations into **niches**: organizational patterns optimized for specific use cases.

```
┌─────────────────────────────────────────────────────────────────┐
│                         biomeOS                                  │
│                   (Niche Orchestration)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────┐      │
│   │ NICHE A       │  │ NICHE B       │  │ NICHE C       │      │
│   │               │  │               │  │               │      │
│   │ 🍄 + 🌾 + 🦴  │  │ 🍄 + 🌾       │  │ 🌾 + 🦴       │      │
│   │               │  │               │  │               │      │
│   │ Deep Archive  │  │ Real-time     │  │ Audit Trail   │      │
│   └───────────────┘  └───────────────┘  └───────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. SweetGrass Configuration Axes

SweetGrass adapts along several dimensions:

| Axis | Range | Affects |
|------|-------|---------|
| **Permanence** | Ephemeral ↔ Archival | How long Braids are retained |
| **Depth** | Shallow ↔ Deep | How much provenance detail |
| **Latency** | Real-time ↔ Batch | When Braids are created |
| **Attribution** | Minimal ↔ Full | How much contributor tracking |
| **Querying** | Simple ↔ Rich | Query capabilities exposed |

```rust
/// SweetGrass niche configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NicheConfig {
    /// Niche identifier
    pub niche_id: NicheId,
    
    /// Permanence strategy
    pub permanence: PermanenceStrategy,
    
    /// Provenance depth
    pub depth: ProvenanceDepth,
    
    /// Processing latency mode
    pub latency: LatencyMode,
    
    /// Attribution level
    pub attribution: AttributionLevel,
    
    /// Query configuration
    pub query: QueryConfig,
    
    /// Compression configuration
    pub compression: CompressionConfig,
    
    /// Integration configuration
    pub integrations: IntegrationConfig,
}
```

---

## 3. Standard Niche Patterns

### 3.1 Distributed Science Niche

Deep attribution chains, permanent Braids, full provenance for reproducibility.

```yaml
# biomeOS niche definition
niche: distributed-science
version: 1.0.0

primals:
  - rhizocrypt:
      role: session-capture
      config:
        session_timeout: 1h
        auto_dehydrate: true
        
  - loamspine:
      role: permanent-record
      config:
        spine_type: community
        replication: 3
        
  - sweetgrass:
      role: provenance-layer
      config:
        permanence: archival
        depth: full
        latency: batch
        attribution: full
        
  - toadstool:
      role: compute
      config:
        capture_provenance: true
        
  - nestgate:
      role: data-storage
      config:
        content_addressed: true

sweetgrass:
  permanence:
    strategy: archival
    retention: forever
    anchor_all: true
    
  depth:
    level: full
    include_inputs: true
    include_environment: true
    capture_parameters: true
    
  attribution:
    level: full
    track_compute: true
    track_storage: true
    track_data: true
    inheritance_decay: 0.7
    
  compression:
    mode: hierarchical
    summary_depth: 3
    preserve_decisions: true
```

```rust
impl SweetGrass {
    /// Configure for distributed science niche
    pub fn configure_science_niche(&mut self) {
        self.config = NicheConfig {
            permanence: PermanenceStrategy::Archival {
                retention: Duration::MAX,
                anchor_all: true,
            },
            depth: ProvenanceDepth::Full {
                include_inputs: true,
                include_environment: true,
                capture_parameters: true,
            },
            latency: LatencyMode::Batch {
                window: Duration::from_secs(60),
            },
            attribution: AttributionLevel::Full {
                track_compute: true,
                track_storage: true,
                track_data: true,
                inheritance_decay: 0.7,
            },
            query: QueryConfig {
                graphql: true,
                sparql: true,
                full_text: true,
            },
            compression: CompressionConfig {
                generate_summaries: true,
                max_summary_depth: 3,
                ..Default::default()
            },
            ..Default::default()
        };
    }
}
```

### 3.2 Gaming Federation Niche

Item provenance, lightweight activity tracking, fast queries.

```yaml
niche: gaming-federation
version: 1.0.0

primals:
  - rhizocrypt:
      role: game-state
      config:
        session_timeout: 5m
        ephemeral_branches: true
        
  - loamspine:
      role: asset-ledger
      config:
        spine_type: personal
        certificates: true
        
  - sweetgrass:
      role: item-provenance
      config:
        permanence: selective
        depth: minimal
        latency: realtime
        attribution: minimal

sweetgrass:
  permanence:
    strategy: selective
    anchor_items: true
    anchor_achievements: true
    discard_gameplay: true
    
  depth:
    level: minimal
    track_ownership: true
    track_transfers: true
    skip_gameplay_details: true
    
  attribution:
    level: minimal
    track_creators: true
    skip_intermediate: true
    
  compression:
    mode: aggressive
    single_braid_only: true
    
  query:
    graphql: true
    sparql: false
    optimized_for:
      - item_history
      - ownership_chain
```

```rust
impl SweetGrass {
    /// Configure for gaming federation niche
    pub fn configure_gaming_niche(&mut self) {
        self.config = NicheConfig {
            permanence: PermanenceStrategy::Selective {
                rules: vec![
                    RetentionRule::Always { 
                        activity_types: vec![
                            ActivityType::CertificateMint,
                            ActivityType::CertificateTransfer,
                        ],
                    },
                    RetentionRule::Discard {
                        activity_types: vec![
                            ActivityType::Custom { type_uri: "game:movement".into() },
                            ActivityType::Custom { type_uri: "game:combat".into() },
                        ],
                    },
                ],
            },
            depth: ProvenanceDepth::Minimal {
                track_ownership: true,
                track_transfers: true,
            },
            latency: LatencyMode::Realtime,
            attribution: AttributionLevel::Minimal {
                track_creators: true,
            },
            compression: CompressionConfig {
                split_threshold: usize::MAX, // Never split
                generate_summaries: false,
                ..Default::default()
            },
            ..Default::default()
        };
    }
}
```

### 3.3 Real-Time Monitoring Niche

Streaming provenance, ephemeral Braids, minimal retention.

```yaml
niche: realtime-monitoring
version: 1.0.0

primals:
  - rhizocrypt:
      role: event-capture
      config:
        streaming: true
        buffer_size: 1000
        
  - sweetgrass:
      role: streaming-provenance
      config:
        permanence: ephemeral
        depth: shallow
        latency: realtime
        
  # Note: No LoamSpine - pure ephemeral

sweetgrass:
  permanence:
    strategy: ephemeral
    ttl: 1h
    anchor: never
    
  depth:
    level: shallow
    summary_only: true
    
  latency:
    mode: streaming
    emit_interval: 100ms
    
  query:
    graphql: true
    realtime_subscriptions: true
```

### 3.4 Audit Trail Niche

Compliance-focused, immutable records, full chain of custody.

```yaml
niche: audit-trail
version: 1.0.0

primals:
  - loamspine:
      role: immutable-ledger
      config:
        spine_type: community
        witnesses: 5
        external_anchor: btc
        
  - sweetgrass:
      role: chain-of-custody
      config:
        permanence: immutable
        depth: full
        attribution: full
        
  - beardog:
      role: identity
      config:
        require_attestation: true

sweetgrass:
  permanence:
    strategy: immutable
    anchor_all: true
    external_witness: true
    
  depth:
    level: full
    chain_of_custody: true
    capture_all_handlers: true
    
  attribution:
    level: full
    require_signatures: true
    multi_party_attestation: true
    
  compliance:
    standards:
      - ISO27001
      - SOC2
    retention_policy: 7y
```

### 3.5 Personal Knowledge Niche

Individual user, private, local-first.

```yaml
niche: personal-knowledge
version: 1.0.0

primals:
  - rhizocrypt:
      role: thought-capture
      config:
        local_only: true
        
  - loamspine:
      role: personal-archive
      config:
        spine_type: personal
        storage: local
        
  - sweetgrass:
      role: knowledge-graph
      config:
        permanence: personal
        depth: semantic
        query: rich

sweetgrass:
  permanence:
    strategy: personal
    local_storage: true
    sync: optional
    
  depth:
    level: semantic
    extract_concepts: true
    build_links: true
    
  attribution:
    level: self
    track_sources: true
    
  query:
    graphql: true
    semantic_search: true
    knowledge_graph: true
```

---

## 4. Dynamic Configuration

Niches can adjust SweetGrass behavior at runtime:

```rust
/// Dynamic niche configuration
pub trait NicheConfigurable {
    /// Apply niche configuration
    fn apply_niche(&mut self, niche: &NicheConfig) -> Result<(), ConfigError>;
    
    /// Get current niche
    fn current_niche(&self) -> Option<&NicheId>;
    
    /// Switch niches (if allowed)
    fn switch_niche(&mut self, niche_id: NicheId) -> Result<(), ConfigError>;
    
    /// Get effective configuration
    fn effective_config(&self) -> &NicheConfig;
}

impl NicheConfigurable for SweetGrass {
    fn apply_niche(&mut self, niche: &NicheConfig) -> Result<(), ConfigError> {
        // Validate niche is compatible with current state
        self.validate_niche_transition(niche)?;
        
        // Apply configuration
        self.config = niche.clone();
        
        // Reconfigure components
        self.compression_engine.reconfigure(&niche.compression)?;
        self.query_engine.reconfigure(&niche.query)?;
        self.anchor_manager.reconfigure(&niche.permanence)?;
        
        // Emit niche change event
        self.emit_event(SweetGrassEvent::NicheChanged {
            niche_id: niche.niche_id.clone(),
        });
        
        Ok(())
    }
    
    fn switch_niche(&mut self, niche_id: NicheId) -> Result<(), ConfigError> {
        // Load niche definition
        let niche = self.niche_registry.get(&niche_id)
            .ok_or(ConfigError::NicheNotFound(niche_id))?;
        
        self.apply_niche(&niche)
    }
}
```

---

## 5. Niche Composition

Complex deployments can compose multiple niche patterns:

```rust
/// Composed niche from multiple patterns
pub struct ComposedNiche {
    /// Base pattern
    base: NichePattern,
    
    /// Overlay patterns
    overlays: Vec<NicheOverlay>,
    
    /// Conflict resolution
    resolution: ConflictResolution,
}

/// How to resolve configuration conflicts
pub enum ConflictResolution {
    /// Last overlay wins
    LastWins,
    
    /// Most restrictive wins
    MostRestrictive,
    
    /// Merge with specified strategy
    Merge(MergeStrategy),
    
    /// Explicit priority ordering
    Priority(Vec<NicheId>),
}

impl ComposedNiche {
    /// Compute effective configuration
    pub fn effective_config(&self) -> NicheConfig {
        let mut config = self.base.config.clone();
        
        for overlay in &self.overlays {
            config = self.apply_overlay(config, overlay);
        }
        
        config
    }
    
    fn apply_overlay(&self, base: NicheConfig, overlay: &NicheOverlay) -> NicheConfig {
        match &self.resolution {
            ConflictResolution::LastWins => overlay.config.clone(),
            ConflictResolution::MostRestrictive => {
                NicheConfig {
                    permanence: self.more_restrictive_permanence(
                        &base.permanence,
                        &overlay.config.permanence,
                    ),
                    depth: self.more_restrictive_depth(
                        &base.depth,
                        &overlay.config.depth,
                    ),
                    attribution: self.more_restrictive_attribution(
                        &base.attribution,
                        &overlay.config.attribution,
                    ),
                    ..base
                }
            }
            // ... other strategies
        }
    }
}
```

---

## 6. biomeOS Integration

SweetGrass receives niche configuration from biomeOS:

```rust
/// biomeOS integration for niche configuration
pub struct BiomeOSIntegration {
    manifest_watcher: ManifestWatcher,
    current_niche: Option<NicheConfig>,
}

impl BiomeOSIntegration {
    /// Watch for niche changes from biomeOS
    pub async fn watch_niche_changes(&self) -> impl Stream<Item = NicheConfig> {
        self.manifest_watcher
            .watch()
            .filter_map(|event| match event {
                ManifestEvent::NicheUpdated(niche) => Some(niche),
                _ => None,
            })
    }
    
    /// Get niche configuration from biomeOS manifest
    pub async fn get_niche_config(&self, niche_id: &NicheId) -> Result<NicheConfig, BiomeError> {
        let manifest = self.manifest_watcher.current_manifest()?;
        
        manifest.get_primal_config("sweetgrass", niche_id)
            .map(|config| config.into())
    }
}
```

---

## 7. Niche Templates

Standard templates for common patterns:

```rust
pub mod templates {
    use super::*;
    
    /// Minimal ephemeral tracking
    pub fn ephemeral() -> NicheConfig {
        NicheConfig {
            permanence: PermanenceStrategy::Ephemeral { ttl: Duration::from_secs(3600) },
            depth: ProvenanceDepth::Minimal { track_ownership: false, track_transfers: false },
            latency: LatencyMode::Realtime,
            attribution: AttributionLevel::None,
            ..Default::default()
        }
    }
    
    /// Standard personal use
    pub fn personal() -> NicheConfig {
        NicheConfig {
            permanence: PermanenceStrategy::Personal { local_storage: true, sync: false },
            depth: ProvenanceDepth::Standard,
            latency: LatencyMode::Batch { window: Duration::from_secs(60) },
            attribution: AttributionLevel::Self_,
            ..Default::default()
        }
    }
    
    /// Community collaboration
    pub fn community() -> NicheConfig {
        NicheConfig {
            permanence: PermanenceStrategy::Community { replication: 3 },
            depth: ProvenanceDepth::Full { 
                include_inputs: true,
                include_environment: false,
                capture_parameters: true,
            },
            latency: LatencyMode::Batch { window: Duration::from_secs(300) },
            attribution: AttributionLevel::Full {
                track_compute: true,
                track_storage: true,
                track_data: true,
                inheritance_decay: 0.7,
            },
            ..Default::default()
        }
    }
    
    /// Archival/scientific
    pub fn archival() -> NicheConfig {
        NicheConfig {
            permanence: PermanenceStrategy::Archival { retention: Duration::MAX, anchor_all: true },
            depth: ProvenanceDepth::Full {
                include_inputs: true,
                include_environment: true,
                capture_parameters: true,
            },
            latency: LatencyMode::Batch { window: Duration::from_secs(3600) },
            attribution: AttributionLevel::Full {
                track_compute: true,
                track_storage: true,
                track_data: true,
                inheritance_decay: 0.7,
            },
            ..Default::default()
        }
    }
}
```

---

## 8. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [biomeOS Specification](../../biomeOS/)
- [Chimera Definitions](../../biomeOS/chimeras/)

---

*SweetGrass: Adapting semantic patterns to ecosystem niches.*

