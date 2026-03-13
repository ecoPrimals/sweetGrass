// SPDX-License-Identifier: AGPL-3.0-only
//! Contribution recording types for inter-primal attribution.
//!
//! When a primal completes work (e.g., rhizoCrypt dehydrates a session),
//! it sends a `ContributionRecord` to sweetGrass. SweetGrass converts this
//! into a provenance Braid with proper W3C PROV-O attribution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::{AgentRole, Did};
use crate::braid::Timestamp;

/// A contribution from an agent to be recorded as provenance.
///
/// This is the inter-primal payload: another primal sends this to sweetGrass
/// to record attribution. SweetGrass converts it into a Braid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContributionRecord {
    /// The agent who made the contribution.
    pub agent: Did,

    /// The role this agent played.
    pub role: AgentRole,

    /// Content hash of the artifact being attributed.
    pub content_hash: String,

    /// MIME type of the content.
    #[serde(default = "default_mime_type")]
    pub mime_type: String,

    /// Size in bytes (0 if unknown).
    #[serde(default)]
    pub size: u64,

    /// When the contribution occurred (nanos since epoch, 0 = now).
    #[serde(default)]
    pub timestamp: Timestamp,

    /// Optional description of what was done.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source primal that is reporting this contribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_primal: Option<String>,

    /// Session ID from the source primal (e.g., rhizoCrypt session).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Domain-specific metadata (extensible for chemistry, ML, games, etc.).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub domain: HashMap<String, serde_json::Value>,
}

fn default_mime_type() -> String {
    "application/octet-stream".to_string()
}

/// A batch of contributions from a session dehydration or similar event.
///
/// When rhizoCrypt dehydrates a session, it may produce multiple contributions
/// (one per participant, or one per changed artifact). This groups them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionContribution {
    /// Session identifier from the source primal.
    pub session_id: String,

    /// The primal reporting these contributions.
    pub source_primal: String,

    /// Niche context (e.g., "rootpulse", "game_engine", "chemistry").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,

    /// Individual contributions within this session.
    pub contributions: Vec<ContributionRecord>,

    /// When the session started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_start: Option<Timestamp>,

    /// When the session ended (dehydration time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_end: Option<Timestamp>,

    /// LoamSpine entry reference (if already committed to permanent record).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loam_entry: Option<String>,

    /// Domain-specific session metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub domain: HashMap<String, serde_json::Value>,
}

/// Well-known domain keys for the `domain` metadata field.
///
/// These are conventions, not enforcement — any string key is valid.
pub mod domain_key {
    /// Chemistry domain (wetSpring): molecule identifier.
    pub const CHEMISTRY_MOLECULE: &str = "chemistry.molecule";
    /// Chemistry domain: basis set used.
    pub const CHEMISTRY_BASIS_SET: &str = "chemistry.basis_set";
    /// Chemistry domain: functional used.
    pub const CHEMISTRY_FUNCTIONAL: &str = "chemistry.functional";
    /// Chemistry domain: campaign identifier.
    pub const CHEMISTRY_CAMPAIGN: &str = "chemistry.campaign";

    /// ML domain: model identifier.
    pub const ML_MODEL: &str = "ml.model";
    /// ML domain: dataset identifier.
    pub const ML_DATASET: &str = "ml.dataset";
    /// ML domain: training run identifier.
    pub const ML_TRAINING_RUN: &str = "ml.training_run";

    /// Game domain (ludoSpring): engagement identifier.
    pub const GAME_ENGAGEMENT: &str = "game.engagement";
    /// Game domain: player identifier.
    pub const GAME_PLAYER: &str = "game.player";
    /// Game domain: session type.
    pub const GAME_SESSION_TYPE: &str = "game.session_type";
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::agent::Did;

    #[test]
    fn test_contribution_record_serialization_roundtrip() {
        let record = ContributionRecord {
            agent: Did::new("did:key:z6MkTest"),
            role: AgentRole::Creator,
            content_hash: "sha256:abc123".to_string(),
            mime_type: "application/json".to_string(),
            size: 100,
            timestamp: 1_234_567_890,
            description: Some("Test contribution".to_string()),
            source_primal: Some("rhizoCrypt".to_string()),
            session_id: Some("session-1".to_string()),
            domain: {
                let mut m = HashMap::new();
                m.insert("test.key".to_string(), serde_json::json!("value"));
                m
            },
        };

        let json = serde_json::to_string(&record).expect("should serialize");
        let parsed: ContributionRecord = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.agent.as_str(), record.agent.as_str());
        assert_eq!(parsed.content_hash, record.content_hash);
        assert_eq!(
            parsed.domain.get("test.key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_session_contribution_serialization_roundtrip() {
        let session = SessionContribution {
            session_id: "session-123".to_string(),
            source_primal: "rhizoCrypt".to_string(),
            niche: Some("chemistry".to_string()),
            contributions: vec![ContributionRecord {
                agent: Did::new("did:key:z6MkAgent1"),
                role: AgentRole::Creator,
                content_hash: "sha256:hash1".to_string(),
                mime_type: "application/json".to_string(),
                size: 50,
                timestamp: 0,
                description: None,
                source_primal: None,
                session_id: None,
                domain: HashMap::new(),
            }],
            session_start: Some(1000),
            session_end: Some(2000),
            loam_entry: None,
            domain: HashMap::new(),
        };

        let json = serde_json::to_string(&session).expect("should serialize");
        let parsed: SessionContribution = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.session_id, session.session_id);
        assert_eq!(parsed.contributions.len(), 1);
    }

    #[test]
    fn test_domain_key_constants_non_empty() {
        assert!(!domain_key::CHEMISTRY_MOLECULE.is_empty());
        assert!(!domain_key::CHEMISTRY_BASIS_SET.is_empty());
        assert!(!domain_key::CHEMISTRY_FUNCTIONAL.is_empty());
        assert!(!domain_key::CHEMISTRY_CAMPAIGN.is_empty());
        assert!(!domain_key::ML_MODEL.is_empty());
        assert!(!domain_key::ML_DATASET.is_empty());
        assert!(!domain_key::ML_TRAINING_RUN.is_empty());
        assert!(!domain_key::GAME_ENGAGEMENT.is_empty());
        assert!(!domain_key::GAME_PLAYER.is_empty());
        assert!(!domain_key::GAME_SESSION_TYPE.is_empty());
    }
}
