// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC 2.0 handler.
//!
//! Implements the wateringHole `UNIVERSAL_IPC_STANDARD_V3` required protocol.
//! Method names follow the `{domain}.{operation}` semantic naming standard
//! from `SEMANTIC_METHOD_NAMING_STANDARD.md`.
//!
//! ## Domain Mapping
//!
//! | Domain        | Operations                                                       |
//! |---------------|------------------------------------------------------------------|
//! | `braid`       | create, get, getByHash, query, delete, commit                    |
//! | `anchoring`   | anchorBraid, verifyAnchor                                        |
//! | `provenance`  | graph, exportProvo, exportGraphProvo                             |
//! | `attribution` | chain, calculateRewards, topContributors                         |
//! | `compression` | compressSession, createMetaBraid                                 |
//! | `contribution`| record, recordSession, recordDehydration                         |
//! | `health`      | check                                                            |

mod anchoring;
mod attribution;
mod braid;
mod compression;
mod contribution;
mod health;
mod provenance;

use std::future::Future;
use std::pin::Pin;

use axum::{extract::State, Json};
use serde::Serialize;
use tracing::instrument;

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
#[derive(Debug, serde::Deserialize)]
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

// ==================== Dispatch Table ====================

pub(crate) type DispatchError = (i64, String);
pub(crate) type DispatchResult = Result<serde_json::Value, DispatchError>;
type DispatchFn = for<'a> fn(
    &'a AppState,
    serde_json::Value,
) -> Pin<Box<dyn Future<Output = DispatchResult> + Send + 'a>>;

struct MethodEntry {
    name: &'static str,
    handler: DispatchFn,
}

/// Static dispatch table — each domain.operation maps to a handler fn.
///
/// Replaces the former giant match statement with a scannable, extendable table.
static METHODS: &[MethodEntry] = &[
    // Braid operations
    MethodEntry {
        name: "braid.create",
        handler: |s, p| Box::pin(braid::handle_braid_create(s, p)),
    },
    MethodEntry {
        name: "braid.get",
        handler: |s, p| Box::pin(braid::handle_braid_get(s, p)),
    },
    MethodEntry {
        name: "braid.getByHash",
        handler: |s, p| Box::pin(braid::handle_braid_get_by_hash(s, p)),
    },
    MethodEntry {
        name: "braid.query",
        handler: |s, p| Box::pin(braid::handle_braid_query(s, p)),
    },
    MethodEntry {
        name: "braid.delete",
        handler: |s, p| Box::pin(braid::handle_braid_delete(s, p)),
    },
    MethodEntry {
        name: "braid.commit",
        handler: |s, p| Box::pin(braid::handle_braid_commit(s, p)),
    },
    MethodEntry {
        name: "anchoring.anchorBraid",
        handler: |s, p| Box::pin(anchoring::handle_anchor_braid(s, p)),
    },
    MethodEntry {
        name: "anchoring.verifyAnchor",
        handler: |s, p| Box::pin(anchoring::handle_verify_anchor(s, p)),
    },
    // Provenance
    MethodEntry {
        name: "provenance.graph",
        handler: |s, p| Box::pin(provenance::handle_provenance_graph(s, p)),
    },
    MethodEntry {
        name: "provenance.exportProvo",
        handler: |s, p| Box::pin(provenance::handle_export_provo(s, p)),
    },
    MethodEntry {
        name: "provenance.exportGraphProvo",
        handler: |s, p| Box::pin(provenance::handle_export_graph_provo(s, p)),
    },
    // Attribution
    MethodEntry {
        name: "attribution.chain",
        handler: |s, p| Box::pin(attribution::handle_attribution_chain(s, p)),
    },
    MethodEntry {
        name: "attribution.calculateRewards",
        handler: |s, p| Box::pin(attribution::handle_calculate_rewards(s, p)),
    },
    MethodEntry {
        name: "attribution.topContributors",
        handler: |s, p| Box::pin(attribution::handle_top_contributors(s, p)),
    },
    // Compression
    MethodEntry {
        name: "compression.compressSession",
        handler: |s, p| Box::pin(async move { compression::handle_compress_session_sync(s, p) }),
    },
    MethodEntry {
        name: "compression.createMetaBraid",
        handler: |s, p| Box::pin(compression::handle_create_meta_braid(s, p)),
    },
    // Contribution recording
    MethodEntry {
        name: "contribution.record",
        handler: |s, p| Box::pin(contribution::handle_record_contribution(s, p)),
    },
    MethodEntry {
        name: "contribution.recordSession",
        handler: |s, p| Box::pin(contribution::handle_record_session(s, p)),
    },
    MethodEntry {
        name: "contribution.recordDehydration",
        handler: |s, p| Box::pin(contribution::handle_record_dehydration(s, p)),
    },
    // Health
    MethodEntry {
        name: "health.check",
        handler: |s, p| Box::pin(health::handle_health(s, p)),
    },
];

fn find_handler(method: &str) -> Option<DispatchFn> {
    METHODS.iter().find(|m| m.name == method).map(|m| m.handler)
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

async fn dispatch(state: &AppState, method: &str, params: serde_json::Value) -> DispatchResult {
    match find_handler(method) {
        Some(handler) => handler(state, params).await,
        None => Err((
            error_code::METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )),
    }
}

// ==================== Helpers (used by domain modules) ====================

pub(crate) fn internal(e: impl std::fmt::Display) -> DispatchError {
    (error_code::INTERNAL_ERROR, e.to_string())
}

pub(crate) fn parse_params<T: serde::de::DeserializeOwned>(
    params: serde_json::Value,
) -> Result<T, DispatchError> {
    serde_json::from_value(params)
        .map_err(|e| (error_code::INVALID_PARAMS, format!("Invalid params: {e}")))
}

pub(crate) fn to_value<T: Serialize>(v: &T) -> DispatchResult {
    serde_json::to_value(v).map_err(|e| {
        (
            error_code::INTERNAL_ERROR,
            format!("Serialization error: {e}"),
        )
    })
}

#[cfg(test)]
mod tests;
