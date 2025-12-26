//! Capability-based primal discovery.
//!
//! `SweetGrass` discovers other primals at runtime based on the capabilities
//! they offer, not hardcoded addresses. This module provides the discovery
//! infrastructure.
//!
//! ## Design Principles
//!
//! - **No hardcoded addresses**: Primals are discovered, not configured
//! - **Capability-based**: Find primals by what they can do, not who they are
//! - **Fault-tolerant**: Handle primal availability changes gracefully
//! - **Self-knowledge only**: A primal knows its own capabilities, discovers others

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sweet_grass_core::config::Capability;
use thiserror::Error;
use tokio::sync::RwLock;

/// Discovery error types.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// No primal found offering the required capability.
    #[error("no primal found offering capability: {0:?}")]
    CapabilityNotFound(Capability),

    /// Connection to discovered primal failed.
    #[error("connection failed to {address}: {reason}")]
    ConnectionFailed { address: String, reason: String },

    /// Discovery service unavailable.
    #[error("discovery service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Timeout during discovery.
    #[error("discovery timeout after {0:?}")]
    Timeout(Duration),
}

/// Information about a discovered primal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Unique primal instance ID.
    pub instance_id: String,

    /// Human-readable primal name.
    pub name: String,

    /// Capabilities this primal offers.
    pub capabilities: Vec<Capability>,

    /// tarpc endpoint address (primary protocol).
    pub tarpc_address: Option<String>,

    /// REST endpoint address (fallback).
    pub rest_address: Option<String>,

    /// When this primal was last seen.
    pub last_seen: std::time::SystemTime,

    /// Health status.
    pub healthy: bool,
}

impl DiscoveredPrimal {
    /// Check if this primal offers a capability.
    #[must_use]
    pub fn offers(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Get the preferred connection address.
    #[must_use]
    pub fn preferred_address(&self) -> Option<&str> {
        self.tarpc_address
            .as_deref()
            .or(self.rest_address.as_deref())
    }
}

/// Trait for primal discovery services.
///
/// Implementations may use various discovery mechanisms:
/// - mDNS/DNS-SD for local networks
/// - Bootstrap nodes for wider networks
/// - Static configuration for testing
#[async_trait::async_trait]
pub trait PrimalDiscovery: Send + Sync {
    /// Find primals offering a specific capability.
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError>;

    /// Find a single primal offering a capability (first healthy one).
    async fn find_one(&self, capability: &Capability) -> Result<DiscoveredPrimal, DiscoveryError> {
        let primals = self.find_by_capability(capability).await?;
        primals
            .into_iter()
            .find(|p| p.healthy)
            .ok_or_else(|| DiscoveryError::CapabilityNotFound(capability.clone()))
    }

    /// Announce this primal's capabilities to the network.
    async fn announce(&self, primal: &DiscoveredPrimal) -> Result<(), DiscoveryError>;

    /// Check if discovery service is available.
    async fn health(&self) -> bool;
}

/// In-memory discovery registry for testing and single-node deployments.
///
/// This is NOT a stub - it's a complete implementation suitable for:
/// - Testing
/// - Single-node deployments
/// - Local development
///
/// For production multi-node deployments, use a distributed discovery
/// implementation.
pub struct LocalDiscovery {
    primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
}

impl LocalDiscovery {
    /// Create a new local discovery registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a primal directly (for testing or single-node).
    pub async fn register(&self, primal: DiscoveredPrimal) {
        let mut primals = self.primals.write().await;
        primals.insert(primal.instance_id.clone(), primal);
    }

    /// Unregister a primal.
    pub async fn unregister(&self, instance_id: &str) {
        let mut primals = self.primals.write().await;
        primals.remove(instance_id);
    }

    /// Get all registered primals.
    pub async fn all(&self) -> Vec<DiscoveredPrimal> {
        let primals = self.primals.read().await;
        primals.values().cloned().collect()
    }
}

impl Default for LocalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PrimalDiscovery for LocalDiscovery {
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let primals = self.primals.read().await;
        let matching: Vec<_> = primals
            .values()
            .filter(|p| p.offers(capability))
            .cloned()
            .collect();
        drop(primals); // Release lock before returning
        Ok(matching)
    }

    async fn announce(&self, primal: &DiscoveredPrimal) -> Result<(), DiscoveryError> {
        self.register(primal.clone()).await;
        Ok(())
    }

    async fn health(&self) -> bool {
        true // Local discovery is always available
    }
}

/// Discovery client that caches results and handles failover.
pub struct CachedDiscovery {
    /// Underlying discovery implementation.
    inner: Arc<dyn PrimalDiscovery>,

