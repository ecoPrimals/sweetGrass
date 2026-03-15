// SPDX-License-Identifier: AGPL-3.0-only
//! Session analyzer for compression strategy selection.
//!
//! Analyzes session structure to determine optimal compression strategy.

use std::collections::HashSet;
use sweet_grass_core::{agent::Did, ContentHash};

use crate::session::{CompressionHint, Session, SessionOutcome};
use crate::strategy::{
    BranchSpec, CompressionConfig, CompressionLevel, CompressionStrategy, DiscardReason,
    GroupingStrategy,
};
use crate::Result;

/// Session analysis result.
#[derive(Clone, Debug)]
pub struct SessionAnalysis {
    /// Number of vertices.
    pub vertex_count: usize,

    /// Number of branch points.
    pub branch_count: usize,

    /// Maximum depth of DAG.
    pub max_depth: usize,

    /// Convergence measure (1.0 = fully linear).
    pub convergence: f64,

    /// Unique output hashes.
    pub unique_outputs: Vec<ContentHash>,

    /// All contributors.
    pub contributors: HashSet<Did>,

    /// Activity type distribution.
    pub activity_types: std::collections::HashMap<String, usize>,

    /// Semantic coherence (0.0 to 1.0).
    pub semantic_coherence: f64,

    /// Temporal span in nanoseconds.
    pub temporal_span_ns: u64,

    /// Session outcome.
    pub outcome: SessionOutcome,

    /// Compression hint.
    pub hint: CompressionHint,

    /// Committed vertex count.
    pub committed_count: usize,
}

/// Session analyzer.
pub struct SessionAnalyzer {
    config: CompressionConfig,
}

impl SessionAnalyzer {
    /// Create a new analyzer.
    #[must_use]
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Analyze a session.
    pub fn analyze(&self, session: &Session) -> Result<SessionAnalysis> {
        let tips = session.tips();
        let tip_count = tips.len();
        let branch_count = session.branch_count();

        // Calculate convergence
        let convergence = if branch_count == 0 {
            1.0 // Fully linear
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "tip_count and branch_count are small; precision loss acceptable for convergence ratio"
            )]
            {
                tip_count as f64 / branch_count.max(1) as f64
            }
        };

        // Calculate semantic coherence
        let semantic_coherence = Self::measure_coherence(session);

        // Activity type distribution
        let mut activity_types = std::collections::HashMap::new();
        for vertex in &session.vertices {
            *activity_types
                .entry(vertex.activity_type.to_string())
                .or_insert(0) += 1;
        }

        Ok(SessionAnalysis {
            vertex_count: session.vertex_count(),
            branch_count,
            max_depth: session.max_depth(),
            convergence,
            unique_outputs: session.unique_outputs().into_iter().cloned().collect(),
            contributors: session.contributors(),
            activity_types,
            semantic_coherence,
            temporal_span_ns: session.temporal_span(),
            outcome: session.outcome.clone(),
            hint: session.compression_hint.clone(),
            committed_count: session.committed_vertices().len(),
        })
    }

    /// Select compression strategy based on analysis.
    #[must_use]
    pub fn select_strategy(&self, analysis: &SessionAnalysis) -> CompressionStrategy {
        // Check hints first (if honoring)
        if self.config.honor_hints {
            match &analysis.hint {
                CompressionHint::Ephemeral => {
                    return CompressionStrategy::Discard(DiscardReason::Ephemeral);
                },
                CompressionHint::Single | CompressionHint::Atomic => {
                    if analysis.committed_count > 0 {
                        return CompressionStrategy::Single;
                    }
                },
                _ => {},
            }
        }

        // Check for discard conditions
        if analysis.outcome == SessionOutcome::Rollback {
            return CompressionStrategy::Discard(DiscardReason::Rollback);
        }

        if analysis.vertex_count == 0 {
            return CompressionStrategy::Discard(DiscardReason::EmptySession);
        }

        if analysis.committed_count == 0 {
            return CompressionStrategy::Discard(DiscardReason::ExploratoryOnly);
        }

        // Check for single Braid
        if analysis.vertex_count < self.config.split_threshold
            && analysis.semantic_coherence > self.config.coherence_threshold
            && analysis.unique_outputs.len() <= 1
        {
            return CompressionStrategy::Single;
        }

        // Check for split
        if analysis.branch_count > 0 && analysis.convergence < 0.5 {
            // Would identify branches here
            return CompressionStrategy::Split(Vec::new());
        }

        // Check for hierarchical
        if analysis.vertex_count > self.config.hierarchical_threshold {
            let levels = self.plan_hierarchy(analysis);
            return CompressionStrategy::Hierarchical(levels);
        }

        // Default to single
        CompressionStrategy::Single
    }

    /// Measure semantic coherence.
    fn measure_coherence(session: &Session) -> f64 {
        let tips = session.tips();

        if tips.len() <= 1 {
            return 1.0;
        }

        // Heuristic: fewer tips relative to vertices = more coherent
        #[expect(
            clippy::cast_precision_loss,
            reason = "tip count and vertex_count are small; precision loss acceptable for coherence heuristic"
        )]
        let tip_ratio = tips.len() as f64 / session.vertex_count().max(1) as f64;

        // Invert: more tips = less coherent
        1.0 - tip_ratio.min(1.0)
    }

    /// Plan hierarchical compression levels.
    fn plan_hierarchy(&self, analysis: &SessionAnalysis) -> Vec<CompressionLevel> {
        let mut levels = Vec::new();

        // Level 0: Group by fixed size
        levels.push(CompressionLevel::new(0, GroupingStrategy::fixed_size(10)).with_max_size(10));

        // Level 1: Group by activity type
        levels.push(CompressionLevel::new(1, GroupingStrategy::ActivityType).with_max_size(20));

        // Level 2: Top-level summary
        if analysis.vertex_count > self.config.hierarchical_threshold * 2 {
            levels
                .push(CompressionLevel::new(2, GroupingStrategy::fixed_size(5)).with_max_size(10));
        }

        levels
    }

    /// Identify branches for splitting.
    #[must_use]
    pub fn identify_branches(&self, session: &Session) -> Vec<BranchSpec> {
        let mut branches = Vec::new();
        let tips = session.tips();

        // Simple strategy: one branch per tip
        for (i, tip) in tips.iter().enumerate() {
            let spec =
                BranchSpec::new(format!("branch-{i}"), tip.id.clone()).with_tip(tip.id.clone());
            branches.push(spec);
        }

        branches
    }
}

