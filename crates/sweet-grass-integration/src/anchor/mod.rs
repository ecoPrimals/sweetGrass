// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Anchoring integration.
//!
//! Provides capability-based discovery for anchoring Braids to primals
//! that offer permanent storage capabilities. Uses `Capability::Anchoring`
//! for discovery - no specific primal names are hardcoded.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use sweet_grass_core::Braid;
use sweet_grass_core::braid::{BraidId, Timestamp};
use sweet_grass_core::config::Capability;

use crate::Result;
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};
use crate::error::{IntegrationError, IpcErrorPhase};

/// Information about an anchor in a permanent storage primal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorInfo {
    /// Braid ID.
    pub braid_id: BraidId,

    /// Spine ID where anchored.
    pub spine_id: String,

    /// Entry hash in the spine.
    pub entry_hash: String,

    /// Index in the spine.
    pub index: u64,

    /// When the anchor was created.
    pub anchored_at: Timestamp,

    /// Whether the anchor has been verified.
    pub verified: bool,
}

/// Receipt from an anchor operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorReceipt {
    /// Anchor information.
    pub anchor: AnchorInfo,

    /// Transaction ID (if applicable).
    pub transaction_id: Option<String>,

    /// Confirmation count.
    pub confirmations: u32,
}

/// Trait for anchoring client connections.
///
/// Implemented by clients connecting to primals with `Capability::Anchoring`.
#[async_trait]
pub trait AnchoringClient: Send + Sync {
    /// Anchor a Braid to a spine.
    async fn anchor(&self, braid: &Braid, spine_id: &str) -> Result<AnchorReceipt>;

    /// Verify an anchor.
    async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>>;

    /// Get all anchors for a Braid.
    async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>>;

    /// Check connection health.
    async fn health(&self) -> Result<bool>;
}

/// Manages Braid anchoring using capability-based discovery.
pub struct AnchorManager {
    discovery: Arc<dyn PrimalDiscovery>,
    anchoring_client: parking_lot::RwLock<Arc<dyn AnchoringClient>>,
    store: Arc<dyn sweet_grass_store::BraidStore>,
}

impl AnchorManager {
    /// Create a new anchor manager using discovery.
    ///
    /// # Errors
    ///
    /// Returns an error if no primal offering `Capability::Anchoring` is discovered.
    #[instrument(skip(discovery, store, client_factory))]
    pub async fn new<F>(
        discovery: Arc<dyn PrimalDiscovery>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
        client_factory: F,
    ) -> Result<Self>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn AnchoringClient>,
    {
        debug!("Discovering anchoring capability");

        let primal = discovery
            .find_one(&Capability::Anchoring)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Found anchoring primal");

        let anchoring_client = client_factory(&primal);

        Ok(Self {
            discovery,
            anchoring_client: parking_lot::RwLock::new(anchoring_client),
            store,
        })
    }

    /// Create with an existing client.
    #[must_use]
    pub fn with_client(
        client: Arc<dyn AnchoringClient>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
    ) -> Self {
        let discovery = Arc::new(crate::discovery::LocalDiscovery::new());
        Self {
            discovery,
            anchoring_client: parking_lot::RwLock::new(client),
            store,
        }
    }

    /// Re-discover the anchoring primal and reconnect.
    ///
    /// Uses capability-based discovery to find a (possibly different) primal
    /// offering `Capability::Anchoring`, then replaces the active client.
    ///
    /// # Errors
    ///
    /// Returns an error if no anchoring-capable primal is discovered or the
    /// client factory fails.
    #[instrument(skip(self, client_factory))]
    pub async fn reconnect<F>(&self, client_factory: F) -> Result<()>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn AnchoringClient>,
    {
        debug!("Re-discovering anchoring capability for reconnection");

        let primal = self
            .discovery
            .find_one(&Capability::Anchoring)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Reconnected to anchoring primal");

        let new_client = client_factory(&primal);
        *self.anchoring_client.write() = new_client;

        Ok(())
    }

