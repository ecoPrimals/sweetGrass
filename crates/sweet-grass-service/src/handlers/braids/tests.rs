// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use axum::extract::State;

fn make_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

async fn create_test_braid(state: &AppState) -> Braid {
    let braid = state
        .factory
        .from_data(b"test data", "text/plain", None)
        .unwrap();
    state.store.put(&braid).await.unwrap();
    braid
}

#[test]
fn test_list_query_defaults() {
    let query = ListBraidsQuery {
        agent: None,
        tag: None,
        mime_type: None,
        limit: None,
        offset: None,
        order: None,
    };

    assert!(query.agent.is_none());
    assert!(query.limit.is_none());
}

#[tokio::test]
async fn test_get_braid_success() {
    let state = make_state();
    let braid = create_test_braid(&state).await;

    let result = get_braid(State(state), Path(braid.id.to_string())).await;

    assert!(result.is_ok());
    let Json(returned) = result.unwrap();
    assert_eq!(returned.id, braid.id);
}

#[tokio::test]
async fn test_get_braid_not_found() {
    let state = make_state();

    let result = get_braid(State(state), Path("urn:braid:nonexistent".to_string())).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_braid_by_hash_success() {
    let state = make_state();
    let braid = create_test_braid(&state).await;

    let result = get_braid_by_hash(State(state), Path(braid.data_hash.as_str().to_string())).await;

    assert!(result.is_ok());
    let Json(returned) = result.unwrap();
    assert_eq!(returned.data_hash, braid.data_hash);
}

#[tokio::test]
async fn test_get_braid_by_hash_not_found() {
    let state = make_state();

    let result = get_braid_by_hash(State(state), Path("sha256:nonexistent".to_string())).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_braid_success() {
    use base64::Engine;
    let state = make_state();

    let request = CreateBraidRequest {
        data: base64::engine::general_purpose::STANDARD.encode(b"test content"),
        mime_type: "text/plain".to_string(),
        title: Some("Test Braid".to_string()),
        description: Some("A test braid".to_string()),
        tags: Some(vec!["test".to_string()]),
    };

    let result = create_braid(State(state.clone()), Json(request)).await;

    assert!(result.is_ok());
    let (status, Json(response)) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert!(!response.id.is_empty());
    assert!(!response.hash.is_empty());

    // Verify it was stored
    let stored = state
        .store
        .get_by_hash(&sweet_grass_core::ContentHash::new(&response.hash))
        .await
        .unwrap();
    assert!(stored.is_some());
}

#[tokio::test]
async fn test_create_braid_invalid_base64() {
    let state = make_state();

    let request = CreateBraidRequest {
        data: "not-valid-base64!!!".to_string(),
        mime_type: "text/plain".to_string(),
        title: None,
        description: None,
        tags: None,
    };

    let result = create_braid(State(state), Json(request)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_braids_empty() {
    let state = make_state();

    let query = ListBraidsQuery {
        agent: None,
        tag: None,
        mime_type: None,
        limit: None,
        offset: None,
        order: None,
    };

    let result = list_braids(State(state), Query(query)).await;

    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert!(response.braids.is_empty());
    assert_eq!(response.total, 0);
    assert!(!response.has_more);
}

#[tokio::test]
async fn test_list_braids_with_data() {
    let state = make_state();

    // Create some braids
    for i in 0..5 {
        let braid = state
            .factory
            .from_data(format!("test data {i}").as_bytes(), "text/plain", None)
            .unwrap();
        state.store.put(&braid).await.unwrap();
    }

    let query = ListBraidsQuery {
        agent: None,
        tag: None,
        mime_type: None,
        limit: Some(3),
        offset: None,
        order: None,
    };

    let result = list_braids(State(state), Query(query)).await;

    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert_eq!(response.braids.len(), 3);
    assert_eq!(response.total, 5);
    assert!(response.has_more);
}

#[tokio::test]
async fn test_list_braids_with_filters() {
    let state = make_state();
    create_test_braid(&state).await;

    let query = ListBraidsQuery {
        agent: None,
        tag: None,
        mime_type: Some("text/".to_string()),
        limit: None,
        offset: None,
        order: Some("oldest".to_string()),
    };

    let result = list_braids(State(state), Query(query)).await;

    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert_eq!(response.braids.len(), 1);
}

#[tokio::test]
async fn test_list_braids_order_variants() {
    let state = make_state();
    create_test_braid(&state).await;

    for order in ["newest", "oldest", "largest", "smallest"] {
        let query = ListBraidsQuery {
            agent: None,
            tag: None,
            mime_type: None,
            limit: None,
            offset: None,
            order: Some(order.to_string()),
        };

        let result = list_braids(State(state.clone()), Query(query)).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_delete_braid() {
    let state = make_state();
    let braid = create_test_braid(&state).await;

    let result = delete_braid(State(state.clone()), Path(braid.id.to_string())).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);

    let stored = state.store.get(&braid.id).await.unwrap();
    assert!(stored.is_none());
}

#[tokio::test]
async fn test_create_provenance_braid_success() {
    let state = make_state();

    let request = CreateProvenanceBraidRequest {
        data_hash: "sha256:abc123def456".to_string(),
        mime_type: "application/pdf".to_string(),
        size: 2048,
        was_attributed_to: "did:key:z6MkCreator".to_string(),
        was_derived_from: vec!["sha256:parent1".to_string()],
        tags: vec!["provenance-test".to_string()],
    };

    let result = create_provenance_braid(State(state.clone()), Json(request)).await;

    assert!(result.is_ok());
    let (status, Json(response)) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
    assert!(!response.id.is_empty());
    assert_eq!(response.hash, "sha256:abc123def456");
}

#[tokio::test]
async fn test_create_provenance_braid_no_derivations() {
    let state = make_state();

    let request = CreateProvenanceBraidRequest {
        data_hash: "sha256:standalone".to_string(),
        mime_type: "text/plain".to_string(),
        size: 512,
        was_attributed_to: "did:key:z6MkSolo".to_string(),
        was_derived_from: vec![],
        tags: vec![],
    };

    let result = create_provenance_braid(State(state), Json(request)).await;
    assert!(result.is_ok());
    let (status, _) = result.unwrap();
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_braids_with_agent_filter() {
    let state = make_state();
    create_test_braid(&state).await;

    let query = ListBraidsQuery {
        agent: Some("did:key:z6MkTest".to_string()),
        tag: None,
        mime_type: None,
        limit: None,
        offset: None,
        order: None,
    };

    let result = list_braids(State(state), Query(query)).await;
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert_eq!(response.braids.len(), 1);
}

#[tokio::test]
async fn test_list_braids_with_tag_filter() {
    let state = make_state();

    let metadata = BraidMetadata {
        tags: vec!["special".into()],
        ..Default::default()
    };
    let braid = state
        .factory
        .from_data(b"tagged data", "text/plain", Some(metadata))
        .unwrap();
    state.store.put(&braid).await.unwrap();

    let query = ListBraidsQuery {
        agent: None,
        tag: Some("special".to_string()),
        mime_type: None,
        limit: None,
        offset: None,
        order: None,
    };

    let result = list_braids(State(state), Query(query)).await;
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert_eq!(response.braids.len(), 1);
}

#[tokio::test]
async fn test_list_braids_with_offset() {
    let state = make_state();

    for i in 0..5 {
        let braid = state
            .factory
            .from_data(format!("data {i}").as_bytes(), "text/plain", None)
            .unwrap();
        state.store.put(&braid).await.unwrap();
    }

    let query = ListBraidsQuery {
        agent: None,
        tag: None,
        mime_type: None,
        limit: Some(2),
        offset: Some(3),
        order: Some("smallest".to_string()),
    };

    let result = list_braids(State(state), Query(query)).await;
    assert!(result.is_ok());
    let Json(response) = result.unwrap();
    assert_eq!(response.braids.len(), 2);
}

#[tokio::test]
async fn test_create_braid_minimal_fields() {
    use base64::Engine;
    let state = make_state();

    let request = CreateBraidRequest {
        data: base64::engine::general_purpose::STANDARD.encode(b"minimal"),
        mime_type: "application/octet-stream".to_string(),
        title: None,
        description: None,
        tags: None,
    };

    let result = create_braid(State(state), Json(request)).await;
    assert!(result.is_ok());
}
