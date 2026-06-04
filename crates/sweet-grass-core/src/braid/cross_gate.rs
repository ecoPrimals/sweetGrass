// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Cross-gate attribution metadata for braids spanning gate boundaries.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::agent::Did;

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
