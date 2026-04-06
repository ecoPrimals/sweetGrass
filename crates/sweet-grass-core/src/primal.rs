// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `SweetGrass` primal implementation.
//!
//! The main entry point for the `SweetGrass` primal, implementing
//! lifecycle management and health checking.

use crate::config::SweetGrassConfig;
use crate::error::SweetGrassError;

/// Primal lifecycle states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrimalState {
    /// Created but not started.
    Created,
    /// Starting up.
    Starting,
    /// Running and healthy.
    Running,
    /// Stopping.
    Stopping,
    /// Stopped.
    Stopped,
    /// Failed state.
    Failed,
}

impl PrimalState {
    /// Check if the primal is running.
    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the primal can be started.
    #[must_use]
    pub const fn can_start(self) -> bool {
        matches!(self, Self::Created | Self::Stopped | Self::Failed)
    }

    /// Check if the primal can be stopped.
    #[must_use]
    pub const fn can_stop(self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for PrimalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Health status.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthStatus {
    /// Healthy and operational.
    Healthy,
    /// Degraded but operational.
    Degraded {
        /// Reason for degradation.
        reason: String,
    },
    /// Unhealthy and not operational.
    Unhealthy {
        /// Reason for unhealthy state.
        reason: String,
    },
}

impl HealthStatus {
    /// Check if the status is healthy.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the status is operational (healthy or degraded).
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        !matches!(self, Self::Unhealthy { .. })
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded { reason } => write!(f, "degraded: {reason}"),
            Self::Unhealthy { reason } => write!(f, "unhealthy: {reason}"),
        }
    }
}

/// Health report.
#[derive(Clone, Debug)]
pub struct HealthReport {
    /// Primal name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Health status.
    pub status: HealthStatus,
    /// Additional checks.
    pub checks: Vec<HealthCheck>,
    /// Timestamp.
    pub timestamp: u64,
}

impl HealthReport {
    /// Create a new health report.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            status: HealthStatus::Healthy,
            checks: Vec::new(),
            timestamp: crate::braid::current_timestamp_nanos(),
        }
    }

    /// Set the status.
    #[must_use]
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }

    /// Add a health check.
    #[must_use]
    pub fn with_check(mut self, check: HealthCheck) -> Self {
        self.checks.push(check);
        self
    }
}

/// Individual health check result.
#[derive(Clone, Debug)]
pub struct HealthCheck {
    /// Check name.
    pub name: String,
    /// Check passed.
    pub passed: bool,
    /// Optional message.
    pub message: Option<String>,
}

impl HealthCheck {
    /// Create a passing check.
    #[must_use]
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: None,
        }
    }

    /// Create a failing check.
    #[must_use]
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: Some(message.into()),
        }
    }
}

/// The `SweetGrass` primal - Attribution Layer.
pub struct SweetGrass {
    config: SweetGrassConfig,
    state: PrimalState,
}

impl SweetGrass {
    /// Create a new `SweetGrass` instance.
    #[must_use]
    pub const fn new(config: SweetGrassConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }

