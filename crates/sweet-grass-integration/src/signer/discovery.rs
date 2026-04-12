// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Discovery-based signing services.
//!
//! Uses capability-based discovery to find signing primals at runtime.
//!
//! ## Zero-Knowledge Architecture
//!
//! - Discovers primals by `Capability::Signing`, not by name
//! - No hardcoded addresses or ports
//! - Runtime discovery via the universal adapter

use std::sync::Arc;

use tracing::{debug, instrument, warn};

use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::config::Capability;

use crate::Result;
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};
use crate::error::IntegrationError;

use super::traits::{Signer, SigningClient};

/// Signing service that uses capability-based discovery to find signing primals.
///
/// This is the recommended signer implementation - it discovers signing
/// services at runtime based on capabilities, not hardcoded names.
pub struct DiscoverySigner {
    discovery: Arc<dyn PrimalDiscovery>,
    client: Arc<dyn SigningClient>,
    did: Did,
}

impl DiscoverySigner {
    /// Create a new signer using discovery to find a signing primal.
    ///
    /// Discovers any primal offering `Capability::Signing` at runtime.
    ///
    /// # Errors
    ///
    /// Returns an error if no signing primal is discovered or client creation fails.
    #[instrument(skip(discovery, client_factory))]
    pub async fn new<F>(discovery: Arc<dyn PrimalDiscovery>, client_factory: F) -> Result<Self>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn SigningClient>,
    {
        debug!("Discovering signing capability");

        let primal = discovery
            .find_one(&Capability::Signing)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Found primal with signing capability");

        let client = client_factory(&primal);
        let did = client.current_did().await?;

        Ok(Self {
            discovery,
            client,
            did,
        })
    }

    /// Create with an existing client (for testing or when discovery is handled externally).
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving the client's DID fails.
    pub async fn with_client(client: Arc<dyn SigningClient>) -> Result<Self> {
        let did = client.current_did().await?;
        // Use a no-op discovery since we already have the client
        let discovery = Arc::new(crate::discovery::LocalDiscovery::new());

        Ok(Self {
            discovery,
            client,
            did,
        })
    }

    /// Reconnect to a new signing primal if the current one becomes unavailable.
    ///
    /// # Errors
    ///
    /// Returns an error if re-discovery fails or the new client is unreachable.
    #[instrument(skip(self, client_factory))]
    pub async fn reconnect<F>(&mut self, client_factory: F) -> Result<()>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn SigningClient>,
    {
        warn!("Reconnecting to signing capability");

        let primal = self
            .discovery
            .find_one(&Capability::Signing)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Found new primal with signing capability");

        let client = client_factory(&primal);
        let did = client.current_did().await?;

        self.client = client;
        self.did = did;

        Ok(())
    }

    /// Get the underlying client for advanced operations.
    #[must_use]
    pub fn client(&self) -> &dyn SigningClient {
        self.client.as_ref()
    }
}

impl Signer for DiscoverySigner {
    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn sign_braid(&self, braid: &Braid) -> Result<Braid> {
        let witness = self.client.sign(braid).await?;

        let mut signed = braid.clone();
        signed.witness = witness;

        debug!("Braid signed successfully");
        Ok(signed)
    }

    #[instrument(skip(self, braid), fields(braid_id = %braid.id))]
    async fn verify_braid(&self, braid: &Braid) -> Result<bool> {
        let info = self.client.verify(braid).await?;
        debug!(valid = info.valid, "Signature verification complete");
        Ok(info.valid)
    }

    fn signer_did(&self) -> &Did {
        &self.did
    }
}

/// Legacy signer for backward compatibility.
///
/// Prefer `DiscoverySigner` for new code as it uses proper capability-based discovery.
pub struct LegacySigner {
    client: Arc<dyn SigningClient>,
    did: Did,
}

impl LegacySigner {
    /// Create a new signer.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving the client's DID fails.
    pub async fn new(client: Arc<dyn SigningClient>) -> Result<Self> {
        let did = client.current_did().await?;
        Ok(Self { client, did })
    }
}

impl Signer for LegacySigner {
    async fn sign_braid(&self, braid: &Braid) -> Result<Braid> {
        let witness = self.client.sign(braid).await?;

        let mut signed = braid.clone();
        signed.witness = witness;

        Ok(signed)
    }

    async fn verify_braid(&self, braid: &Braid) -> Result<bool> {
        let info = self.client.verify(braid).await?;
        Ok(info.valid)
    }

    fn signer_did(&self) -> &Did {
        &self.did
    }
}
