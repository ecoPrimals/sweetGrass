// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_compress_session() {
    let server = make_server();

    let mut session = Session::new("test-session");
    session.outcome = SessionOutcome::Committed;
    session.add_vertex(
        SessionVertex::new(
            "v1",
            "sha256:test",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        )
        .with_size(100)
        .committed(),
    );

    let result = server
        .compress_session(context::current(), session)
        .await
        .unwrap();

    assert!(result.has_braids() || result.discard_reason().is_some());
}

#[tokio::test]
async fn test_compress_session_empty_discards() {
    let server = make_server();

    let session = Session::new("empty-session");
    let result = server
        .compress_session(context::current(), session)
        .await
        .unwrap();

    assert!(result.discard_reason().is_some());
    assert!(!result.has_braids());
}

#[tokio::test]
async fn test_compress_session_rollback_discards() {
    let server = make_server();

    let mut session = Session::new("rollback-session");
    session.outcome = SessionOutcome::Rollback;
    session.add_vertex(
        SessionVertex::new(
            "v1",
            "sha256:test",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        )
        .with_size(100)
        .committed(),
    );

    let result = server
        .compress_session(context::current(), session)
        .await
        .unwrap();

    assert!(result.discard_reason().is_some());
}

#[tokio::test]
async fn test_create_meta_braid() {
    let server = make_server();
    let braid1 = create_test_braid(&server).await;
    let braid2 = create_test_braid(&server).await;

    let meta = server
        .create_meta_braid(
            context::current(),
            vec![braid1.id, braid2.id],
            SummaryType::Session {
                session_id: "test-session".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(matches!(
        meta.braid_type,
        sweet_grass_core::BraidType::Collection { .. }
    ));
}

#[tokio::test]
async fn test_create_meta_braid_single_braid() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let meta = server
        .create_meta_braid(
            context::current(),
            vec![braid.id],
            SummaryType::Session {
                session_id: "single-session".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(matches!(
        meta.braid_type,
        sweet_grass_core::BraidType::Collection { .. }
    ));
}
