// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Registry-based remote discovery via tarpc.
//!
//! Connects to any primal offering `Capability::Discovery` — the interface
//! is vendor-agnostic.  Falls back to [`LocalDiscovery`] when the registry
//! is unreachable.

use serde::{Deserialize, Serialize};
use sweet_grass_core::config::Capability;

use super::{DiscoveredPrimal, DiscoveryError, LocalDiscovery, PrimalDiscovery, RegistryError};

/// tarpc service definition for registry-based discovery.
///
/// Any primal offering `Capability::Discovery` can implement this interface.
/// The name is deliberately vendor-agnostic — primals discover registries
/// by capability, not by name.
#[tarpc::service]
pub trait RegistryRpc {
    /// Discover services by capability.
    async fn discover_services(
        capability: String,
    ) -> std::result::Result<Vec<ServiceInfo>, RegistryError>;

    /// Register a service.
    async fn register_service(info: ServiceInfo) -> std::result::Result<String, RegistryError>;

    /// Unregister a service.
    async fn unregister_service(service_id: String) -> std::result::Result<(), RegistryError>;

    /// Health check.
    async fn health() -> std::result::Result<bool, RegistryError>;
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
                    "No discovery address found. Set DISCOVERY_ADDRESS or \
                     UNIVERSAL_ADAPTER_ADDRESS environment variable"
                        .to_string(),
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
            },
            Ok(Err(e)) => {
                tracing::warn!("Registry discovery error: {e}");
            },
            Err(e) => {
                tracing::warn!("Registry RPC error: {}", e);
            },
        }

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
            Ok(Err(e)) => Err(DiscoveryError::ServiceUnavailable(e.to_string())),
            Err(e) => {
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
