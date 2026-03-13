// SPDX-License-Identifier: AGPL-3.0-only
//! Agent data structures - people, software, and organizations that act.
//!
//! Agents are the "who" of provenance - the entities that perform activities
//! and contribute to data creation.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Decentralized Identifier (DID).
///
/// Uses `Arc<str>` internally so `.clone()` is O(1) (atomic refcount increment).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Did(Arc<str>);

impl Did {
    /// Create a new DID from a string.
    #[must_use]
    pub fn new(did: impl AsRef<str>) -> Self {
        Self(Arc::from(did.as_ref()))
    }

    /// Get the inner string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a valid DID format.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0.starts_with("did:")
    }

    /// Get the DID method (e.g., "key" from "did:key:...").
    #[must_use]
    pub fn method(&self) -> Option<&str> {
        if !self.is_valid() {
            return None;
        }
        self.0.strip_prefix("did:")?.split(':').next()
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(Arc::from(s.into_boxed_str()))
    }
}

/// Agent types in the PROV model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentType {
    /// Human person.
    Person {
        /// Optional display name.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },

    /// Software agent (AI, bot, service).
    SoftwareAgent {
        /// Software name.
        software_name: String,
        /// Software version.
        version: String,
    },

    /// Organization.
    Organization {
        /// Organization name.
        name: String,
        /// Organization type.
        #[serde(skip_serializing_if = "Option::is_none")]
        org_type: Option<String>,
    },

    /// Hardware device.
    Device {
        /// Device type.
        device_type: String,
        /// Device identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        device_id: Option<String>,
    },
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Person { name: None }
    }
}

