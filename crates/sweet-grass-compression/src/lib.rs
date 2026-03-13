// SPDX-License-Identifier: AGPL-3.0-only
//! Compression Engine for `SweetGrass`.
//!
//! This crate implements the 0/1/Many compression model:
//! - **Zero Braids**: Session explored but produced nothing worth recording
//! - **One Braid**: Single coherent record
//! - **Many Braids**: Multiple Braids with optional meta-summary
//!
//! The compression mirrors fungal leather production: grow the mycelium (DAG exploration),
//! then dry and compress (dehydration to linear provenance).

#![forbid(unsafe_code)]
// Note: These pedantic lints are planned for cleanup in v0.3.0
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

pub mod analyzer;
pub mod engine;
pub mod error;
pub mod session;
pub mod strategy;

pub use analyzer::{SessionAnalysis, SessionAnalyzer};
pub use engine::{CompressionEngine, CompressionResult};
pub use error::CompressionError;
pub use session::{Session, SessionOutcome, SessionVertex};
pub use strategy::{CompressionStrategy, DiscardReason, GroupingStrategy};

/// Result type for compression operations.
pub type Result<T> = std::result::Result<T, CompressionError>;
