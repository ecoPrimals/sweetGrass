// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Capability enum for capability-based discovery.
//!
//! Capabilities are the fundamental routing abstraction: a primal advertises
//! what it can do (not who it is), and consumers discover providers at runtime.

use serde::{Deserialize, Serialize};

/// Capabilities that `SweetGrass` can require from other primals.
/// Runtime discovery finds primals offering these capabilities.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Capability {
    /// DID-based signing (offered by identity primals)
    Signing,
    /// Permanent data anchoring (offered by persistence primals)
    Anchoring,
    /// Session event streaming (offered by activity primals)
    SessionEvents,
    /// Service discovery (offered by orchestration primals)
    Discovery,
    /// Compute task execution (offered by compute primals)
    Compute,
    /// Custom capability with identifier
    Custom(String),
}

impl Capability {
    /// Create a custom capability.
    #[must_use]
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Parse a capability from a string representation.
    ///
    /// Uses case-insensitive comparison without allocation for known variants.
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("signing") {
            Some(Self::Signing)
        } else if s.eq_ignore_ascii_case("anchoring") {
            Some(Self::Anchoring)
        } else if s.eq_ignore_ascii_case("sessionevents")
            || s.eq_ignore_ascii_case("session_events")
            || s.eq_ignore_ascii_case("session-events")
        {
            Some(Self::SessionEvents)
        } else if s.eq_ignore_ascii_case("discovery") {
            Some(Self::Discovery)
        } else if s.eq_ignore_ascii_case("compute") {
            Some(Self::Compute)
        } else if let Some(rest) = s
            .strip_prefix("custom:")
            .or_else(|| s.strip_prefix("Custom:"))
            .or_else(|| s.strip_prefix("CUSTOM:"))
        {
            Some(Self::Custom(rest.to_string()))
        } else if !s.is_empty() {
            Some(Self::Custom(s.to_string()))
        } else {
            None
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signing => write!(f, "signing"),
            Self::Anchoring => write!(f, "anchoring"),
            Self::SessionEvents => write!(f, "session_events"),
            Self::Discovery => write!(f, "discovery"),
            Self::Compute => write!(f, "compute"),
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}
