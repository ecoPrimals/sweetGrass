// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC 2.0 handler.
//!
//! Implements the wateringHole `UNIVERSAL_IPC_STANDARD_V3` required protocol.
//! Method names follow the `{domain}.{operation}` semantic naming standard.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use sweet_grass_compression::Session;
use sweet_grass_core::{
    braid::{BraidId, BraidMetadata, ContentHash, SummaryType},
    contribution::{ContributionRecord, SessionContribution},
    entity::EntityReference,
};
use sweet_grass_store::{QueryFilter, QueryOrder};

use crate::state::AppState;

/// JSON-RPC 2.0 error codes per specification.
mod error_code {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
    pub const NOT_FOUND: i64 = -32001;
}

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
            id,
        }
    }
}

/// `POST /jsonrpc` — JSON-RPC 2.0 dispatcher.
///
/// Routes semantic method names to the underlying service logic.
/// Methods follow `{domain}.{operation}` naming per wateringHole standard.
#[instrument(skip_all)]
pub async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Json<JsonRpcResponse> {
    let parsed: JsonRpcRequest = match serde_json::from_value(request) {
        Ok(r) => r,
        Err(e) => {
            return Json(JsonRpcResponse::error(
                serde_json::Value::Null,
                error_code::PARSE_ERROR,
                format!("Parse error: {e}"),
            ));
        },
    };

    if parsed.jsonrpc != "2.0" {
        return Json(JsonRpcResponse::error(
            parsed.id,
            error_code::INVALID_REQUEST,
            "Invalid JSON-RPC version, expected \"2.0\"",
        ));
    }

    let result = dispatch(&state, &parsed.method, parsed.params).await;

    match result {
        Ok(value) => Json(JsonRpcResponse::success(parsed.id, value)),
        Err(e) => Json(JsonRpcResponse::error(parsed.id, e.0, e.1)),
    }
}

type DispatchError = (i64, String);

#[allow(clippy::too_many_lines)]
async fn dispatch(
    state: &AppState,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, DispatchError> {
    match method {
        // ==================== Braid Operations ====================
        "sweetgrass.createBraid" => {
            let p: CreateBraidParams = parse_params(params)?;
            let braid = state
                .factory
                .from_hash(p.data_hash, p.mime_type, p.size, p.metadata)
                .map_err(internal)?;
            state.store.put(&braid).await.map_err(internal)?;
            to_value(&braid)
        },

        "sweetgrass.getBraid" => {
            let p: GetBraidParams = parse_params(params)?;
            let braid = state.store.get(&p.id).await.map_err(internal)?;
            match braid {
                Some(b) => to_value(&b),
                None => Err((error_code::NOT_FOUND, format!("Braid not found: {}", p.id))),
            }
        },

        "sweetgrass.getBraidByHash" => {
            let p: GetByHashParams = parse_params(params)?;
            let filter = QueryFilter {
                data_hash: Some(p.hash.clone()),
                ..QueryFilter::default()
            };
            let result = state
                .store
                .query(&filter, QueryOrder::NewestFirst)
                .await
                .map_err(internal)?;
            match result.braids.into_iter().next() {
                Some(b) => to_value(&b),
                None => Err((
                    error_code::NOT_FOUND,
                    format!("No braid with hash: {}", p.hash),
                )),
            }
        },

        "sweetgrass.queryBraids" => {
            let p: QueryBraidsParams = parse_params(params)?;
            let order = p.order.unwrap_or(QueryOrder::NewestFirst);
            let result = state
                .store
                .query(&p.filter, order)
                .await
                .map_err(internal)?;
            to_value(&result)
        },

        "sweetgrass.deleteBraid" => {
            let p: GetBraidParams = parse_params(params)?;
            let deleted = state.store.delete(&p.id).await.map_err(internal)?;
            to_value(&deleted)
        },

        // ==================== Provenance ====================
        "sweetgrass.provenanceGraph" => {
            let p: ProvenanceParams = parse_params(params)?;
            let graph = state
                .query
                .provenance_graph(p.entity, p.depth)
                .await
                .map_err(internal)?;
            to_value(&graph)
        },

        "sweetgrass.attributionChain" => {
            let p: AttributionParams = parse_params(params)?;
            let chain = state
                .query
                .attribution_chain(&p.hash)
                .await
                .map_err(internal)?;
            to_value(&chain)
        },

        "sweetgrass.calculateRewards" => {
            let p: RewardsParams = parse_params(params)?;
            let chain = state
                .query
                .attribution_chain(&p.hash)
                .await
                .map_err(internal)?;
            let rewards: Vec<serde_json::Value> = chain
                .contributors
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "agent": c.agent.as_str(),
                        "share": c.share,
                        "amount": c.share * p.value,
                        "role": format!("{:?}", c.role),
                    })
                })
                .collect();
            to_value(&rewards)
        },

        // ==================== Compression ====================
        "sweetgrass.compressSession" => {
            let session: Session = parse_params(params)?;
            let result = state.compression.compress(&session).map_err(internal)?;
            to_value(&result)
        },

        "sweetgrass.createMetaBraid" => {
            let p: MetaBraidParams = parse_params(params)?;
            let braid = state
                .factory
                .meta_braid(p.braid_ids, p.summary_type, None)
                .map_err(internal)?;
            state.store.put(&braid).await.map_err(internal)?;
            to_value(&braid)
        },

        // ==================== Contribution Recording ====================
        "sweetgrass.recordContribution" => {
            let record: ContributionRecord = parse_params(params)?;
            let braid = state.factory.from_contribution(&record).map_err(internal)?;
            state.store.put(&braid).await.map_err(internal)?;
            to_value(&braid)
        },

        "sweetgrass.recordSession" => {
            let session: SessionContribution = parse_params(params)?;
            let braids = state.factory.from_session(&session).map_err(internal)?;
            for braid in &braids {
                state.store.put(braid).await.map_err(internal)?;
            }
            to_value(&serde_json::json!({
                "session_id": session.session_id,
                "braids_created": braids.len(),
                "braid_ids": braids.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(),
            }))
        },

        // ==================== Export ====================
        "sweetgrass.exportProvo" => {
            let p: ExportProvoParams = parse_params(params)?;
            let export = state
                .query
                .export_braid_provo(&p.hash)
                .await
                .map_err(internal)?;
            to_value(&export)
        },

        // ==================== Health ====================
        "sweetgrass.health" => {
            let count = state
                .store
                .count(&QueryFilter::default())
                .await
                .map_err(internal)?;
            to_value(&serde_json::json!({
                "status": "healthy",
                "store_status": "ok",
                "braid_count": count,
                "version": env!("CARGO_PKG_VERSION"),
            }))
        },

        _ => Err((
            error_code::METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )),
    }
}

