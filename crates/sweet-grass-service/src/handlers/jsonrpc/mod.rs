// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
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
//! | `braid`       | create, get, get_by_hash, query, delete, commit                  |
//! | `anchoring`   | anchor, verify                                                   |
//! | `provenance`  | graph, export_provo, export_graph_provo                           |
//! | `attribution` | chain, calculate_rewards, top_contributors                       |
//! | `compression` | compress_session, create_meta_braid                              |
//! | `contribution`| record, record_session, record_dehydration                       |
//! | `pipeline`    | attribute (provenance trio coordination)                          |
//! | `health`      | check, liveness, readiness                                       |
//! | `capabilities`| list (canonical per wateringHole v2.1)                            |
//! | `capability`  | list (alias)                                                     |
//! | `tools`       | list, call (MCP exposure for Squirrel AI coordination)            |

mod anchoring;
mod attribution;
mod braid;
mod capability;
mod compression;
mod contribution;
mod health;
mod provenance;

use std::future::Future;
use std::pin::Pin;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::instrument;

use crate::state::AppState;

/// JSON-RPC 2.0 error codes per specification.
pub(crate) mod error_code {
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
    /// Protocol version — must be `"2.0"`.
    pub jsonrpc: String,
    /// Semantic method name (`{domain}.{operation}`).
    pub method: String,
    /// Method parameters (may be omitted).
    #[serde(default)]
    pub params: serde_json::Value,
    /// Caller-supplied request identifier.
    ///
    /// Defaults to `Null` when omitted. Notification detection (absent `id`)
    /// is handled via raw JSON inspection in `process_single` before parsing.
    #[serde(default)]
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version — always `"2.0"`.
    pub jsonrpc: std::borrow::Cow<'static, str>,
    /// Result on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Echoed request identifier.
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code per JSON-RPC 2.0 specification.
    pub code: i64,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// The JSON-RPC 2.0 version string.
const JSONRPC_VERSION: std::borrow::Cow<'static, str> = std::borrow::Cow::Borrowed("2.0");

impl JsonRpcResponse {
    const fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
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

/// Outcome of a JSON-RPC dispatch, separating protocol errors from
/// application-level errors (aligned with rhizoCrypt `DispatchOutcome`).
///
/// Protocol errors (parse, invalid version, method not found) are
/// handled before the handler runs. Application errors come from
/// the handler itself and carry domain-specific codes.
#[derive(Debug)]
pub(crate) enum DispatchOutcome {
    /// Handler executed and returned a JSON result.
    Success(serde_json::Value),
    /// Protocol violation — the request never reached a handler.
    ProtocolError { code: i64, message: String },
    /// Handler ran but returned a domain error.
    ApplicationError { code: i64, message: String },
}

impl DispatchOutcome {
    /// Whether this outcome represents a protocol-level failure (retriable
    /// at the transport layer, not the application layer).
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn is_protocol_error(&self) -> bool {
        matches!(self, Self::ProtocolError { .. })
    }

    /// Whether this outcome represents an application-level error
    /// (handler ran but failed).
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn is_application_error(&self) -> bool {
        matches!(self, Self::ApplicationError { .. })
    }

