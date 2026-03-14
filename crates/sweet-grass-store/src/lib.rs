// SPDX-License-Identifier: AGPL-3.0-only
//! Storage backends for `SweetGrass` Braids.
//!
//! This crate provides the `BraidStore` trait and implementations
//! for persisting and querying Braid provenance records.

#![forbid(unsafe_code)]

pub mod error;
pub mod memory;
pub mod traits;

pub use error::StoreError;
pub use memory::MemoryStore;
pub use traits::{
    BraidStore, IndexStore, QueryFilter, QueryOrder, QueryResult, DEFAULT_QUERY_LIMIT,
};

/// Result type for store operations.
pub type Result<T> = std::result::Result<T, StoreError>;
