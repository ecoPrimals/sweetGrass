// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Session events listener.
//!
//! Provides capability-based discovery for subscribing to session events
//! from primals that offer `Capability::SessionEvents`. No specific primal
//! names are hardcoded.

pub mod tarpc_client;

pub use tarpc_client::create_session_events_client_async;

use std::future::Future;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use sweet_grass_compression::{Session, SessionOutcome};
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::Timestamp;
use sweet_grass_core::config::Capability;

use crate::Result;
use crate::discovery::{DiscoveredPrimal, DiscoveryBackend, PrimalDiscovery};
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

/// Stream of session events.
///
/// Uses native `impl Future + Send` (Rust 2024). Runtime dispatch uses
/// [`SessionEventStreamBackend`].
pub trait SessionEventStream: Send + Sync {
    /// Get the next event.
    fn next(&mut self) -> impl Future<Output = Option<SessionEvent>> + Send;

    /// Close the stream.
    fn close(&mut self) -> impl Future<Output = ()> + Send;
}

/// Unified session event stream for runtime dispatch.
pub enum SessionEventStreamBackend {
    /// Production tarpc-backed stream.
    Tarpc(tarpc_client::TarpcEventStream),
    /// Test-only mock stream.
    #[cfg(any(test, feature = "test"))]
    #[doc(hidden)]
    Mock(testing::MockEventStream),
}

impl SessionEventStream for SessionEventStreamBackend {
    async fn next(&mut self) -> Option<SessionEvent> {
        match self {
            Self::Tarpc(s) => s.next().await,
            #[cfg(any(test, feature = "test"))]
            Self::Mock(m) => m.next().await,
        }
    }

    async fn close(&mut self) {
        match self {
            Self::Tarpc(s) => s.close().await,
            #[cfg(any(test, feature = "test"))]
            Self::Mock(m) => m.close().await,
        }
    }
}

/// Trait for session events client connections.
///
/// Implemented by clients connecting to primals with `Capability::SessionEvents`.
/// Uses native `impl Future + Send` (Rust 2024). Runtime dispatch uses
/// [`SessionEventsBackend`].
pub trait SessionEventsClient: Send + Sync {
    /// Subscribe to session events.
    fn subscribe(&self) -> impl Future<Output = Result<SessionEventStreamBackend>> + Send;

    /// Get a specific session by ID.
    fn get_session(&self, session_id: &str)
    -> impl Future<Output = Result<Option<Session>>> + Send;

    /// Check connection health.
    fn health(&self) -> impl Future<Output = Result<bool>> + Send;
}

/// Unified session events client for runtime dispatch (tarpc production path or test mock).
pub enum SessionEventsBackend {
    /// Production tarpc client.
    Tarpc(tarpc_client::TarpcSessionEventsClient),
    /// Test-only mock client.
    #[cfg(any(test, feature = "test"))]
    #[doc(hidden)]
    Mock(testing::MockSessionEventsClient),
}

impl SessionEventsClient for SessionEventsBackend {
    async fn subscribe(&self) -> Result<SessionEventStreamBackend> {
        match self {
            Self::Tarpc(c) => c.subscribe().await,
            #[cfg(any(test, feature = "test"))]
            Self::Mock(m) => m.subscribe().await,
        }
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        match self {
            Self::Tarpc(c) => c.get_session(session_id).await,
            #[cfg(any(test, feature = "test"))]
            Self::Mock(m) => m.get_session(session_id).await,
        }
    }

    async fn health(&self) -> Result<bool> {
        match self {
            Self::Tarpc(c) => c.health().await,
            #[cfg(any(test, feature = "test"))]
            Self::Mock(m) => m.health().await,
        }
    }
}

/// Event handler that processes session events using discovery.
///
/// Generic over `S: BraidStore` for zero-cost store dispatch.
pub struct EventHandler<S: sweet_grass_store::BraidStore> {
    discovery: Arc<DiscoveryBackend>,
    session_client: parking_lot::RwLock<Arc<SessionEventsBackend>>,
    compression: Arc<sweet_grass_compression::CompressionEngine>,
    store: Arc<S>,
}

impl<S: sweet_grass_store::BraidStore> EventHandler<S> {
    /// Create a new event handler using discovery.
    ///
    /// # Errors
    ///
    /// Returns an error if no primal offering `Capability::SessionEvents` is discovered.
    #[instrument(skip(discovery, compression, store, client_factory))]
    pub async fn new<F>(
        discovery: Arc<DiscoveryBackend>,
        compression: Arc<sweet_grass_compression::CompressionEngine>,
        store: Arc<S>,
        client_factory: F,
    ) -> Result<Self>
    where
        F: FnOnce(&DiscoveredPrimal) -> Arc<SessionEventsBackend>,
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
        client: Arc<SessionEventsBackend>,
        compression: Arc<sweet_grass_compression::CompressionEngine>,
        store: Arc<S>,
    ) -> Self {
        let discovery = Arc::new(DiscoveryBackend::Local(
            crate::discovery::LocalDiscovery::new(),
        ));
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
        F: FnOnce(&DiscoveredPrimal) -> Arc<SessionEventsBackend>,
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
    pub fn client(&self) -> Arc<SessionEventsBackend> {
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
