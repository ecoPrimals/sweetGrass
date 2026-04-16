// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Test-only mock implementations for session events.

use super::{
    Arc, Result, Session, SessionEvent, SessionEventStream, SessionEventStreamBackend,
    SessionEventsClient,
};
use parking_lot::RwLock;
use std::collections::VecDeque;
use tokio::sync::Mutex;

/// Mock session events client for testing.
pub struct MockSessionEventsClient {
    sessions: RwLock<std::collections::HashMap<String, Session>>,
    events: Arc<Mutex<VecDeque<SessionEvent>>>,
    healthy: bool,
}

impl MockSessionEventsClient {
    /// Create a new mock client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(std::collections::HashMap::new()),
            events: Arc::new(Mutex::new(VecDeque::new())),
            healthy: true,
        }
    }

    /// Add a session to the mock.
    pub fn add_session(&self, session: Session) {
        let mut sessions = self.sessions.write();
        sessions.insert(session.id.clone(), session);
    }

    /// Queue an event.
    pub async fn queue_event(&self, event: SessionEvent) {
        let mut events = self.events.lock().await;
        events.push_back(event);
    }

    /// Set health status.
    #[must_use]
    pub const fn with_health(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }
}

impl Default for MockSessionEventsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionEventsClient for MockSessionEventsClient {
    async fn subscribe(&self) -> Result<SessionEventStreamBackend> {
        Ok(SessionEventStreamBackend::Mock(MockEventStream {
            events: Arc::clone(&self.events),
        }))
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read();
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
