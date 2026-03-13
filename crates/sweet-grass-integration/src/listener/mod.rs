// SPDX-License-Identifier: AGPL-3.0-only
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

use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};
use crate::error::IntegrationError;
use crate::Result;

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
#[async_trait]
pub trait SessionEventStream: Send + Sync {
    /// Get the next event.
    async fn next(&mut self) -> Option<SessionEvent>;

    /// Close the stream.
    async fn close(&mut self);
}

/// Event handler that processes session events using discovery.
pub struct EventHandler {
    /// Discovery service for capability-based primal lookup.
    /// Used for reconnection and failover scenarios.
    #[allow(dead_code)]
    discovery: Arc<dyn PrimalDiscovery>,
    session_client: Arc<dyn SessionEventsClient>,
    compression: Arc<sweet_grass_compression::CompressionEngine>,
    store: Arc<dyn sweet_grass_store::BraidStore>,
}

impl EventHandler {
    /// Create a new event handler using discovery.
    #[allow(dead_code)]
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
            session_client,
            compression,
            store,
        })
    }

    /// Create with an existing client.
    #[allow(dead_code)]
    pub fn with_client(
        client: Arc<dyn SessionEventsClient>,
        compression: Arc<sweet_grass_compression::CompressionEngine>,
        store: Arc<dyn sweet_grass_store::BraidStore>,
    ) -> Self {
        let discovery = Arc::new(crate::discovery::LocalDiscovery::new());
        Self {
            discovery,
            session_client: client,
            compression,
            store,
        }
    }

    /// Start processing events.
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut stream = self.session_client.subscribe().await?;

        while let Some(event) = stream.next().await {
            if let Err(e) = self.process_event(event).await {
                tracing::error!("Failed to process event: {e}");
            }
        }

        Ok(())
    }

    /// Process a single event.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    async fn compress_and_store(&self, mut session: Session) -> Result<()> {
        session.finalize(SessionOutcome::Committed);

        let result = self.compression.compress(&session)?;

        for braid in result.braids() {
            self.store.put(braid).await?;
            tracing::info!("Stored Braid: {}", braid.id);
        }

        Ok(())
    }

    /// Get the underlying client.
    #[allow(dead_code)]
    #[must_use]
    pub fn client(&self) -> &dyn SessionEventsClient {
        self.session_client.as_ref()
    }
}

// ============================================================================
// Test-only implementations
// ============================================================================

/// Test-only module containing mock implementations.
#[cfg(any(test, feature = "test-support"))]
#[allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::cast_sign_loss
)]
pub mod testing {
    use super::{
        async_trait, Arc, Result, Session, SessionEvent, SessionEventStream, SessionEventsClient,
    };
    use std::collections::VecDeque;
    use tokio::sync::Mutex;

    /// Mock session events client for testing.
    pub struct MockSessionEventsClient {
        sessions: std::sync::RwLock<std::collections::HashMap<String, Session>>,
        events: Arc<Mutex<VecDeque<SessionEvent>>>,
        healthy: bool,
    }

    impl MockSessionEventsClient {
        /// Create a new mock client.
        #[must_use]
        pub fn new() -> Self {
            Self {
                sessions: std::sync::RwLock::new(std::collections::HashMap::new()),
                events: Arc::new(Mutex::new(VecDeque::new())),
                healthy: true,
            }
        }

        /// Add a session to the mock.
        #[allow(dead_code)]
        pub fn add_session(&self, session: Session) {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session.id.clone(), session);
        }

        /// Queue an event.
        #[allow(dead_code)]
        pub async fn queue_event(&self, event: SessionEvent) {
            let mut events = self.events.lock().await;
            events.push_back(event);
        }

        /// Set health status.
        #[allow(dead_code)]
        #[must_use]
        pub fn with_health(mut self, healthy: bool) -> Self {
            self.healthy = healthy;
            self
        }
    }

    impl Default for MockSessionEventsClient {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl SessionEventsClient for MockSessionEventsClient {
        async fn subscribe(&self) -> Result<Box<dyn SessionEventStream>> {
            Ok(Box::new(MockEventStream {
                events: Arc::clone(&self.events),
            }))
        }

        async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
            let sessions = self.sessions.read().unwrap();
            Ok(sessions.get(session_id).cloned())
        }

        async fn health(&self) -> Result<bool> {
            Ok(self.healthy)
        }
    }

    /// Mock event stream for testing.
    pub struct MockEventStream {
        events: Arc<Mutex<VecDeque<SessionEvent>>>,
    }

    #[async_trait]
    impl SessionEventStream for MockEventStream {
        async fn next(&mut self) -> Option<SessionEvent> {
            let mut events = self.events.lock().await;
            events.pop_front()
        }

        async fn close(&mut self) {
            let mut events = self.events.lock().await;
            events.clear();
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
#[allow(unused_imports)]
pub use testing::MockSessionEventsClient;

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss
)]
mod tests {
    use super::*;
    use sweet_grass_compression::SessionVertex;

    #[test]
    fn test_session_event_type() {
        assert_eq!(SessionEventType::Committed, SessionEventType::Committed);
        assert_ne!(SessionEventType::Started, SessionEventType::Committed);
    }

    #[test]
    fn test_session_event_type_all_variants() {
        let started = SessionEventType::Started;
        let committed = SessionEventType::Committed;
        let rolled_back = SessionEventType::RolledBack;

        assert_ne!(started, committed);
        assert_ne!(committed, rolled_back);
        assert_ne!(started, rolled_back);
    }

