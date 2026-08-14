// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! rootPulse graph step handlers for sweetGrass.
//!
//! Handles sweetGrass's role as the attribution step in rootPulse trio
//! graphs. Three graphs exist in `wateringHole/graphs/`:
//!
//! - **`rootpulse_commit`**: sweetGrass step = `attribute_provenance` —
//!   weave a braid linking cascade provenance (`ledger_ref` + `cas_ref`)
//!   to build/depot attribution.
//! - **`rootpulse_harvest`**: no sweetGrass step (rhizoCrypt + nestGate).
//! - **`rootpulse_diff`**: no sweetGrass step (nestGate + membrane).
//!
//! Wire types are sweetGrass-owned. The graph executor sends step inputs
//! as JSON-RPC params; we deserialize without compile-time coupling to
//! biomeOS or cellMembrane.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use sweet_grass_core::braid::{BraidMetadata, EcoPrimalsAttributes, Timestamp};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

// ==================== Wire types (sweetGrass-owned) ====================

/// Input from the `rootpulse_commit` graph executor for the
/// `attribute_provenance` step (`braid.attribute` operation).
///
/// The graph executor passes outputs from prior steps as inputs:
/// - `ledger_ref`: from loamSpine's `ledger_commit` step
/// - `cas_ref`: from nestGate's `store_provenance` step
///
/// Optional context fields are injected by the graph executor from
/// the trigger payload.
#[derive(Debug, serde::Deserialize)]
struct RootPulseAttributeRequest {
    /// loamSpine ledger reference from the `ledger_commit` step.
    ledger_ref: Option<String>,

    /// nestGate CAS reference from the `store_provenance` step.
    cas_ref: Option<String>,

    /// Graph execution session identifier.
    #[serde(default)]
    session_id: Option<String>,

    /// Agent DID of the entity triggering the rootPulse commit.
    #[serde(default)]
    agent_did: Option<String>,

    /// Wave identifier for temporal anchoring.
    #[serde(default)]
    wave_id: Option<String>,

    /// Which rootPulse graph triggered this step.
    #[serde(default)]
    graph_name: Option<String>,

    /// Target triple for build provenance context.
    #[serde(default)]
    target_triple: Option<String>,

    /// Primal name the harvest was for.
    #[serde(default)]
    primal_name: Option<String>,

    /// Commit SHA from the harvest.
    #[serde(default)]
    commit_sha: Option<String>,

    /// BLAKE3 hash of the built binary.
    #[serde(default)]
    blake3_hash: Option<String>,
}

/// Output from the `attribute_provenance` step.
///
/// Per `rootpulse_commit.toml`, outputs `["braid_ref"]`.
#[derive(Debug, serde::Serialize)]
struct RootPulseAttributeResult {
    /// Braid URN created for this rootPulse provenance record.
    braid_ref: String,

    /// Content hash linking the braid to the CAS/ledger provenance.
    content_hash: String,
}

/// Query input for rootPulse provenance braids.
#[derive(Debug, serde::Deserialize)]
struct RootPulseQueryRequest {
    /// Filter by wave ID.
    #[serde(default)]
    wave_id: Option<String>,

    /// Filter by target triple.
    #[serde(default)]
    target_triple: Option<String>,

    /// Filter by primal name.
    #[serde(default)]
    primal_name: Option<String>,

    /// Filter by graph name (`rootpulse_commit`, `rootpulse_harvest`).
    #[serde(default)]
    graph_name: Option<String>,

    /// Maximum results to return.
    #[serde(default = "default_limit")]
    limit: usize,
}

const fn default_limit() -> usize {
    50
}

/// Query result for rootPulse provenance braids.
#[derive(Debug, serde::Serialize)]
struct RootPulseQueryResult {
    braids: Vec<RootPulseProvenanceSummary>,
    total: usize,
}

/// Summary of a rootPulse provenance braid.
#[derive(Debug, serde::Serialize)]
struct RootPulseProvenanceSummary {
    braid_id: String,
    graph_name: Option<String>,
    wave_id: Option<String>,
    target_triple: Option<String>,
    primal_name: Option<String>,
    created_at: Timestamp,
}

// ==================== Handlers ====================

