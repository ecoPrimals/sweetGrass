// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Sled storage backend for `SweetGrass`.
//!
//! **DEPRECATED**: sled is unmaintained upstream and its optional `zstd-sys`
//! compression feature introduces a C dependency. Use
//! [`sweet-grass-store-redb`](../sweet_grass_store_redb) instead, which is
//! actively maintained and 100% Pure Rust with zero C dependencies.
//!
//! This crate is feature-gated behind `--features sled` and will be
//! removed in a future major release. Follow rhizoCrypt v0.13.0 migration
//! path for guidance.
//!
//! # Primal Sovereignty
//!
//! Uses `sled` — a Pure Rust embedded database. Note: sled's optional
//! compression feature pulls `zstd-sys` (C dependency), violating ecoBin.
//!
//! # Features
//!
//! - **Pure Rust** — No C/C++ dependencies, no bindgen required
//! - **Embedded** — No external database server required
//! - **ACID Transactions** — Crash-safe with atomic operations
//! - **Trees** — Separate storage for braids, activities, indexes
//!
//! # Usage
//!
//! ```rust,ignore
//! use sweet_grass_store_sled::SledStore;
//!
//! let store = SledStore::open("/path/to/data")?;
//! ```
//!
//! # Trees (like `RocksDB` Column Families)
//!
//! - `braids` — Main Braid storage
//! - `by_hash` — Index by content hash
//! - `by_agent` — Index by `attributed_to` DID
//! - `by_time` — Index by generation time
//! - `activities` — Activity storage

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![expect(
    clippy::missing_const_for_fn,
    reason = "sled API closures prevent const evaluation"
)]

mod error;
mod store;

pub use error::{Result, SledError};
#[expect(
    deprecated,
    reason = "re-exporting deprecated type for migration period"
)]
pub use store::SledStore;

/// Default cache capacity in bytes (1 GiB).
pub const DEFAULT_CACHE_CAPACITY: u64 = 1024 * 1024 * 1024;

/// Default flush interval in milliseconds (1 second).
pub const DEFAULT_FLUSH_EVERY_MS: u64 = 1000;

/// Tree names (similar to column families).
pub mod trees {
    /// Main Braid storage.
    pub const BRAIDS: &str = "braids";
    /// Index by content hash.
    pub const BY_HASH: &str = "by_hash";
    /// Index by agent DID.
    pub const BY_AGENT: &str = "by_agent";
    /// Index by generation time.
    pub const BY_TIME: &str = "by_time";
    /// Index by tags.
    pub const BY_TAG: &str = "by_tag";
    /// Activity storage.
    pub const ACTIVITIES: &str = "activities";
}

/// Default database directory path for Sled backend (self-config).
///
/// Overridden by `SledConfig.path`. This is a local-development fallback;
/// production deployments configure paths via `SweetGrassConfig`.
pub const DEFAULT_DB_PATH: &str = "./sweetgrass_sled";

/// Configuration for Sled store.
#[derive(Clone, Debug)]
pub struct SledConfig {
    /// Path to the database directory.
    pub path: String,

    /// Cache capacity in bytes.
    pub cache_capacity: u64,

    /// Flush interval in milliseconds (0 = sync on every write).
    pub flush_every_ms: Option<u64>,

    /// Use compression.
    pub use_compression: bool,
}

impl Default for SledConfig {
    fn default() -> Self {
        Self {
            path: DEFAULT_DB_PATH.to_string(),
            cache_capacity: DEFAULT_CACHE_CAPACITY,
            flush_every_ms: Some(DEFAULT_FLUSH_EVERY_MS),
            use_compression: false, // Requires 'compression' feature in sled
        }
    }
}

impl SledConfig {
    /// Create a new config with the given path.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set cache capacity.
    #[must_use]
    pub fn cache_capacity(mut self, bytes: u64) -> Self {
        self.cache_capacity = bytes;
        self
    }

    /// Set flush interval.
    #[must_use]
    pub fn flush_every_ms(mut self, ms: Option<u64>) -> Self {
        self.flush_every_ms = ms;
        self
    }
}
