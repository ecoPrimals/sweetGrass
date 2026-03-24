// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Compression Engine implementation.
//!
//! Compresses session events into Braids using the 0/1/Many model.

use std::sync::Arc;
use sweet_grass_core::{
    Braid,
    activity::{Activity, ActivityType, UsedEntity},
    agent::{AgentAssociation, AgentRole},
    braid::{BraidId, BraidMetadata, CompressionMeta, EcoPrimalsAttributes},
    entity::EntityReference,
};
use sweet_grass_factory::BraidFactory;

use crate::Result;
use crate::analyzer::{SessionAnalysis, SessionAnalyzer};
use crate::error::CompressionError;
use crate::session::Session;
use crate::strategy::{CompressionConfig, CompressionStrategy, DiscardReason};

/// Default source primal name when `SelfKnowledge` has not been used.
///
/// Prefer [`CompressionEngine::with_source()`] or construction from self-knowledge
/// so the source primal comes from the primal's `SelfKnowledge` at runtime.
pub const DEFAULT_SOURCE_PRIMAL: &str = "unknown";

/// Result of compression.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CompressionResult {
    /// No Braids produced.
    None {
        /// Reason for discarding.
        reason: DiscardReason,
    },

    /// Single Braid produced.
    Single(Braid),

    /// Multiple Braids produced.
    Multiple {
        /// Individual Braids.
        braids: Vec<Braid>,
        /// Optional summary Braid.
        summary: Option<Braid>,
    },
}

impl CompressionResult {
    /// Get all Braids from the result.
    #[must_use]
    pub fn braids(&self) -> Vec<&Braid> {
        match self {
            Self::None { .. } => Vec::new(),
            Self::Single(b) => vec![b],
            Self::Multiple { braids, summary } => {
                let mut all: Vec<_> = braids.iter().collect();
                if let Some(s) = summary {
                    all.push(s);
                }
                all
            },
        }
    }

    /// Get the count of Braids produced.
    #[must_use]
    pub fn count(&self) -> usize {
        match self {
            Self::None { .. } => 0,
            Self::Single(_) => 1,
            Self::Multiple { braids, summary } => braids.len() + usize::from(summary.is_some()),
        }
    }

    /// Check if any Braids were produced.
    #[must_use]
    pub const fn has_braids(&self) -> bool {
        !matches!(self, Self::None { .. })
    }

    /// Get the discard reason if no Braids were produced.
    #[must_use]
    pub const fn discard_reason(&self) -> Option<&DiscardReason> {
        match self {
            Self::None { reason } => Some(reason),
            _ => None,
        }
    }
}

/// Compression Engine.
pub struct CompressionEngine {
    config: CompressionConfig,
    analyzer: SessionAnalyzer,
    factory: Arc<BraidFactory>,
    /// Source primal name (discovered at runtime, not hardcoded).
    source_primal: Arc<str>,
}

impl CompressionEngine {
    /// Create a new compression engine.
    ///
    /// Note: Use `with_source()` to set the source primal name from discovery.
    /// Defaults to "unknown" if not set.
    #[must_use]
    pub fn new(factory: Arc<BraidFactory>) -> Self {
        let config = CompressionConfig::default();
        Self {
            analyzer: SessionAnalyzer::new(config.clone()),
            config,
            factory,
            source_primal: Arc::from(DEFAULT_SOURCE_PRIMAL),
        }
    }

