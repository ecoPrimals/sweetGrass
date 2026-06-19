// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Agent roles in the PROV activity model with attribution weights.

use serde::{Deserialize, Serialize};

/// Roles agents can play in activities.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentRole {
    /// Primary creator/author.
    Creator,

    /// Contributor (partial contribution).
    #[default]
    Contributor,

    /// Publisher/distributor.
    Publisher,

    /// Validator/reviewer.
    Validator,

    /// Data source provider.
    DataProvider,

    /// Compute resource provider.
    ComputeProvider,

    /// Storage resource provider.
    StorageProvider,

    /// Orchestrator/coordinator.
    Orchestrator,

    /// Curator (organized/validated).
    Curator,

    /// Transformer (modified/derived).
    Transformer,

    /// Owner (rights holder).
    Owner,

    /// Custom role.
    Custom(String),
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(name) => write!(f, "{name}"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl AgentRole {
    /// Get the default weight for this role in attribution calculations.
    #[must_use]
    pub const fn default_weight(&self) -> f64 {
        match self {
            Self::Creator => 1.0,
            Self::Contributor => 0.5,
            Self::Publisher | Self::Validator => 0.1,
            Self::DataProvider => 0.4,
            Self::ComputeProvider | Self::Transformer => 0.3,
            Self::StorageProvider | Self::Curator | Self::Custom(_) => 0.2,
            Self::Orchestrator => 0.15,
            Self::Owner => 0.8,
        }
    }
}