/// Roles agents can play in activities.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Agent's association with an activity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentAssociation {
    /// The agent's DID.
    pub agent: Did,

    /// Role in the activity.
    pub role: AgentRole,

    /// Acting on behalf of another agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<Did>,

    /// Plan/protocol followed.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Agent {
    /// Create a new person agent.
    #[must_use]
    pub fn person(did: Did, name: Option<String>) -> Self {
        Self {
            id: did,
            agent_type: AgentType::Person { name: name.clone() },
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
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_did_creation() {
        let did = Did::new("did:key:z6MkTest123");
        assert!(did.is_valid());
        assert_eq!(did.method(), Some("key"));
    }

    #[test]
    fn test_did_invalid() {
        let did = Did::new("not-a-did");
        assert!(!did.is_valid());
        assert_eq!(did.method(), None);
    }

    #[test]
    fn test_did_from_string() {
        let did: Did = "did:web:example.com".into();
        assert!(did.is_valid());
        assert_eq!(did.method(), Some("web"));
    }

    #[test]
    fn test_agent_role_weights() {
        // Use epsilon comparison for floating point
        let epsilon = f64::EPSILON;
        assert!((AgentRole::Creator.default_weight() - 1.0).abs() < epsilon);
        assert!((AgentRole::Contributor.default_weight() - 0.5).abs() < epsilon);
        assert!((AgentRole::ComputeProvider.default_weight() - 0.3).abs() < epsilon);
    }

    #[test]
    fn test_agent_association() {
        let did = Did::new("did:key:z6MkTest");
        let principal = Did::new("did:key:z6MkPrincipal");
        let principal_check = principal.clone();

        let assoc = AgentAssociation::new(did, AgentRole::Creator).on_behalf_of(principal);

        assert!(assoc.is_delegated());
        assert_eq!(assoc.on_behalf_of, Some(principal_check));
    }

    #[test]
    fn test_agent_person() {
        let did = Did::new("did:key:z6MkTest");
        let agent = Agent::person(did.clone(), Some("Alice".to_string()));

        assert_eq!(agent.id, did);
        assert_eq!(agent.name, Some("Alice".to_string()));
        assert!(matches!(agent.agent_type, AgentType::Person { .. }));
    }

    #[test]
    fn test_agent_software() {
        let did = Did::new("did:key:z6MkBot");
        let agent = Agent::software(did, "SweetGrass", "0.1.0");

        assert_eq!(agent.name, Some("SweetGrass".to_string()));
        assert!(matches!(
            agent.agent_type,
            AgentType::SoftwareAgent { software_name, version }
            if software_name == "SweetGrass" && version == "0.1.0"
        ));
    }

    #[test]
    fn test_agent_serialization() {
        let did = Did::new("did:key:z6MkTest");
        let agent = Agent::person(did, Some("Bob".to_string()));

        let json = serde_json::to_string(&agent).expect("should serialize");
        assert!(json.contains("@id"));
        assert!(json.contains("Person"));

        let parsed: Agent = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.name, Some("Bob".to_string()));
    }

    #[test]
    fn test_did_as_str() {
        let did = Did::new("did:key:z6MkHello");
        assert_eq!(did.as_str(), "did:key:z6MkHello");
    }

    #[test]
    fn test_did_display() {
        let did = Did::new("did:key:z6MkDisplay");
        assert_eq!(format!("{did}"), "did:key:z6MkDisplay");
    }

    #[test]
    fn test_did_from_owned_string() {
        let did = Did::from("did:web:example.com".to_string());
        assert!(did.is_valid());
        assert_eq!(did.as_str(), "did:web:example.com");
    }

    #[test]
    fn test_did_roundtrip_json() {
        let did = Did::new("did:key:z6MkRoundtrip");
        let json = serde_json::to_string(&did).expect("serialize");
        let parsed: Did = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, did);
    }

    #[test]
    fn test_agent_type_default() {
        let default = AgentType::default();
        assert!(matches!(default, AgentType::Person { name: None }));
    }

    #[test]
    fn test_agent_role_display_custom() {
        let custom = AgentRole::Custom("MyRole".to_string());
        assert_eq!(format!("{custom}"), "MyRole");
    }

    #[test]
    fn test_agent_role_display_standard() {
        assert_eq!(format!("{}", AgentRole::Creator), "Creator");
        assert_eq!(format!("{}", AgentRole::Contributor), "Contributor");
        assert_eq!(format!("{}", AgentRole::Publisher), "Publisher");
    }

    #[test]
    fn test_agent_role_all_weights() {
        assert!((AgentRole::Publisher.default_weight() - 0.1).abs() < f64::EPSILON);
        assert!((AgentRole::Validator.default_weight() - 0.1).abs() < f64::EPSILON);
        assert!((AgentRole::DataProvider.default_weight() - 0.4).abs() < f64::EPSILON);
        assert!((AgentRole::Transformer.default_weight() - 0.3).abs() < f64::EPSILON);
        assert!((AgentRole::StorageProvider.default_weight() - 0.2).abs() < f64::EPSILON);
        assert!((AgentRole::Curator.default_weight() - 0.2).abs() < f64::EPSILON);
        assert!((AgentRole::Orchestrator.default_weight() - 0.15).abs() < f64::EPSILON);
        assert!((AgentRole::Owner.default_weight() - 0.8).abs() < f64::EPSILON);
        assert!((AgentRole::Custom("x".to_string()).default_weight() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_agent_association_with_plan() {
        let did = Did::new("did:key:z6MkPlanner");
        let assoc = AgentAssociation::new(did, AgentRole::Orchestrator).with_plan("protocol-v2");
        assert_eq!(assoc.had_plan, Some("protocol-v2".to_string()));
        assert!(!assoc.is_delegated());
    }

    #[test]
    fn test_agent_organization() {
        let did = Did::new("did:web:orgexample.com");
        let agent = Agent::organization(did.clone(), "Test Org");
        assert_eq!(agent.id, did);
        assert_eq!(agent.name, Some("Test Org".to_string()));
        assert!(matches!(
            agent.agent_type,
            AgentType::Organization { name, org_type: None } if name == "Test Org"
        ));
    }

    #[test]
    fn test_agent_type_device() {
        let agent_type = AgentType::Device {
            device_type: "sensor".to_string(),
            device_id: Some("sensor-42".to_string()),
        };
        let json = serde_json::to_string(&agent_type).expect("serialize");
        assert!(json.contains("Device"));
        let parsed: AgentType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, AgentType::Device { .. }));
    }
}
