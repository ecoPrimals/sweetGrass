// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
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

use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::BraidSignature;

use crate::Result;
use crate::discovery::DiscoveredPrimal;
use crate::error::{IntegrationError, IpcErrorPhase};

use super::traits::{SIGNING_ALGORITHM, SignatureInfo, SigningClient};

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

impl TarpcSigningClient {
    /// Connect to a signing service at the given address.
    ///
    /// The address is typically discovered via capability-based discovery,
    /// not hardcoded.
    ///
    /// # Errors
    ///
    /// Returns an error if the TCP connection or tarpc handshake fails.
    #[instrument(skip_all, fields(addr = %addr))]
    pub async fn connect(addr: &str) -> Result<Self> {
        use tarpc::serde_transport::tcp;
        use tarpc::tokio_serde::formats::Bincode;

        debug!("Connecting to signing service at {}", addr);

        let transport = tcp::connect(addr, Bincode::default)
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Connect, format!("signing: {e}")))?;

        let client = SigningRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        debug!("Connected to signing service");
        Ok(Self { client })
    }

    /// Create from a discovered primal.
    ///
    /// This is the recommended pattern - discover the primal first,
    /// then create the client from its address.
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
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("sign_braid: {e}")))?
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
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("verify_braid: {e}")))?
            .map_err(IntegrationError::Signing)?;

        let signer = braid.was_attributed_to.clone();
        let now = chrono::Utc::now();

        let signed_at = u64::try_from(now.timestamp().max(0)).unwrap_or(0);

        Ok(SignatureInfo {
            signer,
            algorithm: SIGNING_ALGORITHM.to_string(),
            signed_at,
            valid,
        })
    }

    async fn current_did(&self) -> Result<Did> {
        let did_str = self
            .client
            .current_did(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("current_did: {e}")))?
            .map_err(IntegrationError::Signing)?;

        Ok(Did::new(did_str))
    }

    async fn resolve_did(&self, did: &Did) -> Result<Option<serde_json::Value>> {
        let doc_str = self
            .client
            .resolve_did(tarpc::context::current(), did.as_str().to_string())
            .await
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("resolve_did: {e}")))?
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
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, format!("health: {e}")))?
            .map_err(|e| IntegrationError::ipc(IpcErrorPhase::Read, e))
    }
}

/// Create a signing client by connecting to a discovered primal via tarpc.
///
/// This is the production factory — always connects to the real primal.
/// Tests should construct `MockSigningClient` directly via the `testing`
/// module rather than going through this factory.
///
/// # Errors
///
/// Returns an error if the primal doesn't have a tarpc address configured,
/// or if the connection to the signing service fails.
pub async fn create_signing_client_async(
    primal: &DiscoveredPrimal,
) -> std::result::Result<Arc<dyn SigningClient>, IntegrationError> {
    let client = TarpcSigningClient::from_primal(primal).await?;
    Ok(Arc::new(client))
}
