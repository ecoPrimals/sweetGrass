// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `PostgreSQL` integration tests.
//!
//! Comprehensive tests for the `PostgreSQL` backend, refactored into logical modules.
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
//! These tests require a running `PostgreSQL` instance (via Docker or native):
//!
//! ```bash
//! # Start Postgres
//! docker run --rm -d -p 5432:5432 \
//!   -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=postgres \
//!   -e POSTGRES_DB=sweetgrass_test --name sg-test-pg postgres:16
//!
//! # Run integration tests
//! DATABASE_URL="postgres://postgres:postgres@localhost:5432/sweetgrass_test" \
//!   cargo test -p sweet-grass-store-postgres --test integration \
//!   --features integration-tests
//!
//! # Specific module
//! DATABASE_URL="..." \
//!   cargo test -p sweet-grass-store-postgres --test integration crud \
//!   --features integration-tests
//! ```
//!
//! ## Best Practices
//!
//! - `DATABASE_URL` env var specifies the connection string
//! - Tests can share a single Postgres instance safely
//! - No `testcontainers` / `bollard` / `ring` in the dep tree

#![cfg(feature = "integration-tests")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

// New modular organization (fully migrated)
mod integration {
    pub mod activities;
    pub mod common;
    pub mod concurrency;
    pub mod crud;
    pub mod queries;
    pub mod schema;
}
