//! tarpc server implementation for SweetGrass.
//!
//! Pure Rust RPC server - no gRPC, no protobuf.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use futures::prelude::*;
use tarpc::context::Context;
use tarpc::server::{BaseChannel, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tracing::{info, instrument};

use sweet_grass_compression::{CompressionEngine, CompressionResult, Session};
use sweet_grass_core::{
    agent::Did,
    braid::{Braid, BraidId, ContentHash, SummaryType},
    entity::EntityReference,
};
use sweet_grass_factory::{
    AttributionCalculator, AttributionChain, AttributionConfig, BraidFactory,
};
use sweet_grass_query::{ProvenanceGraph, ProvenanceGraphBuilder, QueryEngine};
use sweet_grass_store::{BraidStore, MemoryStore, QueryFilter, QueryOrder, QueryResult};

use crate::rpc::{
    AgentContributions, CreateBraidRequest, HealthStatus, JsonLdDocument, RewardShare, RpcError,
    ServiceStatus, SweetGrassRpc, TimeRange,
};

/// SweetGrass tarpc server.
#[derive(Clone)]
pub struct SweetGrassServer {
    store: Arc<MemoryStore>,
    factory: Arc<BraidFactory>,
    query: Arc<QueryEngine>,
    compression: Arc<CompressionEngine>,
    attribution: Arc<AttributionCalculator>,
    start_time: Instant,
}

impl SweetGrassServer {
    /// Create a new SweetGrass server.
    #[must_use]
    pub fn new(
        store: Arc<MemoryStore>,
        factory: Arc<BraidFactory>,
        query: Arc<QueryEngine>,
        compression: Arc<CompressionEngine>,
        attribution: Arc<AttributionCalculator>,
    ) -> Self {
        Self {
            store,
            factory,
            query,
            compression,
            attribution,
            start_time: Instant::now(),
        }
    }
}

impl SweetGrassRpc for SweetGrassServer {
    #[instrument(skip(self, _ctx))]
    async fn create_braid(
        self,
        _ctx: Context,
        request: CreateBraidRequest,
    ) -> Result<Braid, RpcError> {
        let braid = self
            .factory
            .from_hash(
                request.data_hash,
                request.mime_type,
                request.size,
                request.metadata,
            )
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        self.store
            .put(&braid)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        info!("Created Braid: {}", braid.id);
        Ok(braid)
    }

    #[instrument(skip(self, _ctx))]
    async fn get_braid(self, _ctx: Context, id: BraidId) -> Result<Option<Braid>, RpcError> {
        self.store
            .get(&id)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn get_braid_by_hash(
        self,
        _ctx: Context,
        hash: ContentHash,
    ) -> Result<Option<Braid>, RpcError> {
        self.store
            .get_by_hash(&hash)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn query_braids(
        self,
        _ctx: Context,
        filter: QueryFilter,
        order: QueryOrder,
    ) -> Result<QueryResult, RpcError> {
        self.store
            .query(&filter, order)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn delete_braid(self, _ctx: Context, id: BraidId) -> Result<bool, RpcError> {
        self.store
            .delete(&id)
            .await
            .map(|_| true)
            .map_err(|e| RpcError::Store(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn provenance_graph(
        self,
        _ctx: Context,
        entity: EntityReference,
        max_depth: u32,
        include_activities: bool,
    ) -> Result<ProvenanceGraph, RpcError> {
        let store = Arc::clone(&self.store) as Arc<dyn BraidStore>;
        let builder = ProvenanceGraphBuilder::new()
            .max_depth(max_depth)
            .include_activities(include_activities);

        builder
            .build(entity, &store)
            .await
            .map_err(|e| RpcError::Query(e.to_string()))
    }

    #[instrument(skip(self, _ctx, config))]
    async fn attribution_chain(
        self,
        _ctx: Context,
        hash: ContentHash,
        config: AttributionConfig,
    ) -> Result<AttributionChain, RpcError> {
        let braid = self
            .store
            .get_by_hash(&hash)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?
            .ok_or_else(|| RpcError::NotFound(format!("Braid not found: {hash}")))?;

        // Use provided config for attribution calculation
        let calculator = AttributionCalculator::with_config(config);
        let chain = calculator.calculate_single(&braid);

        Ok(chain)
    }

    #[instrument(skip(self, _ctx))]
    async fn calculate_rewards(
        self,
        _ctx: Context,
        hash: ContentHash,
        total_value: f64,
    ) -> Result<Vec<RewardShare>, RpcError> {
        let braid = self
            .store
            .get_by_hash(&hash)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?
            .ok_or_else(|| RpcError::NotFound(format!("Braid not found: {hash}")))?;

        let chain = self.attribution.calculate_single(&braid);

        let rewards = chain
            .contributors
            .iter()
            .map(|c| RewardShare {
                agent: c.agent.clone(),
                share: c.share,
                amount: c.share * total_value,
                role: c.role.clone(),
            })
            .collect();

        Ok(rewards)
    }

    #[instrument(skip(self, _ctx))]
    async fn agent_contributions(
        self,
        _ctx: Context,
        agent: Did,
        time_range: Option<TimeRange>,
    ) -> Result<AgentContributions, RpcError> {
        let all_braids = self
            .store
            .by_agent(&agent)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        // Filter by time range if provided
        let braids: Vec<Braid> = if let Some(ref range) = time_range {
            all_braids
                .into_iter()
                .filter(|b| b.generated_at_time >= range.start && b.generated_at_time <= range.end)
                .collect()
        } else {
            all_braids
        };

        let braid_ids: Vec<BraidId> = braids.iter().map(|b| b.id.clone()).collect();

        // Parallelize attribution chain calculations for better performance
        // Each calculation is CPU-bound and independent
        use futures::stream::{self, StreamExt};

        let calculator = Arc::new(self.attribution);
        let agent_clone = agent.clone();

        let shares: Vec<f64> = stream::iter(braids) // Move braids instead of iterating references
            .map(|braid| {
                let calc = Arc::clone(&calculator);
                let agent = agent_clone.clone();
                async move {
                    // Spawn blocking since attribution calculation is CPU-intensive
                    tokio::task::spawn_blocking(move || {
                        let chain = calc.calculate_single(&braid);
                        chain
                            .contributors
                            .iter()
                            .find(|c| c.agent == agent)
                            .map(|c| c.share)
                            .unwrap_or(0.0)
                    })
                    .await
                    .unwrap_or(0.0)
                }
            })
            .buffer_unordered(10) // Process up to 10 braids concurrently
            .collect()
            .await;

        let total_share = shares.iter().sum();

        Ok(AgentContributions {
            agent,
            total_count: braid_ids.len(),
            total_share,
            braids: braid_ids,
        })
    }

    #[instrument(skip(self, _ctx))]
    async fn braids_by_agent(self, _ctx: Context, agent: Did) -> Result<Vec<Braid>, RpcError> {
        self.store
            .by_agent(&agent)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn compress_session(
        self,
        _ctx: Context,
        session: Session,
    ) -> Result<CompressionResult, RpcError> {
        self.compression
            .compress(&session)
            .map_err(|e| RpcError::Compression(e.to_string()))
    }

    #[instrument(skip(self, _ctx))]
    async fn create_meta_braid(
        self,
        _ctx: Context,
        braid_ids: Vec<BraidId>,
        summary_type: SummaryType,
    ) -> Result<Braid, RpcError> {
        let braid = self
            .factory
            .meta_braid(braid_ids, summary_type, None)
            .map_err(|e| RpcError::Internal(e.to_string()))?;

        self.store
            .put(&braid)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        Ok(braid)
    }

    #[instrument(skip(self, _ctx))]
    async fn export_provo(
        self,
        _ctx: Context,
        hash: ContentHash,
    ) -> Result<JsonLdDocument, RpcError> {
        let braid = self
            .store
            .get_by_hash(&hash)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?
            .ok_or_else(|| RpcError::NotFound(format!("Braid not found: {hash}")))?;

        let json_ld = self
            .query
            .export_braid_provo(&braid.data_hash)
            .await
            .map_err(|e| RpcError::Query(e.to_string()))?;

        // Convert to our simplified JsonLdDocument
        Ok(JsonLdDocument {
            content: serde_json::json!({
                "@context": json_ld.context,
                "@graph": json_ld.graph,
            }),
        })
    }

    async fn health_check(self, _ctx: Context) -> Result<HealthStatus, RpcError> {
        let count = self
            .store
            .count(&QueryFilter::default())
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        Ok(HealthStatus {
            status: "UP".to_string(),
            store_status: "ok".to_string(),
            braid_count: count,
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    async fn status(self, _ctx: Context) -> Result<ServiceStatus, RpcError> {
        let count = self
            .store
            .count(&QueryFilter::default())
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        Ok(ServiceStatus {
            healthy: true,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            braid_count: count,
            store_type: "memory".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// Start the tarpc server.
pub async fn start_tarpc_server(
    addr: SocketAddr,
    server: SweetGrassServer,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = tarpc::serde_transport::tcp::listen(&addr, Bincode::default).await?;

    info!("🌾 SweetGrass tarpc server listening on {}", addr);

    // Accept connections and serve
    tokio::pin!(listener);

    while let Some(result) = listener.next().await {
        match result {
            Ok(transport) => {
                let server = server.clone();
                tokio::spawn(async move {
                    let channel = BaseChannel::with_defaults(transport);
                    let () = channel.execute(server.serve()).for_each(|f| f).await;
                });
            },
            Err(e) => {
                tracing::warn!("Failed to accept connection: {}", e);
            },
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::clone_on_ref_ptr
)]
mod tests {
    use super::*;
    use sweet_grass_compression::{SessionOutcome, SessionVertex};
    use sweet_grass_core::agent::Did;
    use tarpc::context;

    fn make_server() -> SweetGrassServer {
        let store = Arc::new(MemoryStore::new());
        let did = Did::new("did:key:z6MkTest");
        let factory = Arc::new(BraidFactory::new(did));
        let query = Arc::new(QueryEngine::new(store.clone()));
        let compression = Arc::new(CompressionEngine::new(factory.clone()));
        let attribution = Arc::new(AttributionCalculator::new());

        SweetGrassServer::new(store, factory, query, compression, attribution)
    }

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    async fn create_test_braid(server: &SweetGrassServer) -> Braid {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let request = CreateBraidRequest {
            data_hash: format!("sha256:test{id}"),
            mime_type: "text/plain".to_string(),
            size: 1024,
            attributed_to: Did::new("did:key:z6MkTest"),
            activity: None,
            derived_from: vec![],
            metadata: None,
        };
        server
            .clone()
            .create_braid(context::current(), request)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = make_server();
        let status = server.health_check(context::current()).await.unwrap();
        assert_eq!(status.status, "UP");
        assert_eq!(status.braid_count, 0);
    }

    #[tokio::test]
    async fn test_status() {
        let server = make_server();
        let status = server.status(context::current()).await.unwrap();
        assert!(status.healthy);
        assert_eq!(status.store_type, "memory");
        assert_eq!(status.braid_count, 0);
    }

    #[tokio::test]
    async fn test_create_and_get_braid() {
        let server = make_server();

        let request = CreateBraidRequest {
            data_hash: "sha256:abc123".to_string(),
            mime_type: "text/plain".to_string(),
            size: 1024,
            attributed_to: Did::new("did:key:z6MkTest"),
            activity: None,
            derived_from: vec![],
            metadata: None,
        };

        let braid = server
            .clone()
            .create_braid(context::current(), request)
            .await
            .unwrap();

        assert_eq!(braid.data_hash, "sha256:abc123");

        let retrieved = server
            .get_braid(context::current(), braid.id.clone())
            .await
            .unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data_hash, "sha256:abc123");
    }

    #[tokio::test]
    async fn test_get_braid_not_found() {
        let server = make_server();
        let result = server
            .get_braid(context::current(), BraidId::new())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_braid_by_hash() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let retrieved = server
            .clone()
            .get_braid_by_hash(context::current(), braid.data_hash.clone())
            .await
            .unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, braid.id);
    }

    #[tokio::test]
    async fn test_get_braid_by_hash_not_found() {
        let server = make_server();
        let result = server
            .get_braid_by_hash(context::current(), "sha256:nonexistent".to_string())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_query_braids() {
        let server = make_server();
        create_test_braid(&server).await;
        create_test_braid(&server).await;

        let result = server
            .query_braids(
                context::current(),
                QueryFilter::new(),
                QueryOrder::NewestFirst,
            )
            .await
            .unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.braids.len(), 2);
    }

    #[tokio::test]
    async fn test_query_braids_with_filter() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let filter = QueryFilter::new().with_hash(braid.data_hash.clone());
        let result = server
            .query_braids(context::current(), filter, QueryOrder::NewestFirst)
            .await
            .unwrap();

        assert_eq!(result.total_count, 1);
    }

    #[tokio::test]
    async fn test_delete_braid() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let deleted = server
            .clone()
            .delete_braid(context::current(), braid.id.clone())
            .await
            .unwrap();

        assert!(deleted);

        let retrieved = server
            .get_braid(context::current(), braid.id)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_braids_by_agent() {
        let server = make_server();
        create_test_braid(&server).await;

        let agent = Did::new("did:key:z6MkTest");
        let braids = server
            .braids_by_agent(context::current(), agent)
            .await
            .unwrap();

        assert_eq!(braids.len(), 1);
    }

    #[tokio::test]
    async fn test_attribution_chain() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let chain = server
            .attribution_chain(
                context::current(),
                braid.data_hash.clone(),
                AttributionConfig::default(),
            )
            .await
            .unwrap();

        // Chain was created successfully
        assert!(!chain.contributors.is_empty());
    }

    #[tokio::test]
    async fn test_attribution_chain_not_found() {
        let server = make_server();

        let result = server
            .attribution_chain(
                context::current(),
                "sha256:nonexistent".to_string(),
                AttributionConfig::default(),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculate_rewards() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let rewards = server
            .calculate_rewards(context::current(), braid.data_hash.clone(), 100.0)
            .await
            .unwrap();

        // Should have at least one contributor
        assert!(!rewards.is_empty());
        // Total should sum close to 100
        let total: f64 = rewards.iter().map(|r| r.amount).sum();
        assert!((total - 100.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_calculate_rewards_not_found() {
        let server = make_server();

        let result = server
            .calculate_rewards(context::current(), "sha256:nonexistent".to_string(), 100.0)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_contributions() {
        let server = make_server();
        create_test_braid(&server).await;
        create_test_braid(&server).await;

        let agent = Did::new("did:key:z6MkTest");
        let contributions = server
            .agent_contributions(context::current(), agent.clone(), None)
            .await
            .unwrap();

        assert_eq!(contributions.agent, agent);
        assert_eq!(contributions.total_count, 2);
        assert_eq!(contributions.braids.len(), 2);
    }

    #[tokio::test]
    async fn test_compress_session() {
        let server = make_server();

        let mut session = Session::new("test-session");
        session.outcome = SessionOutcome::Committed;
        session.add_vertex(
            SessionVertex::new(
                "v1",
                "sha256:test",
                "text/plain",
                Did::new("did:key:z6MkTest"),
            )
            .with_size(100)
            .committed(),
        );

        let result = server
            .compress_session(context::current(), session)
            .await
            .unwrap();

        // Should produce some result
        assert!(result.has_braids() || result.discard_reason().is_some());
    }

    #[tokio::test]
    async fn test_create_meta_braid() {
        let server = make_server();
        let braid1 = create_test_braid(&server).await;
        let braid2 = create_test_braid(&server).await;

        let meta = server
            .create_meta_braid(
                context::current(),
                vec![braid1.id, braid2.id],
                SummaryType::Session {
                    session_id: "test-session".to_string(),
                },
            )
            .await
            .unwrap();

        assert!(matches!(
            meta.braid_type,
            sweet_grass_core::BraidType::Collection { .. }
        ));
    }

    #[tokio::test]
    async fn test_provenance_graph() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let entity = EntityReference::by_hash(&braid.data_hash);
        let graph = server
            .provenance_graph(context::current(), entity, 5, true)
            .await
            .unwrap();

        assert!(!graph.entities.is_empty());
    }

    #[tokio::test]
    async fn test_export_provo() {
        let server = make_server();
        let braid = create_test_braid(&server).await;

        let doc = server
            .clone()
            .export_provo(context::current(), braid.data_hash.clone())
            .await
            .unwrap();

        assert!(doc.content.get("@context").is_some());
    }

    #[tokio::test]
    async fn test_export_provo_not_found() {
        let server = make_server();

        let result = server
            .export_provo(context::current(), "sha256:nonexistent".to_string())
            .await;

        assert!(result.is_err());
    }
}