    /// Cached primal information.
    cache: Arc<RwLock<HashMap<Capability, Vec<DiscoveredPrimal>>>>,

    /// Cache TTL.
    cache_ttl: Duration,
}

impl CachedDiscovery {
    /// Create a new cached discovery client.
    #[allow(dead_code)]
    #[must_use]
    pub fn new(inner: Arc<dyn PrimalDiscovery>, cache_ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Invalidate cache for a capability.
    #[allow(dead_code)]
    pub async fn invalidate(&self, capability: &Capability) {
        let mut cache = self.cache.write().await;
        cache.remove(capability);
    }

    /// Invalidate all cache entries.
    #[allow(dead_code)]
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[async_trait::async_trait]
impl PrimalDiscovery for CachedDiscovery {
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(capability) {
                // Check if any entries are still valid (within TTL)
                let valid: Vec<_> = cached
                    .iter()
                    .filter(|p| {
                        p.last_seen
                            .elapsed()
                            .map(|e| e < self.cache_ttl)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                if !valid.is_empty() {
                    return Ok(valid);
                }
            }
        }

        // Cache miss or expired - query underlying discovery
        let primals = self.inner.find_by_capability(capability).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(capability.clone(), primals.clone());
        }

        Ok(primals)
    }

    async fn announce(&self, primal: &DiscoveredPrimal) -> Result<(), DiscoveryError> {
        self.inner.announce(primal).await
    }

    async fn health(&self) -> bool {
        self.inner.health().await
    }
}

// ============================================================================
// Songbird-based Discovery
// ============================================================================

/// Songbird tarpc service definition for discovery.
#[tarpc::service]
pub trait SongbirdRpc {
    /// Discover services by capability.
    async fn discover_services(capability: String)
        -> std::result::Result<Vec<ServiceInfo>, String>;

    /// Register a service.
    async fn register_service(info: ServiceInfo) -> std::result::Result<String, String>;

    /// Unregister a service.
    async fn unregister_service(service_id: String) -> std::result::Result<(), String>;

    /// Health check.
    async fn health() -> std::result::Result<bool, String>;
}

/// Service information from Songbird.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID.
    pub id: String,
    /// Service name.
    pub name: String,
    /// Service version.
    pub version: String,
    /// tarpc address.
    pub tarpc_address: Option<String>,
    /// REST address.
    pub rest_address: Option<String>,
    /// Capabilities offered.
    pub capabilities: Vec<String>,
    /// Last heartbeat time.
    pub last_seen: u64,
    /// Whether service is healthy.
    pub healthy: bool,
}

impl ServiceInfo {
    /// Convert to `DiscoveredPrimal`.
    #[must_use]
    pub fn to_primal(&self) -> DiscoveredPrimal {
        DiscoveredPrimal {
            instance_id: self.id.clone(),
            name: self.name.clone(),
            capabilities: self
                .capabilities
                .iter()
                .filter_map(|c| Capability::from_string(c))
                .collect(),
            tarpc_address: self.tarpc_address.clone(),
            rest_address: self.rest_address.clone(),
            last_seen: std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(self.last_seen),
            healthy: self.healthy,
        }
    }
}

/// Discovery implementation using Songbird service mesh.
///
/// Connects to a running Songbird rendezvous server for real service discovery.
pub struct SongbirdDiscovery {
    /// tarpc client.
    client: SongbirdRpcClient,
    /// Local fallback for when Songbird is unavailable.
    fallback: LocalDiscovery,
}

impl SongbirdDiscovery {
    /// Connect to a Songbird rendezvous server.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub async fn connect(addr: &str) -> Result<Self, DiscoveryError> {
        use tarpc::serde_transport::tcp;
        use tarpc::tokio_serde::formats::Bincode;

        let transport = tcp::connect(addr, Bincode::default).await.map_err(|e| {
            DiscoveryError::ConnectionFailed {
                address: addr.to_string(),
                reason: e.to_string(),
            }
        })?;

        let client = SongbirdRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        Ok(Self {
            client,
            fallback: LocalDiscovery::new(),
        })
    }

    /// Create from environment configuration.
    ///
    /// Looks for `SONGBIRD_ADDRESS` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable is not set or connection fails.
    pub async fn from_env() -> Result<Self, DiscoveryError> {
        let addr = std::env::var("SONGBIRD_ADDRESS").map_err(|_| {
            DiscoveryError::ServiceUnavailable(
                "SONGBIRD_ADDRESS environment variable not set".to_string(),
            )
        })?;
        Self::connect(&addr).await
    }

    /// Register local fallback primals (for hybrid discovery).
    #[allow(dead_code)]
    pub async fn register_fallback(&self, primal: DiscoveredPrimal) {
        self.fallback.register(primal).await;
    }
}

