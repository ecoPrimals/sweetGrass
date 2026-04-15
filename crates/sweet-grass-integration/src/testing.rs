// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Test utilities for integration testing.
//!
//! Provides helpers for creating test fixtures and managing test infrastructure
//! without hardcoding.

#![cfg(any(test, feature = "test"))]

use std::net::TcpListener;

/// Test bind address for mock services (OS-allocated port).
pub const TEST_BIND_ADDR: &str = "127.0.0.1:0";

/// Test base URL for mock HTTP services (OS-allocated port).
pub const TEST_HTTP_BASE: &str = "http://127.0.0.1:0";

/// Test REST URL for discovery test fixtures.
///
/// Used only as mock data in `DiscoveredPrimal` structs — never bound to.
/// Capability-based discovery resolves real addresses at runtime.
pub const TEST_REST_URL: &str = "http://localhost:8080";

/// Test tarpc address for discovery test fixtures.
///
/// Used only as mock data in `DiscoveredPrimal` structs — never bound to.
/// Capability-based discovery resolves real addresses at runtime.
pub const TEST_TARPC_ADDR: &str = "localhost:9000";

/// Test tarpc URI for discovery test fixtures.
///
/// Used only as mock data in `DiscoveredPrimal` structs — never bound to.
/// Capability-based discovery resolves real addresses at runtime.
pub const TEST_TARPC_URI: &str = "tcp://localhost:9000";

/// Invalid address for testing connection failure (reserved port).
pub const TEST_INVALID_ADDR: &str = "127.0.0.1:1";

// ---------------------------------------------------------------------------
// PostgreSQL test database URLs
// ---------------------------------------------------------------------------

/// Fallback `PostgreSQL` URL when `TEST_DATABASE_URL` is not set (e.g. local dev).
pub const TEST_DB_URL_FALLBACK: &str = "postgresql://postgres:postgres@localhost/sweetgrass_test";

/// `PostgreSQL` URL for generic test database (factory/config tests).
pub const TEST_DB_URL: &str = "postgresql://localhost/test";

/// `PostgreSQL` URL for primary database (preference tests).
pub const TEST_DB_URL_PRIMARY: &str = "postgresql://localhost/primary";

/// `PostgreSQL` URL for secondary database (preference tests).
pub const TEST_DB_URL_SECONDARY: &str = "postgresql://localhost/secondary";

/// Get test database URL from `TEST_DATABASE_URL` env var or fallback.
///
/// Use this for integration tests that need a real `PostgreSQL` connection
/// (migrations, CRUD). Prefer constants for unit tests that only need valid URLs.
#[must_use]
pub fn test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| TEST_DB_URL_FALLBACK.to_string())
}

/// Format a `PostgreSQL` URL for `testcontainers` (dynamic host/port).
///
/// Use when connecting to a containerized `PostgreSQL` instance.
#[must_use]
pub fn postgres_test_url_for_port(port: u16) -> String {
    format!("postgresql://postgres:postgres@127.0.0.1:{port}/postgres")
}

/// Allocate a random port from the operating system.
///
/// This avoids port conflicts in CI/CD pipelines and follows the
/// Infant Discovery principle of zero hardcoding.
///
/// # Panics
///
/// Panics if the OS cannot allocate a port (extremely rare).
/// This is acceptable in test code where failure indicates a system-level issue.
///
/// # Example
///
/// ```rust,ignore
/// let port = allocate_test_port();
/// let addr = format!("127.0.0.1:{port}");
/// let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
/// ```
#[expect(
    clippy::expect_used,
    reason = "test helper: panic on system failure is acceptable"
)]
#[must_use]
pub fn allocate_test_port() -> u16 {
    TcpListener::bind(TEST_BIND_ADDR)
        .expect("OS should allocate port")
        .local_addr()
        .expect("should have local address")
        .port()
}

/// Allocate multiple ports at once.
///
/// Useful when you need multiple test services running simultaneously.
///
/// # Example
///
/// ```rust,ignore
/// let [tarpc_port, rest_port] = allocate_test_ports::<2>();
/// ```
#[must_use]
pub fn allocate_test_ports<const N: usize>() -> [u16; N] {
    let mut ports = [0u16; N];
    for port in &mut ports {
        *port = allocate_test_port();
    }
    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_port() {
        let port = allocate_test_port();
        assert!(port > 0);
    }

    #[test]
    fn test_allocate_multiple_ports() {
        let ports = allocate_test_ports::<3>();
        assert_eq!(ports.len(), 3);

        // All ports should be unique
        assert_ne!(ports[0], ports[1]);
        assert_ne!(ports[1], ports[2]);
        assert_ne!(ports[0], ports[2]);
    }

    #[test]
    fn test_ports_are_available() {
        let port = allocate_test_port();

        // Should be able to bind to the allocated port
        let result = TcpListener::bind(format!("127.0.0.1:{port}"));
        assert!(result.is_ok(), "Port {port} should be available");
    }

    #[test]
    fn test_postgres_test_url_for_port() {
        let url = postgres_test_url_for_port(15432);
        assert_eq!(
            url,
            "postgresql://postgres:postgres@127.0.0.1:15432/postgres"
        );
    }

    #[test]
    fn test_postgres_test_url_for_port_different_ports() {
        let url1 = postgres_test_url_for_port(5432);
        let url2 = postgres_test_url_for_port(5433);
        assert_ne!(url1, url2);
        assert!(url1.contains("5432"));
        assert!(url2.contains("5433"));
    }

    #[test]
    fn test_db_url_returns_string() {
        let url = test_db_url();
        assert!(url.starts_with("postgresql://"));
    }

    #[test]
    fn test_constants_are_valid() {
        assert!(TEST_BIND_ADDR.contains("127.0.0.1"));
        assert!(TEST_HTTP_BASE.starts_with("http://"));
        assert!(TEST_REST_URL.starts_with("http://"));
        assert!(!TEST_TARPC_ADDR.is_empty());
        assert!(!TEST_TARPC_URI.is_empty());
        assert!(TEST_DB_URL.starts_with("postgresql://"));
        assert!(TEST_DB_URL_PRIMARY.starts_with("postgresql://"));
        assert!(TEST_DB_URL_SECONDARY.starts_with("postgresql://"));
        assert!(TEST_DB_URL_FALLBACK.starts_with("postgresql://"));
    }

    #[test]
    fn test_allocate_zero_ports() {
        let ports = allocate_test_ports::<0>();
        assert!(ports.is_empty());
    }
}
