# SweetGrass — Integration Specification

**Version**: 0.2.0  
**Status**: Draft  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass integrates with the ecoPrimals ecosystem as the semantic provenance layer, drawing data from RhizoCrypt and LoamSpine while serving queries to applications, gAIa, and sunCloud.

```
┌─────────────────────────────────────────────────────────────────┐
│                        SweetGrass                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    ┌─────────────────┐                          │
│                    │   SweetGrass    │                          │
│                    │     Core        │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│     ┌───────────────────────┼───────────────────────┐           │
│     │           │           │           │           │           │
│ ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐        │
│ │RhizoCr│  │LoamSp │  │BearDog│  │ToadSt │  │sunCld │        │
│ │Adapter│  │Adapter│  │Adapter│  │Adapter│  │Adapter│        │
│ └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘        │
│     │           │           │           │           │           │
└─────┼───────────┼───────────┼───────────┼───────────┼───────────┘
      │           │           │           │           │
      ▼           ▼           ▼           ▼           ▼
 ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
 │RhizoCrpt│ │LoamSpine│ │ BearDog │ │ToadStool│ │ sunCloud│
 │   🍄    │ │   🦴    │ │   🐻    │ │   🍄    │ │   ☀️    │
 └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

---

## 2. RhizoCrypt Integration

RhizoCrypt is a primary source of provenance data through session dehydration.

### 2.1 Event Subscription

```rust
/// RhizoCrypt client for SweetGrass
#[async_trait]
pub trait RhizoCryptClient: Send + Sync {
    /// Subscribe to session resolution events
    fn subscribe_resolutions(&self) -> impl Stream<Item = DehydrationSummary> + Send;
    
    /// Get session details
    async fn get_session(&self, session_id: &SessionId) -> Result<Session, RhizoCryptError>;
    
