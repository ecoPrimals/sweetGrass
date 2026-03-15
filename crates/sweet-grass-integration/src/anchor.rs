// SPDX-License-Identifier: AGPL-3.0-only
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
use crate::error::IntegrationError;

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
    /// Discovery service for capability-based primal lookup.
    /// Reserved for v0.8.0 deployment (reconnection and failover scenarios).
    #[expect(
        dead_code,
        reason = "Reserved for v0.8.0 reconnection and failover; will be used when discovery-based reconnection is implemented"
    )]
    discovery: Arc<dyn PrimalDiscovery>,
    anchoring_client: Arc<dyn AnchoringClient>,
    /// Braid store for fetching braids by ID in `anchor_by_id`.
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
            anchoring_client,
            store,
        })
    }

    /// Create with an existing client.
    pub fn with_client(
        client: Arc<dyn AnchoringClient>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
    ) -> Self {
        let discovery = Arc::new(crate::discovery::LocalDiscovery::new());
        Self {
            discovery,
            anchoring_client: client,
            store,
        }
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

        self.anchoring_client.anchor(&braid, spine_id).await
    }

    /// Verify a Braid's anchor.
    ///
    /// # Errors
    ///
    /// Returns an error if the verification request fails.
    pub async fn verify(&self, braid_id: &BraidId) -> Result<Option<AnchorInfo>> {
        self.anchoring_client.verify(braid_id).await
    }

    /// Get all anchors for a Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if the anchor lookup fails.
    pub async fn get_anchors(&self, braid_id: &BraidId) -> Result<Vec<AnchorInfo>> {
        self.anchoring_client.get_anchors(braid_id).await
    }

    /// Get the underlying client.
    #[must_use]
    pub fn client(&self) -> &dyn AnchoringClient {
        self.anchoring_client.as_ref()
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

        let transport = tcp::connect(addr, Bincode::default)
            .await
            .map_err(|e| IntegrationError::Connection(format!("Failed to connect: {e}")))?;

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
        let addr = primal.tarpc_address.as_ref().ok_or_else(|| {
            IntegrationError::Discovery("Primal has no tarpc address".to_string())
        })?;
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
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
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
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
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
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Anchoring)?;

        let anchors: Vec<AnchorInfo> = serde_json::from_slice(&anchors_bytes)
            .map_err(|e| IntegrationError::Serialization(e.to_string()))?;

        Ok(anchors)
    }

    async fn health(&self) -> Result<bool> {
        self.client
            .health(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Connection)
    }
}