    fn into_response(self, id: serde_json::Value) -> JsonRpcResponse {
        match self {
            Self::Success(value) => JsonRpcResponse::success(id, value),
            Self::ProtocolError { code, message } | Self::ApplicationError { code, message } => {
                JsonRpcResponse::error(id, code, message)
            },
        }
    }
}

pub(super) struct MethodEntry {
    pub(super) name: &'static str,
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
        name: "braid.get_by_hash",
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
        name: "anchoring.anchor",
        handler: |s, p| Box::pin(anchoring::handle_anchor_braid(s, p)),
    },
    MethodEntry {
        name: "anchoring.verify",
        handler: |s, p| Box::pin(anchoring::handle_verify_anchor(s, p)),
    },
    // Provenance
    MethodEntry {
        name: "provenance.graph",
        handler: |s, p| Box::pin(provenance::handle_provenance_graph(s, p)),
    },
    MethodEntry {
        name: "provenance.export_provo",
        handler: |s, p| Box::pin(provenance::handle_export_provo(s, p)),
    },
    MethodEntry {
        name: "provenance.export_graph_provo",
        handler: |s, p| Box::pin(provenance::handle_export_graph_provo(s, p)),
    },
    // Attribution
    MethodEntry {
        name: "attribution.chain",
        handler: |s, p| Box::pin(attribution::handle_attribution_chain(s, p)),
    },
    MethodEntry {
        name: "attribution.calculate_rewards",
        handler: |s, p| Box::pin(attribution::handle_calculate_rewards(s, p)),
    },
    MethodEntry {
        name: "attribution.top_contributors",
        handler: |s, p| Box::pin(attribution::handle_top_contributors(s, p)),
    },
    // Compression
    MethodEntry {
        name: "compression.compress_session",
        handler: |s, p| Box::pin(async move { compression::handle_compress_session_sync(s, p) }),
    },
    MethodEntry {
        name: "compression.create_meta_braid",
        handler: |s, p| Box::pin(compression::handle_create_meta_braid(s, p)),
    },
    // Contribution recording
    MethodEntry {
        name: "contribution.record",
        handler: |s, p| Box::pin(contribution::handle_record_contribution(s, p)),
    },
    MethodEntry {
        name: "contribution.record_session",
        handler: |s, p| Box::pin(contribution::handle_record_session(s, p)),
    },
    MethodEntry {
        name: "contribution.record_dehydration",
        handler: |s, p| Box::pin(contribution::handle_record_dehydration(s, p)),
    },
    // Pipeline (provenance trio coordination)
    MethodEntry {
        name: "pipeline.attribute",
        handler: |s, p| Box::pin(contribution::handle_pipeline_attribute(s, p)),
    },
    // Health (wateringHole PRIMAL_IPC_PROTOCOL v3.0)
    MethodEntry {
        name: "health.check",
        handler: |s, p| Box::pin(health::handle_health(s, p)),
    },
    MethodEntry {
        name: "health.liveness",
        handler: |s, p| Box::pin(async move { health::handle_liveness(s, p) }),
    },
    MethodEntry {
        name: "health.readiness",
        handler: |s, p| Box::pin(health::handle_readiness(s, p)),
    },
    // Capability discovery (wateringHole SEMANTIC_METHOD_NAMING v2.1)
    // `capabilities.list` is canonical; `capability.list` retained as alias
    MethodEntry {
        name: "capabilities.list",
        handler: |s, p| Box::pin(async move { capability::handle_capability_list(s, p) }),
    },
    MethodEntry {
        name: "capability.list",
        handler: |s, p| Box::pin(async move { capability::handle_capability_list(s, p) }),
    },
    // MCP tool exposure (airSpring v0.10 pattern for Squirrel AI coordination)
    MethodEntry {
        name: "tools.list",
        handler: |s, p| Box::pin(async move { capability::handle_tools_list(s, p) }),
    },
    MethodEntry {
        name: "tools.call",
        handler: |s, p| Box::pin(capability::handle_tools_call(s, p)),
    },
];

pub(super) fn find_handler(method: &str) -> Option<DispatchFn> {
    METHODS.iter().find(|m| m.name == method).map(|m| m.handler)
}

/// Process a single JSON-RPC request, returning `None` for notifications.
///
/// A notification is a request without an `id` field (per JSON-RPC 2.0 spec,
/// Section 4.1). The server MUST NOT reply to a notification. Note that
/// `"id": null` is a valid request identifier, not a notification.
///
/// Used by both the HTTP handler and the UDS transport.
pub(crate) async fn process_single(
    state: &AppState,
    raw: serde_json::Value,
) -> Option<JsonRpcResponse> {
    let is_notification = raw.as_object().is_some_and(|obj| !obj.contains_key("id"));

    let parsed: JsonRpcRequest = match serde_json::from_value(raw) {
        Ok(r) => r,
        Err(e) => {
            return Some(JsonRpcResponse::error(
                serde_json::Value::Null,
                error_code::PARSE_ERROR,
                format!("Parse error: {e}"),
            ));
        },
    };

    if is_notification {
        return None;
    }

    if parsed.jsonrpc != "2.0" {
        return Some(
            DispatchOutcome::ProtocolError {
                code: error_code::INVALID_REQUEST,
                message: "Invalid JSON-RPC version, expected \"2.0\"".into(),
            }
            .into_response(parsed.id),
        );
    }

    let outcome = dispatch_classified(state, &parsed.method, parsed.params).await;
    Some(outcome.into_response(parsed.id))
}

/// `POST /jsonrpc` — JSON-RPC 2.0 dispatcher with batch and notification support.
///
/// Per JSON-RPC 2.0 spec:
/// - **Single request** (object): dispatched to the appropriate handler.
/// - **Batch request** (array): each element processed independently;
///   responses collected into an array.
/// - **Notifications** (absent `id`): executed but produce no response.
/// - **All-notification batch**: returns `204 No Content`.
/// - **Empty batch**: returns an `Invalid Request` error.
#[instrument(skip_all)]
pub async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Response {
    if let serde_json::Value::Array(batch) = request {
        if batch.is_empty() {
            return Json(JsonRpcResponse::error(
                serde_json::Value::Null,
                error_code::INVALID_REQUEST,
                "Invalid Request: empty batch",
            ))
            .into_response();
        }

        let mut responses = Vec::with_capacity(batch.len());
        for req in batch {
            if let Some(resp) = process_single(&state, req).await {
                responses.push(resp);
            }
        }

        if responses.is_empty() {
            return StatusCode::NO_CONTENT.into_response();
        }

        return Json(responses).into_response();
    }

    (process_single(&state, request).await).map_or_else(
        || StatusCode::NO_CONTENT.into_response(),
        |resp| Json(resp).into_response(),
    )
}

#[cfg(test)]
async fn dispatch(state: &AppState, method: &str, params: serde_json::Value) -> DispatchResult {
    match find_handler(method) {
        Some(handler) => handler(state, params).await,
        None => Err((
            error_code::METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )),
    }
}

/// Dispatch with protocol/application error classification.
async fn dispatch_classified(
    state: &AppState,
    method: &str,
    params: serde_json::Value,
) -> DispatchOutcome {
    let Some(handler) = find_handler(method) else {
        return DispatchOutcome::ProtocolError {
            code: error_code::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
        };
    };

    match handler(state, params).await {
        Ok(value) => DispatchOutcome::Success(value),
        Err((code, message)) => DispatchOutcome::ApplicationError { code, message },
    }
}

// ==================== Helpers (used by domain modules) ====================

pub(crate) fn internal(e: impl std::fmt::Display) -> DispatchError {
    (error_code::INTERNAL_ERROR, e.to_string())
}

/// # Errors
///
/// Returns an error if JSON deserialization of params fails.
pub(crate) fn parse_params<T: serde::de::DeserializeOwned>(
    params: serde_json::Value,
) -> Result<T, DispatchError> {
    serde_json::from_value(params)
        .map_err(|e| (error_code::INVALID_PARAMS, format!("Invalid params: {e}")))
}

/// # Errors
///
/// Returns an error if JSON serialization fails.
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
#[cfg(test)]
mod tests_anchoring;
#[cfg(test)]
mod tests_attribution;
#[cfg(test)]
mod tests_compression;
#[cfg(test)]
mod tests_contribution;
#[cfg(test)]
mod tests_protocol;
#[cfg(test)]
mod tests_provenance;
