//! # SweetGrass
//!
//! Attribution Layer - Semantic Provenance & PROV-O
//!
//! ## Overview
//!
//! SweetGrass is part of the ecoPrimals ecosystem. It provides query and
//! attribution capabilities on top of RhizoCrypt's DAG engine.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use sweet_grass_core::SweetGrass;
//!
//! let primal = SweetGrass::new(config);
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

use sourdough_core::{
    PrimalLifecycle, PrimalHealth, PrimalState,
    HealthStatus, health::HealthReport, PrimalError,
};

/// SweetGrass configuration.
pub use config::SweetGrassConfig;

/// SweetGrass errors.
pub use error::SweetGrassError;

/// The SweetGrass primal - Attribution Layer.
pub struct SweetGrass {
    #[allow(dead_code)]
    config: SweetGrassConfig,
    state: PrimalState,
}

impl SweetGrass {
    /// Create a new SweetGrass instance.
    #[must_use]
    pub fn new(config: SweetGrassConfig) -> Self {
        Self {
            config,
            state: PrimalState::Created,
        }
    }
}

impl PrimalLifecycle for SweetGrass {
    fn state(&self) -> PrimalState {
        self.state
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Starting;
        tracing::info!("SweetGrass starting...");
        
        // TODO: Initialize resources
        
        self.state = PrimalState::Running;
        tracing::info!("SweetGrass running");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        self.state = PrimalState::Stopping;
        tracing::info!("SweetGrass stopping...");
        
        // TODO: Clean up resources
        
        self.state = PrimalState::Stopped;
        tracing::info!("SweetGrass stopped");
        Ok(())
    }
}

impl PrimalHealth for SweetGrass {
    fn health_status(&self) -> HealthStatus {
        if self.state.is_running() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: format!("state: {}", self.state),
            }
        }
    }

    async fn health_check(&self) -> Result<HealthReport, PrimalError> {
        Ok(HealthReport::new("SweetGrass", env!("CARGO_PKG_VERSION"))
            .with_status(self.health_status()))
    }
}