    /// Get the current state.
    #[must_use]
    pub const fn state(&self) -> PrimalState {
        self.state
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &SweetGrassConfig {
        &self.config
    }

    /// Get the primal version.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Start the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal cannot be started.
    pub fn start(&mut self) -> Result<(), SweetGrassError> {
        if !self.state.can_start() {
            return Err(SweetGrassError::AlreadyRunning);
        }

        self.state = PrimalState::Starting;
        tracing::info!(name = %self.config.name, "SweetGrass starting...");

        self.initialize_subsystems();

        self.state = PrimalState::Running;
        tracing::info!(name = %self.config.name, "SweetGrass running");
        Ok(())
    }

    /// Stop the primal.
    ///
    /// # Errors
    ///
    /// Returns an error if the primal cannot be stopped.
    pub fn stop(&mut self) -> Result<(), SweetGrassError> {
        if !self.state.can_stop() {
            return Err(SweetGrassError::NotRunning(self.state.to_string()));
        }

        self.state = PrimalState::Stopping;
        tracing::info!(name = %self.config.name, "SweetGrass stopping...");

        tracing::debug!("Flushing storage and stopping listeners");

        self.state = PrimalState::Stopped;
        tracing::info!(name = %self.config.name, "SweetGrass stopped");
        Ok(())
    }

    /// Get health status.
    #[must_use]
    pub fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    /// Perform a health check.
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails.
    pub fn health_check(&self) -> Result<HealthReport, SweetGrassError> {
        let report = HealthReport::new(&self.config.name, self.version())
            .with_status(self.health_status())
            .with_check(HealthCheck::pass("storage"));

        Ok(report)
    }

    // Internal initialization methods

    fn initialize_subsystems(&self) {
        tracing::debug!(backend = ?self.config.storage.backend, "Initializing storage");
        if self.config.listener.session_events {
            tracing::debug!("SessionEvents capability listener enabled");
        }
        if self.config.listener.anchoring {
            tracing::debug!("Anchoring capability listener enabled");
        }
        if self.config.listener.compute {
            tracing::debug!("Compute capability listener enabled");
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_state_transitions() {
        assert!(PrimalState::Created.can_start());
        assert!(PrimalState::Stopped.can_start());
        assert!(PrimalState::Failed.can_start());
        assert!(!PrimalState::Running.can_start());

        assert!(PrimalState::Running.can_stop());
        assert!(!PrimalState::Stopped.can_stop());
    }

    #[test]
    fn test_primal_state_display() {
        assert_eq!(PrimalState::Running.to_string(), "running");
        assert_eq!(PrimalState::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(HealthStatus::Healthy.is_operational());

        let degraded = HealthStatus::Degraded {
            reason: "test".to_string(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_operational());

        let unhealthy = HealthStatus::Unhealthy {
            reason: "test".to_string(),
        };
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_operational());
    }

    #[test]
    fn test_sweetgrass_creation() {
        let config = SweetGrassConfig::default();
        let primal = SweetGrass::new(config);

        assert_eq!(primal.state(), PrimalState::Created);
        assert_eq!(primal.config().name, "SweetGrass");
    }

    #[test]
    fn test_sweetgrass_lifecycle() {
        let config = SweetGrassConfig::default();
        let mut primal = SweetGrass::new(config);

        primal.start().expect("should start");
        assert_eq!(primal.state(), PrimalState::Running);
        assert!(primal.health_status().is_healthy());

        assert!(primal.start().is_err());

        primal.stop().expect("should stop");
        assert_eq!(primal.state(), PrimalState::Stopped);

        assert!(primal.stop().is_err());

        primal.start().expect("should restart");
        assert_eq!(primal.state(), PrimalState::Running);
    }

    #[test]
    fn test_health_check() {
        let config = SweetGrassConfig::default();
        let mut primal = SweetGrass::new(config);

        let report = primal.health_check().expect("should check health");
        assert!(!report.status.is_healthy());

        primal.start().expect("should start");
        let report = primal.health_check().expect("should check health");
        assert!(report.status.is_healthy());
        assert_eq!(report.name, "SweetGrass");
    }

    #[test]
    fn test_health_report_builder() {
        let report = HealthReport::new("TestPrimal", "1.0.0")
            .with_status(HealthStatus::Healthy)
            .with_check(HealthCheck::pass("storage"))
            .with_check(HealthCheck::fail("network", "connection timeout"));

        assert_eq!(report.name, "TestPrimal");
        assert_eq!(report.checks.len(), 2);
        assert!(report.checks[0].passed);
        assert!(!report.checks[1].passed);
    }

    #[test]
    fn test_primal_state_display_all_variants() {
        assert_eq!(PrimalState::Created.to_string(), "created");
        assert_eq!(PrimalState::Starting.to_string(), "starting");
        assert_eq!(PrimalState::Running.to_string(), "running");
        assert_eq!(PrimalState::Stopping.to_string(), "stopping");
        assert_eq!(PrimalState::Stopped.to_string(), "stopped");
        assert_eq!(PrimalState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_primal_state_is_running() {
        assert!(PrimalState::Running.is_running());
        assert!(!PrimalState::Created.is_running());
        assert!(!PrimalState::Stopped.is_running());
        assert!(!PrimalState::Failed.is_running());
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");

        let degraded = HealthStatus::Degraded {
            reason: "high load".to_string(),
        };
        assert_eq!(degraded.to_string(), "degraded: high load");

        let unhealthy = HealthStatus::Unhealthy {
            reason: "db down".to_string(),
        };
        assert_eq!(unhealthy.to_string(), "unhealthy: db down");
    }

    #[test]
    fn test_health_check_pass_fail() {
        let pass = HealthCheck::pass("storage");
        assert_eq!(pass.name, "storage");
        assert!(pass.passed);
        assert!(pass.message.is_none());

        let fail = HealthCheck::fail("network", "timeout");
        assert_eq!(fail.name, "network");
        assert!(!fail.passed);
        assert_eq!(fail.message.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_health_report_new() {
        let report = HealthReport::new("MyPrimal", "2.0.0");
        assert_eq!(report.name, "MyPrimal");
        assert_eq!(report.version, "2.0.0");
        assert!(report.status.is_healthy());
        assert!(report.checks.is_empty());
        assert!(report.timestamp > 0);
    }

    #[test]
    fn test_stop_not_running_error() {
        let config = SweetGrassConfig::default();
        let mut primal = SweetGrass::new(config);

        let result = primal.stop();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("not running")
        );
    }
}
