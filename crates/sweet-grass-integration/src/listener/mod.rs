// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Session events listener.
//!
//! Provides capability-based discovery for subscribing to session events
//! from primals that offer `Capability::SessionEvents`. No specific primal
//! names are hardcoded.

pub mod tarpc_client;

pub use tarpc_client::create_session_events_client_async;

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use sweet_grass_compression::{Session, SessionOutcome};
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::Timestamp;
use sweet_grass_core::config::Capability;

use crate::Result;
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};
use crate::error::IntegrationError;

/// Session event from a primal with `Capability::SessionEvents`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Session ID.
    pub session_id: String,

    /// Event type.
    pub event_type: SessionEventType,

    /// Session snapshot (for resolution events).
    pub session: Option<Session>,

    /// Timestamp.
    pub timestamp: Timestamp,

    /// Agent who triggered the event.
    pub agent: Did,
}

/// Types of session events.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SessionEventType {
    /// Session started.
    Started,

    /// Session committed (ready for compression).
    Committed,

    /// Session rolled back.
    RolledBack,

    /// Vertex added.
    VertexAdded,

    /// Branch created.
    BranchCreated,

    /// Branches merged.
    BranchesMerged,
}

/// Trait for session events client connections.
///
/// Implemented by clients connecting to primals with `Capability::SessionEvents`.
/// Uses `#[async_trait]` for `Arc<dyn SessionEventsClient>` object safety.
#[async_trait]
pub trait SessionEventsClient: Send + Sync {
    /// Subscribe to session events.
    async fn subscribe(&self) -> Result<Box<dyn SessionEventStream>>;

    /// Get a specific session by ID.
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>>;

    /// Check connection health.
    async fn health(&self) -> Result<bool>;
}

/// Stream of session events.
///
/// Uses `#[async_trait]` for `Box<dyn SessionEventStream>` object safety.
#[async_trait]
pub trait SessionEventStream: Send + Sync {
    /// Get the next event.
    async fn next(&mut self) -> Option<SessionEvent>;

    /// Close the stream.
    async fn close(&mut self);
}

/// Event handler that processes session events using discovery.
pub struct EventHandler {
    discovery: Arc<dyn PrimalDiscovery>,
    session_client: parking_lot::RwLock<Arc<dyn SessionEventsClient>>,
    compression: Arc<sweet_grass_compression::CompressionEngine>,
    store: Arc<dyn sweet_grass_store::BraidStore>,
}

impl EventHandler {
    /// Create a new event handler using discovery.
    ///
    /// # Errors
    ///
    /// Returns an error if no primal offering `Capability::SessionEvents` is discovered.
    #[instrument(skip(discovery, compression, store, client_factory))]
    pub async fn new<F>(
        discovery: Arc<dyn PrimalDiscovery>,
        compression: Arc<sweet_grass_compression::CompressionEngine>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
        client_factory: F,
    ) -> Result<Self>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn SessionEventsClient>,
    {
        debug!("Discovering session events capability");

        let primal = discovery
            .find_one(&Capability::SessionEvents)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Found session events primal");

        let session_client = client_factory(&primal);

        Ok(Self {
            discovery,
            session_client: parking_lot::RwLock::new(session_client),
            compression,
            store,
        })
    }

    /// Create with an existing client.
    #[must_use]
    pub fn with_client(
        client: Arc<dyn SessionEventsClient>,
        compression: Arc<sweet_grass_compression::CompressionEngine>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
    ) -> Self {
        let discovery = Arc::new(crate::discovery::LocalDiscovery::new());
        Self {
            discovery,
            session_client: parking_lot::RwLock::new(client),
            compression,
            store,
        }
    }

    /// Re-discover the session events primal and reconnect.
    ///
    /// Uses capability-based discovery to find a (possibly different) primal
    /// offering `Capability::SessionEvents`, then replaces the active client.
    ///
    /// # Errors
    ///
    /// Returns an error if no session-events-capable primal is discovered.
    #[instrument(skip(self, client_factory))]
    pub async fn reconnect<F>(&self, client_factory: F) -> Result<()>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<dyn SessionEventsClient>,
    {
        debug!("Re-discovering session events capability for reconnection");

        let primal = self
            .discovery
            .find_one(&Capability::SessionEvents)
            .await
            .map_err(|e| IntegrationError::Discovery(e.to_string()))?;

        debug!(primal = %primal.name, "Reconnected to session events primal");

        let new_client = client_factory(&primal);
        *self.session_client.write() = new_client;

        Ok(())
    }

    /// Start processing events.
    ///
    /// # Errors
    ///
    /// Returns an error if subscribing to the event stream fails.
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let client = Arc::clone(&self.session_client.read());
        let mut stream = client.subscribe().await?;

        while let Some(event) = stream.next().await {
            if let Err(e) = self.process_event(event).await {
                tracing::error!("Failed to process event: {e}");
            }
        }

        Ok(())
    }

    /// Process a single event.
    #[instrument(skip(self, event), fields(session_id = %event.session_id))]
    async fn process_event(&self, event: SessionEvent) -> Result<()> {
        match event.event_type {
            SessionEventType::Committed => {
                if let Some(session) = event.session {
                    self.compress_and_store(session).await?;
                }
            },
            SessionEventType::RolledBack => {
                debug!("Session rolled back, no Braids created");
            },
            _ => {
                trace!("Received event: {:?}", event.event_type);
            },
        }

        Ok(())
    }

    /// Compress a session and store resulting Braids.
    async fn compress_and_store(&self, mut session: Session) -> Result<()> {
        session.finalize(SessionOutcome::Committed);

        let result = self.compression.compress(&session)?;

        for braid in result.braids() {
            self.store.put(braid).await?;
            tracing::info!("Stored Braid: {}", braid.id);
        }

        Ok(())
    }

    /// Get the underlying client reference.
    #[must_use]
    pub fn client(&self) -> Arc<dyn SessionEventsClient> {
        Arc::clone(&self.session_client.read())
    }
}

// ============================================================================
// Test-only implementations
// ============================================================================

#[cfg(any(test, feature = "test"))]
pub mod testing;

#[cfg(any(test, feature = "test"))]
pub use testing::MockSessionEventsClient;

#[cfg(test)]
mod tests;