/// Async factory function to create an anchoring client from a discovered primal.
///
/// ## `#[cfg]` branching (compile-time, not runtime)
///
/// This function uses `#[cfg(any(test, feature = "test"))]` branching.
/// The mock is **only** returned when compiled with `cargo test` or the
/// `test` feature. Production builds always get the real tarpc client.
///
/// # Errors
///
/// Returns an error if the primal has no tarpc address or connection fails.
pub async fn create_anchoring_client_async(
    primal: &DiscoveredPrimal,
) -> std::result::Result<Arc<dyn AnchoringClient>, IntegrationError> {
    #[cfg(any(test, feature = "test"))]
    {
        let _ = primal;
        Ok(Arc::new(testing::MockAnchoringClient::new()))
    }
    #[cfg(not(any(test, feature = "test")))]
    {
        let client = TarpcAnchoringClient::from_primal(primal).await?;
        Ok(Arc::new(client))
    }
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

        /// Set health status.
        #[must_use]
        #[allow(dead_code)]
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
#[allow(unused_imports)]
pub use testing::MockAnchoringClient;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;
    use sweet_grass_core::braid::BraidBuilder;
    use sweet_grass_store::MemoryStore;

    fn create_test_braid() -> Braid {
        BraidBuilder::default()
            .data_hash("sha256:test123")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid")
    }

    fn create_test_braid_with_hash(hash: &str) -> Braid {
        BraidBuilder::default()
            .data_hash(hash)
            .mime_type("text/plain")
            .size(100)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid")
    }

    #[tokio::test]
    async fn test_mock_client_anchor() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid();

        let receipt = client.anchor(&braid, "spine-1").await.expect("anchor");
        assert_eq!(receipt.anchor.spine_id, "spine-1");
        assert!(receipt.transaction_id.is_some());
    }

    #[tokio::test]
    async fn test_mock_client_verify() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid();

        // Anchor first
        client.anchor(&braid, "spine-1").await.expect("anchor");

        // Then verify
        let info = client.verify(&braid.id).await.expect("verify");
        assert!(info.is_some());
        assert_eq!(info.unwrap().braid_id, braid.id);
    }

    #[tokio::test]
    async fn test_mock_client_health() {
        let client = MockAnchoringClient::new();
        assert!(client.health().await.expect("health"));

        let unhealthy = MockAnchoringClient::new().with_health(false);
        assert!(!unhealthy.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_mock_client_get_anchors() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid();

        // No anchors initially
        let anchors = client.get_anchors(&braid.id).await.expect("get");
        assert!(anchors.is_empty());

        // Anchor the braid
        client.anchor(&braid, "spine-1").await.expect("anchor");

        // Now should have one anchor
        let anchors = client.get_anchors(&braid.id).await.expect("get");
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].spine_id, "spine-1");
    }

    #[tokio::test]
    async fn test_mock_client_verify_not_found() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid();

        // Verify without anchoring first
        let info = client.verify(&braid.id).await.expect("verify");
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_anchor_manager_with_client() {
        let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());

        let manager = AnchorManager::with_client(client, store);
        assert!(manager.client().health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_anchor_manager_anchor_by_id() {
        let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
        let braid = create_test_braid_with_hash("sha256:anchor_test");

        // Store the braid first
        store.put(&braid).await.expect("store");

        let manager = AnchorManager::with_client(client, store);
        let receipt = manager
            .anchor_by_id(&braid.id, "test-spine")
            .await
            .expect("anchor");
        assert_eq!(receipt.anchor.spine_id, "test-spine");
    }

    #[tokio::test]
    async fn test_anchor_manager_anchor_by_id_not_found() {
        let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
        let braid = create_test_braid();

        // Don't store the braid
        let manager = AnchorManager::with_client(client, store);
        let result = manager.anchor_by_id(&braid.id, "test-spine").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_anchor_manager_verify() {
        let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
        let braid = create_test_braid_with_hash("sha256:verify_test");

        store.put(&braid).await.expect("store");

        let manager = AnchorManager::with_client(Arc::clone(&client), store);

        // Anchor via the client directly
        client.anchor(&braid, "spine-1").await.expect("anchor");

        // Verify via manager
        let info = manager.verify(&braid.id).await.expect("verify");
        assert!(info.is_some());
    }

    #[tokio::test]
    async fn test_anchor_manager_get_anchors() {
        let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
        let braid = create_test_braid_with_hash("sha256:get_anchors_test");

        store.put(&braid).await.expect("store");

        let manager = AnchorManager::with_client(Arc::clone(&client), store);

        // Anchor via client
        client.anchor(&braid, "spine-1").await.expect("anchor");
        client.anchor(&braid, "spine-2").await.expect("anchor");

        // Get all anchors via manager
        let anchors = manager.get_anchors(&braid.id).await.expect("get");
        assert_eq!(anchors.len(), 2);
    }

    #[tokio::test]
    async fn test_anchor_receipt_structure() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid_with_hash("sha256:receipt_test");

        let receipt = client.anchor(&braid, "test-spine").await.expect("anchor");

        assert_eq!(receipt.anchor.braid_id, braid.id);
        assert_eq!(receipt.anchor.spine_id, "test-spine");
        assert!(receipt.anchor.entry_hash.starts_with("entry:"));
        assert!(!receipt.anchor.verified);
        assert!(receipt.confirmations >= 1);
    }

    #[tokio::test]
    async fn test_anchor_info_structure() {
        let client = MockAnchoringClient::new();
        let braid = create_test_braid_with_hash("sha256:info_test");

        client.anchor(&braid, "spine-test").await.expect("anchor");

        let info = client.verify(&braid.id).await.expect("verify").unwrap();

        assert_eq!(info.braid_id, braid.id);
        assert_eq!(info.spine_id, "spine-test");
        assert!(info.anchored_at > 0);
    }

    #[tokio::test]
    async fn test_create_anchoring_client_async() {
        use crate::discovery::DiscoveredPrimal;
        use sweet_grass_core::config::Capability;

        // Use environment variable or OS-allocated port (zero hardcoding)
        let test_address = std::env::var("TEST_ANCHORING_ADDR")
            .unwrap_or_else(|_| format!("localhost:{}", crate::testing::allocate_test_port()));

        let primal = DiscoveredPrimal {
            instance_id: "anchor-1".to_string(),
            name: "TestAnchoringService".to_string(),
            capabilities: vec![Capability::Anchoring],
            tarpc_address: Some(test_address),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };

        // In test mode, returns a mock client
        let client = create_anchoring_client_async(&primal)
            .await
            .expect("create client");
        assert!(client.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_anchor_manager_new_discovery_failure() {
        use crate::discovery::{LocalDiscovery, PrimalDiscovery};

        let discovery: Arc<dyn PrimalDiscovery> = Arc::new(LocalDiscovery::new());
        let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());

        let result = AnchorManager::new(discovery, store, |_| {
            Arc::new(MockAnchoringClient::new()) as Arc<dyn AnchoringClient>
        })
        .await;

        let err = result.err().expect("should fail");
        assert!(
            err.to_string().to_lowercase().contains("capability"),
            "error should mention capability: {err}"
        );
    }
}
