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

/// `capabilities.list` / `capability.list` — Wire Standard L3 (Composable).
///
/// Returns the full self-advertisement envelope per
/// `wateringHole/CAPABILITY_WIRE_STANDARD.md` v1.0:
/// - L1: parseable response with `methods`
/// - L2: `{primal, version, methods}` envelope + `identity.get`
/// - L3: `provided_capabilities`, `consumed_capabilities`,
///   `cost_estimates` (per-method), `operation_dependencies`
pub(super) fn handle_capability_list(
    _state: &AppState,
    _params: serde_json::Value,
) -> DispatchResult {
    use sweet_grass_core::niche;

    let methods: Vec<&str> = METHODS.iter().map(|m| m.name).collect();
    let version = env!("CARGO_PKG_VERSION");

    let mut domains = std::collections::BTreeMap::<&str, Vec<&str>>::new();
    for method in &methods {
        if let Some((domain, operation)) = method.split_once('.') {
            domains.entry(domain).or_default().push(operation);
        }
    }

    let desc_map: std::collections::HashMap<&str, &str> =
        niche::DOMAIN_DESCRIPTIONS.iter().copied().collect();

    let provided_capabilities: Vec<serde_json::Value> = domains
        .iter()
        .map(|(domain, ops)| {
            let mut group = serde_json::json!({
                "type": domain,
                "methods": ops,
            });
            if let Some(desc) = desc_map.get(domain) {
                group["description"] = serde_json::json!(desc);
            }
            group["version"] = serde_json::json!(version);
            group
        })
        .collect();

    let ops = niche::operation_dependencies();

    let mut operations = serde_json::Map::new();
    let mut operation_dependencies = serde_json::Map::new();
    let mut cost_estimates = serde_json::Map::new();

    for op in &ops {
        operations.insert(
            op.method.to_string(),
            serde_json::json!({
                "depends_on": op.depends_on,
                "cost": op.cost,
            }),
        );
        if !op.depends_on.is_empty() {
            operation_dependencies.insert(op.method.to_string(), serde_json::json!(op.depends_on));
        }
        cost_estimates.insert(
            op.method.to_string(),
            serde_json::json!({
                "cpu": op.cost,
                "latency_ms": op.latency_ms,
            }),
        );
    }

    to_value(&serde_json::json!({
        "primal": niche::NICHE_ID,
        "version": version,
        "description": niche::NICHE_DESCRIPTION,
        "protocol": "jsonrpc-2.0",
        "transport": ["http", "uds"],
        "methods": methods,
        "provided_capabilities": provided_capabilities,
        "consumed_capabilities": niche::CONSUMED_CAPABILITIES,
        "cost_estimates": cost_estimates,
        "operation_dependencies": operation_dependencies,
        // Backward-compatible fields (pre-Wire Standard consumers)
        "capabilities": &methods,
        "domains": domains,
        "operations": operations,
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
