// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! CLI support extracted from the `UniBin` binary entry point.
//!
//! Contains testable business logic for CLI subcommands (`capabilities`,
//! `socket`, `status`). The binary delegates to these functions so the
//! logic can be exercised by unit tests and contribute to coverage.

use std::net::SocketAddr;

use sweet_grass_core::niche;

/// Errors from the raw TCP health check.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HealthCheckError {
    /// TCP connection or IO failure.
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Server responded with a non-200 status.
    #[error("unhealthy response: {0}")]
    Unhealthy(String),
}

/// Parsed capabilities report.
#[derive(Debug, Clone)]
pub struct CapabilitiesReport {
    /// Niche identifier.
    pub niche_id: String,
    /// Package version.
    pub version: String,
    /// Niche description.
    pub description: String,
    /// Capabilities grouped by domain.
    pub domains: Vec<DomainCapabilities>,
    /// Consumed capabilities.
    pub consumed: Vec<String>,
    /// Dependencies.
    pub dependencies: Vec<DependencyInfo>,
}

/// Capabilities within a single semantic domain.
#[derive(Debug, Clone)]
pub struct DomainCapabilities {
    /// Domain name (e.g. `braid`, `health`).
    pub domain: String,
    /// Operations offered.
    pub operations: Vec<String>,
}

/// Dependency information.
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Capability name.
    pub capability: String,
    /// Whether the dependency is required.
    pub required: bool,
    /// Fallback behavior when unavailable (`"skip"`, `"warn"`, or `"fail"`).
    pub fallback: String,
}

/// Build the capabilities report from niche constants.
#[must_use]
pub fn capabilities_report(version: &str) -> CapabilitiesReport {
    let mut domain_map = std::collections::BTreeMap::<String, Vec<String>>::new();

    for cap in niche::CAPABILITIES {
        if let Some((domain, op)) = cap.split_once('.') {
            domain_map
                .entry(domain.to_string())
                .or_default()
                .push(op.to_string());
        }
    }

    let domains = domain_map
        .into_iter()
        .map(|(domain, operations)| DomainCapabilities { domain, operations })
        .collect();

    let consumed = niche::CONSUMED_CAPABILITIES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let dependencies = niche::DEPENDENCIES
        .iter()
        .map(|dep| DependencyInfo {
            capability: dep.capability.to_string(),
            required: dep.required,
            fallback: dep.fallback.to_string(),
        })
        .collect();

    CapabilitiesReport {
        niche_id: niche::NICHE_ID.to_string(),
        version: version.to_string(),
        description: niche::NICHE_DESCRIPTION.to_string(),
        domains,
        consumed,
        dependencies,
    }
}

/// Format the capabilities report for human display.
///
/// # Panics
///
/// Cannot panic — all `write!` targets are `String` (infallible).
#[must_use]
pub fn format_capabilities_report(report: &CapabilitiesReport) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    let _ = writeln!(
        out,
        "{} v{} — {}\n",
        report.niche_id, report.version, report.description
    );

    let method_count: usize = report.domains.iter().map(|d| d.operations.len()).sum();
    let _ = writeln!(out, "Capabilities ({method_count} methods):");
    for domain in &report.domains {
        let _ = writeln!(out, "  {}:", domain.domain);
        for op in &domain.operations {
            let _ = writeln!(out, "    - {}.{op}", domain.domain);
        }
    }
    out.push('\n');

    let _ = writeln!(out, "Consumed capabilities:");
    for cap in &report.consumed {
        let _ = writeln!(out, "  - {cap}");
    }
    out.push('\n');

    let _ = writeln!(out, "Dependencies:");
    for dep in &report.dependencies {
        let _ = writeln!(
            out,
            "  - {} (required: {}, fallback: {})",
            dep.capability, dep.required, dep.fallback,
        );
    }

    out
}

/// Parse a socket address string, returning a structured error.
///
/// # Errors
///
/// Returns a human-readable message when the address cannot be parsed.
pub fn parse_socket_addr(addr: &str) -> Result<SocketAddr, String> {
    addr.parse()
        .map_err(|e| format!("invalid address '{addr}': {e}"))
}

