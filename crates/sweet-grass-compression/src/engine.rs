// SPDX-License-Identifier: AGPL-3.0-only
//! Compression Engine implementation.
//!
//! Compresses session events into Braids using the 0/1/Many model.

use std::sync::Arc;
use sweet_grass_core::{
    activity::{Activity, ActivityType, UsedEntity},
    agent::{AgentAssociation, AgentRole},
    braid::{BraidId, BraidMetadata, CompressionMeta, EcoPrimalsAttributes},
    entity::EntityReference,
    Braid,
};
use sweet_grass_factory::BraidFactory;

use crate::analyzer::{SessionAnalysis, SessionAnalyzer};
use crate::error::CompressionError;
use crate::session::Session;
use crate::strategy::{CompressionConfig, CompressionStrategy, DiscardReason};
use crate::Result;

/// Default source primal name when discovery has not been used.
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
    pub fn has_braids(&self) -> bool {
        !matches!(self, Self::None { .. })
    }

    /// Get the discard reason if no Braids were produced.
    #[must_use]
    pub fn discard_reason(&self) -> Option<&DiscardReason> {
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
    source_primal: String,
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
            source_primal: DEFAULT_SOURCE_PRIMAL.to_string(),
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
    ///     .with_source(&primal.name);
    /// ```
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
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
            title: Some(format!("Session {}", session.id)),
            description: Some(format!(
                "Compressed from {} vertices ({} committed)",
                analysis.vertex_count, analysis.committed_count
            )),
            ..Default::default()
        };

        // Build ecoPrimals attributes
        // Note: source_primal is discovered at runtime via with_source(),
        // not hardcoded. "unknown" indicates discovery was not used.
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            rhizo_session: Some(session.id.clone()),
            compression: Some(CompressionMeta {
                vertex_count: analysis.vertex_count as u64,
                branch_count: analysis.branch_count as u64,
                #[allow(clippy::cast_precision_loss)]
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
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::session::{SessionOutcome, SessionVertex};
    use sweet_grass_core::agent::Did;

    fn make_factory() -> Arc<BraidFactory> {
        Arc::new(BraidFactory::new(Did::new("did:key:z6MkTestFactory")))
    }

    fn make_vertex(id: &str, hash: &str) -> SessionVertex {
        SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkTest"))
            .with_size(1024)
    }

    #[test]
    fn test_empty_session_discarded() {
        let engine = CompressionEngine::new(make_factory());
        let session = Session::new("empty");

        let result = engine.compress(&session).expect("should compress");

        assert!(!result.has_braids());
        assert_eq!(result.count(), 0);
        assert!(matches!(
            result.discard_reason(),
            Some(DiscardReason::EmptySession)
        ));
    }

    #[test]
    fn test_rollback_discarded() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("rollback");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.finalize(SessionOutcome::Rollback);

        let result = engine.compress(&session).expect("should compress");

        assert!(!result.has_braids());
        assert!(matches!(
            result.discard_reason(),
            Some(DiscardReason::Rollback)
        ));
    }

    #[test]
    fn test_exploratory_discarded() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("exploratory");
        session.add_vertex(make_vertex("v1", "sha256:a")); // Not committed

        let result = engine.compress(&session).expect("should compress");

        assert!(!result.has_braids());
        assert!(matches!(
            result.discard_reason(),
            Some(DiscardReason::ExploratoryOnly)
        ));
    }

    #[test]
    fn test_single_braid() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("single");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");

        assert!(result.has_braids());
        assert_eq!(result.count(), 1);

        if let CompressionResult::Single(braid) = result {
            assert_eq!(braid.data_hash.as_str(), "sha256:b");
            assert!(braid.was_generated_by.is_some());
            assert!(!braid.was_derived_from.is_empty());
            assert!(braid.ecop.compression.is_some());
        } else {
            panic!("Expected Single result");
        }
    }

    #[test]
    fn test_compression_metadata() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("metadata-test");
        session.compute_units = 2.5;
        session.add_vertex(make_vertex("v1", "sha256:root").committed());
        session.add_vertex(
            make_vertex("v2", "sha256:derived")
                .with_parent("v1")
                .committed(),
        );
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");

        if let CompressionResult::Single(braid) = result {
            let compression = braid.ecop.compression.unwrap();
            assert_eq!(compression.vertex_count, 2);
            assert_eq!(compression.branch_count, 0);
            assert!(compression.ratio > 0.0 && compression.ratio < 1.0);

            let activity = braid.was_generated_by.unwrap();
            assert_eq!(activity.ecop.compute_units, Some(2.5));
            assert_eq!(
                activity.ecop.rhizo_session,
                Some("metadata-test".to_string())
            );
        } else {
            panic!("Expected Single result");
        }
    }

    #[test]
    fn test_result_braids_accessor() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("accessor");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");
        let braids = result.braids();

        assert_eq!(braids.len(), 1);
    }

    #[test]
    fn test_with_config() {
        let config = CompressionConfig {
            split_threshold: 5,
            hierarchical_threshold: 10,
            generate_summaries: true,
            ..Default::default()
        };
        let engine = CompressionEngine::new(make_factory()).with_config(config);

        assert_eq!(engine.config().split_threshold, 5);
        assert!(engine.config().generate_summaries);
    }

    #[test]
    fn test_multiple_result_braids() {
        // Test the Multiple variant's braids() accessor
        let braid1 = Braid::builder()
            .data_hash("sha256:multi1")
            .mime_type("application/json")
            .size(100)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("should build");

        let braid2 = Braid::builder()
            .data_hash("sha256:multi2")
            .mime_type("application/json")
            .size(200)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("should build");

        let summary = Braid::builder()
            .data_hash("sha256:summary")
            .mime_type("application/json")
            .size(50)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("should build");

        let result = CompressionResult::Multiple {
            braids: vec![braid1, braid2],
            summary: Some(summary),
        };

        assert!(result.has_braids());
        assert_eq!(result.count(), 3); // 2 braids + 1 summary
        assert_eq!(result.braids().len(), 3);
        assert!(result.discard_reason().is_none());
    }

    #[test]
    fn test_multiple_result_without_summary() {
        let braid = Braid::builder()
            .data_hash("sha256:nosummary")
            .mime_type("application/json")
            .size(100)
            .attributed_to(Did::new("did:key:z6MkTest"))
            .build()
            .expect("should build");

        let result = CompressionResult::Multiple {
            braids: vec![braid],
            summary: None,
        };

        assert!(result.has_braids());
        assert_eq!(result.count(), 1);
        assert_eq!(result.braids().len(), 1);
    }

    #[test]
    fn test_branching_session_produces_multiple() {
        // Create a session with branches to trigger Split strategy
        let config = CompressionConfig {
            split_threshold: 3,
            generate_summaries: true,
            ..Default::default()
        };
        let engine = CompressionEngine::new(make_factory()).with_config(config);

        let mut session = Session::new("branching");
        // Root
        session.add_vertex(make_vertex("root", "sha256:root").committed());
        // Branch 1
        session.add_vertex(
            make_vertex("b1-1", "sha256:b1-1")
                .with_parent("root")
                .committed(),
        );
        session.add_vertex(
            make_vertex("b1-2", "sha256:b1-2")
                .with_parent("b1-1")
                .committed(),
        );
        // Branch 2 from root
        session.add_vertex(
            make_vertex("b2-1", "sha256:b2-1")
                .with_parent("root")
                .committed(),
        );
        session.add_vertex(
            make_vertex("b2-2", "sha256:b2-2")
                .with_parent("b2-1")
                .committed(),
        );
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");
        assert!(result.has_braids());
    }

    #[test]
    fn test_deep_session() {
        // Create a deep session to potentially trigger hierarchical strategy
        let config = CompressionConfig {
            hierarchical_threshold: 3,
            generate_summaries: true,
            ..Default::default()
        };
        let engine = CompressionEngine::new(make_factory()).with_config(config);

        let mut session = Session::new("deep");
        session.add_vertex(make_vertex("v1", "sha256:l1").committed());
        session.add_vertex(make_vertex("v2", "sha256:l2").with_parent("v1").committed());
        session.add_vertex(make_vertex("v3", "sha256:l3").with_parent("v2").committed());
        session.add_vertex(make_vertex("v4", "sha256:l4").with_parent("v3").committed());
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");
        assert!(result.has_braids());
    }

    #[test]
    fn test_session_with_compute_units() {
        let engine = CompressionEngine::new(make_factory());
        let mut session = Session::new("compute");
        // Using a specific test value (not PI)
        let test_compute_units = 3.5;
        session.compute_units = test_compute_units;
        session.add_vertex(make_vertex("v1", "sha256:compute").committed());
        session.finalize(SessionOutcome::Committed);

        let result = engine.compress(&session).expect("should compress");

        if let CompressionResult::Single(braid) = result {
            let activity = braid.was_generated_by.expect("should have activity");
            assert_eq!(activity.ecop.compute_units, Some(test_compute_units));
        } else {
            panic!("Expected Single result");
        }
    }
}
