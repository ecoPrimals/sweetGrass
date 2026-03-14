// SPDX-License-Identifier: AGPL-3.0-only
//! redb storage backend for `SweetGrass`.
//!
//! This crate provides a high-performance embedded storage backend
//! implementing the `BraidStore` trait from `sweet-grass-store`.
//!
//! # Primal Sovereignty
//!
//! Uses `redb` — a **100% Pure Rust** embedded database with zero C dependencies.
//! This aligns with ecoPrimals' commitment to vendor independence.
//!
//! # Features
//!
//! - **Pure Rust** — No C/C++ dependencies
//! - **Embedded** — No external database server required
//! - **ACID Transactions** — Crash-safe with atomic operations
//! - **Tables** — Separate storage for braids, activities, indexes

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

mod store;

pub use store::RedbStore;

/// Table definitions (similar to sled trees / column families).
pub mod tables {
    use redb::TableDefinition;

    /// Main Braid storage.
    pub const BRAIDS: TableDefinition<&str, &[u8]> = TableDefinition::new("braids");
    /// Index by content hash.
    pub const BY_HASH: TableDefinition<&str, &[u8]> = TableDefinition::new("by_hash");
    /// Index by agent DID.
    pub const BY_AGENT: TableDefinition<&str, &[u8]> = TableDefinition::new("by_agent");
    /// Index by generation time.
    pub const BY_TIME: TableDefinition<&str, &[u8]> = TableDefinition::new("by_time");
    /// Index by tags.
    pub const BY_TAG: TableDefinition<&str, &[u8]> = TableDefinition::new("by_tag");
    /// Activity storage.
    pub const ACTIVITIES: TableDefinition<&str, &[u8]> = TableDefinition::new("activities");
}

/// Configuration for redb store.
#[derive(Clone, Debug)]
pub struct RedbConfig {
    /// Path to the database file.
    pub path: std::path::PathBuf,
}

impl Default for RedbConfig {
    fn default() -> Self {
        Self {
            path: std::path::PathBuf::from("./sweetgrass.redb"),
        }
    }
}

impl RedbConfig {
    /// Create a new config with the given path.
    #[must_use]
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}
