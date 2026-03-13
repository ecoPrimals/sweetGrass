// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc signing client implementation.
//!
//! Production implementation for connecting to signing services
//! using tarpc over TCP with bincode serialization.
//!
//! ## Zero-Knowledge Architecture
//!
//! This client connects to any primal offering `Capability::Signing`.
//! No hardcoded primal names, ports, or addresses - all discovered at runtime.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, instrument};

use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::BraidSignature;
use sweet_grass_core::Braid;

use crate::discovery::DiscoveredPrimal;
use crate::error::IntegrationError;
use crate::Result;

use super::traits::{SignatureInfo, SigningClient};

/// tarpc service definition for signing capability.
///
/// This interface is implemented by any primal offering `Capability::Signing`.
#[tarpc::service]
pub trait SigningRpc {
    /// Sign a braid.
    async fn sign_braid(braid_bytes: bytes::Bytes) -> std::result::Result<bytes::Bytes, String>;

    /// Verify a braid signature.
    async fn verify_braid(braid_bytes: bytes::Bytes) -> std::result::Result<bool, String>;

    /// Get the current signer DID.
    async fn current_did() -> std::result::Result<String, String>;

    /// Resolve a DID to its document.
    async fn resolve_did(did: String) -> std::result::Result<Option<String>, String>;

    /// Health check.
    async fn health() -> std::result::Result<bool, String>;
}

// Note: BearDogRpc alias removed - use SigningRpc directly
// The tarpc macro creates SigningRpc, SigningRpcClient, etc.

/// Real tarpc client for connecting to a signing service.
///
/// This is the production implementation that connects to any primal
/// offering the `Capability::Signing` capability.
///
/// ## Discovery Pattern
///
/// ```rust,ignore
/// // Discover signing capability at runtime
/// let discovery = create_discovery().await;
/// let primal = discovery.find_one(&Capability::Signing).await?;
/// let client = TarpcSigningClient::from_primal(&primal).await?;
/// ```
pub struct TarpcSigningClient {
    client: SigningRpcClient,
}

// ============================================================================
// CAPABILITY-BASED ARCHITECTURE (v0.5.0+)
// ============================================================================
// Deprecated primal-specific type aliases removed (Dec 24, 2025).
// Use TarpcSigningClient for capability-based architecture.
// See DEPRECATED_ALIASES_REMOVAL_PLAN.md for migration details.
// ============================================================================

impl TarpcSigningClient {
    /// Connect to a signing service at the given address.
    ///
    /// The address is typically discovered via capability-based discovery,
    /// not hardcoded.
    #[instrument(skip_all, fields(addr = %addr))]
    pub async fn connect(addr: &str) -> Result<Self> {
        use tarpc::serde_transport::tcp;
        use tarpc::tokio_serde::formats::Bincode;

        debug!("Connecting to signing service at {}", addr);

        let transport = tcp::connect(addr, Bincode::default)
            .await
            .map_err(|e| IntegrationError::Connection(format!("Failed to connect: {e}")))?;

        let client = SigningRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        debug!("Connected to signing service");
        Ok(Self { client })
    }

    /// Create from a discovered primal.
    ///
    /// This is the recommended pattern - discover the primal first,
    /// then create the client from its address.
    pub async fn from_primal(primal: &DiscoveredPrimal) -> Result<Self> {
        let addr = primal.tarpc_address.as_ref().ok_or_else(|| {
            IntegrationError::Discovery("Primal has no tarpc address".to_string())
        })?;
        Self::connect(addr).await
    }
}

#[async_trait]
impl SigningClient for TarpcSigningClient {
    async fn sign(&self, braid: &Braid) -> Result<BraidSignature> {
        let braid_bytes = bytes::Bytes::from(
            serde_json::to_vec(braid)
                .map_err(|e| IntegrationError::Serialization(e.to_string()))?,
        );

        let result = self
            .client
            .sign_braid(tarpc::context::current(), braid_bytes)
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Signing)?;

        let signature: BraidSignature = serde_json::from_slice(&result)
            .map_err(|e| IntegrationError::Serialization(e.to_string()))?;

        Ok(signature)
    }

    async fn verify(&self, braid: &Braid) -> Result<SignatureInfo> {
        let braid_bytes = bytes::Bytes::from(
            serde_json::to_vec(braid)
                .map_err(|e| IntegrationError::Serialization(e.to_string()))?,
        );

        let valid = self
            .client
            .verify_braid(tarpc::context::current(), braid_bytes)
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Signing)?;

        let signer = braid.was_attributed_to.clone();
        let now = chrono::Utc::now();

        #[allow(clippy::cast_sign_loss)]
        let signed_at = now.timestamp().max(0) as u64;

        Ok(SignatureInfo {
            signer,
            algorithm: "Ed25519Signature2020".to_string(),
            signed_at,
            valid,
        })
    }

    async fn current_did(&self) -> Result<Did> {
        let did_str = self
            .client
            .current_did(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Signing)?;

        Ok(Did::new(did_str))
    }

    async fn resolve_did(&self, did: &Did) -> Result<Option<serde_json::Value>> {
        let doc_str = self
            .client
            .resolve_did(tarpc::context::current(), did.as_str().to_string())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Signing)?;

        match doc_str {
            Some(s) => {
                let doc: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|e| IntegrationError::Serialization(e.to_string()))?;
                Ok(Some(doc))
            },
            None => Ok(None),
        }
    }

    async fn health(&self) -> Result<bool> {
        self.client
            .health(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Connection)
    }
}

/// Async factory function to create a signing client from a discovered primal.
///
/// This is the recommended way to create clients in production code.
/// It uses capability-based discovery and connects via tarpc.
///
/// In test mode (with `test-support` feature), returns a mock client.
///
/// # Errors
///
/// Returns an error if the primal doesn't have a tarpc address configured,
/// or if the connection to the signing service fails.
pub async fn create_signing_client_async(
    primal: &DiscoveredPrimal,
) -> std::result::Result<Arc<dyn SigningClient>, IntegrationError> {
    #[cfg(any(test, feature = "test-support"))]
    {
        // In test mode, return mock client
        let _ = primal; // Silence unused warning
        Ok(Arc::new(super::testing::MockSigningClient::new()))
    }
    #[cfg(not(any(test, feature = "test-support")))]
    {
        // In production, connect via tarpc
        let client = TarpcSigningClient::from_primal(primal).await?;
        Ok(Arc::new(client))
    }
}

// ============================================================================
// CAPABILITY-BASED ARCHITECTURE (v0.5.0+)
// ============================================================================
// Deprecated primal-specific functions removed (Dec 24, 2025).
// Use create_signing_client_async() for capability-based discovery.
// See DEPRECATED_ALIASES_REMOVAL_PLAN.md for migration details.
// ============================================================================
