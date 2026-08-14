// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Core tests for JSON-RPC 2.0 dispatch: protocol, braid CRUD, health,
//! helpers, and `DispatchOutcome` classification.
//!
//! Domain-specific handler tests live in sibling modules:
//! - `tests_anchoring` — `anchoring.*`
//! - `tests_attribution` — `attribution.*`
//! - `tests_composition` — provenance trio contract tests, NFT seal, witness
//! - `tests_compression` — `compression.*`
//! - `tests_contribution` — `contribution.*` + `pipeline.*`
//! - `tests_provenance` — `provenance.*`
//! - `tests_cross_gate` — cross-gate attribution braids, `source_gate` query

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

mod aliases;
mod braid;
mod contribution;
mod dispatch;
mod health;
mod helpers;
mod helpers_unit;
mod lifecycle;
mod protocol;
