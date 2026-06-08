// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Cross-gate attribution metadata for braids spanning gate boundaries.
//!
//! Provides both the schema types (`CrossGateAttribution`, `CrossGateTrustEvent`)
//! and the weaving logic that maps trust events to fully-populated PROV-O braids.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::activity::{Activity, ActivityMetadata, ActivityType};
use crate::agent::{AgentAssociation, AgentRole, Did};
use crate::braid::Timestamp;

/// Attribution context for braids that span gate boundaries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossGateAttribution {
    /// Gate that originated the cross-gate event.
    pub origin_gate: Arc<str>,
    /// Gate that received/verified the event.
    pub target_gate: Arc<str>,
    /// Type of cross-gate trust event.
    pub trust_event: CrossGateTrustEvent,
    /// DID of the signing agent on the origin gate.
    pub origin_agent: Did,
    /// DID of the verifying agent on the target gate (if known).
    #[serde(default)]
    pub target_agent: Option<Did>,
    /// Family ID binding (BTSP transport trust boundary).
    #[serde(default)]
    pub family_id: Option<String>,
}

impl CrossGateAttribution {
    /// Gate traversal context string (e.g. `"ironGate->strandGate"`).
    #[must_use]
    pub fn gate_context(&self) -> String {
        format!("{}->{}", self.origin_gate, self.target_gate)
    }

    /// Build a PROV-O `Activity` representing this trust event.
    ///
    /// Wires `origin_agent` as the primary creator and, when present,
    /// `target_agent` as a delegation principal via `actedOnBehalfOf`.
    #[must_use]
    pub fn to_activity(&self, started_at: Timestamp) -> Activity {
        let activity_type = self.trust_event.to_activity_type();

        let mut assoc = AgentAssociation::new(self.origin_agent.clone(), AgentRole::Creator);
        if let Some(target) = &self.target_agent {
            assoc = assoc.on_behalf_of(target.clone());
        }

        let metadata = ActivityMetadata {
            description: Some(format!(
                "Cross-gate {} ({})",
                self.trust_event.label(),
                self.gate_context(),
            )),
            ..ActivityMetadata::default()
        };

        Activity::builder(activity_type)
            .associated_with(assoc)
            .started_at(started_at)
            .metadata(metadata)
            .build()
    }

    /// Content hash seed for a trust event braid.
    ///
    /// Deterministic: same `(origin_gate, target_gate, trust_event, origin_agent)`
    /// always yields the same seed so repeated weaves produce the same braid ID.
    #[must_use]
    pub fn content_hash_seed(&self) -> String {
        format!(
            "trust:{}:{}:{}:{}",
            self.origin_gate,
            self.target_gate,
            self.trust_event.label(),
            self.origin_agent.as_str(),
        )
    }
}

/// Cross-gate trust event types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrossGateTrustEvent {
    /// Ed25519 key exchange between gates.
    KeyExchange,
    /// Trust issuer registration.
    TrustIssuerRegistered,
    /// Gate enrollment into mesh.
    GateEnrollment,
    /// Family enrollment completed.
    FamilyEnrollment,
    /// Cross-gate attestation/verification.
    CrossGateAttestation,
    /// Mesh join event.
    MeshJoin,
    /// Mesh leave event.
    MeshLeave,
}

