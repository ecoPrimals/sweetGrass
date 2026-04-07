# SweetGrass — Integration Specification

**Version**: 2.0.0  
**Status**: Canonical  
**Last Updated**: December 2025

---

## 1. Overview

SweetGrass integrates with the ecoPrimals ecosystem using **pure Rust tarpc** for primal-to-primal communication. All integrations follow primal sovereignty principles—no gRPC, no protobuf.

### 1.1 Capability-Based Architecture

SweetGrass uses **capability-based discovery** rather than hardcoded primal names. Each integration is defined by the capability it provides, not the specific primal that implements it.

| Capability | Purpose | Example Primals |
|------------|---------|-----------------|
| `Signing` | Braid signatures, DID resolution | BearDog |
| `SessionEvents` | Session capture, activity tracking | RhizoCrypt |
| `Anchoring` | Permanent storage anchoring | LoamSpine |
| `Compute` | Task execution, compute activity | ToadStool |
| `Discovery` | Service mesh, primal discovery | Songbird |

```
┌─────────────────────────────────────────────────────────────────┐
│                        SweetGrass 🌾                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                    ┌─────────────────┐                          │
│                    │   SweetGrass    │                          │
│                    │     Core        │                          │
│                    └────────┬────────┘                          │
│                             │                                    │
│     ┌───────────────────────┼───────────────────────┐           │
│     │ tarpc      │ tarpc    │ tarpc    │ tarpc      │           │
│     │           │           │           │           │           │
│ ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐  ┌───▼───┐        │
│ │Session│  │Anchor │  │Signing│  │Compute│  │Attrib │        │
│ │Events │  │Client │  │Client │  │Client │  │Provider│        │
│ └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘  └───┬───┘        │
│     │           │           │           │           │           │
└─────┼───────────┼───────────┼───────────┼───────────┼───────────┘
      │ tarpc     │ tarpc     │ tarpc     │ tarpc     │ tarpc
      ▼           ▼           ▼           ▼           ▼
 ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
 │ Session │ │Anchoring│ │ Signing │ │ Compute │ │ Attrib  │
 │ Events  │ │ Service │ │ Service │ │ Service │ │ Service │
 └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

### 1.2 Zero-Knowledge Startup

SweetGrass starts with minimal configuration and discovers capabilities at runtime:

```rust
// Discovery-based client creation (no hardcoded addresses)
let discovery = create_discovery().await;
let primal = discovery.find_one(&Capability::Signing).await?;
let client = create_signing_client_async(&primal).await?;
```

---

## 2. Session Events Integration

Session events capability provides activity tracking and session data through dehydration.

*Note: Currently implemented by RhizoCrypt, but SweetGrass discovers by capability, not name.*

### 2.1 tarpc Client Trait

```rust
/// Session events tarpc service (capability-based)
#[tarpc::service]
pub trait SessionEventsRpc {
    /// Subscribe to session resolution events (returns stream ID)
    async fn subscribe_resolutions() -> Result<SubscriptionId, RhizoCryptError>;
    
    /// Poll for new resolutions
    async fn poll_resolutions(
        subscription_id: SubscriptionId,
    ) -> Result<Vec<DehydrationSummary>, RhizoCryptError>;
    
    /// Get session details
    async fn get_session(session_id: SessionId) -> Result<Session, RhizoCryptError>;
    
    /// Get vertices for a session
    async fn get_vertices(session_id: SessionId) -> Result<Vec<Vertex>, RhizoCryptError>;
    
    /// Get session graph structure
    async fn get_session_graph(session_id: SessionId) -> Result<SessionGraph, RhizoCryptError>;
}

/// Dehydration summary from RhizoCrypt
#[derive(Clone, Debug, Serialize, Deserialize)]
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

### 2.2 Client Implementation

