// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc transport implementation for session events.

use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, instrument};

use sweet_grass_compression::Session;

use super::{SessionEvent, SessionEventStream, SessionEventsClient};
use crate::discovery::DiscoveredPrimal;
use crate::error::IntegrationError;
use crate::Result;

/// tarpc service definition for session events.
///
/// Generic service interface for any primal offering `Capability::SessionEvents`.
#[tarpc::service]
pub trait SessionEventsRpc {
    /// Subscribe to session events (returns serialized events).
    async fn subscribe() -> std::result::Result<Vec<u8>, String>;

    /// Get a session by ID.
    async fn get_session(session_id: String) -> std::result::Result<Option<Vec<u8>>, String>;

    /// Health check.
    async fn health() -> std::result::Result<bool, String>;
}

/// Real tarpc client for connecting to a session events service.
///
/// This is the production implementation that connects to any primal
/// offering `Capability::SessionEvents` using tarpc over TCP with bincode serialization.
pub struct TarpcSessionEventsClient {
    client: SessionEventsRpcClient,
}

impl TarpcSessionEventsClient {
    /// Connect to a session events service at the given address.
    #[instrument(skip_all, fields(addr = %addr))]
    pub async fn connect(addr: &str) -> Result<Self> {
        use tarpc::serde_transport::tcp;
        use tarpc::tokio_serde::formats::Bincode;

        debug!("Connecting to session events service at {}", addr);

        let transport = tcp::connect(addr, Bincode::default)
            .await
            .map_err(|e| IntegrationError::Connection(format!("Failed to connect: {e}")))?;

        let client =
            SessionEventsRpcClient::new(tarpc::client::Config::default(), transport).spawn();

        debug!("Connected to session events service");
        Ok(Self { client })
    }

    /// Create from a discovered primal.
    pub async fn from_primal(primal: &DiscoveredPrimal) -> Result<Self> {
        let addr = primal.tarpc_address.as_ref().ok_or_else(|| {
            IntegrationError::Discovery("Primal has no tarpc address".to_string())
        })?;
        Self::connect(addr).await
    }
}

#[async_trait]
impl SessionEventsClient for TarpcSessionEventsClient {
    async fn subscribe(&self) -> Result<Box<dyn SessionEventStream>> {
        let events_bytes = self
            .client
            .subscribe(tarpc::context::current())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Subscription)?;

        let events: Vec<SessionEvent> = serde_json::from_slice(&events_bytes)
            .map_err(|e| IntegrationError::Serialization(e.to_string()))?;

        Ok(Box::new(TarpcEventStream {
            events: VecDeque::from(events),
        }))
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let session_bytes = self
            .client
            .get_session(tarpc::context::current(), session_id.to_string())
            .await
            .map_err(|e| IntegrationError::Rpc(e.to_string()))?
            .map_err(IntegrationError::Subscription)?;

        match session_bytes {
            Some(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)
                    .map_err(|e| IntegrationError::Serialization(e.to_string()))?;
                Ok(Some(session))
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

/// tarpc-backed event stream.
struct TarpcEventStream {
    events: VecDeque<SessionEvent>,
}

#[async_trait]
impl SessionEventStream for TarpcEventStream {
    async fn next(&mut self) -> Option<SessionEvent> {
        self.events.pop_front()
    }

    async fn close(&mut self) {
        self.events.clear();
    }
}

/// Async factory function to create a session events client from a discovered primal.
///
/// In test mode, returns a mock client. In production, connects via tarpc.
pub async fn create_session_events_client_async(
    primal: &DiscoveredPrimal,
) -> std::result::Result<Arc<dyn SessionEventsClient>, IntegrationError> {
    #[cfg(any(test, feature = "test-support"))]
    {
        let _ = primal;
        Ok(Arc::new(super::testing::MockSessionEventsClient::new()))
    }
    #[cfg(not(any(test, feature = "test-support")))]
    {
        let client = TarpcSessionEventsClient::from_primal(primal).await?;
        Ok(Arc::new(client))
    }
}