impl CrossGateTrustEvent {
    /// Map to the corresponding PROV-O `ActivityType`.
    #[must_use]
    pub const fn to_activity_type(&self) -> ActivityType {
        match self {
            Self::KeyExchange => ActivityType::KeyExchange,
            Self::TrustIssuerRegistered => ActivityType::TrustEstablishment,
            Self::GateEnrollment | Self::FamilyEnrollment => ActivityType::GateEnrollment,
            Self::CrossGateAttestation => ActivityType::CrossGateAttestation,
            Self::MeshJoin => ActivityType::MeshJoin,
            Self::MeshLeave => ActivityType::MeshLeave,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::KeyExchange => "key_exchange",
            Self::TrustIssuerRegistered => "trust_issuer_registered",
            Self::GateEnrollment => "gate_enrollment",
            Self::FamilyEnrollment => "family_enrollment",
            Self::CrossGateAttestation => "cross_gate_attestation",
            Self::MeshJoin => "mesh_join",
            Self::MeshLeave => "mesh_leave",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_event_activity_type_mapping() {
        assert_eq!(
            CrossGateTrustEvent::KeyExchange.to_activity_type(),
            ActivityType::KeyExchange
        );
        assert_eq!(
            CrossGateTrustEvent::TrustIssuerRegistered.to_activity_type(),
            ActivityType::TrustEstablishment
        );
        assert_eq!(
            CrossGateTrustEvent::GateEnrollment.to_activity_type(),
            ActivityType::GateEnrollment
        );
        assert_eq!(
            CrossGateTrustEvent::FamilyEnrollment.to_activity_type(),
            ActivityType::GateEnrollment
        );
        assert_eq!(
            CrossGateTrustEvent::CrossGateAttestation.to_activity_type(),
            ActivityType::CrossGateAttestation
        );
        assert_eq!(
            CrossGateTrustEvent::MeshJoin.to_activity_type(),
            ActivityType::MeshJoin
        );
        assert_eq!(
            CrossGateTrustEvent::MeshLeave.to_activity_type(),
            ActivityType::MeshLeave
        );
    }

    #[test]
    fn gate_context_format() {
        let cga = CrossGateAttribution {
            origin_gate: Arc::from("ironGate"),
            target_gate: Arc::from("strandGate"),
            trust_event: CrossGateTrustEvent::KeyExchange,
            origin_agent: Did::new("did:key:z6MkOrigin"),
            target_agent: Some(Did::new("did:key:z6MkTarget")),
            family_id: Some("test-family".into()),
        };
        assert_eq!(cga.gate_context(), "ironGate->strandGate");
    }

    #[test]
    fn content_hash_seed_deterministic() {
        let cga = CrossGateAttribution {
            origin_gate: Arc::from("eastGate"),
            target_gate: Arc::from("westGate"),
            trust_event: CrossGateTrustEvent::MeshJoin,
            origin_agent: Did::new("did:key:z6MkAgent"),
            target_agent: None,
            family_id: None,
        };
        let seed1 = cga.content_hash_seed();
        let seed2 = cga.content_hash_seed();
        assert_eq!(seed1, seed2);
        assert!(seed1.starts_with("trust:eastGate:westGate:mesh_join:"));
    }

    #[test]
    fn to_activity_wires_delegation() {
        let cga = CrossGateAttribution {
            origin_gate: Arc::from("ironGate"),
            target_gate: Arc::from("strandGate"),
            trust_event: CrossGateTrustEvent::KeyExchange,
            origin_agent: Did::new("did:key:z6MkOrigin"),
            target_agent: Some(Did::new("did:key:z6MkTarget")),
            family_id: None,
        };
        let now = Timestamp::now();
        let activity = cga.to_activity(now);
        assert_eq!(activity.activity_type, ActivityType::KeyExchange);
        assert_eq!(activity.was_associated_with.len(), 1);
        let assoc = &activity.was_associated_with[0];
        assert_eq!(assoc.agent.as_str(), "did:key:z6MkOrigin");
        assert_eq!(
            assoc.on_behalf_of.as_ref().map(Did::as_str),
            Some("did:key:z6MkTarget")
        );
    }

    #[test]
    fn to_activity_no_delegation_when_target_absent() {
        let cga = CrossGateAttribution {
            origin_gate: Arc::from("eastGate"),
            target_gate: Arc::from("westGate"),
            trust_event: CrossGateTrustEvent::MeshJoin,
            origin_agent: Did::new("did:key:z6MkSolo"),
            target_agent: None,
            family_id: None,
        };
        let activity = cga.to_activity(Timestamp::now());
        assert_eq!(activity.activity_type, ActivityType::MeshJoin);
        assert!(activity.was_associated_with[0].on_behalf_of.is_none());
    }
}
