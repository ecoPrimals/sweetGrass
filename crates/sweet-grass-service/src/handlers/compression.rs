// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Compression handlers.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sweet_grass_compression::{CompressionResult, Session, SessionOutcome, SessionVertex};
use sweet_grass_core::{ActivityType, Braid, agent::Did};

use sweet_grass_store::BraidStore;

use crate::{error::ServiceError, state::AppState};

/// Request to compress a session.
#[derive(Debug, Deserialize)]
pub struct CompressSessionRequest {
    /// Session ID.
    pub session_id: String,

    /// Session vertices.
    pub vertices: Vec<VertexInput>,

    /// Session outcome.
    pub outcome: String,

    /// Compression hint.
    pub hint: Option<String>,

    /// Compute units consumed.
    pub compute_units: Option<f64>,
}

/// Input vertex in a session.
#[derive(Debug, Deserialize)]
pub struct VertexInput {
    /// Vertex ID.
    pub id: String,

    /// Content hash.
    pub data_hash: String,

    /// MIME type.
    pub mime_type: String,

    /// Size in bytes.
    pub size: u64,

    /// Parent vertex IDs.
    pub parents: Vec<String>,

    /// Agent DID.
    pub agent: String,

    /// Activity type.
    pub activity_type: Option<String>,

    /// Whether committed.
    pub committed: bool,
}

/// Response from compression.
#[derive(Debug, Serialize)]
pub struct CompressSessionResponse {
    /// Result type: "none", "single", "multiple".
    pub result_type: String,

    /// Reason for discarding (if none).
    pub discard_reason: Option<String>,

    /// Created Braids.
    pub braids: Vec<BraidSummary>,

    /// Summary Braid (if multiple).
    pub summary: Option<BraidSummary>,
}

/// Summary of a created Braid.
#[derive(Debug, Serialize)]
pub struct BraidSummary {
    /// Braid ID.
    pub id: String,

    /// Content hash.
    pub hash: String,

    /// Size.
    pub size: u64,
}

impl From<&Braid> for BraidSummary {
    fn from(braid: &Braid) -> Self {
        Self {
            id: braid.id.to_string(),
            hash: braid.data_hash.as_str().to_string(),
            size: braid.size,
        }
    }
}

/// Compress a session to Braids.
///
/// # Errors
///
/// Returns an error if compression fails or storing created Braids fails.
pub async fn compress_session(
    State(state): State<AppState>,
    Json(request): Json<CompressSessionRequest>,
) -> Result<(StatusCode, Json<CompressSessionResponse>), ServiceError> {
    // Build session
    let mut session = Session::new(&request.session_id);
    session.compute_units = request.compute_units.unwrap_or(0.0);

    // Set outcome
    session.outcome = match request.outcome.as_str() {
        "committed" => SessionOutcome::Committed,
        "rollback" => SessionOutcome::Rollback,
        "noop" => SessionOutcome::NoOp,
        _ => SessionOutcome::InProgress,
    };

    // Add vertices
    for v in request.vertices {
        let mut vertex = SessionVertex::new(
            &v.id,
            v.data_hash.as_str(),
            &v.mime_type,
            Did::new(&v.agent),
        )
        .with_size(v.size);

        for parent in v.parents {
            vertex = vertex.with_parent(parent);
        }

        if let Some(activity_type) = v.activity_type {
            let at = parse_activity_type(&activity_type);
            vertex = vertex.with_activity_type(at);
        }

        if v.committed {
            vertex = vertex.committed();
        }

        session.add_vertex(vertex);
    }

    // Compress
    let result = state.compression.compress(&session)?;

    // Store created Braids
    for braid in result.braids() {
        state.store.put(braid).await?;
    }

    // Build response
    let response = match result {
        CompressionResult::None { reason } => CompressSessionResponse {
            result_type: "none".to_string(),
            discard_reason: Some(reason.to_string()),
            braids: Vec::new(),
            summary: None,
        },
        CompressionResult::Single(ref braid) => CompressSessionResponse {
            result_type: "single".to_string(),
            discard_reason: None,
            braids: vec![BraidSummary::from(braid)],
            summary: None,
        },
        CompressionResult::Multiple {
            ref braids,
            ref summary,
        } => CompressSessionResponse {
            result_type: "multiple".to_string(),
            discard_reason: None,
            braids: braids.iter().map(BraidSummary::from).collect(),
            summary: summary.as_ref().map(BraidSummary::from),
        },
        _ => CompressSessionResponse {
            result_type: "unknown".to_string(),
            discard_reason: None,
            braids: Vec::new(),
            summary: None,
        },
    };

    let status = if response.result_type == "none" {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(response)))
}

