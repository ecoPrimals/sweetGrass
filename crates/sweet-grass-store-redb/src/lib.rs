// SPDX-License-Identifier: AGPL-3.0-only
//! redb storage backend for `SweetGrass`.
//!
//! This crate provides a Pure Rust embedded storage backend implementing the
//! `BraidStore` trait from `sweet-grass-store`, using redb.
//!
//! # Primal Sovereignty
//!
//! Uses `redb` — a **100% Pure Rust** embedded database with zero C dependencies.
//! This aligns with ecoPrimals' commitment to vendor independence.
//!
//! # Features
//!
//! - **Pure Rust** — No C/C++ dependencies, no bindgen required
//! - **Embedded** — No external database server required
//! - **ACID Transactions** — Crash-safe with atomic operations
//! - **Tables** — Separate storage for braids, activities, indexes
//!
//! # Usage
//!
//! ```rust,ignore
//! use sweet_grass_store_redb::RedbStore;
//!
//! let store = RedbStore::open_path("/path/to/data.redb")?;
//! ```
//!
//! # Tables (like `RocksDB` Column Families)
//!
//! - `braids` — Main Braid storage
//! - `by_hash` — Index by content hash
//! - `by_agent` — Index by `attributed_to` DID
//! - `by_time` — Index by generation time
//! - `activities` — Activity storage

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod store;

pub use error::{RedbError, Result};
pub use store::RedbStore;

/// Configuration for redb store.
#[derive(Clone, Debug)]
pub struct RedbConfig {
    /// Path to the database file.
    pub path: String,
}

impl Default for RedbConfig {
    fn default() -> Self {
        Self {
            path: "./sweetgrass_redb".to_string(),
        }
    }
}

impl RedbConfig {
    /// Create a new config with the given path.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

/// Table name constants (similar to sled trees / column families).
pub mod tables {
    use redb::TableDefinition;

    /// Main Braid storage.
    pub const BRAIDS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("braids");
    /// Index by content hash.
    pub const BY_HASH: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_hash");
    /// Index by agent DID.
    pub const BY_AGENT: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_agent");
    /// Index by generation time.
    pub const BY_TIME: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_time");
    /// Index by tags.
    pub const BY_TAG: TableDefinition<&[u8], &[u8]> = TableDefinition::new("by_tag");
    /// Activity storage.
    pub const ACTIVITIES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("activities");
}
