// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! tarpc server implementation for `SweetGrass`.
//!
//! Pure Rust RPC server - no gRPC, no protobuf.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use base64::Engine;
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
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder, QueryResult};

use crate::rpc::{
    AgentContributions, CreateBraidRequest, HealthStatus, JsonLdDocument, RewardShare, RpcError,
    ServiceStatus, SweetGrassRpc, TimeRange,
};

/// Default maximum concurrent tarpc requests when processing agent contributions.
const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 10;

/// `SweetGrass` tarpc server.
///
/// Generic over `BraidStore` — shares the same store as the HTTP/JSON-RPC
/// stack when constructed from `AppState`.
#[derive(Clone)]
pub struct SweetGrassServer {
    store: Arc<dyn BraidStore>,
    factory: Arc<BraidFactory>,
    query: Arc<QueryEngine>,
    compression: Arc<CompressionEngine>,
    attribution: Arc<AttributionCalculator>,
    store_backend: String,
    start_time: Instant,
    /// Maximum concurrent requests for parallel operations (e.g. `agent_contributions`).
    /// Configurable via `TARPC_MAX_CONCURRENT_REQUESTS` env var or struct field.
    max_concurrent_requests: usize,
}

impl SweetGrassServer {
    /// Create a new `SweetGrass` server.
    ///
    /// `max_concurrent_requests` defaults to `TARPC_MAX_CONCURRENT_REQUESTS` env var
    /// or 10 if unset.
    #[must_use]
    pub fn new(
        store: Arc<dyn BraidStore>,
        factory: Arc<BraidFactory>,
        query: Arc<QueryEngine>,
        compression: Arc<CompressionEngine>,
        attribution: Arc<AttributionCalculator>,
    ) -> Self {
        let max_concurrent_requests = std::env::var("TARPC_MAX_CONCURRENT_REQUESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_REQUESTS);
        Self {
            store,
            factory,
            query,
            compression,
            attribution,
            store_backend: "unknown".to_string(),
            start_time: Instant::now(),
            max_concurrent_requests,
        }
    }

    /// Create from `AppState` — shares the same store as the HTTP/JSON-RPC stack.
    #[must_use]
    pub fn from_app_state(state: &crate::state::AppState) -> Self {
        let max_concurrent_requests = std::env::var("TARPC_MAX_CONCURRENT_REQUESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_CONCURRENT_REQUESTS);
        Self {
            store: Arc::clone(&state.store),
            factory: Arc::clone(&state.factory),
            query: Arc::clone(&state.query),
            compression: Arc::clone(&state.compression),
            attribution: Arc::new(AttributionCalculator::new()),
            store_backend: state.store_backend.clone(),
            start_time: Instant::now(),
            max_concurrent_requests,
        }
    }

    /// Set the maximum concurrent requests for parallel operations.
    #[must_use]
    pub const fn with_max_concurrent_requests(mut self, n: usize) -> Self {
        self.max_concurrent_requests = n;
        self
    }

    /// Set the store backend label for status reporting.
    #[must_use]
    pub fn with_store_backend(mut self, backend: impl Into<String>) -> Self {
        self.store_backend = backend.into();
        self
    }

    async fn require_braid_by_hash(&self, hash: &ContentHash) -> Result<Braid, RpcError> {
        self.store
            .get_by_hash(hash)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?
            .ok_or_else(|| RpcError::NotFound(format!("Braid not found: {hash}")))
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
        let store = Arc::clone(&self.store);
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
        let braid = self.require_braid_by_hash(&hash).await?;
        let calculator = AttributionCalculator::with_config(config);
        Ok(calculator.calculate_single(&braid))
    }

    #[instrument(skip(self, _ctx))]
    async fn calculate_rewards(
        self,
        _ctx: Context,
        hash: ContentHash,
        total_value: f64,
    ) -> Result<Vec<RewardShare>, RpcError> {
        let braid = self.require_braid_by_hash(&hash).await?;
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
    async fn top_contributors(
        self,
        _ctx: Context,
        hash: ContentHash,
        limit: u32,
    ) -> Result<Vec<RewardShare>, RpcError> {
        let braid = self.require_braid_by_hash(&hash).await?;
        let chain = self.attribution.calculate_single(&braid);

        let mut contributors: Vec<RewardShare> = chain
            .contributors
            .iter()
            .map(|c| RewardShare {
                agent: c.agent.clone(),
                share: c.share,
                amount: 0.0,
                role: c.role.clone(),
            })
            .collect();

        contributors.sort_by(|a, b| {
            b.share
                .partial_cmp(&a.share)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        contributors.truncate(limit as usize);
        Ok(contributors)
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
                            .map_or(0.0, |c| c.share)
                    })
                    .await
                    .unwrap_or(0.0)
                }
            })
            .buffer_unordered(self.max_concurrent_requests)
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
        let braid = self.require_braid_by_hash(&hash).await?;
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

    #[instrument(skip(self, _ctx))]
    async fn export_graph_provo(
        self,
        _ctx: Context,
        entity: EntityReference,
        depth: u32,
    ) -> Result<JsonLdDocument, RpcError> {
        let json_ld = self
            .query
            .export_graph_provo(entity, Some(depth))
            .await
            .map_err(|e| RpcError::Query(e.to_string()))?;

        Ok(JsonLdDocument {
            content: serde_json::json!({
                "@context": json_ld.context,
                "@graph": json_ld.graph,
            }),
        })
    }

    #[instrument(skip(self, _ctx))]
    async fn anchor_braid(
        self,
        _ctx: Context,
        braid_id: BraidId,
        spine_id: String,
    ) -> Result<serde_json::Value, RpcError> {
        let braid = self
            .store
            .get(&braid_id)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?
            .ok_or_else(|| RpcError::NotFound(format!("Braid not found: {braid_id}")))?;

        let hash_bytes = braid
            .data_hash
            .to_bytes32()
            .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
            .ok_or_else(|| {
                RpcError::InvalidInput("Content hash must be sha256 (32 bytes)".to_string())
            })?;

        let uuid_str = braid_id
            .as_str()
            .strip_prefix("urn:braid:")
            .unwrap_or(braid_id.as_str());

        Ok(serde_json::json!({
            "braid_id": uuid_str,
            "spine_id": spine_id,
            "content_hash": hash_bytes,
            "anchored": false,
            "status": "prepared",
        }))
    }

    #[instrument(skip(self, _ctx))]
    async fn verify_anchor(
        self,
        _ctx: Context,
        braid_id: BraidId,
    ) -> Result<serde_json::Value, RpcError> {
        let exists = self
            .store
            .exists(&braid_id)
            .await
            .map_err(|e| RpcError::Store(e.to_string()))?;

        if !exists {
            return Err(RpcError::NotFound(format!("Braid not found: {braid_id}")));
        }

        Ok(serde_json::json!({
            "braid_id": braid_id.as_str(),
            "anchored": false,
            "verification_status": "pending_integration",
        }))
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

    async fn health_liveness(self, _ctx: Context) -> bool {
        true
    }

    async fn health_readiness(self, _ctx: Context) -> bool {
        self.store.count(&QueryFilter::default()).await.is_ok()
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
            store_type: self.store_backend.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}

/// Start the tarpc server with coordinated shutdown.
///
/// Accepts connections until `shutdown` signals. Existing connections
/// continue until their in-flight requests complete.
///
/// # Errors
///
/// Returns an error if binding to the address fails.
pub async fn start_tarpc_server(
    addr: SocketAddr,
    server: SweetGrassServer,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> std::result::Result<(), crate::ServiceError> {
    let listener = tarpc::serde_transport::tcp::listen(&addr, Bincode::default)
        .await
        .map_err(|e| crate::ServiceError::Internal(format!("tarpc bind failed: {e}")))?;

    info!("🌾 SweetGrass tarpc server listening on {}", addr);

    tokio::pin!(listener);

    loop {
        tokio::select! {
            result = listener.next() => {
                match result {
                    Some(Ok(transport)) => {
                        let server = server.clone();
                        tokio::spawn(async move {
                            let channel = BaseChannel::with_defaults(transport);
                            let () = channel.execute(server.serve()).for_each(|f| f).await;
                        });
                    },
                    Some(Err(e)) => {
                        tracing::warn!("Failed to accept connection: {}", e);
                    },
                    None => break,
                }
            }
            _ = shutdown.changed() => {
                info!("tarpc server shutting down");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
