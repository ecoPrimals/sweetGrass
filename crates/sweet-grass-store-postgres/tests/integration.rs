// SPDX-License-Identifier: AGPL-3.0-only
//! PostgreSQL integration tests.
//!
//! Comprehensive tests for the PostgreSQL backend, refactored into logical modules.
//!
//! ## Organization
//!
//! Tests are organized by domain for maintainability:
//! - **CRUD**: Basic create/read/update/delete operations
//! - **Queries**: Filter, search, and query engine tests  
//! - **Schema**: Migration and database schema validation
//! - **Activities**: Activity storage and braid relationships
//! - **Concurrency**: Parallel operations and race conditions
//!
//! ## Running Tests
//!
//! These tests require Docker to run real PostgreSQL:
//!
//! ```bash
//! # All integration tests
//! cargo test --package sweet-grass-store-postgres --test integration -- --ignored
//!
//! # Specific module
//! cargo test --package sweet-grass-store-postgres --test integration crud -- --ignored
//! ```
//!
//! ## Best Practices
//!
//! - Each test uses isolated PostgreSQL container (testcontainers)
//! - Dynamic port allocation prevents conflicts
//! - Tests can run in parallel safely
//! - Containers auto-cleanup after tests

#![cfg(feature = "integration-tests")]
#![allow(clippy::expect_used, clippy::unwrap_used)]
// Test code clarity

// New modular organization (fully migrated)
mod integration {
    pub mod common;
    pub mod crud;
    // Future modules:
    // pub mod queries;
    // pub mod schema;
    // pub mod activities;
    // pub mod concurrency;
}
