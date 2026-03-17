// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
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
    ConnectionFailed {
        /// Socket or network address that was unreachable.
        address: String,
        /// Underlying connection failure detail.
        reason: String,
    },

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
///
/// Wraps any [`PrimalDiscovery`] implementation with a TTL-based cache
/// to reduce repeated network queries for the same capability.
pub struct CachedDiscovery {
    inner: Arc<dyn PrimalDiscovery>,
    cache: Arc<RwLock<HashMap<Capability, Vec<DiscoveredPrimal>>>>,
    cache_ttl: Duration,
}

impl CachedDiscovery {
    /// Create a new cached discovery client.
    #[must_use]
    pub fn new(inner: Arc<dyn PrimalDiscovery>, cache_ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Invalidate cache for a capability.
    pub async fn invalidate(&self, capability: &Capability) {
        let mut cache = self.cache.write().await;
        cache.remove(capability);
    }

    /// Invalidate all cache entries.
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
// Registry-based Discovery (vendor-agnostic)
// ============================================================================

/// tarpc service definition for registry-based discovery.
///
/// Any primal offering `Capability::Discovery` can implement this interface.
/// The name is deliberately vendor-agnostic — primals discover registries
/// by capability, not by name.
#[tarpc::service]
pub trait RegistryRpc {
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

/// Service information returned by a discovery registry.
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
    /// Convert to [`DiscoveredPrimal`].
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

/// Discovery implementation backed by a remote registry service.
///
/// Connects to any primal offering `Capability::Discovery` via tarpc.
/// Falls back to `LocalDiscovery` when the registry is unreachable.
pub struct RegistryDiscovery {
    client: RegistryRpcClient,
    fallback: LocalDiscovery,
}

impl RegistryDiscovery {
    /// Connect to a discovery registry service.
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

        let client = RegistryRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        Ok(Self {
            client,
            fallback: LocalDiscovery::new(),
        })
    }

    /// Create from environment configuration.
    ///
    /// Looks for discovery address in environment variables (in order of preference):
    /// 1. `DISCOVERY_ADDRESS` — generic discovery service
    /// 2. `UNIVERSAL_ADAPTER_ADDRESS` — universal adapter (e.g., service mesh)
    /// 3. `DISCOVERY_BOOTSTRAP` — bootstrap node address
    ///
    /// # Errors
    ///
    /// Returns an error if no environment variable is set or connection fails.
    pub async fn from_env() -> Result<Self, DiscoveryError> {
        Self::from_reader(|key| std::env::var(key).ok()).await
    }

    /// Create from an injectable key reader (DI-friendly).
    ///
    /// # Errors
    ///
    /// Returns an error if no address is found or connection fails.
    pub async fn from_reader(
        reader: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, DiscoveryError> {
        let addr = reader("DISCOVERY_ADDRESS")
            .or_else(|| reader("UNIVERSAL_ADAPTER_ADDRESS"))
            .or_else(|| reader("DISCOVERY_BOOTSTRAP"))
            .ok_or_else(|| {
                DiscoveryError::ServiceUnavailable(
                    "No discovery address found. Set DISCOVERY_ADDRESS or UNIVERSAL_ADAPTER_ADDRESS environment variable".to_string(),
                )
            })?;
        Self::connect(&addr).await
    }

    /// Register local fallback primals (for hybrid discovery).
    pub async fn register_fallback(&self, primal: DiscoveredPrimal) {
        self.fallback.register(primal).await;
    }
}

#[async_trait::async_trait]
impl PrimalDiscovery for RegistryDiscovery {
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let cap_string = capability.to_string();

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
                tracing::warn!("Registry discovery error: {}", e);
            },
            Err(e) => {
                tracing::warn!("Registry RPC error: {}", e);
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
                tracing::warn!("Registry unavailable, registered locally: {}", e);
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
/// If discovery address is set (via `DISCOVERY_ADDRESS`, `UNIVERSAL_ADAPTER_ADDRESS`,
/// or `DISCOVERY_BOOTSTRAP`), connects to that universal adapter service.
///
/// Otherwise, returns a local discovery instance for single-node deployments.
pub async fn create_discovery() -> Arc<dyn PrimalDiscovery> {
    create_discovery_with_reader(|key| std::env::var(key).ok()).await
}

/// Create a discovery client using an injectable key reader (DI-friendly).
pub async fn create_discovery_with_reader(
    reader: impl Fn(&str) -> Option<String>,
) -> Arc<dyn PrimalDiscovery> {
    match RegistryDiscovery::from_reader(reader).await {
        Ok(discovery) => {
            tracing::info!(
                "Using network discovery service (universal adapter) for primal coordination"
            );
            Arc::new(discovery)
        },
        Err(e) => {
            tracing::info!(
                "Using local discovery for single-node deployment (network discovery unavailable: {})",
                e
            );
            Arc::new(LocalDiscovery::new())
        },
    }
}

/// Extract capability method names from a `capability.list` JSON-RPC response.
///
/// Handles multiple response formats across the ecosystem:
///
/// - **Format A** (flat array): `{"methods": ["braid.create", "health.check"]}`
/// - **Format B** (structured domains): `{"domains": {"braid": ["create"], "health": ["check"]}}`
/// - **`capabilities` alias**: Falls back to `capabilities` key if `methods` is absent
///   (neuralSpring S156 ecosystem compat)
/// - **`result` wrapper**: Unwraps `{"result": {...}}` if present
///
/// Returns a sorted, deduplicated `Vec<String>` of `domain.operation` method names.
///
/// # Examples
///
/// ```
/// # use serde_json::json;
/// # use sweet_grass_integration::discovery::extract_capabilities;
/// let flat = json!({"methods": ["braid.create", "health.check"]});
/// assert_eq!(extract_capabilities(&flat), vec!["braid.create", "health.check"]);
///
/// let structured = json!({"domains": {"braid": ["create", "get"], "health": ["check"]}});
/// let caps = extract_capabilities(&structured);
/// assert_eq!(caps, vec!["braid.create", "braid.get", "health.check"]);
/// ```
#[must_use]
pub fn extract_capabilities(response: &serde_json::Value) -> Vec<String> {
    let source = response.get("result").unwrap_or(response);

    if let Some(methods) = source
        .get("methods")
        .or_else(|| source.get("capabilities"))
        .and_then(serde_json::Value::as_array)
    {
        let mut caps: Vec<String> = methods
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(String::from)
            .collect();
        caps.sort();
        caps.dedup();
        return caps;
    }

    if let Some(domains) = source.get("domains").and_then(serde_json::Value::as_object) {
        let mut caps = Vec::new();
        for (domain, ops) in domains {
            if let Some(arr) = ops.as_array() {
                for op in arr {
                    if let Some(s) = op.as_str() {
                        caps.push(format!("{domain}.{s}"));
                    }
                }
            }
        }
        caps.sort();
        caps.dedup();
        return caps;
    }

    Vec::new()
}

#[cfg(test)]
mod tests;