/// Perform a minimal HTTP GET `/health` check using raw TCP.
///
/// Pure Rust implementation — no reqwest or hyper dependency needed.
/// Parses the HTTP status code numerically rather than matching on reason phrases.
///
/// # Errors
///
/// Returns an error if the TCP connection fails or the response indicates
/// an unhealthy status.
pub async fn http_health_check(address: &str) -> Result<String, HealthCheckError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::net::TcpStream::connect(address).await?;

    let request = format!("GET /health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    stream.shutdown().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();

    let status_code = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);

    if (200..300).contains(&status_code) {
        Ok(body)
    } else {
        let status_line = response
            .lines()
            .next()
            .unwrap_or("(empty response)")
            .to_string();
        Err(HealthCheckError::Unhealthy(status_line))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn parse_socket_addr_valid() {
        let addr = parse_socket_addr("127.0.0.1:8080").expect("valid");
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn parse_socket_addr_ephemeral() {
        let addr = parse_socket_addr("0.0.0.0:0").expect("valid");
        assert_eq!(addr.port(), 0);
    }

    #[test]
    fn parse_socket_addr_invalid() {
        let err = parse_socket_addr("not-an-address").unwrap_err();
        assert!(err.contains("invalid address"), "got: {err}");
    }

    #[test]
    fn parse_socket_addr_missing_port() {
        let err = parse_socket_addr("127.0.0.1").unwrap_err();
        assert!(err.contains("invalid address"), "got: {err}");
    }

    #[test]
    fn capabilities_report_populated() {
        let report = capabilities_report("0.7.27");
        assert_eq!(report.niche_id, "sweetgrass");
        assert!(!report.version.is_empty());
        assert!(!report.description.is_empty());
        assert!(!report.domains.is_empty());
        assert!(!report.consumed.is_empty());
    }

    #[test]
    fn capabilities_report_has_health_domain() {
        let report = capabilities_report("0.7.27");
        let health = report
            .domains
            .iter()
            .find(|d| d.domain == "health")
            .expect("health domain");
        assert!(health.operations.contains(&"check".to_string()));
        assert!(health.operations.contains(&"liveness".to_string()));
        assert!(health.operations.contains(&"readiness".to_string()));
    }

    #[test]
    fn capabilities_report_has_braid_domain() {
        let report = capabilities_report("0.7.27");
        let braid = report
            .domains
            .iter()
            .find(|d| d.domain == "braid")
            .expect("braid domain");
        assert!(braid.operations.contains(&"create".to_string()));
        assert!(braid.operations.contains(&"get".to_string()));
        assert!(braid.operations.contains(&"query".to_string()));
    }

    #[test]
    fn capabilities_report_has_capabilities_domain() {
        let report = capabilities_report("0.7.27");
        let caps = report
            .domains
            .iter()
            .find(|d| d.domain == "capabilities")
            .expect("capabilities domain");
        assert!(caps.operations.contains(&"list".to_string()));
    }

    #[test]
    fn capabilities_report_semantic_naming() {
        let report = capabilities_report("0.7.27");
        for domain in &report.domains {
            assert!(!domain.domain.is_empty(), "domain name must not be empty");
            for op in &domain.operations {
                assert!(
                    !op.is_empty(),
                    "operation name must not be empty in domain {}",
                    domain.domain
                );
            }
        }
    }

    #[test]
    fn capabilities_report_version_passthrough() {
        let report = capabilities_report("1.2.3");
        assert_eq!(report.version, "1.2.3");
    }

    #[test]
    fn format_capabilities_report_includes_sections() {
        let report = capabilities_report("0.7.27");
        let output = format_capabilities_report(&report);

        assert!(output.contains("sweetgrass"));
        assert!(output.contains("0.7.27"));
        assert!(output.contains("Capabilities ("));
        assert!(output.contains("Consumed capabilities:"));
        assert!(output.contains("Dependencies:"));
        assert!(output.contains("health.check"));
        assert!(output.contains("braid.create"));
    }

    #[test]
    fn format_capabilities_report_method_count() {
        let report = capabilities_report("0.7.27");
        let output = format_capabilities_report(&report);
        let expected_count = niche::CAPABILITIES.len();
        assert!(
            output.contains(&format!("Capabilities ({expected_count} methods)")),
            "output: {output}"
        );
    }

    #[test]
    fn capabilities_report_dependencies_present() {
        let report = capabilities_report("0.7.27");
        assert!(
            !report.dependencies.is_empty(),
            "dependencies should not be empty"
        );
        for dep in &report.dependencies {
            assert!(!dep.capability.is_empty());
        }
    }

    #[test]
    fn health_check_error_display() {
        let io_err = HealthCheckError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "refused",
        ));
        assert!(io_err.to_string().contains("refused"));

        let unhealthy = HealthCheckError::Unhealthy("HTTP/1.1 503".to_string());
        assert!(unhealthy.to_string().contains("503"));
    }

    #[tokio::test]
    async fn http_health_check_connection_refused() {
        let result = http_health_check("127.0.0.1:1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn http_health_check_invalid_address() {
        let result = http_health_check("not-a-host:99999").await;
        assert!(result.is_err());
    }
}
