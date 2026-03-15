// SPDX-License-Identifier: AGPL-3.0-only
//! Test utilities for integration testing.
//!
//! Provides helpers for creating test fixtures and managing test infrastructure
//! without hardcoding.

#![cfg(any(test, feature = "test-support"))]

use std::net::TcpListener;

/// Test bind address for mock services (OS-allocated port).
pub const TEST_BIND_ADDR: &str = "127.0.0.1:0";

/// Test base URL for mock HTTP services (OS-allocated port).
pub const TEST_HTTP_BASE: &str = "http://127.0.0.1:0";

/// Test REST URL for discovery test fixtures (arbitrary port, not for binding).
pub const TEST_REST_URL: &str = "http://localhost:8080";

/// Test tarpc address for discovery test fixtures (arbitrary port, not for binding).
pub const TEST_TARPC_ADDR: &str = "localhost:9000";

/// Test tarpc URI for discovery test fixtures (arbitrary port, not for binding).
pub const TEST_TARPC_URI: &str = "tcp://localhost:9000";

/// Invalid address for testing connection failure (reserved port).
pub const TEST_INVALID_ADDR: &str = "127.0.0.1:1";

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
}
