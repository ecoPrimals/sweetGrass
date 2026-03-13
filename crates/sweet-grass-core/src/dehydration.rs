// SPDX-License-Identifier: AGPL-3.0-only
//! Dehydration summary types for rhizoCrypt → sweetGrass coordination.
//!
//! When rhizoCrypt dehydrates a session (ephemeral DAG → permanent record),
//! it produces a `DehydrationSummary` describing the collapsed state. This
//! is the shared contract between rhizoCrypt and sweetGrass for the
//! provenance trio workflow:
//!
//! ```text
//! rhizoCrypt.dehydrate() → DehydrationSummary → sweetGrass.recordSession
//!                                             → LoamSpine.commit
//! ```

use serde::{Deserialize, Serialize};

use crate::agent::Did;
use crate::braid::{ContentHash, Timestamp};

/// Summary produced when rhizoCrypt dehydrates an ephemeral DAG session
/// into a form suitable for permanence (LoamSpine) and attribution (sweetGrass).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DehydrationSummary {
    /// The primal that performed the dehydration (discovered at runtime).
    pub source_primal: String,

    /// The session that was dehydrated.
    pub session_id: String,

    /// Merkle root of the collapsed DAG.
    pub merkle_root: ContentHash,

    /// Number of vertices in the original DAG.
    pub vertex_count: u64,

    /// Number of branches explored.
    pub branch_count: u64,

    /// Agents who participated in the session.
    pub agents: Vec<Did>,

    /// Cryptographic attestations from participating agents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attestations: Vec<Attestation>,

    /// Operations performed during the session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<SessionOperation>,

    /// When the session was created.
    pub session_start: Timestamp,

    /// When dehydration occurred.
    pub dehydrated_at: Timestamp,

    /// The frontier hashes (leaf nodes of the DAG at dehydration time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<ContentHash>,

    /// Niche context (e.g., "rootpulse", "chemistry", "game_engine").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,

    /// Compression ratio if the DAG was compressed before dehydration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
}

/// An agent's attestation that they participated in a session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attestation {
    /// The agent who attested.
    pub agent: Did,

    /// Ed25519 signature over the Merkle root (base64-encoded).
    pub signature: String,

    /// When the attestation was created.
    pub attested_at: Timestamp,
}

/// A high-level operation recorded during a session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionOperation {
    /// Operation type (e.g., "create", "modify", "derive", "merge").
    pub op_type: String,

    /// Content hash of the artifact affected.
    pub content_hash: ContentHash,

    /// The agent who performed the operation.
    pub agent: Did,

    /// When the operation occurred.
    pub timestamp: Timestamp,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::agent::Did;
    use crate::test_fixtures::TEST_SOURCE_PRIMAL;

    fn sample_summary() -> DehydrationSummary {
        DehydrationSummary {
            source_primal: TEST_SOURCE_PRIMAL.to_string(),
            session_id: "rhizo-session-42".to_string(),
            merkle_root: ContentHash::new("sha256:abcdef0123456789"),
            vertex_count: 15,
            branch_count: 3,
            agents: vec![Did::new("did:key:z6MkAlice"), Did::new("did:key:z6MkBob")],
            attestations: vec![Attestation {
                agent: Did::new("did:key:z6MkAlice"),
                signature: "base64sig==".to_string(),
                attested_at: 1_000_000,
            }],
            operations: vec![SessionOperation {
                op_type: "create".to_string(),
                content_hash: ContentHash::new("sha256:op1hash"),
                agent: Did::new("did:key:z6MkAlice"),
                timestamp: 500_000,
                description: Some("Initial creation".to_string()),
            }],
            session_start: 100_000,
            dehydrated_at: 2_000_000,
            frontier: vec![ContentHash::new("sha256:frontier1")],
            niche: Some("rootpulse".to_string()),
            compression_ratio: Some(0.42),
        }
    }

    #[test]
    fn test_dehydration_summary_roundtrip() {
        let summary = sample_summary();
        let json = serde_json::to_string(&summary).expect("should serialize");
        let parsed: DehydrationSummary = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.session_id, "rhizo-session-42");
        assert_eq!(parsed.merkle_root.as_str(), "sha256:abcdef0123456789");
        assert_eq!(parsed.vertex_count, 15);
        assert_eq!(parsed.agents.len(), 2);
        assert_eq!(parsed.attestations.len(), 1);
        assert_eq!(parsed.operations.len(), 1);
    }

    #[test]
    fn test_dehydration_summary_minimal() {
        let summary = DehydrationSummary {
            source_primal: "testPrimal".to_string(),
            session_id: "minimal-session".to_string(),
            merkle_root: ContentHash::new("sha256:minimal"),
            vertex_count: 1,
            branch_count: 0,
            agents: vec![Did::new("did:key:z6MkSolo")],
            attestations: Vec::new(),
            operations: Vec::new(),
            session_start: 0,
            dehydrated_at: 1,
            frontier: Vec::new(),
            niche: None,
            compression_ratio: None,
        };

        let json = serde_json::to_string(&summary).expect("should serialize");
        assert!(!json.contains("attestations"));
        assert!(!json.contains("operations"));
        assert!(!json.contains("niche"));
    }

    #[test]
    fn test_attestation_roundtrip() {
        let att = Attestation {
            agent: Did::new("did:key:z6MkTest"),
            signature: "dGVzdA==".to_string(),
            attested_at: 42,
        };
        let json = serde_json::to_string(&att).expect("serialize");
        let parsed: Attestation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.agent.as_str(), "did:key:z6MkTest");
        assert_eq!(parsed.attested_at, 42);
    }

    #[test]
    fn test_session_operation_roundtrip() {
        let op = SessionOperation {
            op_type: "derive".to_string(),
            content_hash: ContentHash::new("sha256:derived"),
            agent: Did::new("did:key:z6MkDeriver"),
            timestamp: 999,
            description: None,
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let parsed: SessionOperation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.op_type, "derive");
        assert!(!json.contains("description"));
    }
}
