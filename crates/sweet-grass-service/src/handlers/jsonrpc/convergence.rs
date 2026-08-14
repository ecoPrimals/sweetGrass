// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Convergence domain handler: one-call provenance chain verification.
//!
//! `convergence.check` verifies the provenance chain for a given content hash:
//! CAS → DAG → Spine → Braid → Signed?
//!
//! This is the trust gate for spring data consumption — a dataset is only
//! "converged" when all five stages are confirmed present.

use serde::{Deserialize, Serialize};
use sweet_grass_core::braid::ContentHash;
use sweet_grass_store::{BraidStore, QueryFilter};

use crate::state::AppState;

use super::{DispatchResult, internal, parse_params, to_value};

/// Parameters for `convergence.check`.
#[derive(Debug, Deserialize)]
pub(super) struct ConvergenceCheckParams {
    /// Content hash to verify provenance chain for.
    data_hash: ContentHash,
}

/// Individual stage in the provenance chain.
#[derive(Debug, Clone, Serialize)]
pub(super) struct ConvergenceStage {
    /// Stage name (e.g., "cas", "dag", "spine", "braid", "signed").
    pub stage: &'static str,
    /// Whether this stage is present/verified.
    pub present: bool,
    /// Optional detail about the stage state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Response from `convergence.check`.
#[derive(Debug, Clone, Serialize)]
pub(super) struct ConvergenceCheckResponse {
    /// The content hash that was checked.
    pub data_hash: String,
    /// Whether the full provenance chain is converged.
    pub converged: bool,
    /// Individual stage results.
    pub stages: Vec<ConvergenceStage>,
    /// Braid ID (if a braid exists for this hash).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub braid_id: Option<String>,
}

/// Handle `convergence.check` — one-call provenance chain verification.
///
/// Verifies: CAS (braid exists) → DAG (session ref) → Spine (ledger commit
/// or loam anchor) → Braid (valid structure) → Signed (crypto witness).
pub(super) async fn handle_convergence_check(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ConvergenceCheckParams = parse_params(params)?;

    let filter = QueryFilter {
        data_hash: Some(p.data_hash.clone()),
        ..QueryFilter::default()
    };

    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .map_err(internal)?;

    let braid = result.braids.into_iter().next();

    let (cas_present, dag_present, spine_present, braid_present, signed_present, braid_id) = braid
        .as_ref()
        .map_or((false, false, false, false, false, None), |b| {
            let dag = b.ecop.session_ref.is_some();
            let spine = b.loam_anchor.is_some() || b.ecop.ledger_commit.is_some();
            let signed = b.is_signed();
            (true, dag, spine, true, signed, Some(b.id.to_string()))
        });

    let dag_detail = braid
        .as_ref()
        .and_then(|b| b.ecop.session_ref.as_ref().map(|s| format!("session: {s}")));

    let spine_detail = braid.as_ref().and_then(|b| {
        b.loam_anchor.as_ref().map_or_else(
            || {
                b.ecop
                    .ledger_commit
                    .as_ref()
                    .map(|c| format!("commit: {}@{}", c.spine_id, c.index))
            },
            |anchor| Some(format!("anchor: {}@{}", anchor.spine_id, anchor.index)),
        )
    });

    let signed_detail = braid.as_ref().filter(|b| b.is_signed()).and_then(|b| {
        b.witness
            .algorithm
            .as_ref()
            .map(|alg| format!("algorithm: {alg}"))
    });

    let stages = vec![
        ConvergenceStage {
            stage: "cas",
            present: cas_present,
            detail: None,
        },
        ConvergenceStage {
            stage: "dag",
            present: dag_present,
            detail: dag_detail,
        },
        ConvergenceStage {
            stage: "spine",
            present: spine_present,
            detail: spine_detail,
        },
        ConvergenceStage {
            stage: "braid",
            present: braid_present,
            detail: None,
        },
        ConvergenceStage {
            stage: "signed",
            present: signed_present,
            detail: signed_detail,
        },
    ];

    let converged = cas_present && dag_present && spine_present && braid_present && signed_present;

    let response = ConvergenceCheckResponse {
        data_hash: p.data_hash.to_string(),
        converged,
        stages,
        braid_id,
    };

    to_value(&response)
}

/// Parameters for `convergence.batch_check`.
#[derive(Debug, Deserialize)]
pub(super) struct ConvergenceBatchCheckParams {
    /// Content hashes to verify.
    data_hashes: Vec<ContentHash>,
}

/// Summary for batch convergence response.
#[derive(Debug, Clone, Serialize)]
struct BatchConvergenceSummary {
    /// Total hashes checked.
    total: usize,
    /// Fully converged count.
    converged: usize,
    /// Partially converged (some stages present).
    partial: usize,
    /// No provenance at all.
    primordial: usize,
}

/// Single item in batch response.
#[derive(Debug, Clone, Serialize)]
struct BatchConvergenceItem {
    data_hash: String,
    converged: bool,
    /// Highest confirmed stage (0-5).
    depth: u8,
}

/// Batch response from `convergence.batch_check`.
#[derive(Debug, Clone, Serialize)]
struct ConvergenceBatchResponse {
    summary: BatchConvergenceSummary,
    items: Vec<BatchConvergenceItem>,
}

const MAX_BATCH_HASHES: usize = 1_000;

/// Handle `convergence.batch_check` — batch provenance chain verification.
pub(super) async fn handle_convergence_batch_check(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ConvergenceBatchCheckParams = parse_params(params)?;

    if p.data_hashes.len() > MAX_BATCH_HASHES {
        return Err(super::DispatchError {
            code: super::error_code::INVALID_PARAMS,
            message: format!(
                "batch_check limited to {MAX_BATCH_HASHES} hashes, got {}",
                p.data_hashes.len()
            ),
            source_detail: None,
        });
    }

    let mut items = Vec::with_capacity(p.data_hashes.len());
    let mut converged_count = 0usize;
    let mut partial_count = 0usize;
    let mut primordial_count = 0usize;

    for hash in &p.data_hashes {
        let filter = QueryFilter {
            data_hash: Some(hash.clone()),
            ..QueryFilter::default()
        };
        let result = state
            .store
            .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
            .await
            .map_err(internal)?;

        let (is_converged, depth) = result.braids.into_iter().next().map_or((false, 0), |b| {
            let d = compute_depth(&b);
            (d == 5, d)
        });

        if is_converged {
            converged_count += 1;
        } else if depth > 0 {
            partial_count += 1;
        } else {
            primordial_count += 1;
        }

        items.push(BatchConvergenceItem {
            data_hash: hash.to_string(),
            converged: is_converged,
            depth,
        });
    }

    let response = ConvergenceBatchResponse {
        summary: BatchConvergenceSummary {
            total: p.data_hashes.len(),
            converged: converged_count,
            partial: partial_count,
            primordial: primordial_count,
        },
        items,
    };

    to_value(&response)
}

/// Compute provenance depth for a braid (count of confirmed stages, 0–5).
///
/// Stages: CAS (1) + DAG (1) + Spine (1) + Braid (1) + Signed (1).
/// If we found a braid object, CAS and Braid are always implicitly present.
fn compute_depth(b: &sweet_grass_core::braid::Braid) -> u8 {
    let cas: u8 = 1;
    let braid_struct: u8 = 1;
    let dag = u8::from(b.ecop.session_ref.is_some());
    let spine = u8::from(b.loam_anchor.is_some() || b.ecop.ledger_commit.is_some());
    let signed = u8::from(b.is_signed());
    cas + dag + spine + braid_struct + signed
}

// ==================== Backpressure ====================

/// Parameters for `convergence.pressure`.
#[derive(Debug, Deserialize)]
pub(super) struct ConvergencePressureParams {
    /// Optional filter (e.g., by `source_primal` or niche).
    #[serde(default)]
    filter: QueryFilter,
    /// Maximum items to scan for pressure calculation.
    #[serde(default = "default_scan_limit")]
    scan_limit: usize,
}

const fn default_scan_limit() -> usize {
    10_000
}

/// Backpressure response: convergence lag as a throttling signal.
#[derive(Debug, Clone, Serialize)]
struct ConvergencePressureResponse {
    /// Total braids scanned.
    total_scanned: usize,
    /// Fully converged (depth 5).
    converged: usize,
    /// Backlog at each depth (0=no braid, 2=CAS+braid only, 3=+DAG, 4=+spine, 5=full).
    backlog_by_depth: [usize; 6],
    /// Pressure metric: ratio of unconverged to total (`0.0` = all converged, `1.0` = none).
    pressure: f64,
    /// Whether downstream should throttle ingestion.
    throttle: bool,
}

/// Pressure threshold above which `throttle` is `true`.
const PRESSURE_THROTTLE_THRESHOLD: f64 = 0.8;

/// Handle `convergence.pressure` — backpressure signal from convergence lag.
///
/// Scans the braid store and reports how much content is piling up at each
/// convergence depth. Downstream pipelines (convoy, `bulk_braid.py`) use this
/// to decide whether to slow down ingestion.
pub(super) async fn handle_convergence_pressure(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: ConvergencePressureParams = parse_params(params)?;

    let filter = QueryFilter {
        limit: Some(p.scan_limit),
        ..p.filter
    };

    let result = state
        .store
        .query(&filter, sweet_grass_store::QueryOrder::NewestFirst)
        .await
        .map_err(internal)?;

    let total = result.braids.len();
    let mut backlog: [usize; 6] = [0; 6];
    let mut converged = 0usize;

    for braid in &result.braids {
        let depth = compute_depth(braid) as usize;
        let idx = depth.min(5);
        backlog[idx] += 1;
        if depth >= 5 {
            converged += 1;
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "pressure ratio is approximate — sub-ulp precision irrelevant"
    )]
    let pressure = if total == 0 {
        0.0
    } else {
        1.0 - (converged as f64 / total as f64)
    };

    let response = ConvergencePressureResponse {
        total_scanned: total,
        converged,
        backlog_by_depth: backlog,
        pressure,
        throttle: pressure > PRESSURE_THROTTLE_THRESHOLD,
    };

    to_value(&response)
}
