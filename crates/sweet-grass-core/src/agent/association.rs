// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Agent association with activities — delegation and plan tracking.

use serde::{Deserialize, Serialize};

use super::{AgentRole, Did};

/// Agent's association with an activity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentAssociation {
    /// The agent's DID.
    pub agent: Did,

    /// Role in the activity.
    pub role: AgentRole,

    /// Acting on behalf of another agent.
    #[serde(default)]
    pub on_behalf_of: Option<Did>,

    /// Plan/protocol followed.
    #[serde(default)]
    pub had_plan: Option<String>,
}

impl AgentAssociation {
    /// Create a new agent association.
    #[must_use]
    pub const fn new(agent: Did, role: AgentRole) -> Self {
        Self {
            agent,
            role,
            on_behalf_of: None,
            had_plan: None,
        }
    }

    /// Set the delegation principal.
    #[must_use]
    pub fn on_behalf_of(mut self, principal: Did) -> Self {
        self.on_behalf_of = Some(principal);
        self
    }

    /// Set the plan reference.
    #[must_use]
    pub fn with_plan(mut self, plan: impl Into<String>) -> Self {
        self.had_plan = Some(plan.into());
        self
    }

    /// Check if this is a delegated action.
    #[must_use]
    pub const fn is_delegated(&self) -> bool {
        self.on_behalf_of.is_some()
    }
}
