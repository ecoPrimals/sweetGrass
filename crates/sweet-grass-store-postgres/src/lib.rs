// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! `PostgreSQL` storage backend for `SweetGrass`.
//!
//! This crate provides a persistent `PostgreSQL` backend implementing
//! the `BraidStore` trait from `sweet-grass-store`.
//!
//! # Features
//!
//! - **Persistent Storage** — Durable Braid storage across restarts
//! - **Connection Pooling** — Efficient connection management via sqlx
//! - **Migrations** — Automatic schema management
//! - **Indexing** — Optimized queries via database indexes
//!
//! # Usage
//!
//! ```rust,ignore
//! use sweet_grass_store_postgres::PostgresStore;
//!
//! let store = PostgresStore::connect("postgresql://localhost/sweetgrass").await?;
//! store.run_migrations().await?;
//! ```
//!
//! # Primal Sovereignty
//!
//! Uses `sqlx` with pure Rust TLS (rustls) — no `OpenSSL` dependency.

#![warn(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, deny(unsafe_code))]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod migrations;
mod store;

pub use error::{PostgresError, Result};
pub use store::PostgresStore;

/// Default maximum number of connections in the pool.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Default connection timeout in seconds.
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;

/// Default idle timeout in seconds.
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

/// Database pool type alias.
pub type Pool = sqlx::PgPool;

/// Database connection options.
#[derive(Clone, Debug)]
pub struct PostgresConfig {
    /// Database connection URL.
    pub database_url: String,

    /// Maximum number of connections in the pool.
    pub max_connections: u32,

    /// Minimum number of idle connections to maintain.
    pub min_connections: u32,

    /// Connection timeout in seconds.
    pub connect_timeout_secs: u64,

    /// Idle timeout in seconds.
    pub idle_timeout_secs: u64,
}

impl PostgresConfig {
    /// Sentinel value indicating no URL was configured.
    const UNCONFIGURED: &str = "";

    /// Check whether a database URL has been explicitly provided.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        !self.database_url.is_empty()
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            // No hardcoded fallback — require explicit configuration.
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| Self::UNCONFIGURED.to_string()),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: 1,
            connect_timeout_secs: DEFAULT_CONNECT_TIMEOUT_SECS,
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
        }
    }
}

impl PostgresConfig {
    /// Create a new config with the given database URL.
    #[must_use]
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }

    /// Create config from environment variables.
    ///
    /// Reads `DATABASE_URL` for the connection string.
    /// Returns `None` if the environment variable is not set.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        std::env::var("DATABASE_URL").ok().map(Self::new)
    }

    /// Set max connections.
    #[must_use]
    pub fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    /// Set min connections.
    #[must_use]
    pub fn min_connections(mut self, n: u32) -> Self {
        self.min_connections = n;
        self
    }
}
