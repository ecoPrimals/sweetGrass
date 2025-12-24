//! Braid Factory - creates Braids from various sources.
//!
//! The factory handles:
//! - Creating Braids from data with computed hashes
//! - Creating Braids from `RhizoCrypt` session summaries
//! - Creating Braids from `LoamSpine` entries
//! - Creating meta-Braids that summarize other Braids
//! - Signing Braids with agent credentials

#![forbid(unsafe_code)]

pub mod attribution;
pub mod error;
pub mod factory;

pub use attribution::{
    AttributionCalculator, AttributionChain, AttributionConfig, ContributorShare,
};
pub use error::FactoryError;
pub use factory::BraidFactory;

/// Result type for factory operations.
pub type Result<T> = std::result::Result<T, FactoryError>;
