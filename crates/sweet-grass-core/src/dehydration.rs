// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Dehydration summary types for rhizoCrypt → sweetGrass coordination.
//!
//! When rhizoCrypt dehydrates a session (ephemeral DAG → permanent record),
//! it sends a JSON-RPC payload that sweetGrass deserializes into its own
//! [`DehydrationSummary`]. No shared crate — each primal owns its types
//! and unknown wire fields are silently ignored by serde.
//!
//! ```text
//! rhizoCrypt.dehydrate() → JSON-RPC → sweetGrass.contribution.record_dehydration
//!                        → JSON-RPC → loamSpine.commit.session
//! ```

use serde::{Deserialize, Serialize};

use crate::agent::Did;
use crate::braid::{ContentHash, Timestamp};

/// Summary produced when `rhizoCrypt` dehydrates an ephemeral DAG session
/// into a form suitable for permanence (`LoamSpine`) and attribution (`sweetGrass`).
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

    /// Number of branches explored (defaults to 0 when omitted by older callers).
    #[serde(default)]
    pub branch_count: u64,

    /// Agents who participated in the session.
    pub agents: Vec<Did>,

    /// Session witnesses (signatures, hash observations, checkpoints, markers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<Witness>,

    /// Operations performed during the session.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<SessionOperation>,

    /// When the session was created (defaults to 0 when omitted by older callers).
    #[serde(default)]
    pub session_start: Timestamp,

    /// When dehydration occurred (defaults to 0 when omitted by older callers).
    #[serde(default)]
    pub dehydrated_at: Timestamp,

    /// The frontier hashes (leaf nodes of the DAG at dehydration time).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<ContentHash>,

    /// Niche context (e.g., `rootpulse`, `chemistry`, `game_engine`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,

    /// Compression ratio if the DAG was compressed before dehydration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<f64>,
}

/// A witness that something occurred in a session.
///
/// The trio is agnostic to what a witness contains. A witness may be a
/// cryptographic signature, a hash observation, a game-state checkpoint,
/// a conversation marker, or a bare timestamp. The `kind` field
/// discriminates; the `evidence` field carries the payload (opaque string,
/// empty when the witness needs no payload).
///
/// When the witness is cryptographic (`kind: "signature"`), verification
/// is delegated to `BearDog` (`crypto.verify`) or an external verifier.
/// `sweetGrass` never decodes or validates evidence — it attributes and
/// retains on braids.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Witness {
    /// Agent or system that produced this witness.
    pub agent: Did,

    /// What this witness represents.
    /// `"signature"` = cryptographic signature,
    /// `"hash"` = hash observation, `"checkpoint"` = state snapshot,
    /// `"marker"` = boundary/event marker, `"timestamp"` = bare time witness.
    #[serde(default = "default_witness_kind")]
    pub kind: String,

    /// Evidence payload (opaque). For signatures this is the encoded
    /// signature bytes; for non-crypto witnesses this may be empty or
    /// carry a hash, checkpoint token, or other payload.
    #[serde(default)]
    pub evidence: String,

    /// When the witness was created (defaults to 0 when omitted by callers).
    #[serde(default)]
    pub witnessed_at: Timestamp,

    /// How the evidence payload is encoded. Only meaningful when `evidence`
    /// is non-empty. Values: `"hex"`, `"base64"`, `"base64url"`, `"multibase"`,
    /// `"utf8"` (plain text), `"none"` (no encoding / empty payload).
    #[serde(default = "default_witness_encoding")]
    pub encoding: String,

    /// Cryptographic algorithm (when `kind` = `"signature"`).
    /// `None` for non-crypto witnesses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,

    /// Provenance tier.
    /// `"local"` = same gate, `"gateway"` = remote gate,
    /// `"anchor"` = public chain, `"external"` = third-party,
    /// `"open"` = unsigned / no cryptographic backing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,

    /// Freeform context for the witness.
    /// `"game:tick:42"`, `"conversation:thread:abc"`, `"experiment:run:7"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl Witness {
    /// Create an unsigned placeholder witness (open tier, no evidence).
    #[must_use]
    pub fn unsigned() -> Self {
        Self {
            agent: Did::new(""),
            kind: "marker".to_string(),
            evidence: String::new(),
            witnessed_at: super::braid::types::current_timestamp_nanos(),
            encoding: "none".to_string(),
            algorithm: None,
            tier: Some("open".to_string()),
            context: None,
        }
    }

    /// Create an Ed25519 signature witness from raw bytes.
    #[must_use]
    pub fn from_ed25519(agent: &Did, signature_bytes: &[u8]) -> Self {
        use base64::Engine;
        Self {
            agent: agent.clone(),
            kind: "signature".to_string(),
            evidence: base64::engine::general_purpose::STANDARD.encode(signature_bytes),
            witnessed_at: super::braid::types::current_timestamp_nanos(),
            encoding: "base64".to_string(),
            algorithm: Some("ed25519".to_string()),
            tier: Some("local".to_string()),
            context: None,
        }
    }

    /// Whether this witness carries a cryptographic signature.
    #[must_use]
    pub fn is_signed(&self) -> bool {
        self.kind == "signature" && !self.evidence.is_empty()
    }
}