fn internal(e: impl std::fmt::Display) -> DispatchError {
    (error_code::INTERNAL_ERROR, e.to_string())
}

fn parse_params<T: serde::de::DeserializeOwned>(
    params: serde_json::Value,
) -> Result<T, DispatchError> {
    serde_json::from_value(params)
        .map_err(|e| (error_code::INVALID_PARAMS, format!("Invalid params: {e}")))
}

fn to_value<T: Serialize>(v: &T) -> Result<serde_json::Value, DispatchError> {
    serde_json::to_value(v).map_err(|e| {
        (
            error_code::INTERNAL_ERROR,
            format!("Serialization error: {e}"),
        )
    })
}

// ==================== Parameter Types ====================

#[derive(Debug, Deserialize)]
struct CreateBraidParams {
    data_hash: ContentHash,
    mime_type: String,
    size: u64,
    #[serde(default)]
    metadata: Option<BraidMetadata>,
}

#[derive(Debug, Deserialize)]
struct GetBraidParams {
    id: BraidId,
}

#[derive(Debug, Deserialize)]
struct GetByHashParams {
    hash: ContentHash,
}

#[derive(Debug, Deserialize)]
struct QueryBraidsParams {
    filter: QueryFilter,
    #[serde(default)]
    order: Option<QueryOrder>,
}

#[derive(Debug, Deserialize)]
struct ProvenanceParams {
    entity: EntityReference,
    #[serde(default)]
    depth: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AttributionParams {
    hash: ContentHash,
}

#[derive(Debug, Deserialize)]
struct RewardsParams {
    hash: ContentHash,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct MetaBraidParams {
    braid_ids: Vec<BraidId>,
    summary_type: SummaryType,
}

#[derive(Debug, Deserialize)]
struct ExportProvoParams {
    hash: ContentHash,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use sweet_grass_core::agent::Did;

    fn test_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    #[test]
    fn test_parse_error_response() {
        let resp = JsonRpcResponse::error(
            serde_json::Value::Null,
            error_code::PARSE_ERROR,
            "test parse error",
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_some());
        assert!(resp.result.is_none());
        assert_eq!(resp.error.unwrap().code, error_code::PARSE_ERROR);
    }

    #[test]
    fn test_success_response() {
        let resp =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let state = test_state();
        let result = dispatch(&state, "nonexistent.method", serde_json::json!({})).await;
        assert!(result.is_err());
        let (code, _msg) = result.unwrap_err();
        assert_eq!(code, error_code::METHOD_NOT_FOUND);
    }