```rust
use tarpc::{client, context};

pub struct TarpcSessionEventsClient {
    client: SessionEventsRpcClient,
}

impl TarpcSessionEventsClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let transport = tarpc::serde_transport::tcp::connect(
            addr,
            tarpc::tokio_serde::formats::Bincode::default,
        ).await?;
        
        let client = SessionEventsRpcClient::new(
            client::Config::default(),
            transport,
        ).spawn();
        
        Ok(Self { client })
    }
    
    pub async fn get_session(&self, session_id: SessionId) -> Result<Session> {
        self.client.get_session(context::current(), session_id).await?
            .map_err(Into::into)
    }
    
    pub async fn subscribe_and_process<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(DehydrationSummary) -> Result<()>,
    {
        let sub_id = self.client.subscribe_resolutions(context::current()).await??;
        
        loop {
            let summaries = self.client.poll_resolutions(context::current(), sub_id).await??;
            for summary in summaries {
                handler(summary)?;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

### 2.3 Session Processing

```rust
/// Process session events for Braid creation
pub struct SessionEventsProcessor {
    client: Arc<dyn SessionEventsClient>,
    compression: Arc<CompressionEngine>,
    factory: Arc<BraidFactory>,
    store: Arc<dyn BraidStore>,
}

impl RhizoCryptProcessor {
    /// Process a dehydration event
    pub async fn process(&self, summary: DehydrationSummary) -> Result<ProcessingResult> {
        // 1. Get full session details if needed
        let session = if summary.vertex_count > self.config.detail_threshold {
            Some(self.client.get_session(summary.session_id.clone()).await?)
        } else {
            None
        };
        
        // 2. Compress to Braids
        let compression_result = self.compression.compress_summary(&summary, session.as_ref())?;
        
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
            self.store.put(braid).await?;
        }
        
        Ok(ProcessingResult::Created(braids))
    }
}
```

---

## 3. Anchoring Integration

Anchoring capability provides permanent storage and certificate provenance.

*Note: Currently implemented by LoamSpine, but SweetGrass discovers by capability, not name.*

### 3.1 tarpc Client Trait

```rust
/// Anchoring tarpc service (capability-based)
#[tarpc::service]
pub trait AnchoringRpc {
    // ==================== Anchoring ====================
    
    /// Anchor a Braid to a spine
    async fn anchor_braid(
        braid_hash: ContentHash,
        spine_id: SpineId,
    ) -> Result<LoamAnchor, LoamError>;
    
    /// Verify Braid anchor
    async fn verify_anchor(braid_id: BraidId) -> Result<Option<LoamAnchor>, LoamError>;
    
    // ==================== Event Subscription ====================
    
    /// Subscribe to spine entries
    async fn subscribe_entries(spine_id: SpineId) -> Result<SubscriptionId, LoamError>;
    
    /// Poll for new entries
    async fn poll_entries(subscription_id: SubscriptionId) -> Result<Vec<LoamEntry>, LoamError>;
    
    /// Subscribe to certificate events
    async fn subscribe_certificates() -> Result<SubscriptionId, LoamError>;
    
    /// Poll for certificate events
    async fn poll_certificates(
        subscription_id: SubscriptionId,
    ) -> Result<Vec<CertificateEvent>, LoamError>;
    
    // ==================== Queries ====================
    
    /// Get entry by hash
    async fn get_entry(entry_hash: EntryHash) -> Result<Option<LoamEntry>, LoamError>;
    
    /// Get certificate
    async fn get_certificate(cert_id: CertificateId) -> Result<Option<Certificate>, LoamError>;
    
    /// Get certificate history
    async fn get_certificate_history(
        cert_id: CertificateId,
    ) -> Result<Vec<LoamEntry>, LoamError>;
    
    /// Get spine hierarchy
    async fn get_spine_hierarchy(spine_id: SpineId) -> Result<SpineHierarchy, LoamError>;
}
```

### 3.2 Client Implementation

```rust
pub struct TarpcAnchoringClient {
    client: AnchoringRpcClient,
}

impl TarpcAnchoringClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let transport = tarpc::serde_transport::tcp::connect(
            addr,
            tarpc::tokio_serde::formats::Bincode::default,
        ).await?;
        
        let client = AnchoringRpcClient::new(
            client::Config::default(),
            transport,
        ).spawn();
        
        Ok(Self { client })
    }
    
    pub async fn anchor_braid(
        &self,
        braid: &Braid,
        spine_id: SpineId,
    ) -> Result<AnchorReceipt> {
        self.client.anchor_braid(
            context::current(),
            braid.data_hash.clone(),
            spine_id,
        ).await?.map_err(Into::into)
    }
    
    pub async fn verify_anchor(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>> {
        self.client.verify_anchor(context::current(), braid_id.clone()).await?
            .map_err(Into::into)
    }
}
```

### 3.3 Anchor Manager

```rust
/// Manage Braid anchoring (capability-based)
pub struct AnchorManager {
    anchoring_client: Arc<dyn AnchoringClient>,
    store: Arc<dyn BraidStore>,
}