fn default_witness_kind() -> String {
    "signature".to_string()
}

fn default_witness_encoding() -> String {
    "hex".to_string()
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

    /// When the operation occurred (defaults to 0 when omitted by callers).
    #[serde(default)]
    pub timestamp: Timestamp,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
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
            witnesses: vec![Witness {
                agent: Did::new("did:key:z6MkAlice"),
                kind: "signature".to_string(),
                evidence: "deadbeef01234567".to_string(),
                witnessed_at: 1_000_000,
                encoding: "hex".to_string(),
                algorithm: Some("ed25519".to_string()),
                tier: Some("local".to_string()),
                context: None,
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
        assert_eq!(parsed.witnesses.len(), 1);
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
            witnesses: Vec::new(),
            operations: Vec::new(),
            session_start: 0,
            dehydrated_at: 1,
            frontier: Vec::new(),
            niche: None,
            compression_ratio: None,
        };

        let json = serde_json::to_string(&summary).expect("should serialize");
        assert!(!json.contains("witnesses"));
        assert!(!json.contains("operations"));
        assert!(!json.contains("niche"));
    }

    #[test]
    fn test_witness_roundtrip() {
        let w = Witness {
            agent: Did::new("did:key:z6MkTest"),
            kind: "signature".to_string(),
            evidence: "dGVzdA==".to_string(),
            witnessed_at: 42,
            encoding: "base64".to_string(),
            algorithm: Some("ed25519".to_string()),
            tier: None,
            context: None,
        };
        let json = serde_json::to_string(&w).expect("serialize");
        let parsed: Witness = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.agent.as_str(), "did:key:z6MkTest");
        assert_eq!(parsed.witnessed_at, 42);
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

    #[test]
    fn test_rhizocrypt_payload_compatibility() {
        let payload = serde_json::json!({
            "session_id": "rhizo-session-99",
            "source_primal": "rhizoCrypt",
            "merkle_root": "sha256:abc123",
            "vertex_count": 42,
            "agents": ["did:key:z6MkAlice", "did:key:z6MkBob"],
            "session_type": "experiment",
            "outcome": "Success"
        });

        let parsed: DehydrationSummary =
            serde_json::from_value(payload).expect("rhizoCrypt payload should deserialize");
        assert_eq!(parsed.session_id, "rhizo-session-99");
        assert_eq!(parsed.source_primal, "rhizoCrypt");
        assert_eq!(parsed.vertex_count, 42);
        assert_eq!(parsed.agents.len(), 2);
        assert_eq!(parsed.branch_count, 0);
        assert_eq!(parsed.session_start, 0);
        assert_eq!(parsed.dehydrated_at, 0);
    }

    #[test]
    fn test_enriched_rhizocrypt_payload() {
        let payload = serde_json::json!({
            "session_id": "rhizo-session-100",
            "source_primal": "rhizoCrypt",
            "merkle_root": "sha256:def456",
            "vertex_count": 100,
            "branch_count": 5,
            "agents": ["did:key:z6MkAlice"],
            "session_start": 1_000_000_u64,
            "dehydrated_at": 2_000_000_u64,
            "session_type": "rootpulse",
            "outcome": "Success"
        });

        let parsed: DehydrationSummary =
            serde_json::from_value(payload).expect("enriched payload should deserialize");
        assert_eq!(parsed.vertex_count, 100);
        assert_eq!(parsed.branch_count, 5);
        assert_eq!(parsed.session_start, 1_000_000);
        assert_eq!(parsed.dehydrated_at, 2_000_000);
    }
}
