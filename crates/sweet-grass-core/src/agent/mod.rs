// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Agent data structures — people, software, and organizations that act.
//!
//! Agents are the "who" of provenance — the entities that perform activities
//! and contribute to data creation.

mod agent_type;
mod association;
mod did;
mod role;

pub use agent_type::AgentType;
pub use association::AgentAssociation;
pub use did::Did;
pub use role::AgentRole;

use serde::{Deserialize, Serialize};

/// An agent (person, software, organization).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// Agent identifier (DID).
    #[serde(rename = "@id")]
    pub id: Did,

    /// Agent type.
    #[serde(rename = "@type")]
    pub agent_type: AgentType,

    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
}

impl Agent {
    /// Create a new person agent.
    #[must_use]
    pub fn person(did: Did, name: Option<impl Into<String>>) -> Self {
        let name = name.map(Into::into);
        let agent_type = AgentType::Person { name: name.clone() };
        Self {
            id: did,
            agent_type,
            name,
        }
    }

    /// Create a new software agent.
    #[must_use]
    pub fn software(
        did: Did,
        software_name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        let software_name = software_name.into();
        Self {
            id: did,
            agent_type: AgentType::SoftwareAgent {
                software_name: software_name.clone(),
                version: version.into(),
            },
            name: Some(software_name),
        }
    }

    /// Create a new organization agent.
    #[must_use]
    pub fn organization(did: Did, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: did,
            agent_type: AgentType::Organization {
                name: name.clone(),
                org_type: None,
            },
            name: Some(name),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests;