/// Handle the `attribute_provenance` step from `rootpulse_commit` graph.
///
/// Creates a provenance attribution braid linking the loamSpine ledger
/// entry and nestGate CAS reference into a W3C PROV-O record. This
/// braid is the semantic provenance layer for rootPulse — it says
/// "this build produced this binary, signed by this gate, at this time."
///
/// Gracefully handles missing inputs (per graph `fallback = "skip"`
/// semantics) — if `ledger_ref` or `cas_ref` are absent, the braid records
/// what's available.
pub(super) async fn handle_rootpulse_attribute(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let request: RootPulseAttributeRequest = parse_params(params)?;

    let data_hash = build_rootpulse_content_hash(&request);
    let agent_did = request
        .agent_did
        .as_deref()
        .unwrap_or("did:primal:rootpulse");

    let mut metadata = BraidMetadata {
        description: Some(Arc::from("rootPulse provenance attribution braid")),
        ..BraidMetadata::default()
    };

    if let Some(ref ledger_ref) = request.ledger_ref {
        metadata.custom.insert(
            "rootpulse.ledger_ref".to_string(),
            ledger_ref.clone().into(),
        );
    }
    if let Some(ref cas_ref) = request.cas_ref {
        metadata
            .custom
            .insert("rootpulse.cas_ref".to_string(), cas_ref.clone().into());
    }
    if let Some(ref wave_id) = request.wave_id {
        metadata
            .custom
            .insert("rootpulse.wave_id".to_string(), wave_id.clone().into());
    }
    if let Some(ref graph_name) = request.graph_name {
        metadata.custom.insert(
            "rootpulse.graph_name".to_string(),
            graph_name.clone().into(),
        );
    }
    if let Some(ref target) = request.target_triple {
        metadata
            .custom
            .insert("rootpulse.target_triple".to_string(), target.clone().into());
    }
    if let Some(ref primal) = request.primal_name {
        metadata
            .custom
            .insert("rootpulse.primal_name".to_string(), primal.clone().into());
    }
    if let Some(ref commit) = request.commit_sha {
        metadata
            .custom
            .insert("rootpulse.commit_sha".to_string(), commit.clone().into());
    }
    if let Some(ref blake3) = request.blake3_hash {
        metadata
            .custom
            .insert("rootpulse.blake3_hash".to_string(), blake3.clone().into());
    }

    let session_ref = request
        .session_id
        .as_deref()
        .or(request.wave_id.as_deref())
        .unwrap_or("rootpulse");

    let braid = sweet_grass_core::Braid::builder()
        .data_hash(&data_hash)
        .mime_type(sweet_grass_core::identity::MIME_OCTET_STREAM)
        .size(0)
        .attributed_to(sweet_grass_core::agent::Did::new(agent_did))
        .metadata(metadata)
        .ecop(EcoPrimalsAttributes {
            source_primal: Some(Arc::from(sweet_grass_core::identity::PRIMAL_NAME)),
            session_ref: Some(Arc::from(session_ref)),
            niche: Some(Arc::from("rootpulse")),
            ..EcoPrimalsAttributes::default()
        })
        .build()
        .map_err(internal)?;

    let braid_ref = braid.id.to_string();
    state.store.put(&braid).await.map_err(internal)?;

    tracing::info!(
        braid_ref = %braid_ref,
        graph = ?request.graph_name,
        wave = ?request.wave_id,
        "rootPulse attribution braid created"
    );

    to_value(&RootPulseAttributeResult {
        braid_ref,
        content_hash: data_hash,
    })
}

/// Query rootPulse provenance braids by metadata filters.
///
/// Uses the store's `query()` with `niche = "rootpulse"` filter, then
/// applies custom metadata filters (`wave_id`, `target_triple`, etc.) that
/// the store's `QueryFilter` doesn't natively support.
pub(super) async fn handle_rootpulse_query(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let request: RootPulseQueryRequest = parse_params(params)?;

    let filter = QueryFilter {
        niche: Some("rootpulse".to_string()),
        limit: Some(request.limit),
        ..QueryFilter::default()
    };

    let query_result = state
        .store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .map_err(internal)?;

    let mut results = Vec::new();

    for braid in &query_result.braids {
        if let Some(ref filter_wave) = request.wave_id {
            let wave_match = braid
                .metadata
                .custom
                .get("rootpulse.wave_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|w| w == filter_wave);
            if !wave_match {
                continue;
            }
        }

        if let Some(ref filter_target) = request.target_triple {
            let target_match = braid
                .metadata
                .custom
                .get("rootpulse.target_triple")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|t| t == filter_target);
            if !target_match {
                continue;
            }
        }

        if let Some(ref filter_primal) = request.primal_name {
            let primal_match = braid
                .metadata
                .custom
                .get("rootpulse.primal_name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|p| p == filter_primal);
            if !primal_match {
                continue;
            }
        }

        if let Some(ref filter_graph) = request.graph_name {
            let graph_match = braid
                .metadata
                .custom
                .get("rootpulse.graph_name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|g| g == filter_graph);
            if !graph_match {
                continue;
            }
        }

        results.push(RootPulseProvenanceSummary {
            braid_id: braid.id.to_string(),
            graph_name: braid
                .metadata
                .custom
                .get("rootpulse.graph_name")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            wave_id: braid
                .metadata
                .custom
                .get("rootpulse.wave_id")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            target_triple: braid
                .metadata
                .custom
                .get("rootpulse.target_triple")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            primal_name: braid
                .metadata
                .custom
                .get("rootpulse.primal_name")
                .and_then(serde_json::Value::as_str)
                .map(String::from),
            created_at: braid.generated_at_time,
        });

        if results.len() >= request.limit {
            break;
        }
    }

    let total = results.len();
    to_value(&RootPulseQueryResult {
        braids: results,
        total,
    })
}

/// Build a deterministic content hash for rootPulse provenance data.
///
/// Hashes all available provenance references to create a stable
/// content-addressed identifier for this rootPulse attribution.
fn build_rootpulse_content_hash(request: &RootPulseAttributeRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"rootpulse:");

    if let Some(ref ledger_ref) = request.ledger_ref {
        hasher.update(b"ledger:");
        hasher.update(ledger_ref.as_bytes());
    }
    if let Some(ref cas_ref) = request.cas_ref {
        hasher.update(b"cas:");
        hasher.update(cas_ref.as_bytes());
    }
    if let Some(ref wave_id) = request.wave_id {
        hasher.update(b"wave:");
        hasher.update(wave_id.as_bytes());
    }
    if let Some(ref target) = request.target_triple {
        hasher.update(b"target:");
        hasher.update(target.as_bytes());
    }
    if let Some(ref primal) = request.primal_name {
        hasher.update(b"primal:");
        hasher.update(primal.as_bytes());
    }
    if let Some(ref commit) = request.commit_sha {
        hasher.update(b"commit:");
        hasher.update(commit.as_bytes());
    }
    if let Some(ref blake3) = request.blake3_hash {
        hasher.update(b"blake3:");
        hasher.update(blake3.as_bytes());
    }

    format!("{:x}", hasher.finalize())
}