impl AnchorManager {
    pub async fn anchor(&self, braid: &Braid, spine_id: SpineId) -> Result<AnchorReceipt> {
        // 1. Request anchor from anchoring service
        let anchor = self.anchoring_client.anchor(braid, spine_id).await?;
        
        // 2. Update Braid with anchor
        let mut updated = braid.clone();
        updated.loam_anchor = Some(anchor.anchor.clone());
        self.store.put(&updated).await?;
        
        Ok(anchor)
    }
    
    pub async fn verify(&self, braid_id: &BraidId) -> Result<AnchorVerification> {
        let anchor = self.anchoring_client.verify(braid_id).await?;
        
        Ok(AnchorVerification {
            anchored: anchor.is_some(),
            anchor,
            verified: true,
            verification_time: Some(current_timestamp_nanos()),
        })
    }
}
```

---

## 4. Signing Integration

Signing capability provides identity and cryptographic signatures for Braids.

*Note: Currently implemented by BearDog, but SweetGrass discovers by capability, not name.*

### 4.1 tarpc Client Trait

```rust
/// Signing tarpc service (capability-based)
#[tarpc::service]
pub trait SigningRpc {
    /// Resolve DID to document
    async fn resolve_did(did: Did) -> Result<DidDocument, BearDogError>;
    
    /// Sign data with agent key
    async fn sign(
        data: Vec<u8>,
        key_id: KeyId,
    ) -> Result<Signature, BearDogError>;
    
    /// Verify signature
    async fn verify(
        data: Vec<u8>,
        signature: Signature,
        did: Did,
    ) -> Result<bool, BearDogError>;
    
    /// Get agent profile
    async fn get_agent_profile(did: Did) -> Result<AgentProfile, BearDogError>;
}
```

### 4.2 Braid Signing

```rust
pub struct DiscoverySigner {
    signing_client: Arc<dyn SigningClient>,
    key_id: KeyId,
}

impl DiscoverySigner {
    pub async fn sign_braid(&self, braid: &mut Braid) -> Result<()> {
        // 1. Canonicalize Braid for signing
        let canonical = self.canonicalize(braid)?;
        
        // 2. Request signature from signing service
        let sig_bytes = self.signing_client.sign(canonical, self.key_id.clone()).await?;
        
        // 3. Set the witness (WireWitnessRef vocabulary)
        braid.witness = Witness::from_ed25519(&braid.was_attributed_to, &sig_bytes);
        
        Ok(())
    }
    
    pub async fn verify_signature(&self, braid: &Braid) -> Result<bool> {
        let canonical = self.canonicalize(braid)?;
        let evidence = base64::decode(&braid.witness.evidence)?;
        
        self.signing_client.verify(
            canonical,
            evidence,
            braid.was_attributed_to.clone(),
        ).await
    }
    
    fn canonicalize(&self, braid: &Braid) -> Result<Vec<u8>> {
        let mut signing_input = braid.clone();
        signing_input.witness = Witness::unsigned();
        Ok(serde_json::to_vec(&signing_input)?)
    }
}
```

---

## 5. Compute Integration

Compute capability provides task execution events for provenance tracking.

*Note: Currently implemented by ToadStool, but SweetGrass discovers by capability, not name.*

### 5.1 tarpc Client Trait

```rust
/// Compute tarpc service (capability-based)
#[tarpc::service]
pub trait ComputeRpc {
    /// Subscribe to task completion events
    async fn subscribe_completions() -> Result<SubscriptionId, ToadStoolError>;
    
    /// Poll for completions
    async fn poll_completions(
        subscription_id: SubscriptionId,
    ) -> Result<Vec<TaskCompletion>, ToadStoolError>;
    
    /// Get task details
    async fn get_task(task_id: TaskId) -> Result<TaskDetails, ToadStoolError>;
    
