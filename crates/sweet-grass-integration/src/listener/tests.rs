// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for session events listener.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    reason = "test module: expect/unwrap are standard in tests"
)]

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

#[tokio::test]
async fn test_event_handler_new_discovery_failure() {
    use crate::discovery::{LocalDiscovery, PrimalDiscovery};
    use std::sync::Arc;

    let discovery: Arc<dyn PrimalDiscovery> = Arc::new(LocalDiscovery::new());
    let compression = Arc::new(sweet_grass_compression::CompressionEngine::new(Arc::new(
        sweet_grass_factory::BraidFactory::new(Did::new("did:key:z6MkTest")),
    )));
    let store: Arc<dyn sweet_grass_store::BraidStore> =
        Arc::new(sweet_grass_store::MemoryStore::new());

    let result = EventHandler::new(discovery, compression, store, |_| {
        Arc::new(testing::MockSessionEventsClient::new())
    })
    .await;

    let err = result.err().expect("should fail");
    assert!(
        err.to_string().to_lowercase().contains("capability"),
        "error should mention capability: {err}"
    );
}

#[tokio::test]
async fn test_event_handler_start_processes_committed_event() {
    use std::sync::Arc;

    let client = Arc::new(testing::MockSessionEventsClient::new());
    let compression = Arc::new(sweet_grass_compression::CompressionEngine::new(Arc::new(
        sweet_grass_factory::BraidFactory::new(Did::new("did:key:z6MkTest")),
    )));
    let store: Arc<dyn sweet_grass_store::BraidStore> =
        Arc::new(sweet_grass_store::MemoryStore::new());

    let mut session = Session::new("compress-test");
    session.add_vertex(
        SessionVertex::new(
            "v1",
            "sha256:root",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        )
        .with_size(100)
        .committed(),
    );
    session.add_vertex(
        SessionVertex::new(
            "v2",
            "sha256:derived",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        )
        .with_parent("v1")
        .with_size(200)
        .committed(),
    );

    client
        .queue_event(SessionEvent {
            session_id: "compress-test".to_string(),
            event_type: SessionEventType::Committed,
            session: Some(session),
            timestamp: chrono::Utc::now().timestamp() as u64,
            agent: Did::new("did:key:z6MkTest"),
        })
        .await;

    let handler = EventHandler::with_client(client, compression, Arc::clone(&store));
    handler.start().await.expect("start");

    let result = store
        .query(
            &sweet_grass_store::QueryFilter::new(),
            sweet_grass_store::QueryOrder::NewestFirst,
        )
        .await
        .expect("query");
    assert!(
        !result.braids.is_empty(),
        "compress_and_store should have stored Braids"
    );
}

#[tokio::test]
async fn test_event_handler_start_ignores_rolled_back() {
    use std::sync::Arc;

    let client = Arc::new(testing::MockSessionEventsClient::new());
    let compression = Arc::new(sweet_grass_compression::CompressionEngine::new(Arc::new(
        sweet_grass_factory::BraidFactory::new(Did::new("did:key:z6MkTest")),
    )));
    let store: Arc<dyn sweet_grass_store::BraidStore> =
        Arc::new(sweet_grass_store::MemoryStore::new());

    client
        .queue_event(SessionEvent {
            session_id: "rollback-session".to_string(),
            event_type: SessionEventType::RolledBack,
            session: None,
            timestamp: chrono::Utc::now().timestamp() as u64,
            agent: Did::new("did:key:z6MkTest"),
        })
        .await;

    let handler = EventHandler::with_client(client, compression, Arc::clone(&store));
    handler.start().await.expect("start");

    let result = store
        .query(
            &sweet_grass_store::QueryFilter::new(),
            sweet_grass_store::QueryOrder::NewestFirst,
        )
        .await
        .expect("query");
    assert!(
        result.braids.is_empty(),
        "RolledBack should not store Braids"
    );
}