#[async_trait::async_trait]
impl PrimalDiscovery for SongbirdDiscovery {
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        // Convert capability to string for Songbird query
        let cap_string = capability.to_string();

        // Try Songbird first
        match self
            .client
            .discover_services(tarpc::context::current(), cap_string)
            .await
        {
            Ok(Ok(services)) => {
                let primals: Vec<_> = services.iter().map(ServiceInfo::to_primal).collect();
                if !primals.is_empty() {
                    return Ok(primals);
                }
                // Fall through to local fallback
            },
            Ok(Err(e)) => {
                tracing::warn!("Songbird discovery error: {}", e);
            },
            Err(e) => {
                tracing::warn!("Songbird RPC error: {}", e);
            },
        }

        // Use local fallback
        self.fallback.find_by_capability(capability).await
    }

    async fn announce(&self, primal: &DiscoveredPrimal) -> Result<(), DiscoveryError> {
        let info = ServiceInfo {
            id: primal.instance_id.clone(),
            name: primal.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            tarpc_address: primal.tarpc_address.clone(),
            rest_address: primal.rest_address.clone(),
            capabilities: primal
                .capabilities
                .iter()
                .map(ToString::to_string)
                .collect(),
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            healthy: primal.healthy,
        };

        match self
            .client
            .register_service(tarpc::context::current(), info)
            .await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(DiscoveryError::ServiceUnavailable(e)),
            Err(e) => {
                // Fall back to local registration
                self.fallback.announce(primal).await?;
                tracing::warn!("Songbird unavailable, registered locally: {}", e);
                Ok(())
            },
        }
    }

    async fn health(&self) -> bool {
        match self.client.health(tarpc::context::current()).await {
            Ok(Ok(healthy)) => healthy,
            _ => self.fallback.health().await,
        }
    }
}