    /// Get task inputs
    async fn get_task_inputs(task_id: TaskId) -> Result<Vec<TaskInput>, ToadStoolError>;
    
    /// Get task outputs
    async fn get_task_outputs(task_id: TaskId) -> Result<Vec<TaskOutput>, ToadStoolError>;
}

/// Task completion event
#[derive(Clone, Debug, Serialize, Deserialize)]
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

### 5.2 Task Processing

```rust
/// Process compute tasks for Braid creation
pub struct ComputeProcessor {
    client: Arc<dyn ComputeClient>,
    factory: Arc<BraidFactory>,
    store: Arc<dyn BraidStore>,
}

impl ToadStoolProcessor {
    pub async fn process_completion(&self, completion: TaskCompletion) -> Result<Vec<Braid>> {
        // Skip failed tasks
        if completion.status != TaskStatus::Completed {
            return Ok(vec![]);
        }
        
        let mut braids = Vec::new();
        
        for output in &completion.outputs {
            let activity = Activity::builder(ActivityType::Computation)
                .with_id(ActivityId::from_task(&completion.task_id))
                .started_at_time(completion.started_at)
                .ended_at_time(Some(completion.ended_at))
                .associated_with(AgentAssociation::new(
                    completion.executor.clone(),
                    AgentRole::ComputeProvider,
                ))
                .ecop(ActivityEcoPrimals {
                    compute_units: Some(completion.compute_units),
                    toadstool_task: Some(completion.task_id.clone()),
                    rhizo_session: completion.rhizo_session.clone(),
                    ..Default::default()
                })
                .build();
            
            let braid = self.factory.create_from_data_spec(
                output.data_hash.clone(),
                output.mime_type.clone(),
                output.size,
                completion.executor.clone(),
                Some(activity),
                completion.inputs.iter().map(|i| {
                    EntityReference::by_hash(&i.data_hash)
                }).collect(),
                BraidMetadata::default(),
                Default::default(),
            )?;
            
            self.store.put(&braid).await?;
            braids.push(braid);
        }
        
        Ok(braids)
    }
}
```

---

## 6. sunCloud Integration

sunCloud queries SweetGrass for attribution data.

### 6.1 SweetGrass as Attribution Provider

```rust
/// SweetGrass implements attribution provider for sunCloud
#[tarpc::service]
pub trait SunCloudAttributionRpc {
    /// Get attribution chain for value distribution
    async fn get_attribution(entity: EntityReference) -> Result<AttributionChain, AttributionError>;
    
    /// Calculate reward shares
    async fn calculate_shares(
        entity: EntityReference,
        total_value: f64,
    ) -> Result<Vec<RewardShare>, AttributionError>;
    
    /// Get agent's total contributions
    async fn get_agent_contributions(
        agent: Did,
        time_range: Option<TimeRange>,
    ) -> Result<AgentContributions, AttributionError>;
    
    /// Record distribution for auditing
    async fn record_distribution(
        entity: EntityReference,
        distributions: Vec<RewardDistribution>,
    ) -> Result<DistributionReceipt, AttributionError>;
    
    /// Verify attribution calculation
    async fn verify_attribution(chain: AttributionChain) -> Result<VerificationResult, AttributionError>;
}

impl SunCloudAttributionRpc for SweetGrassServer {
    async fn get_attribution(
        self,
        _: Context,
        entity: EntityReference,
    ) -> Result<AttributionChain, AttributionError> {
        let hash = entity.content_hash()
            .ok_or(AttributionError::InvalidEntity)?;
        self.query.attribution_chain(hash).await
            .map_err(|e| AttributionError::Query(e.to_string()))
    }
    
    async fn calculate_shares(
        self,
        _: Context,
        entity: EntityReference,
        total_value: f64,
    ) -> Result<Vec<RewardShare>, AttributionError> {
        let chain = self.get_attribution(entity).await?;
        
        Ok(chain.contributors.iter().map(|c| {
            RewardShare {
                agent: c.agent.clone(),
                share: c.share,
                amount: c.share * total_value,
                role: c.role.clone(),
            }
        }).collect())
    }
    
    async fn record_distribution(
        self,
        _: Context,
        entity: EntityReference,
        distributions: Vec<RewardDistribution>,
    ) -> Result<DistributionReceipt, AttributionError> {
        // Create a Braid recording the distribution
        let braid = self.factory.create_distribution_record(&entity, &distributions)?;
        
        // Anchor to LoamSpine for permanent record
        let anchor = self.anchor_manager.anchor(&braid, self.config.distribution_spine.clone()).await
            .map_err(|e| AttributionError::Anchor(e.to_string()))?;
        
        Ok(DistributionReceipt {
            entity,
            distributions,
            total_distributed: distributions.iter().map(|d| d.amount).sum(),
            timestamp: current_timestamp_nanos(),
            braid_id: braid.id,
            anchor: Some(anchor),
        })
    }
}
```

