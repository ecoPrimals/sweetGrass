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
#![forbid(unsafe_code)]
// Note: These pedantic lints are planned for cleanup in v0.3.0
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]

mod error;
mod migrations;
mod store;

pub use error::{PostgresError, Result};
pub use store::PostgresStore;

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

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            // Prefer environment variable for 12-factor app compatibility
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/sweetgrass".to_string()),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
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