    /// Anchor a Braid by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the Braid is not found or the anchoring request fails.
    #[instrument(skip(self))]
    pub async fn anchor_by_id(&self, braid_id: &BraidId, spine_id: &str) -> Result<AnchorReceipt> {
        let braid =
            self.store.get(braid_id).await?.ok_or_else(|| {
                IntegrationError::Anchoring(format!("Braid not found: {braid_id}"))
            })?;

        let client = Arc::clone(&self.anchoring_client.read());
        client.anchor(&braid, spine_id).await
    }

    /// Verify a Braid's anchor.
    ///
    /// # Errors
    ///
    /// Returns an error if the verification request fails.
    pub async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>> {
        let client = Arc::clone(&self.anchoring_client.read());
        client.verify(braid_id).await
    }

    /// Get all anchors for a Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if the anchor lookup fails.
    pub async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>> {
        let client = Arc::clone(&self.anchoring_client.read());
        client.get_anchors(braid_id).await
    }

    /// Get the underlying client reference.
    #[must_use]
    pub fn client(&self) -> Arc<dyn AnchoringClient> {
        Arc::clone(&self.anchoring_client.read())
    }
}

// ============================================================================
// tarpc Service Definition
// ============================================================================

/// tarpc service definition for anchoring.
///
/// Generic service interface for any primal offering `Capability::Anchoring`.
#[tarpc::service]
pub trait AnchoringRpc {
    /// Anchor a braid (serialized as JSON bytes).
    async fn anchor(
        braid_bytes: bytes::Bytes,
        spine_id: String,
    ) -> std::result::Result<bytes::Bytes, String>;

    /// Verify an anchor.
    async fn verify(braid_id: String) -> std::result::Result<Option<bytes::Bytes>, String>;

    /// Get all anchors for a braid.
    async fn get_anchors(braid_id: String) -> std::result::Result<bytes::Bytes, String>;

    /// Health check.
    async fn health() -> std::result::Result<bool, String>;
}

/// Real tarpc client for connecting to an anchoring service.
///
/// This is the production implementation that connects to any primal
/// offering `Capability::Anchoring` using tarpc over TCP with bincode serialization.
pub struct TarpcAnchoringClient {
    client: AnchoringRpcClient,
}

impl TarpcAnchoringClient {
    /// Connect to an anchoring service at the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if the TCP connection or tarpc handshake fails.
    #[instrument(skip_all, fields(addr = %addr))]
    pub async fn connect(addr: &str) -> Result<Self> {
        use tarpc::serde_transport::tcp;
        use tarpc::tokio_serde::formats::Bincode;

        debug!("Connecting to anchoring service at {}", addr);

        let transport = tcp::connect(addr, Bincode::default).await.map_err(|e| {
            IntegrationError::ipc(IpcErrorPhase::Connect, format!("anchoring: {e}"))
        })?;

        let client = AnchoringRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        debug!("Connected to anchoring service");
        Ok(Self { client })
    }

    /// Create from a discovered primal.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal has no tarpc address or connection fails.
    pub async fn from_primal(primal: &DiscoveredPrimal) -> Result<Self> {
        let addr = primal
            .tarpc_address
            .as_ref()
            .ok_or(IntegrationError::MissingTarpcAddress)?;
        Self::connect(addr).await
    }
}

