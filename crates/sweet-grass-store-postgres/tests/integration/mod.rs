//! PostgreSQL integration tests organized by domain.
//!
//! This module contains comprehensive integration tests for the PostgreSQL
//! backend, organized into logical domains for maintainability.
//!
//! ## Test Organization
//!
//! - `common` — Shared test utilities and helpers
//! - `crud` — Basic CRUD operations (put, get, delete, exists)
//! - `queries` — Query engine and filtering tests
//! - `schema` — Migration and schema validation tests
//! - `activities` — Activity storage and relationships
//! - `concurrency` — Concurrent operations and race conditions
//!
//! ## Running Tests
//!
//! These tests require Docker to run a real PostgreSQL instance:
//!
//! ```bash
//! # Run all integration tests
//! cargo test --package sweet-grass-store-postgres --features integration-tests -- --ignored
//!
//! # Run specific domain
//! cargo test --package sweet-grass-store-postgres --test integration::crud -- --ignored
//! ```

#![cfg(feature = "integration-tests")]
#![allow(clippy::expect_used, clippy::unwrap_used)] // Test code clarity

pub mod common;
pub mod crud;
pub mod queries;
pub mod schema;
pub mod activities;
pub mod concurrency;