---

## 7. Songbird Integration

Songbird provides service discovery for SweetGrass.

### 7.1 Registration

```rust
/// Register SweetGrass with Songbird via tarpc
pub async fn register_with_songbird(
    songbird_addr: &str,
    sweetgrass_config: &SweetGrassConfig,
) -> Result<RegistrationReceipt> {
    let client = SongbirdClient::connect(songbird_addr).await?;
    
    let service_info = ServiceInfo {
        name: "sweetgrass".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        address: sweetgrass_config.bind_address.clone(),
        tarpc_port: sweetgrass_config.tarpc_port,
        http_port: sweetgrass_config.http_port,
        capabilities: vec![
            "sweetgrass:braid:create".to_string(),
            "sweetgrass:braid:query".to_string(),
            "sweetgrass:provenance:graph".to_string(),
            "sweetgrass:attribution:calculate".to_string(),
            "sweetgrass:prov-o:export".to_string(),
        ],
        metadata: Default::default(),
    };
    
    client.register_service(service_info).await
}
```

---

## 8. Connection Pool

For high-performance inter-primal communication:

```rust
/// Connection pool for tarpc clients
pub struct TarpcPool<C> {
    connections: Arc<RwLock<Vec<C>>>,
    factory: Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<C>> + Send>> + Send + Sync>,
    max_connections: usize,
}

impl<C: Clone + Send + Sync + 'static> TarpcPool<C> {
    pub async fn get(&self) -> Result<C> {
        // Try to get existing connection
        {
            let conns = self.connections.read().await;
            if !conns.is_empty() {
                return Ok(conns[fastrand::usize(..conns.len())].clone());
            }
        }
        
        // Create new connection
        let conn = (self.factory)().await?;
        {
            let mut conns = self.connections.write().await;
            if conns.len() < self.max_connections {
                conns.push(conn.clone());
            }
        }
        Ok(conn)
    }
}
```

---

## 9. Error Handling

```rust
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum IntegrationError {
    #[error("Session events error: {0}")]
    SessionEvents(String),
    
    #[error("Anchoring error: {0}")]
    Anchoring(String),
    
    #[error("Signing error: {0}")]
    Signing(String),
    
    #[error("Compute error: {0}")]
    Compute(String),
    
    #[error("Discovery error: {0}")]
    Discovery(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Processing error: {0}")]
    Processing(String),
}
```

---

## 10. Configuration

Configuration uses environment variables for zero-knowledge startup:

```bash
# Discovery service (optional - enables runtime discovery)
SONGBIRD_ADDRESS=localhost:8091

# Capability-specific overrides (optional - discovered if not set)
SESSION_EVENTS_ADDRESS=localhost:8092
ANCHORING_ADDRESS=localhost:8093
SIGNING_ADDRESS=localhost:8094
COMPUTE_ADDRESS=localhost:8095
```

```toml
[integration]
# Connection pool settings
pool_size = 4

# Retry settings
max_retries = 3
retry_delay_ms = 100
timeout_ms = 5000

# Capability requirements
required_capabilities = ["signing", "anchoring"]
optional_capabilities = ["session_events", "compute"]
```

*Note: Addresses are discovered at runtime via Songbird. Environment variables are fallbacks.*

---

## 11. References

- [PRIMAL_SOVEREIGNTY.md](./PRIMAL_SOVEREIGNTY.md) — Pure Rust principles
- [API_SPECIFICATION.md](./API_SPECIFICATION.md) — API definitions
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture

---

*SweetGrass: Pure Rust integration with the ecoPrimals ecosystem.*