    /// Get vertices for a session
    async fn get_vertices(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<Vertex>, RhizoCryptError>;
    
    /// Get session graph structure
    async fn get_session_graph(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionGraph, RhizoCryptError>;
}

/// Dehydration summary from RhizoCrypt
#[derive(Clone, Debug)]
pub struct DehydrationSummary {
    pub session_id: SessionId,
    pub session_type: SessionType,
    pub outcome: SessionOutcome,
    pub merkle_root: ContentHash,
    pub vertex_count: u64,
    pub branch_count: u64,
    pub inputs: Vec<EntityReference>,
    pub outputs: Vec<OutputEntity>,
    pub contributors: Vec<AgentContribution>,
    pub compute_units: f64,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub slices: Vec<SliceInfo>,
    pub loam_commit: Option<LoamCommitRef>,
}
```

### 2.2 Session Processing

```rust
/// Process RhizoCrypt session for Braid creation
pub struct RhizoCryptProcessor {
    client: Arc<dyn RhizoCryptClient>,
    compression: Arc<CompressionEngine>,
    factory: Arc<BraidFactory>,
    store: Arc<dyn BraidStore>,
}

impl RhizoCryptProcessor {
    /// Process a dehydration event
    pub async fn process(
        &self,
        summary: DehydrationSummary,
    ) -> Result<ProcessingResult, ProcessingError> {
        // 1. Get full session details if needed
        let session = if summary.vertex_count > self.config.detail_threshold {
            Some(self.client.get_session(&summary.session_id).await?)
        } else {
            None
        };
        
        // 2. Compress to Braids
        let compression_result = self.compression.compress(
            session.as_ref(),
            &summary,
        ).await?;
        
        // 3. Store Braids
        let braids = match compression_result {
            CompressionResult::None { reason } => {
                return Ok(ProcessingResult::Discarded(reason));
            }
            CompressionResult::Single(braid) => vec![braid],
            CompressionResult::Multiple { braids, summary } => {
                let mut all = braids;
                if let Some(s) = summary {
                    all.push(s);
                }
                all
            }
        };
        
        for braid in &braids {
            self.store.put(braid.clone()).await?;
        }
        
        // 4. Emit events
        for braid in &braids {
            self.emit_braid_created(braid);
        }
        
        Ok(ProcessingResult::Created(braids))
    }
}
```

### 2.3 Slice Provenance

SweetGrass tracks slice operations for lending provenance:

```rust
/// Create Braid for slice operation
pub async fn create_slice_braid(
    &self,
    slice_info: &SliceInfo,
    session_summary: &DehydrationSummary,
) -> Result<Braid, BraidError> {
    let activity = Activity {
        activity_type: match slice_info.mode {
            SliceMode::Loan { .. } => ActivityType::CertificateLoan,
            SliceMode::Consignment { .. } => ActivityType::Custom { 
                type_uri: "ecop:SliceConsignment".to_string() 
            },
            SliceMode::Copy => ActivityType::Derivation,
            SliceMode::Escrow { .. } => ActivityType::Custom {
                type_uri: "ecop:SliceEscrow".to_string()
            },
            SliceMode::Waypoint { .. } => ActivityType::Custom {
                type_uri: "ecop:SliceWaypoint".to_string()
            },
            SliceMode::Transfer => ActivityType::CertificateTransfer,
        },
        was_associated_with: vec![
            AgentAssociation {
                agent: slice_info.owner.clone(),
                role: AgentRole::Creator,
                on_behalf_of: None,
                had_plan: None,
            },
        ],
        ecop: ActivityEcoPrimals {
            rhizo_session: Some(session_summary.session_id.clone()),
            ..Default::default()
        },
        ..Default::default()
    };
    
    self.factory.create(BraidSpec {
        braid_type: BraidType::Slice {
            slice_mode: slice_info.mode.clone(),
            origin_spine: slice_info.origin_spine.clone(),
        },
        was_generated_by: Some(activity),
        ecop: EcoPrimalsAttributes {
            slice: Some(SliceRef {
                slice_id: slice_info.slice_id.clone(),
                mode: slice_info.mode.clone(),
                origin_spine: slice_info.origin_spine.clone(),
                origin_entry: slice_info.origin_entry.clone(),
                checkout_time: slice_info.checkout_time,
                return_time: slice_info.return_time,
            }),
            ..Default::default()
        },
        ..Default::default()
    }).await
}
```

---

## 3. LoamSpine Integration

LoamSpine provides permanent anchoring and certificate provenance.

### 3.1 Client Interface

```rust
/// LoamSpine client for SweetGrass
#[async_trait]
pub trait LoamSpineClient: Send + Sync {
    // ==================== Anchoring ====================
    
    /// Anchor a Braid to a spine
    async fn anchor_braid(
        &self,
        braid: &Braid,
        spine_id: SpineId,
    ) -> Result<LoamAnchor, LoamError>;
    
    /// Verify Braid anchor
    async fn verify_anchor(
        &self,
        braid_id: &BraidId,
    ) -> Result<Option<LoamAnchor>, LoamError>;
    
    // ==================== Event Subscription ====================
    
    /// Subscribe to spine entries
    fn subscribe_entries(&self, spine_id: SpineId) -> impl Stream<Item = LoamEntry> + Send;
    
    /// Subscribe to certificate events
    fn subscribe_certificates(&self) -> impl Stream<Item = CertificateEvent> + Send;
    
    // ==================== Queries ====================
    
    /// Get entry by hash
    async fn get_entry(&self, entry_hash: &EntryHash) -> Result<Option<LoamEntry>, LoamError>;
    
    /// Get certificate
    async fn get_certificate(&self, cert_id: &CertificateId) -> Result<Option<Certificate>, LoamError>;
    
    /// Get certificate history
    async fn get_certificate_history(
        &self,
        cert_id: &CertificateId,
    ) -> Result<Vec<LoamEntry>, LoamError>;
    
    /// Get spine hierarchy
    async fn get_spine_hierarchy(&self, spine_id: SpineId) -> Result<SpineHierarchy, LoamError>;
}
```

### 3.2 Entry Processing

```rust
/// Process LoamSpine entries for provenance
pub struct LoamSpineProcessor {
    client: Arc<dyn LoamSpineClient>,
    factory: Arc<BraidFactory>,
    store: Arc<dyn BraidStore>,
}

impl LoamSpineProcessor {
    /// Process LoamSpine entry
    pub async fn process_entry(
        &self,
        entry: &LoamEntry,
    ) -> Result<Option<Braid>, ProcessingError> {
        // Only create Braids for certain entry types
        match &entry.entry_type {
            EntryType::SessionCommit { summary, .. } => {
                // RhizoCrypt already created Braids, just update anchor
                self.update_anchors(summary).await?;
                Ok(None)
            }
            EntryType::CertificateMint { certificate, .. } => {
                let braid = self.create_certificate_braid(certificate, entry).await?;
                self.store.put(braid.clone()).await?;
                Ok(Some(braid))
            }
            EntryType::CertificateTransfer { from, to, .. } => {
                let braid = self.create_transfer_braid(entry, from, to).await?;
                self.store.put(braid.clone()).await?;
                Ok(Some(braid))
            }
            EntryType::CertificateLoan { .. } => {
                let braid = self.create_loan_braid(entry).await?;
                self.store.put(braid.clone()).await?;
                Ok(Some(braid))
            }
            EntryType::BraidCommit { braid_id, .. } => {
                // Update existing Braid with anchor
                self.update_braid_anchor(braid_id, entry).await?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    
    /// Create Braid for certificate mint
    async fn create_certificate_braid(
        &self,
        certificate: &Certificate,
        entry: &LoamEntry,
    ) -> Result<Braid, ProcessingError> {
        let activity = Activity {
            activity_type: ActivityType::CertificateMint,
            was_associated_with: vec![
                AgentAssociation {
                    agent: certificate.mint.minter.clone(),
                    role: AgentRole::Creator,
                    on_behalf_of: None,
                    had_plan: None,
                },
            ],
            started_at_time: entry.timestamp,
            ended_at_time: Some(entry.timestamp),
            ..Default::default()
        };
        
        self.factory.create(BraidSpec {
            braid_type: BraidType::Entity,
            data_hash: certificate.id.to_string(),
            was_generated_by: Some(activity),
            was_attributed_to: certificate.owner.clone(),
            ecop: EcoPrimalsAttributes {
                certificate: Some(certificate.id.clone()),
                loam_commit: Some(LoamCommitRef {
                    spine_id: entry.spine_id.clone(),
                    entry_hash: entry.hash.clone(),
                    index: entry.index,
                }),
                ..Default::default()
            },
            loam_anchor: Some(LoamAnchor {
                spine_id: entry.spine_id.clone(),
                entry_hash: entry.hash.clone(),
                index: entry.index,
                anchored_at: entry.timestamp,
                verified: true,
            }),
            ..Default::default()
        }).await
    }
}
```

### 3.3 Spine Hierarchy for Attribution

```rust
/// Get spine hierarchy for radiating attribution
pub async fn get_attribution_hierarchy(
    &self,
    entity: &EntityReference,
) -> Result<AttributionHierarchy, AttributionError> {
    // 1. Find the Braid
    let braid = self.resolve_to_braid(entity).await?;
    
    // 2. Get LoamSpine location
    let anchor = braid.loam_anchor.as_ref()
        .ok_or(AttributionError::NotAnchored)?;
    
    // 3. Get spine hierarchy
    let hierarchy = self.loam_client.get_spine_hierarchy(anchor.spine_id.clone()).await?;
    
    // 4. Map to attribution levels
    Ok(AttributionHierarchy {
        entity: entity.clone(),
        levels: hierarchy.spines.iter().enumerate().map(|(i, spine)| {
            AttributionLevel {
                level: i as u32,
                spine_id: spine.id.clone(),
                spine_type: spine.spine_type.clone(),
                owner: spine.owner.clone(),
            }
        }).collect(),
    })
}
```

---

## 4. BearDog Integration

BearDog provides identity and signing for Braids.

### 4.1 Client Interface

```rust
/// BearDog client for SweetGrass
#[async_trait]
pub trait BearDogClient: Send + Sync {
    /// Resolve DID to document
    async fn resolve_did(&self, did: &Did) -> Result<DidDocument, BearDogError>;
    
    /// Sign a Braid
    async fn sign_braid(
        &self,
        braid: &Braid,
        key_id: &KeyId,
    ) -> Result<BraidSignature, BearDogError>;
    
    /// Verify Braid signature
    async fn verify_braid_signature(&self, braid: &Braid) -> Result<bool, BearDogError>;
    
    /// Get agent profile
    async fn get_agent_profile(&self, did: &Did) -> Result<AgentProfile, BearDogError>;
}
```

### 4.2 Signature Operations

```rust
impl BraidFactory {
    /// Sign a Braid with BearDog
    pub async fn sign_braid(
        &self,
        braid: &mut Braid,
    ) -> Result<(), BraidError> {
        // 1. Compute signature input (canonicalized JSON-LD)
        let canonical = self.canonicalize_for_signing(braid)?;
        
        // 2. Request signature from BearDog
        let signature = self.beardog.sign_braid(braid, &self.key_id).await?;
        
        // 3. Attach signature
        braid.signature = signature;
        
        Ok(())
    }
    
    /// Verify a Braid signature
    pub async fn verify_signature(&self, braid: &Braid) -> Result<bool, BraidError> {
        self.beardog.verify_braid_signature(braid).await
            .map_err(|e| BraidError::Verification(e.to_string()))
    }
}
```

---

## 5. ToadStool Integration

ToadStool provides compute activity events for provenance.

### 5.1 Client Interface

```rust
/// ToadStool client for SweetGrass
#[async_trait]
pub trait ToadStoolClient: Send + Sync {
    /// Subscribe to task completion events
    fn subscribe_completions(&self) -> impl Stream<Item = TaskCompletion> + Send;
    
    /// Get task details
    async fn get_task(&self, task_id: &TaskId) -> Result<TaskDetails, ToadStoolError>;
    
    /// Get task inputs
    async fn get_task_inputs(&self, task_id: &TaskId) -> Result<Vec<TaskInput>, ToadStoolError>;
    
    /// Get task outputs
    async fn get_task_outputs(&self, task_id: &TaskId) -> Result<Vec<TaskOutput>, ToadStoolError>;
}

/// Task completion event
#[derive(Clone, Debug)]
pub struct TaskCompletion {
    pub task_id: TaskId,
    pub task_type: String,
    pub inputs: Vec<TaskInput>,
    pub outputs: Vec<TaskOutput>,
    pub executor: Did,
    pub compute_units: f64,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub status: TaskStatus,
    pub rhizo_session: Option<SessionId>,
}
```

### 5.2 Activity Processing

```rust
/// Process ToadStool task for Braid creation
pub struct ToadStoolProcessor {
    client: Arc<dyn ToadStoolClient>,
    factory: Arc<BraidFactory>,
    store: Arc<dyn BraidStore>,
}

impl ToadStoolProcessor {
    /// Process task completion
    pub async fn process_completion(
        &self,
        completion: TaskCompletion,
    ) -> Result<Vec<Braid>, ProcessingError> {
        // Skip failed tasks
        if completion.status != TaskStatus::Completed {
            return Ok(vec![]);
        }
        
        // Create Braid for each output
        let mut braids = Vec::new();
        
        for output in &completion.outputs {
            let activity = Activity {
                id: format!("urn:activity:toadstool:{}", completion.task_id),
                activity_type: self.map_task_type(&completion.task_type),
                used: completion.inputs.iter().map(|i| UsedEntity {
                    entity: EntityReference::ByHash {
                        data_hash: i.data_hash.clone(),
                        mime_type: Some(i.mime_type.clone()),
                    },
                    role: EntityRole::Input,
                    time: Some(completion.started_at),
                    extent: None,
                }).collect(),
                was_associated_with: vec![
                    AgentAssociation {
                        agent: completion.executor.clone(),
                        role: AgentRole::ComputeProvider,
                        on_behalf_of: None,
                        had_plan: None,
                    },
                ],
                started_at_time: completion.started_at,
                ended_at_time: Some(completion.ended_at),
                ecop: ActivityEcoPrimals {
                    compute_units: Some(completion.compute_units),
                    toadstool_task: Some(completion.task_id.clone()),
                    rhizo_session: completion.rhizo_session.clone(),
                    ..Default::default()
                },
                ..Default::default()
            };
            
            let braid = self.factory.create(BraidSpec {
                braid_type: BraidType::Entity,
                data_hash: output.data_hash.clone(),
                mime_type: output.mime_type.clone(),
                size: output.size,
                was_generated_by: Some(activity),
                was_derived_from: completion.inputs.iter().map(|i| {
                    EntityReference::ByHash {
                        data_hash: i.data_hash.clone(),
                        mime_type: Some(i.mime_type.clone()),
                    }
                }).collect(),
                ..Default::default()
            }).await?;
            
            self.store.put(braid.clone()).await?;
            braids.push(braid);
        }
        
        Ok(braids)
    }
}
```

---

## 6. sunCloud Integration

sunCloud queries SweetGrass for attribution data.

### 6.1 Interface

```rust
/// sunCloud interface for attribution queries
#[async_trait]
pub trait SunCloudProvider: Send + Sync {
    /// Get attribution chain for value distribution
    async fn get_attribution(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, AttributionError>;
    
    /// Get attribution with radiating distribution
    async fn radiate_attribution(
        &self,
        entity: EntityReference,
        value: f64,
    ) -> Result<RadiationResult, AttributionError>;
    
    /// Get agent's total contributions
    async fn get_agent_contributions(
        &self,
        agent: &Did,
        time_range: Option<TimeRange>,
    ) -> Result<AgentContributions, AttributionError>;
    
    /// Record distribution for auditing
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

impl SunCloudProvider for SweetGrass {
    async fn get_attribution(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, AttributionError> {
        self.attribution_calculator.calculate(entity).await
    }
    
    async fn radiate_attribution(
        &self,
        entity: EntityReference,
        value: f64,
    ) -> Result<RadiationResult, AttributionError> {
        self.radiating_attribution.radiate(entity, value).await
    }
    
    async fn record_distribution(
        &self,
        entity: EntityReference,
        distributions: Vec<RewardDistribution>,
    ) -> Result<DistributionReceipt, AttributionError> {
        // Create a Braid recording the distribution
        let activity = Activity {
            activity_type: ActivityType::Custom {
                type_uri: "ecop:RewardDistribution".to_string(),
            },
            was_associated_with: distributions.iter().map(|d| {
                AgentAssociation {
                    agent: d.agent.clone(),
                    role: d.role.clone(),
                    on_behalf_of: None,
                    had_plan: None,
                }
            }).collect(),
            ..Default::default()
        };
        
        let braid = self.factory.create(BraidSpec {
            braid_type: BraidType::Custom {
                type_uri: "ecop:DistributionRecord".to_string(),
            },
            was_generated_by: Some(activity),
            metadata: BraidMetadata {
                custom: serde_json::to_value(&distributions)?,
                ..Default::default()
            },
            ..Default::default()
        }).await?;
        
        // Anchor to LoamSpine for permanent record
        let anchor = self.anchor_manager.anchor(&braid, self.config.distribution_spine).await?;
        
        Ok(DistributionReceipt {
            entity,
            distributions,
            total_distributed: distributions.iter().map(|d| d.amount).sum(),
            timestamp: current_timestamp_nanos(),
            braid_id: braid.id,
        })
    }
}
```

---

## 7. gAIa Integration

gAIa queries SweetGrass for trust assessment and knowledge graph.

### 7.1 Interface

```rust
/// gAIa query interface
#[async_trait]
pub trait GaiaQueryInterface: Send + Sync {
    /// Assess trust for an entity
    async fn assess_trust(
        &self,
        entity: EntityReference,
    ) -> Result<TrustAssessment, GaiaError>;
    
    /// Get attribution for entity
    async fn get_attribution(
        &self,
        entity: EntityReference,
    ) -> Result<AttributionChain, GaiaError>;
    
    /// Search knowledge graph
    async fn search(
        &self,
        query: SemanticQuery,
    ) -> Result<SearchResults, GaiaError>;
    
    /// Get provenance graph
    async fn provenance_graph(
        &self,
        entity: EntityReference,
        depth: u32,
    ) -> Result<ProvenanceGraph, GaiaError>;
}

/// Trust assessment result
#[derive(Clone, Debug)]
pub struct TrustAssessment {
    pub entity: EntityReference,
    pub trust_score: f64,
    pub factors: Vec<TrustFactor>,
    pub provenance_depth: u32,
    pub agent_reputations: HashMap<Did, f64>,
}

#[derive(Clone, Debug)]
pub struct TrustFactor {
    pub name: String,
    pub weight: f64,
    pub score: f64,
    pub evidence: Vec<BraidId>,
}

impl GaiaQueryInterface for SweetGrass {
    async fn assess_trust(
        &self,
        entity: EntityReference,
    ) -> Result<TrustAssessment, GaiaError> {
        // 1. Get provenance graph
        let graph = self.query_engine.provenance_graph(entity.clone(), 10).await?;
        
        // 2. Calculate trust factors
        let factors = vec![
            self.calculate_depth_factor(&graph),
            self.calculate_signature_factor(&graph).await?,
            self.calculate_anchor_factor(&graph).await?,
            self.calculate_reputation_factor(&graph).await?,
        ];
        
        // 3. Combine factors
        let total_weight: f64 = factors.iter().map(|f| f.weight).sum();
        let weighted_score: f64 = factors.iter()
            .map(|f| f.weight * f.score)
            .sum::<f64>() / total_weight;
        
        // 4. Get agent reputations
        let agent_reputations = self.get_agent_reputations(&graph).await?;
        
        Ok(TrustAssessment {
            entity,
            trust_score: weighted_score,
            factors,
            provenance_depth: graph.depth,
            agent_reputations,
        })
    }
}
```

---

## 8. Songbird Integration

Songbird provides service discovery for SweetGrass.

### 8.1 Registration

```rust
/// Register SweetGrass with Songbird
pub async fn register_with_songbird(
    sweetgrass: &SweetGrass,
    songbird: &impl SongbirdClient,
) -> Result<RegistrationReceipt, SongbirdError> {
    let capabilities = vec![
        Capability::new("sweetgrass:braid:create"),
        Capability::new("sweetgrass:braid:query"),
        Capability::new("sweetgrass:provenance:graph"),
        Capability::new("sweetgrass:attribution:calculate"),
        Capability::new("sweetgrass:graphql"),
        Capability::new("sweetgrass:sparql"),
    ];
    
    let service_info = ServiceInfo {
        name: "sweetgrass".to_string(),
        version: sweetgrass.version().to_string(),
        capabilities,
        endpoints: vec![
            Endpoint::Grpc {
                host: sweetgrass.grpc_host(),
                port: sweetgrass.grpc_port(),
            },
            Endpoint::Rest {
                base_url: sweetgrass.rest_url(),
            },
            Endpoint::GraphQL {
                url: sweetgrass.graphql_url(),
            },
        ],
        health_check: Some(HealthCheck {
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
        }),
    };
    
    songbird.register(service_info).await
}
```

---

## 9. Adapter Pattern

All integrations use a common adapter pattern:

```rust
/// Generic primal adapter
pub struct PrimalAdapter<C> {
    client: C,
    config: AdapterConfig,
    metrics: AdapterMetrics,
    circuit_breaker: CircuitBreaker,
}

impl<C> PrimalAdapter<C> {
    pub fn new(client: C, config: AdapterConfig) -> Self {
        Self {
            client,
            config,
            metrics: AdapterMetrics::default(),
            circuit_breaker: CircuitBreaker::new(config.circuit_breaker.clone()),
        }
    }
    
    /// Execute with retry and circuit breaker
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, AdapterError>
    where
        F: Fn(&C) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + 'static,
    {
        // Check circuit breaker
        if self.circuit_breaker.is_open() {
            return Err(AdapterError::CircuitOpen);
        }
        
        let mut attempts = 0;
        loop {
            attempts += 1;
            
            match tokio::time::timeout(
                self.config.timeout,
                operation(&self.client),
            ).await {
                Ok(Ok(result)) => {
                    self.circuit_breaker.record_success();
                    self.metrics.record_success(attempts);
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    self.circuit_breaker.record_failure();
                    if attempts >= self.config.retry.max_attempts {
                        self.metrics.record_failure();
                        return Err(AdapterError::Operation(e.to_string()));
                    }
                    tokio::time::sleep(self.config.retry.delay(attempts)).await;
                }
                Err(_) => {
                    self.circuit_breaker.record_failure();
                    if attempts >= self.config.retry.max_attempts {
                        self.metrics.record_timeout();
                        return Err(AdapterError::Timeout);
                    }
                    tokio::time::sleep(self.config.retry.delay(attempts)).await;
                }
            }
        }
    }
}
```

---

## 10. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("RhizoCrypt error: {0}")]
    RhizoCrypt(#[from] RhizoCryptError),
    
    #[error("LoamSpine error: {0}")]
    LoamSpine(#[from] LoamError),
    
    #[error("BearDog error: {0}")]
    BearDog(#[from] BearDogError),
    
    #[error("ToadStool error: {0}")]
    ToadStool(#[from] ToadStoolError),
    
    #[error("Songbird error: {0}")]
    Songbird(#[from] SongbirdError),
    
    #[error("Adapter error: {0}")]
    Adapter(#[from] AdapterError),
    
    #[error("Processing error: {0}")]
    Processing(String),
}
```

---

## 11. References

- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — API definitions
- [RhizoCrypt Integration](../../rhizoCrypt/specs/INTEGRATION_SPECIFICATION.md)
- [LoamSpine Integration](../../loamSpine/specs/INTEGRATION_SPECIFICATION.md)

---

*SweetGrass: The semantic bridge connecting the ecosystem.*

