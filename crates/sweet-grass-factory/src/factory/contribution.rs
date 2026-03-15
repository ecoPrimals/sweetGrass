// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Inter-primal Braid creation: `from_contribution`, `from_session`.

use sweet_grass_core::{
    ContentHash,
    activity::{Activity, ActivityMetadata, ActivityType},
    agent::AgentAssociation,
    braid::{Braid, BraidMetadata, EcoPrimalsAttributes, LoamCommitRef},
    contribution::{ContributionRecord, SessionContribution},
};

use crate::Result;
use crate::error::FactoryError;

/// Parse `loam_entry` string into [`LoamCommitRef`].
/// Format: `spine_id|entry_hash|index` (pipe-separated).
fn parse_loam_entry(s: Option<&str>) -> Option<LoamCommitRef> {
    let s = s?;
    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() != 3 {
        return None;
    }
    let index = parts[2].parse::<u64>().ok()?;
    Some(LoamCommitRef {
        spine_id: parts[0].to_string(),
        entry_hash: ContentHash::new(parts[1]),
        index,
    })
}

impl super::BraidFactory {
    /// Create a Braid from a single contribution record.
    ///
    /// This is the primary inter-primal interface: another primal sends a
    /// `ContributionRecord`, and sweetGrass creates a provenance Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if braid construction fails.
    pub fn from_contribution(&self, record: &ContributionRecord) -> Result<Braid> {
        let activity_type = if record.session_id.is_some() {
            ActivityType::SessionCommit
        } else {
            ActivityType::Creation
        };

        let activity = Activity::builder(activity_type)
            .associated_with(AgentAssociation::new(
                record.agent.clone(),
                record.role.clone(),
            ))
            .metadata(ActivityMetadata {
                description: record.description.clone(),
                ..ActivityMetadata::default()
            })
            .build();

        let mut metadata = BraidMetadata {
            custom: record.domain.clone(),
            ..BraidMetadata::default()
        };
        if let Some(desc) = &record.description {
            metadata.description = Some(desc.clone());
        }

        let source_primal = record
            .source_primal
            .clone()
            .unwrap_or_else(|| self.source_primal.clone());

        let ecop = EcoPrimalsAttributes {
            source_primal: Some(source_primal),
            niche: self.niche.clone(),
            rhizo_session: record.session_id.clone(),
            ..EcoPrimalsAttributes::default()
        };

        let mut braid = Braid::builder()
            .data_hash(record.content_hash.clone())
            .mime_type(record.mime_type.clone())
            .size(record.size)
            .attributed_to(record.agent.clone())
            .generated_by(activity)
            .metadata(metadata)
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        if record.timestamp != 0 {
            braid.generated_at_time = record.timestamp;
        }

        Ok(braid)
    }

    /// Create braids from a session contribution (batch).
    ///
    /// When rhizoCrypt dehydrates a session, it sends a `SessionContribution`
    /// containing multiple contribution records. This creates one braid per
    /// contribution and returns them all.
    ///
    /// # Errors
    ///
    /// Returns an error if any braid construction fails.
    pub fn from_session(&self, session: &SessionContribution) -> Result<Vec<Braid>> {
        let loam_commit = parse_loam_entry(session.loam_entry.as_deref());

        let mut braids = Vec::with_capacity(session.contributions.len());
        for contrib in &session.contributions {
            let mut record = contrib.clone();
            if record.session_id.is_none() {
                record.session_id = Some(session.session_id.clone());
            }
            if record.source_primal.is_none() {
                record.source_primal = Some(session.source_primal.clone());
            }

            let mut braid = self.from_contribution(&record)?;

            if let Some(ref niche) = session.niche {
                braid.ecop.niche = Some(niche.clone());
            }
            if let Some(ref loam) = loam_commit {
                braid.ecop.loam_commit = Some(loam.clone());
            }

            braids.push(braid);
        }

        Ok(braids)
    }
}