    /// Set the source primal name from discovery.
    ///
    /// This should be called with the discovered primal's name, not hardcoded.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let primal = discovery.find_one(&Capability::SessionEvents).await?;
    /// let engine = CompressionEngine::new(factory)
    ///     .with_source(primal.name.as_str());
    /// ```
    #[must_use]
    pub fn with_source(mut self, source: impl Into<Arc<str>>) -> Self {
        self.source_primal = source.into();
        self
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(mut self, config: CompressionConfig) -> Self {
        self.config = config.clone();
        self.analyzer = SessionAnalyzer::new(config);
        self
    }

    /// Compress a session to Braids.
    ///
    /// # Errors
    ///
    /// Returns an error if session analysis fails, no committed vertices exist,
    /// Braid construction fails, or the factory fails to create a summary Braid.
    pub fn compress(&self, session: &Session) -> Result<CompressionResult> {
        // 1. Analyze session structure
        let analysis = self.analyzer.analyze(session)?;

        // 2. Select strategy
        let strategy = self.analyzer.select_strategy(&analysis);

        // 3. Execute compression
        match strategy {
            CompressionStrategy::Discard(reason) => Ok(CompressionResult::None { reason }),

            CompressionStrategy::Single => {
                let braid = self.compress_single(session, &analysis)?;
                Ok(CompressionResult::Single(braid))
            },

            CompressionStrategy::Split(_branches) => {
                // For now, fall back to single for each branch
                // Full implementation would create per-branch Braids
                let braid = self.compress_single(session, &analysis)?;
                let summary = if self.config.generate_summaries {
                    Some(self.create_meta_braid(std::slice::from_ref(&braid), &session.id)?)
                } else {
                    None
                };
                Ok(CompressionResult::Multiple {
                    braids: vec![braid],
                    summary,
                })
            },

            CompressionStrategy::Hierarchical(_levels) => {
                // For now, fall back to single with summary
                let braid = self.compress_single(session, &analysis)?;
                let summary = if self.config.generate_summaries {
                    Some(self.create_meta_braid(std::slice::from_ref(&braid), &session.id)?)
                } else {
                    None
                };
                Ok(CompressionResult::Multiple {
                    braids: vec![braid],
                    summary,
                })
            },
        }
    }

    /// Compress to a single Braid.
    fn compress_single(&self, session: &Session, analysis: &SessionAnalysis) -> Result<Braid> {
        // Get the primary output
        let committed = session.committed_vertices();
        let output = committed
            .last()
            .ok_or_else(|| CompressionError::InvalidSession("No committed vertices".to_string()))?;

        // Build activity
        let mut activity_builder = Activity::builder(ActivityType::SessionCommit)
            .started_at(session.started_at)
            .rhizo_session(&session.id)
            .compute_units(session.compute_units);

        if let Some(ended) = session.ended_at {
            activity_builder = activity_builder.ended_at(ended);
        }

        // Add contributor associations
        for contributor in &analysis.contributors {
            activity_builder = activity_builder.associated_with(AgentAssociation::new(
                contributor.clone(),
                AgentRole::Contributor,
            ));
        }

        // Add input entities
        for root in session.roots() {
            activity_builder =
                activity_builder.uses(UsedEntity::new(EntityReference::by_hash(&root.data_hash)));
        }

        let activity = activity_builder.build();

        // Build derivation chain
        let derived_from: Vec<EntityReference> = session
            .roots()
            .iter()
            .map(|r| EntityReference::by_hash(&r.data_hash))
            .collect();

        // Build metadata
        let metadata = BraidMetadata {
            title: Some(format!("Session {}", session.id).into()),
            description: Some(
                format!(
                    "Compressed from {} vertices ({} committed)",
                    analysis.vertex_count, analysis.committed_count
                )
                .into(),
            ),
            ..Default::default()
        };

        // Build ecoPrimals attributes
        // Note: source_primal is discovered at runtime via with_source(),
        // not hardcoded. "unknown" indicates discovery was not used.
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(Arc::clone(&self.source_primal)),
            rhizo_session: Some(session.id.clone()),
            compression: Some(CompressionMeta {
                vertex_count: analysis.vertex_count as u64,
                branch_count: analysis.branch_count as u64,
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "vertex_count is small; u64->f64 precision loss is acceptable for compression ratio"
                )]
                ratio: 1.0 / analysis.vertex_count.max(1) as f64,
                summarizes: Vec::new(),
            }),
            ..Default::default()
        };

        // Create the Braid
        let mut braid = Braid::builder()
            .data_hash(&output.data_hash)
            .mime_type(&output.mime_type)
            .size(output.size)
            .attributed_to(output.agent.clone())
            .generated_by(activity)
            .metadata(metadata)
            .ecop(ecop)
            .build()
            .map_err(CompressionError::Core)?;

        braid.was_derived_from = derived_from;

        Ok(braid)
    }

    /// Create a meta-Braid summarizing other Braids.
    fn create_meta_braid(&self, braids: &[Braid], session_id: &str) -> Result<Braid> {
        let braid_ids: Vec<BraidId> = braids.iter().map(|b| b.id.clone()).collect();

        self.factory
            .session_summary(session_id, braid_ids, None)
            .map_err(CompressionError::Factory)
    }

    /// Get the current configuration.
    #[must_use]
    pub const fn config(&self) -> &CompressionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests;
