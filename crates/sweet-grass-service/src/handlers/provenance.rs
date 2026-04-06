// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Provenance graph handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use sweet_grass_core::entity::EntityReference;
use sweet_grass_query::JsonLdDocument;

use crate::{error::ServiceError, state::AppState};

/// Query parameters for provenance.
#[derive(Debug, Deserialize)]
pub struct ProvenanceQuery {
    /// Maximum depth to traverse.
    pub depth: Option<u32>,
}

/// Get provenance graph for an entity.
///
/// # Errors
///
/// Returns an error if the provenance query fails.
pub async fn get_provenance(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Query(query): Query<ProvenanceQuery>,
) -> Result<Json<JsonLdDocument>, ServiceError> {
    let entity_ref = EntityReference::by_hash(hash.as_str());
    let graph = state
        .query
        .export_graph_provo(entity_ref, query.depth)
        .await?;

    Ok(Json(graph))
}

/// PROV-O response wrapper.
#[derive(Debug, Serialize)]
pub struct ProvOResponse {
    /// PROV-O JSON-LD document.
    #[serde(flatten)]
    pub document: JsonLdDocument,
}

/// Export provenance as PROV-O JSON-LD.
///
/// # Errors
///
/// Returns an error if the provenance query fails.
pub async fn export_prov_o(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Query(query): Query<ProvenanceQuery>,
) -> Result<Json<ProvOResponse>, ServiceError> {
    let entity_ref = EntityReference::by_hash(hash.as_str());
    let graph = state
        .query
        .export_graph_provo(entity_ref, query.depth)
        .await?;

    Ok(Json(ProvOResponse { document: graph }))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;
    use axum::extract::{Path, Query, State};
    use std::sync::Arc;
    use sweet_grass_core::agent::Did;
    use sweet_grass_factory::BraidFactory;

    fn create_test_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    async fn create_state_with_braid() -> (AppState, String) {
        let state = create_test_state();
        let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkCreator")));
        let braid = factory.from_data(b"test data", "text/plain", None).unwrap();
        let hash = braid.data_hash.as_str().to_string();
        state.store.put(&braid).await.unwrap();
        (state, hash)
    }

    #[tokio::test]
    async fn test_get_provenance() {
        let (state, hash) = create_state_with_braid().await;

        let result = get_provenance(
            State(state),
            Path(hash.clone()),
            Query(ProvenanceQuery { depth: None }),
        )
        .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        // Verify it's valid JSON-LD structure (context should be an object)
        assert!(doc.context.is_object());
    }

    #[tokio::test]
    async fn test_get_provenance_with_depth() {
        let (state, hash) = create_state_with_braid().await;

        let result = get_provenance(
            State(state),
            Path(hash),
            Query(ProvenanceQuery { depth: Some(5) }),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_prov_o() {
        let (state, hash) = create_state_with_braid().await;

        let result = export_prov_o(
            State(state),
            Path(hash.clone()),
            Query(ProvenanceQuery { depth: None }),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // The document should have proper PROV-O context (an object)
        assert!(response.document.context.is_object());
    }

    #[tokio::test]
    async fn test_provenance_not_found() {
        let state = create_test_state();

        let result = get_provenance(
            State(state),
            Path("nonexistent".to_string()),
            Query(ProvenanceQuery { depth: None }),
        )
        .await;

        // Empty provenance for non-existent entity is still valid
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provenance_with_derivation_chain() {
        use sweet_grass_core::entity::EntityReference;

        let state = create_test_state();

        // Create parent
        let factory = BraidFactory::new(Did::new("did:key:z6MkAlice"));
        let parent = factory.from_data(b"parent", "text/plain", None).unwrap();
        let parent_hash = parent.data_hash.clone();
        state.store.put(&parent).await.unwrap();

        // Create child derived from parent
        let child_factory = BraidFactory::new(Did::new("did:key:z6MkBob"));
        let mut child = child_factory
            .from_data(b"child", "text/plain", None)
            .unwrap();
        child.was_derived_from = vec![EntityReference::by_hash(&parent_hash)];
        let child_hash = child.data_hash.clone();
        state.store.put(&child).await.unwrap();

        // Get provenance for child
        let result = get_provenance(
            State(state),
            Path(child_hash.as_str().to_string()),
            Query(ProvenanceQuery { depth: Some(10) }),
        )
        .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        // Graph should be an array (possibly empty for simple case)
        // Graph is a Vec<Value>, check it's accessible
        let _ = &doc.graph;
    }
}
