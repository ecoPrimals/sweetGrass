// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability and MCP tool exposure handlers.
//!
//! Required by wateringHole `SEMANTIC_METHOD_NAMING_STANDARD v2.1`:
//! - `capabilities.list` (canonical) / `capability.list` (alias)
//! - `tools.list` + `tools.call` (MCP pattern from airSpring v0.10)
//!
//! `capabilities.list` advertises what this primal offers so that Songbird
//! and other primals can discover it at runtime.
//!
//! `tools.list` / `tools.call` expose braid operations as MCP tools
//! for Squirrel AI coordination (airSpring v0.10 pattern).
//!
//! Delegates to `sweet_grass_core::niche` for the canonical source of
//! truth — no inline duplication of capability metadata.

use super::{DispatchResult, METHODS, error_code, parse_params, to_value};
use crate::state::AppState;

/// `capabilities.list` / `capability.list` — advertise every domain,
/// operation, dependency, and cost hint this primal serves.
///
/// Returns a structured response with primal identity, version, protocol
/// metadata, domain-grouped methods, and per-operation dependency/cost
/// information for intelligent niche dispatch.
pub(super) fn handle_capability_list(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    use sweet_grass_core::niche;

    let methods: Vec<&str> = METHODS.iter().map(|m| m.name).collect();

    let mut domains = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for method in &methods {
        if let Some((domain, operation)) = method.split_once('.') {
            domains.entry(domain).or_default().push(operation);
        }
    }

    let mut operations = serde_json::Map::new();
    for op in niche::operation_dependencies() {
        operations.insert(
            op.method.to_string(),
            serde_json::json!({
                "depends_on": op.depends_on,
                "cost": op.cost,
            }),
        );
    }

    to_value(&serde_json::json!({
        "primal": niche::NICHE_ID,
        "version": env!("CARGO_PKG_VERSION"),
        "description": niche::NICHE_DESCRIPTION,
        "protocol": "jsonrpc-2.0",
        "transport": ["http", "uds"],
        "capabilities": &methods,
        "domains": domains,
        "methods": methods,
        "operations": operations,
        "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        "cost_estimates": niche::cost_estimates().into_iter().collect::<std::collections::BTreeMap<_, _>>(),
    }))
}

/// MCP tool descriptor for AI coordination.
#[derive(serde::Serialize)]
struct McpTool {
    name: &'static str,
    description: &'static str,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

/// `tools.list` — MCP-compatible tool listing for Squirrel AI coordination.
///
/// Returns tools in the MCP `tools/list` format so Squirrel (or other
/// MCP-aware agents) can discover sweetGrass braid operations.
/// Pattern adopted from airSpring v0.10.
pub(super) fn handle_tools_list(_state: &AppState, _params: serde_json::Value) -> DispatchResult {
    let tools = vec![
        McpTool {
            name: "braid.create",
            description: "Create a new attribution braid from content hash and metadata",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "data_hash": {"type": "string", "description": "Content hash (sha256:...)"},
                    "mime_type": {"type": "string", "description": "MIME type of the content"},
                    "size": {"type": "integer", "description": "Content size in bytes"},
                },
                "required": ["data_hash", "mime_type", "size"]
            }),
        },
        McpTool {
            name: "braid.get",
            description: "Retrieve an attribution braid by its ID",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Braid ID (urn:braid:uuid:...)"},
                },
                "required": ["id"]
            }),
        },
        McpTool {
            name: "braid.query",
            description: "Query braids with filters (agent, hash, mime type, tags, time range)",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "attributed_to": {"type": "string", "description": "Filter by agent DID"},
                    "data_hash": {"type": "string", "description": "Filter by content hash"},
                    "mime_type": {"type": "string", "description": "Filter by MIME type prefix"},
                    "tag": {"type": "string", "description": "Filter by tag"},
                    "limit": {"type": "integer", "description": "Max results (default 100)"},
                    "offset": {"type": "integer", "description": "Pagination offset"},
                }
            }),
        },
        McpTool {
            name: "provenance.graph",
            description: "Build the provenance graph for an entity (who created what, derived from what)",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "entity_hash": {"type": "string", "description": "Content hash to trace"},
                    "max_depth": {"type": "integer", "description": "Max traversal depth (default 10)"},
                },
                "required": ["entity_hash"]
            }),
        },
        McpTool {
            name: "attribution.chain",
            description: "Calculate the full attribution chain for content (who contributed what share)",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "hash": {"type": "string", "description": "Content hash to trace attribution for"},
                },
                "required": ["hash"]
            }),
        },
        McpTool {
            name: "health.check",
            description: "Check sweetGrass health status including store and integration status",
            input_schema: serde_json::json!({"type": "object"}),
        },
        McpTool {
            name: "capabilities.list",
            description: "List all capabilities, methods, and domains this primal offers",
            input_schema: serde_json::json!({"type": "object"}),
        },
    ];

    to_value(&serde_json::json!({ "tools": tools }))
}

/// `tools.call` — MCP-compatible tool invocation for AI coordination.
///
/// Dispatches a named tool with parameters through the standard JSON-RPC
/// dispatch table. Pattern adopted from airSpring v0.10.
///
/// # Errors
///
/// Returns `INVALID_PARAMS` if the request lacks a `name` field, or
/// `METHOD_NOT_FOUND` if the named tool is not in the dispatch table.
pub(super) async fn handle_tools_call(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    #[derive(serde::Deserialize)]
    struct ToolCallParams {
        name: String,
        #[serde(default)]
        arguments: serde_json::Value,
    }

    let call: ToolCallParams = parse_params(params)?;

    let handler = super::find_handler(&call.name).ok_or_else(|| {
        (
            error_code::METHOD_NOT_FOUND,
            format!("Tool not found: {}", call.name),
        )
    })?;

    let result = handler(state, call.arguments).await?;
    to_value(&serde_json::json!({
        "content": [{"type": "text", "text": result}],
        "isError": false,
    }))
}
