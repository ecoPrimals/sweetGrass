// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query Engine for `SweetGrass`.
//!
//! This crate provides:
//! - Query execution over Braid stores
//! - Provenance graph traversal
//! - PROV-O JSON-LD export
//! - Attribution chain queries

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod engine;
pub mod error;
pub mod provo;
pub mod traversal;

pub use engine::QueryEngine;
pub use error::QueryError;
pub use provo::{JsonLdDocument, ProvoExport};
pub use traversal::{ProvenanceGraph, ProvenanceGraphBuilder};

/// Result type for query operations.
pub type Result<T> = std::result::Result<T, QueryError>;
