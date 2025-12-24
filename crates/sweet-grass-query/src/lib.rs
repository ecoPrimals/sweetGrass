//! Query Engine for `SweetGrass`.
//!
//! This crate provides:
//! - Query execution over Braid stores
//! - Provenance graph traversal
//! - PROV-O JSON-LD export
//! - Attribution chain queries

#![forbid(unsafe_code)]
// Note: These pedantic lints are planned for cleanup in v0.3.0
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

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
