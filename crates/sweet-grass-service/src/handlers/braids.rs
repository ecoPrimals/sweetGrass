//! Braid management handlers.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sweet_grass_core::{
    agent::Did,
    braid::{BraidId, BraidMetadata},
    Braid,
};
use sweet_grass_store::{QueryFilter, QueryOrder};

use crate::{error::ServiceError, state::AppState};

/// Request to create a new Braid.
#[derive(Debug, Deserialize)]
pub struct CreateBraidRequest {
    /// Raw data (base64 encoded).
    pub data: String,

    /// MIME type.
    pub mime_type: String,

    /// Optional title.
    pub title: Option<String>,

    /// Optional description.
    pub description: Option<String>,

    /// Optional tags.
    pub tags: Option<Vec<String>>,
}

/// Response for Braid creation.
#[derive(Debug, Serialize)]
pub struct CreateBraidResponse {
    /// Created Braid ID.
    pub id: String,

    /// Content hash.
    pub hash: String,
}

/// Query parameters for listing Braids.
#[derive(Debug, Deserialize)]
pub struct ListBraidsQuery {
    /// Filter by agent DID.
    pub agent: Option<String>,

    /// Filter by tag.
    pub tag: Option<String>,

    /// Filter by MIME type.
    pub mime_type: Option<String>,

    /// Limit results.
    pub limit: Option<usize>,

    /// Offset for pagination.
    pub offset: Option<usize>,

    /// Order by (newest, oldest, largest, smallest).
    pub order: Option<String>,
}

/// Paginated list response.
#[derive(Debug, Serialize)]
pub struct ListBraidsResponse {
    /// Braids on this page.
    pub braids: Vec<Braid>,

    /// Total matching Braids.
    pub total: usize,

    /// Whether there are more results.
    pub has_more: bool,
}

/// Get a Braid by ID.
pub async fn get_braid(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Braid>, ServiceError> {
    let braid_id = BraidId::from_string(id);

    let braid = state
        .store
        .get(&braid_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Braid not found: {braid_id}")))?;

    Ok(Json(braid))
}

/// Get a Braid by content hash.
pub async fn get_braid_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<Braid>, ServiceError> {
    let braid = state
        .store
        .get_by_hash(&hash)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Braid not found for hash: {hash}")))?;

    Ok(Json(braid))
}

/// Create a new Braid.
pub async fn create_braid(
    State(state): State<AppState>,
    Json(request): Json<CreateBraidRequest>,
) -> Result<(StatusCode, Json<CreateBraidResponse>), ServiceError> {
    // Decode base64 data
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&request.data)
        .map_err(|e| ServiceError::BadRequest(format!("Invalid base64 data: {e}")))?;

    // Build metadata
    let metadata = BraidMetadata {
        title: request.title,
        description: request.description,
        tags: request.tags.unwrap_or_default(),
        ..Default::default()
    };

    // Create the Braid
    let braid = state
        .factory
        .from_data(&data, &request.mime_type, Some(metadata))?;

    // Store it
    state.store.put(&braid).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateBraidResponse {
            id: braid.id.to_string(),
            hash: braid.data_hash.clone(),
        }),
    ))
}

/// List/query Braids.
pub async fn list_braids(
    State(state): State<AppState>,
    Query(query): Query<ListBraidsQuery>,
) -> Result<Json<ListBraidsResponse>, ServiceError> {
    // Build filter using builder pattern
    let mut filter = QueryFilter::new();

    if let Some(agent) = query.agent {
        filter = filter.with_agent(Did::new(&agent));
    }

    if let Some(mime_type) = query.mime_type {
        filter = filter.with_mime_type(mime_type);
    }

    if let Some(limit) = query.limit {
        filter = filter.with_limit(limit);
    }

    if let Some(offset) = query.offset {
        filter = filter.with_offset(offset);
    }

    // Handle tag separately since it's a direct field
    if let Some(tag) = query.tag {
        filter.tag = Some(tag);
    }

    // Determine order
    let order = match query.order.as_deref() {
        Some("oldest") => QueryOrder::OldestFirst,
        Some("largest") => QueryOrder::LargestFirst,
        Some("smallest") => QueryOrder::SmallestFirst,
        _ => QueryOrder::NewestFirst,
    };

    // Execute query
    let result = state.store.query(&filter, order).await?;

    Ok(Json(ListBraidsResponse {
        braids: result.braids,
        total: result.total_count,
        has_more: result.has_more,
    }))
}

/// Delete a Braid.
pub async fn delete_braid(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServiceError> {
    let braid_id = BraidId::from_string(id);
    state.store.delete(&braid_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
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

        let result = get_braid_by_hash(State(state), Path(braid.data_hash.clone())).await;

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
        let stored = state.store.get_by_hash(&response.hash).await.unwrap();
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

        // Verify it was deleted
        let stored = state.store.get(&braid.id).await.unwrap();
        assert!(stored.is_none());
    }
}