    #[test]
    fn test_session_event_structure() {
        let event = SessionEvent {
            session_id: "test-session".to_string(),
            event_type: SessionEventType::Started,
            session: None,
            timestamp: 1_234_567_890,
            agent: Did::new("did:key:z6MkTest"),
        };

        assert_eq!(event.session_id, "test-session");
        assert_eq!(event.event_type, SessionEventType::Started);
        assert!(event.session.is_none());
        assert_eq!(event.timestamp, 1_234_567_890);
    }

    #[test]
    fn test_session_event_with_session() {
        let mut session = Session::new("test-session");
        session.add_vertex(SessionVertex::new(
            "v1",
            "sha256:test",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        ));

        let event = SessionEvent {
            session_id: "test-session".to_string(),
            event_type: SessionEventType::Committed,
            session: Some(session),
            timestamp: 1_234_567_890,
            agent: Did::new("did:key:z6MkTest"),
        };

        assert!(event.session.is_some());
        assert_eq!(event.session.as_ref().unwrap().id, "test-session");
    }

    #[tokio::test]
    async fn test_mock_client_health() {
        let client = testing::MockSessionEventsClient::new();
        let health = client.health().await.expect("health check");
        assert!(health);
    }

    #[tokio::test]
    async fn test_mock_client_health_unhealthy() {
        let client = testing::MockSessionEventsClient::new().with_health(false);
        let health = client.health().await.expect("health check");
        assert!(!health);
    }

    #[tokio::test]
    async fn test_mock_client_subscribe() {
        let client = testing::MockSessionEventsClient::new();

        client
            .queue_event(SessionEvent {
                session_id: "test-session".to_string(),
                event_type: SessionEventType::Started,
                session: None,
                timestamp: chrono::Utc::now().timestamp() as u64,
                agent: Did::new("did:key:z6MkTest"),
            })
            .await;

        let mut stream = client.subscribe().await.expect("subscribe");
        let event = stream.next().await;
        assert!(event.is_some());
        assert_eq!(event.unwrap().session_id, "test-session");

        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_mock_client_subscribe_multiple_events() {
        let client = testing::MockSessionEventsClient::new();

        for i in 0..3 {
            client
                .queue_event(SessionEvent {
                    session_id: format!("session-{i}"),
                    event_type: SessionEventType::Started,
                    session: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    agent: Did::new("did:key:z6MkTest"),
                })
                .await;
        }

        let mut stream = client.subscribe().await.expect("subscribe");
        let mut count = 0;
        while stream.next().await.is_some() {
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_mock_client_get_session() {
        let client = testing::MockSessionEventsClient::new();

        let mut session = Session::new("test-session");
        session.add_vertex(SessionVertex::new(
            "v1",
            "sha256:test",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        ));
        client.add_session(session);

        let retrieved = client.get_session("test-session").await.expect("get");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-session");
    }

    #[tokio::test]
    async fn test_mock_client_get_session_not_found() {
        let client = testing::MockSessionEventsClient::new();

        let retrieved = client.get_session("nonexistent").await.expect("get");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_mock_client_multiple_sessions() {
        let client = testing::MockSessionEventsClient::new();

        for i in 0..5 {
            let mut session = Session::new(format!("session-{i}"));
            session.add_vertex(SessionVertex::new(
                format!("v{i}"),
                format!("sha256:data{i}"),
                "text/plain",
                Did::new("did:key:z6MkTest"),
            ));
            client.add_session(session);
        }

        for i in 0..5 {
            let retrieved = client
                .get_session(&format!("session-{i}"))
                .await
                .expect("get");
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test]
    async fn test_create_session_events_client_async() {
        use crate::discovery::DiscoveredPrimal;
        use sweet_grass_core::config::Capability;

        let test_address = std::env::var("TEST_SESSION_EVENTS_ADDR")
            .unwrap_or_else(|_| format!("localhost:{}", crate::testing::allocate_test_port()));

        let primal = DiscoveredPrimal {
            instance_id: "session-events-1".to_string(),
            name: "TestSessionEventsService".to_string(),
            capabilities: vec![Capability::SessionEvents],
            tarpc_address: Some(test_address),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };

        let client = create_session_events_client_async(&primal)
            .await
            .expect("create client");
        assert!(client.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_mock_event_stream_close() {
        let client = testing::MockSessionEventsClient::new();

        for i in 0..3 {
            client
                .queue_event(SessionEvent {
                    session_id: format!("session-{i}"),
                    event_type: SessionEventType::Started,
                    session: None,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    agent: Did::new("did:key:z6MkTest"),
                })
                .await;
        }

        let mut stream = client.subscribe().await.expect("subscribe");

        assert!(stream.next().await.is_some());

        stream.close().await;

        assert!(stream.next().await.is_none());
    }

    #[test]
    fn test_session_event_type_additional_variants() {
        let vertex_added = SessionEventType::VertexAdded;
        let branch_created = SessionEventType::BranchCreated;
        let branches_merged = SessionEventType::BranchesMerged;

        assert_ne!(vertex_added, branch_created);
        assert_ne!(branch_created, branches_merged);
        assert_ne!(vertex_added, branches_merged);

        assert!(!format!("{vertex_added:?}").is_empty());
        assert!(!format!("{branch_created:?}").is_empty());
        assert!(!format!("{branches_merged:?}").is_empty());
    }

    #[test]
    fn test_session_event_serialization() {
        let event = SessionEvent {
            session_id: "test-session".to_string(),
            event_type: SessionEventType::Committed,
            session: None,
            timestamp: 1_234_567_890,
            agent: Did::new("did:key:z6MkTest"),
        };

        let json = serde_json::to_string(&event).expect("serialize");
        let parsed: SessionEvent = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.session_id, event.session_id);
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.timestamp, event.timestamp);
    }

    #[tokio::test]
    async fn test_mock_client_default() {
        let client = testing::MockSessionEventsClient::default();
        assert!(client.health().await.expect("health"));
    }
}
