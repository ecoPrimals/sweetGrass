// SPDX-License-Identifier: AGPL-3.0-only
//! Attribution chain types — contributor shares and chain structure.

use std::collections::HashMap;

use sweet_grass_core::{
    agent::{AgentRole, Did},
    entity::EntityReference,
};

/// A share of attribution for a contributor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContributorShare {
    /// The agent receiving the share.
    pub agent: Did,

    /// The share amount (0.0 to 1.0).
    pub share: f64,

    /// The role that earned this share.
    pub role: AgentRole,

    /// Whether this is a direct contribution or inherited.
    pub direct: bool,

    /// Depth in the derivation chain (0 = direct).
    pub depth: u32,
}

impl ContributorShare {
    /// Create a new contributor share.
    #[must_use]
    pub fn new(agent: Did, share: f64, role: AgentRole, direct: bool) -> Self {
        Self {
            agent,
            share,
            role,
            direct,
            depth: u32::from(!direct),
        }
    }

    /// Create with explicit depth.
    #[must_use]
    pub const fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

/// Attribution chain for an entity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributionChain {
    /// The entity being attributed.
    pub entity: EntityReference,

    /// All contributors and their shares.
    pub contributors: Vec<ContributorShare>,

    /// Total compute units involved.
    pub total_compute: f64,

    /// Maximum depth of derivation chain.
    pub max_depth: u32,
}

impl AttributionChain {
    /// Create a new attribution chain for an entity.
    #[must_use]
    pub const fn new(entity: EntityReference) -> Self {
        Self {
            entity,
            contributors: Vec::new(),
            total_compute: 0.0,
            max_depth: 0,
        }
    }

    /// Add a contributor share.
    pub fn add_contributor(&mut self, share: ContributorShare) {
        if share.depth > self.max_depth {
            self.max_depth = share.depth;
        }
        self.contributors.push(share);
    }

    /// Normalize shares to sum to 1.0.
    pub fn normalize(&mut self) {
        let total: f64 = self.contributors.iter().map(|c| c.share).sum();
        if total > 0.0 {
            for contributor in &mut self.contributors {
                contributor.share /= total;
            }
        }
    }

    /// Aggregate shares by agent.
    #[must_use]
    pub fn aggregate_by_agent(&self) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();
        for contributor in &self.contributors {
            *aggregated
                .entry(contributor.agent.as_str().to_string())
                .or_insert(0.0) += contributor.share;
        }
        aggregated
    }

    /// Get direct contributors only.
    #[must_use]
    pub fn direct_contributors(&self) -> Vec<&ContributorShare> {
        self.contributors.iter().filter(|c| c.direct).collect()
    }

    /// Get inherited contributors only.
    #[must_use]
    pub fn inherited_contributors(&self) -> Vec<&ContributorShare> {
        self.contributors.iter().filter(|c| !c.direct).collect()
    }

    /// Check if the chain is valid (shares sum to ~1.0).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let total: f64 = self.contributors.iter().map(|c| c.share).sum();
        (total - 1.0).abs() < 0.001
    }
}