fn parse_activity_type(s: &str) -> ActivityType {
    match s.to_lowercase().as_str() {
        "creation" => ActivityType::Creation,
        "transformation" => ActivityType::Transformation,
        "aggregation" => ActivityType::Aggregation,
        "derivation" => ActivityType::Derivation,
        "validation" => ActivityType::Validation,
        "import" => ActivityType::Import,
        "computation" => ActivityType::Computation,
        "inference" => ActivityType::Inference,
        "editing" => ActivityType::Editing,
        "sessioncommit" => ActivityType::SessionCommit,
        _ => ActivityType::Custom {
            type_uri: s.to_string(),
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sweet_grass_factory::BraidFactory;

    fn make_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    #[test]
    fn test_parse_activity_type() {
        assert!(matches!(
            parse_activity_type("creation"),
            ActivityType::Creation
        ));
        assert!(matches!(
            parse_activity_type("CREATION"),
            ActivityType::Creation
        ));
        assert!(matches!(
            parse_activity_type("unknown"),
            ActivityType::Custom { .. }
        ));
    }

    #[test]
    fn test_parse_activity_type_all_variants() {
        assert!(matches!(
            parse_activity_type("transformation"),
            ActivityType::Transformation
        ));
        assert!(matches!(
            parse_activity_type("aggregation"),
            ActivityType::Aggregation
        ));
        assert!(matches!(
            parse_activity_type("derivation"),
            ActivityType::Derivation
        ));
        assert!(matches!(
            parse_activity_type("validation"),
            ActivityType::Validation
        ));
        assert!(matches!(
            parse_activity_type("import"),
            ActivityType::Import
        ));
        assert!(matches!(
            parse_activity_type("computation"),
            ActivityType::Computation
        ));
        assert!(matches!(
            parse_activity_type("inference"),
            ActivityType::Inference
        ));
        assert!(matches!(
            parse_activity_type("editing"),
            ActivityType::Editing
        ));
        assert!(matches!(
            parse_activity_type("sessioncommit"),
            ActivityType::SessionCommit
        ));
    }

    #[test]
    fn test_braid_summary_from_braid() {
        let factory = BraidFactory::new(Did::new("did:key:z6MkTest"));
        let braid = factory.from_data(b"test", "text/plain", None).unwrap();

        let summary = BraidSummary::from(&braid);

        assert_eq!(summary.id, braid.id.to_string());
        assert_eq!(summary.hash, braid.data_hash.as_str());
        assert_eq!(summary.size, braid.size);
    }

    #[tokio::test]
    async fn test_compress_session_with_committed_vertex() {
        let state = make_state();

        let request = CompressSessionRequest {
            session_id: "test-session-1".to_string(),
            vertices: vec![VertexInput {
                id: "v1".to_string(),
                data_hash: "sha256:test123".to_string(),
                mime_type: "text/plain".to_string(),
                size: 100,
                parents: vec![],
                agent: "did:key:z6MkTest".to_string(),
                activity_type: Some("creation".to_string()),
                committed: true,
            }],
            outcome: "committed".to_string(),
            hint: None,
            compute_units: Some(1.5),
        };

        let result = compress_session(State(state), Json(request)).await;

        assert!(result.is_ok());
        let (status, Json(response)) = result.unwrap();
        assert!(status == StatusCode::CREATED || status == StatusCode::OK);
        // Response should indicate some result
        assert!(!response.result_type.is_empty());
    }

    #[tokio::test]
    async fn test_compress_session_noop() {
        let state = make_state();

        let request = CompressSessionRequest {
            session_id: "test-session-noop".to_string(),
            vertices: vec![],
            outcome: "noop".to_string(),
            hint: None,
            compute_units: None,
        };

        let result = compress_session(State(state), Json(request)).await;

        assert!(result.is_ok());
        let (status, Json(response)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.result_type, "none");
        assert!(response.discard_reason.is_some());
    }

    #[tokio::test]
    async fn test_compress_session_rollback() {
        let state = make_state();

        let request = CompressSessionRequest {
            session_id: "test-session-rollback".to_string(),
            vertices: vec![VertexInput {
                id: "v1".to_string(),
                data_hash: "sha256:test456".to_string(),
                mime_type: "text/plain".to_string(),
                size: 50,
                parents: vec![],
                agent: "did:key:z6MkTest".to_string(),
                activity_type: None,
                committed: false,
            }],
            outcome: "rollback".to_string(),
            hint: None,
            compute_units: None,
        };

        let result = compress_session(State(state), Json(request)).await;

        assert!(result.is_ok());
        let (status, Json(response)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(response.result_type, "none");
    }

    #[tokio::test]
    async fn test_compress_session_with_parents() {
        let state = make_state();

        let request = CompressSessionRequest {
            session_id: "test-session-parents".to_string(),
            vertices: vec![
                VertexInput {
                    id: "v1".to_string(),
                    data_hash: "sha256:parent".to_string(),
                    mime_type: "text/plain".to_string(),
                    size: 100,
                    parents: vec![],
                    agent: "did:key:z6MkTest".to_string(),
                    activity_type: Some("creation".to_string()),
                    committed: true,
                },
                VertexInput {
                    id: "v2".to_string(),
                    data_hash: "sha256:child".to_string(),
                    mime_type: "text/plain".to_string(),
                    size: 200,
                    parents: vec!["v1".to_string()],
                    agent: "did:key:z6MkTest".to_string(),
                    activity_type: Some("transformation".to_string()),
                    committed: true,
                },
            ],
            outcome: "committed".to_string(),
            hint: None,
            compute_units: Some(2.5),
        };

        let result = compress_session(State(state), Json(request)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compress_session_unknown_outcome() {
        let state = make_state();

        let request = CompressSessionRequest {
            session_id: "test-session-unknown".to_string(),
            vertices: vec![],
            outcome: "unknown_outcome".to_string(),
            hint: None,
            compute_units: None,
        };

        let result = compress_session(State(state), Json(request)).await;

        // Should default to InProgress
        assert!(result.is_ok());
    }
}