/// Create a discovery client based on environment.
///
/// If `SONGBIRD_ADDRESS` is set, connects to Songbird.
/// Otherwise, returns a local discovery instance.
pub async fn create_discovery() -> Arc<dyn PrimalDiscovery> {
    match SongbirdDiscovery::from_env().await {
        Ok(songbird) => {
            tracing::info!("Using Songbird for primal discovery");
            Arc::new(songbird)
        },
        Err(e) => {
            tracing::info!("Using local discovery (Songbird unavailable: {})", e);
            Arc::new(LocalDiscovery::new())
        },
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_test_primal(name: &str, capabilities: Vec<Capability>) -> DiscoveredPrimal {
        // Use OS-allocated ports for test primals
        let [tarpc_port, rest_port] = crate::testing::allocate_test_ports::<2>();
        
        DiscoveredPrimal {
            instance_id: format!("{name}-instance"),
            name: name.to_string(),
            capabilities,
            tarpc_address: Some(format!("localhost:{tarpc_port}")),
            rest_address: Some(format!("localhost:{rest_port}")),
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        }
    }

    #[tokio::test]
    async fn test_local_discovery_register_and_find() {
        let discovery = LocalDiscovery::new();

        let signer = make_test_primal("signer", vec![Capability::Signing]);
        let anchor = make_test_primal("anchor", vec![Capability::Anchoring]);

        discovery.register(signer).await;
        discovery.register(anchor).await;

        let signers = discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(signers.len(), 1);
        assert_eq!(signers[0].name, "signer");

        let anchors = discovery
            .find_by_capability(&Capability::Anchoring)
            .await
            .expect("find");
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].name, "anchor");
    }

    #[tokio::test]
    async fn test_local_discovery_find_one() {
        let discovery = LocalDiscovery::new();
        let signer = make_test_primal("signer", vec![Capability::Signing]);
        discovery.register(signer).await;

        let found = discovery
            .find_one(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(found.name, "signer");

        let not_found = discovery.find_one(&Capability::Compute).await;
        assert!(not_found.is_err());
    }

    #[tokio::test]
    async fn test_discovered_primal_offers() {
        let primal = make_test_primal(
            "multi",
            vec![Capability::Signing, Capability::custom("custom")],
        );

        assert!(primal.offers(&Capability::Signing));
        assert!(primal.offers(&Capability::Custom("custom".to_string())));
        assert!(!primal.offers(&Capability::Anchoring));
    }

    #[tokio::test]
    async fn test_cached_discovery() {
        let inner = Arc::new(LocalDiscovery::new());
        let signer = make_test_primal("signer", vec![Capability::Signing]);
        inner.register(signer).await;

        let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

        // First call populates cache
        let result1 = cached
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(result1.len(), 1);

        // Second call uses cache
        let result2 = cached
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(result2.len(), 1);
    }

    #[tokio::test]
    async fn test_local_discovery_health() {
        let discovery = LocalDiscovery::new();
        assert!(discovery.health().await);
    }

    #[tokio::test]
    async fn test_multiple_primals_same_capability() {
        let discovery = LocalDiscovery::new();
        discovery
            .register(make_test_primal("signer1", vec![Capability::Signing]))
            .await;
        discovery
            .register(make_test_primal("signer2", vec![Capability::Signing]))
            .await;

        let signers = discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(signers.len(), 2);
    }

    #[tokio::test]
    async fn test_unregister() {
        let discovery = LocalDiscovery::new();
        let primal = make_test_primal("signer", vec![Capability::Signing]);
        let id = primal.instance_id.clone();

        discovery.register(primal).await;
        assert_eq!(
            discovery
                .find_by_capability(&Capability::Signing)
                .await
                .expect("find")
                .len(),
            1
        );

        discovery.unregister(&id).await;
        assert_eq!(
            discovery
                .find_by_capability(&Capability::Signing)
                .await
                .expect("find")
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn test_cached_discovery_find_one() {
        let inner = Arc::new(LocalDiscovery::new());
        let signer = make_test_primal("cached-signer", vec![Capability::Signing]);
        inner.register(signer).await;

        let cached = CachedDiscovery::new(inner, Duration::from_secs(60));
        let found = cached.find_one(&Capability::Signing).await.expect("find");
        assert_eq!(found.name, "cached-signer");
    }

    #[tokio::test]
    async fn test_cached_discovery_health() {
        let inner = Arc::new(LocalDiscovery::new());
        let cached = CachedDiscovery::new(inner, Duration::from_secs(60));
        assert!(cached.health().await);
    }

    #[tokio::test]
    async fn test_primal_with_all_capabilities() {
        let primal = make_test_primal(
            "full",
            vec![
                Capability::Signing,
                Capability::Anchoring,
                Capability::SessionEvents,
            ],
        );
        assert!(primal.offers(&Capability::Signing));
        assert!(primal.offers(&Capability::Anchoring));
        assert!(primal.offers(&Capability::SessionEvents));
        assert!(!primal.offers(&Capability::Compute));
    }

    #[tokio::test]
    async fn test_discovery_empty_capabilities() {
        let primal = make_test_primal("empty", vec![]);
        assert!(!primal.offers(&Capability::Signing));
        assert!(!primal.offers(&Capability::Anchoring));
    }

    #[tokio::test]
    async fn test_discovery_update_primal() {
        let discovery = LocalDiscovery::new();

        // Register initial version
        let mut primal = make_test_primal("updatable", vec![Capability::Signing]);
        discovery.register(primal.clone()).await;

        // Re-register with additional capability
        primal.capabilities.push(Capability::Anchoring);
        discovery.register(primal).await;

        // Should have updated capabilities (still only one primal)
        let signers = discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(signers.len(), 1);

        let anchors = discovery
            .find_by_capability(&Capability::Anchoring)
            .await
            .expect("find");
        assert_eq!(anchors.len(), 1);
    }

    #[tokio::test]
    async fn test_discovery_custom_capability() {
        let discovery = LocalDiscovery::new();
        let custom_cap = Capability::custom("my-custom-feature");
        let primal = make_test_primal("custom", vec![custom_cap.clone()]);
        discovery.register(primal).await;

        let found = discovery
            .find_by_capability(&custom_cap)
            .await
            .expect("find");
        assert_eq!(found.len(), 1);
        assert!(found[0].offers(&custom_cap));
    }

    #[tokio::test]
    async fn test_primal_tarpc_address() {
        let primal = make_test_primal("test", vec![Capability::Signing]);
        assert!(primal.tarpc_address.is_some());
        assert!(primal.rest_address.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_discovery_operations() {
        let discovery = Arc::new(LocalDiscovery::new());
        let mut handles = vec![];

        // Spawn multiple concurrent registrations
        for i in 0..10 {
            let d = Arc::clone(&discovery);
            let handle = tokio::spawn(async move {
                let primal = DiscoveredPrimal {
                    instance_id: format!("primal-{i}"),
                    name: format!("primal-{i}"),
                    capabilities: vec![Capability::Signing],
                    tarpc_address: Some(format!("localhost:{}", 8090 + i)),
                    rest_address: None,
                    last_seen: std::time::SystemTime::now(),
                    healthy: true,
                };
                d.register(primal).await;
            });
            handles.push(handle);
        }

        // Wait for all registrations
        for handle in handles {
            handle.await.expect("join");
        }

        // All should be registered
        let found = discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find");
        assert_eq!(found.len(), 10);
    }
}