    #[test]
    fn test_invalid_version_detection() {
        let request = serde_json::json!({
            "jsonrpc": "1.0",
            "method": "sweetgrass.health",
            "params": {},
            "id": 1
        });
        let parsed: JsonRpcRequest = serde_json::from_value(request).unwrap();
        assert_ne!(parsed.jsonrpc, "2.0");
    }

    #[tokio::test]
    async fn test_health_method() {
        let state = test_state();
        let result = dispatch(&state, "sweetgrass.health", serde_json::json!({})).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["status"], "healthy");
        assert_eq!(value["braid_count"], 0);
    }

    #[tokio::test]
    async fn test_create_and_get_braid() {
        let state = test_state();

        let create_params = serde_json::json!({
            "data_hash": "sha256:testjsonrpc",
            "mime_type": "application/json",
            "size": 512
        });
        let result = dispatch(&state, "sweetgrass.createBraid", create_params).await;
        assert!(result.is_ok());
        let braid = result.unwrap();
        let braid_id = braid["@id"].as_str().unwrap().to_string();

        let get_result = dispatch(
            &state,
            "sweetgrass.getBraid",
            serde_json::json!({"id": braid_id}),
        )
        .await;
        assert!(get_result.is_ok());
    }

    #[tokio::test]
    async fn test_get_braid_not_found() {
        let state = test_state();
        let result = dispatch(
            &state,
            "sweetgrass.getBraid",
            serde_json::json!({"id": "nonexistent"}),
        )
        .await;
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, error_code::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_query_braids() {
        let state = test_state();
        let result = dispatch(
            &state,
            "sweetgrass.queryBraids",
            serde_json::json!({"filter": {}}),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_params() {
        let state = test_state();
        let result = dispatch(
            &state,
            "sweetgrass.createBraid",
            serde_json::json!({"wrong": "params"}),
        )
        .await;
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, error_code::INVALID_PARAMS);
    }

    #[tokio::test]
    async fn test_delete_braid() {
        let state = test_state();

        let create_result = dispatch(
            &state,
            "sweetgrass.createBraid",
            serde_json::json!({
                "data_hash": "sha256:deleteme",
                "mime_type": "text/plain",
                "size": 10
            }),
        )
        .await
        .unwrap();
        let braid_id = create_result["@id"].as_str().unwrap().to_string();

        let delete_result = dispatch(
            &state,
            "sweetgrass.deleteBraid",
            serde_json::json!({"id": braid_id}),
        )
        .await;
        assert!(delete_result.is_ok());
    }

    #[test]
    fn test_all_error_codes() {
        assert_eq!(error_code::PARSE_ERROR, -32700);
        assert_eq!(error_code::INVALID_REQUEST, -32600);
        assert_eq!(error_code::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_code::INVALID_PARAMS, -32602);
        assert_eq!(error_code::INTERNAL_ERROR, -32603);
        assert_eq!(error_code::NOT_FOUND, -32001);
    }

    #[tokio::test]
    async fn test_record_contribution_dispatch() {
        let state = test_state();
        let params = serde_json::json!({
            "agent": "did:key:z6MkContributor",
            "role": "Creator",
            "content_hash": "sha256:rpc-contrib-test",
            "mime_type": "application/json",
            "size": 64
        });

        let result = dispatch(&state, "sweetgrass.recordContribution", params).await;
        assert!(result.is_ok());
        let braid = result.unwrap();
        assert_eq!(braid["data_hash"], "sha256:rpc-contrib-test");
        assert!(braid["@id"].as_str().unwrap().starts_with("urn:braid:"));
    }

    #[tokio::test]
    async fn test_record_session_dispatch() {
        let state = test_state();
        let params = serde_json::json!({
            "session_id": "rpc-session-123",
            "source_primal": "rhizoCrypt",
            "contributions": [
                {
                    "agent": "did:key:z6MkAgent1",
                    "role": "Creator",
                    "content_hash": "sha256:session-hash-1",
                    "mime_type": "text/plain",
                    "size": 10
                },
                {
                    "agent": "did:key:z6MkAgent2",
                    "role": "Contributor",
                    "content_hash": "sha256:session-hash-2",
                    "mime_type": "application/json",
                    "size": 20
                }
            ]
        });

        let result = dispatch(&state, "sweetgrass.recordSession", params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["session_id"], "rpc-session-123");
        assert_eq!(response["braids_created"], 2);
        let braid_ids = response["braid_ids"].as_array().unwrap();
        assert_eq!(braid_ids.len(), 2);
    }
}