impl Default for SessionAnalyzer {
    fn default() -> Self {
        Self::new(CompressionConfig::default())
    }
}

#[cfg(test)]
#[expect(
    clippy::float_cmp,
    clippy::expect_used,
    reason = "test module: float comparison, expect are standard in tests"
)]
mod tests {
    use super::*;
    use crate::session::SessionVertex;

    fn make_vertex(id: &str, hash: &str) -> SessionVertex {
        SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkTest"))
    }

    #[test]
    fn test_empty_session_analysis() {
        let session = Session::new("empty");
        let analyzer = SessionAnalyzer::default();

        let analysis = analyzer.analyze(&session).expect("should analyze");

        assert_eq!(analysis.vertex_count, 0);
        assert_eq!(analysis.committed_count, 0);
    }

    #[test]
    fn test_single_vertex_analysis() {
        let mut session = Session::new("single");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");

        assert_eq!(analysis.vertex_count, 1);
        assert_eq!(analysis.committed_count, 1);
        assert_eq!(analysis.convergence, 1.0);
        assert_eq!(analysis.semantic_coherence, 1.0);
    }

    #[test]
    fn test_branching_analysis() {
        let mut session = Session::new("branching");
        session.add_vertex(make_vertex("root", "sha256:root").committed());
        session.add_vertex(
            make_vertex("b1", "sha256:b1")
                .with_parent("root")
                .committed(),
        );
        session.add_vertex(
            make_vertex("b2", "sha256:b2")
                .with_parent("root")
                .committed(),
        );

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");

        assert_eq!(analysis.branch_count, 1);
        assert!(analysis.convergence > 1.0); // 2 tips / 1 branch = 2.0
        assert!(analysis.semantic_coherence < 1.0);
    }

    #[test]
    fn test_discard_strategy_rollback() {
        let mut session = Session::new("rollback");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.finalize(SessionOutcome::Rollback);

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");
        let strategy = analyzer.select_strategy(&analysis);

        matches!(
            strategy,
            CompressionStrategy::Discard(DiscardReason::Rollback)
        );
    }

    #[test]
    fn test_discard_strategy_empty() {
        let session = Session::new("empty");

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");
        let strategy = analyzer.select_strategy(&analysis);

        matches!(
            strategy,
            CompressionStrategy::Discard(DiscardReason::EmptySession)
        );
    }

    #[test]
    fn test_discard_strategy_exploratory() {
        let mut session = Session::new("exploratory");
        session.add_vertex(make_vertex("v1", "sha256:a")); // Not committed

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");
        let strategy = analyzer.select_strategy(&analysis);

        matches!(
            strategy,
            CompressionStrategy::Discard(DiscardReason::ExploratoryOnly)
        );
    }

    #[test]
    fn test_single_strategy() {
        let mut session = Session::new("single");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");
        let strategy = analyzer.select_strategy(&analysis);

        matches!(strategy, CompressionStrategy::Single);
    }

    #[test]
    fn test_ephemeral_hint() {
        let mut session = Session::new("ephemeral");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.compression_hint = CompressionHint::Ephemeral;

        let analyzer = SessionAnalyzer::default();
        let analysis = analyzer.analyze(&session).expect("should analyze");
        let strategy = analyzer.select_strategy(&analysis);

        matches!(
            strategy,
            CompressionStrategy::Discard(DiscardReason::Ephemeral)
        );
    }

    #[test]
    fn test_identify_branches() {
        let mut session = Session::new("branches");
        session.add_vertex(make_vertex("root", "sha256:root").committed());
        session.add_vertex(
            make_vertex("b1", "sha256:b1")
                .with_parent("root")
                .committed(),
        );
        session.add_vertex(
            make_vertex("b2", "sha256:b2")
                .with_parent("root")
                .committed(),
        );

        let analyzer = SessionAnalyzer::default();
        let branches = analyzer.identify_branches(&session);

        assert_eq!(branches.len(), 2);
    }
}
