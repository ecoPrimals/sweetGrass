//! Test utilities for integration testing.
//!
//! Provides helpers for creating test fixtures and managing test infrastructure
//! without hardcoding.

#![cfg(any(test, feature = "test-support"))]

use std::net::TcpListener;

/// Allocate a random port from the operating system.
///
/// This avoids port conflicts in CI/CD pipelines and follows the
/// Infant Discovery principle of zero hardcoding.
///
/// # Panics
///
/// Panics if the OS cannot allocate a port (extremely rare).
///
/// # Example
///
/// ```rust,ignore
/// let port = allocate_test_port();
/// let addr = format!("127.0.0.1:{port}");
/// let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
/// ```
pub fn allocate_test_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
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

