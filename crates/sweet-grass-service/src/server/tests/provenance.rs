// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_provenance_graph() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let entity = EntityReference::by_hash(&braid.data_hash);
    let graph = server
        .provenance_graph(context::current(), entity, 5, true)
        .await
        .unwrap();

    assert!(!graph.entities.is_empty());
}

#[tokio::test]
async fn test_provenance_graph_without_activities() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let entity = EntityReference::by_hash(&braid.data_hash);
    let graph = server
        .provenance_graph(context::current(), entity, 5, false)
        .await
        .unwrap();

    assert!(!graph.entities.is_empty());
}

#[tokio::test]
async fn test_export_provo() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let doc = server
        .clone()
        .export_provo(context::current(), braid.data_hash.clone())
        .await
        .unwrap();

    assert!(doc.content.get("@context").is_some());
}

#[tokio::test]
async fn test_export_provo_not_found() {
    let server = make_server();

    let result = server
        .export_provo(context::current(), "sha256:nonexistent".to_string().into())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_export_graph_provo() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let entity = EntityReference::by_hash(&braid.data_hash);
    let doc = server
        .export_graph_provo(context::current(), entity, 5)
        .await
        .unwrap();

    assert!(doc.content.get("@context").is_some());
    assert!(doc.content.get("@graph").is_some());
}