#[async_trait]
impl AnchoringClient for TarpcAnchoringClient {
    async fn anchor(&self, braid: &Braid, spine_id: &str) -> Result<AnchorReceipt> {
        let braid_bytes = bytes::Bytes::from(
            serde_json::to_vec(braid)
                .map_err(|e| IntegrationError::Serialization(e.to_string()))?,
        );

        let receipt_bytes = self
            .client
            .anchor(tarpc::context::current(), braid_bytes, spine_id.to_string())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("anchor: {e}")))?
            .map_err(IntegrationError::Anchoring)?;

        let receipt: AnchorReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|e| IntegrationError::Serialization(e.to_string()))?;

        Ok(receipt)
    }

    async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>> {
        let anchor_bytes = self
            .client
            .verify(tarpc::context::current(), braid_id.to_string())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("verify: {e}")))?
            .map_err(IntegrationError::Anchoring)?;

        match anchor_bytes {
            Some(bytes) => {
                let anchor: AnchorInfo = serde_json::from_slice(&bytes)
                    .map_err(|e| IntegrationError::Serialization(e.to_string()))?;
                Ok(Some(anchor))
            },
            None => Ok(None),
        }
    }

    async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>> {
        let anchors_bytes = self
            .client
            .get_anchors(tarpc::context::current(), braid_id.to_string())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("get_anchors: {e}")))?
            .map_err(IntegrationError::Anchoring)?;

        let anchors: Vec<AnchorInfo> = serde_json::from_slice(&anchors_bytes)
            .map_err(|e| IntegrationError::Serialization(e.to_string()))?;

        Ok(anchors)
    }

    async fn health(&self) -> Result<bool> {
        self.client
            .health(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("health: {e}")))?
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, e))
    }
}

/// Create an anchoring client by connecting to a discovered primal via tarpc.
///
/// This is the production factory — always connects to the real primal.
/// Tests should construct `MockAnchoringClient` directly via the `testing`
/// module rather than going through this factory.
///
/// # Errors
///
/// Returns an error if the primal has no tarpc address or connection fails.
pub async fn create_anchoring_client_async(
    primal: &DiscoveredPrimal,
) -> std::result::Result<Arc<dyn AnchoringClient>, IntegrationError> {
    let client = TarpcAnchoringClient::from_primal(primal).await?;
    Ok(Arc::new(client))
}

// ============================================================================
// Test-only implementations
// ============================================================================

/// Test-only module containing mock implementations.
#[cfg(any(test, feature = "test"))]
pub mod testing {
    use super::{AnchorInfo, AnchorReceipt, AnchoringClient, Braid, BraidId, Result, async_trait};
    use parking_lot::{RwLock, const_rwlock};

    /// Mock anchoring client for testing.
    pub struct MockAnchoringClient {
        healthy: bool,
        anchors: RwLock<Vec<AnchorInfo>>,
    }

    impl MockAnchoringClient {
        /// Create a new mock client.
        #[must_use]
        pub const fn new() -> Self {
            Self {
                healthy: true,
                anchors: const_rwlock(Vec::new()),
            }
        }

        /// Set health status for testing failure scenarios.
        #[must_use]
        pub const fn with_health(mut self, healthy: bool) -> Self {
            self.healthy = healthy;
            self
        }
    }

    impl Default for MockAnchoringClient {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl AnchoringClient for MockAnchoringClient {
        async fn anchor(&self, braid: &Braid, spine_id: &str) -> Result<AnchorReceipt> {
            let anchor = AnchorInfo {
                braid_id: braid.id.clone(),
                spine_id: spine_id.to_string(),
                entry_hash: format!("entry:{}", braid.data_hash),
                index: 0,
                anchored_at: u64::try_from(chrono::Utc::now().timestamp()).unwrap_or(0),
                verified: false,
            };

            self.anchors.write().push(anchor.clone());

            Ok(AnchorReceipt {
                anchor,
                transaction_id: Some("mock-tx-001".to_string()),
                confirmations: 1,
            })
        }

        async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>> {
            let anchors = self.anchors.read();
            Ok(anchors.iter().find(|a| &a.braid_id == braid_id).cloned())
        }

        async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>> {
            let anchors = self.anchors.read();
            Ok(anchors
                .iter()
                .filter(|a| &a.braid_id == braid_id)
                .cloned()
                .collect())
        }

        async fn health(&self) -> Result<bool> {
            Ok(self.healthy)
        }
    }
}

#[cfg(any(test, feature = "test"))]
pub use testing::MockAnchoringClient;

#[cfg(test)]
mod tests;
