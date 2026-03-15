// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Compression strategy types.
//!
//! This module defines the different strategies the compression engine
//! can use to convert sessions into Braids.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Compression strategy to apply.
#[derive(Clone, Debug)]
pub enum CompressionStrategy {
    /// Produce no Braids.
    Discard(DiscardReason),

    /// Produce single Braid.
    Single,

    /// Split into multiple Braids by branch.
    Split(Vec<BranchSpec>),

    /// Hierarchical compression with meta-levels.
    Hierarchical(Vec<CompressionLevel>),
}

/// Reasons for discarding a session (0 Braids).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscardReason {
    /// Session explicitly rolled back.
    Rollback,

    /// Session had no vertices.
    EmptySession,

    /// All branches were exploratory (no commits).
    ExploratoryOnly,

    /// Content below significance threshold.
    BelowThreshold,

    /// Duplicate of existing provenance.
    Duplicate,

    /// Session marked as ephemeral.
    Ephemeral,
}

impl std::fmt::Display for DiscardReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rollback => write!(f, "session was rolled back"),
            Self::EmptySession => write!(f, "session has no vertices"),
            Self::ExploratoryOnly => write!(f, "all branches were exploratory"),
            Self::BelowThreshold => write!(f, "content below threshold"),
            Self::Duplicate => write!(f, "duplicate provenance"),
            Self::Ephemeral => write!(f, "session marked as ephemeral"),
        }
    }
}

/// Specification for a branch to compress.
#[derive(Clone, Debug)]
pub struct BranchSpec {
    /// Branch identifier.
    pub id: String,

    /// Root vertex ID.
    pub root: String,

    /// Tip vertex IDs.
    pub tips: Vec<String>,

    /// Vertex IDs in this branch.
    pub vertices: Vec<String>,
}

impl BranchSpec {
    /// Create a new branch spec.
    #[must_use]
    pub fn new(id: impl Into<String>, root: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            root: root.into(),
            tips: Vec::new(),
            vertices: Vec::new(),
        }
    }

    /// Add a tip.
    #[must_use]
    pub fn with_tip(mut self, tip: impl Into<String>) -> Self {
        self.tips.push(tip.into());
        self
    }

    /// Add vertices.
    #[must_use]
    pub fn with_vertices(mut self, vertices: Vec<String>) -> Self {
        self.vertices = vertices;
        self
    }
}

/// Compression level for hierarchical compression.
#[derive(Clone, Debug)]
pub struct CompressionLevel {
    /// Level number (0 = leaf, higher = more summary).
    pub level: u32,

    /// How to segment/group at this level.
    pub grouping: GroupingStrategy,

    /// Maximum items per group.
    pub max_group_size: usize,
}

impl CompressionLevel {
    /// Create a new compression level.
    #[must_use]
    pub fn new(level: u32, grouping: GroupingStrategy) -> Self {
        Self {
            level,
            grouping,
            max_group_size: 10,
        }
    }

    /// Set max group size.
    #[must_use]
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_group_size = size;
        self
    }
}

/// How to group Braids at each level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GroupingStrategy {
    /// Group by time window.
    Temporal {
        /// Window size.
        window_secs: u64,
    },

    /// Group by activity type.
    ActivityType,

    /// Group by contributor.
    Contributor,

    /// Group by branch in DAG.
    Branch,

    /// Fixed size groups.
    FixedSize {
        /// Size of each group.
        size: usize,
    },
}

impl GroupingStrategy {
    /// Create temporal grouping.
    #[must_use]
    pub fn temporal(window: Duration) -> Self {
        Self::Temporal {
            window_secs: window.as_secs(),
        }
    }

    /// Create fixed size grouping.
    #[must_use]
    pub fn fixed_size(size: usize) -> Self {
        Self::FixedSize { size }
    }
}

/// Compression configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum vertices for compression (below = single or none).
    pub min_vertices: usize,

    /// Threshold for splitting into multiple Braids.
    pub split_threshold: usize,

    /// Threshold for hierarchical compression.
    pub hierarchical_threshold: usize,

    /// Coherence threshold for single Braid (0.0 - 1.0).
    pub coherence_threshold: f64,

    /// Maximum Braids per session.
    pub max_braids_per_session: usize,

    /// Enable meta-Braid generation.
    pub generate_summaries: bool,

    /// Maximum summary depth.
    pub max_summary_depth: u32,

    /// Whether to honor compression hints.
    pub honor_hints: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_vertices: 1,
            split_threshold: 100,
            hierarchical_threshold: 1000,
            coherence_threshold: 0.7,
            max_braids_per_session: 100,
            generate_summaries: true,
            max_summary_depth: 3,
            honor_hints: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discard_reason_display() {
        assert_eq!(
            DiscardReason::Rollback.to_string(),
            "session was rolled back"
        );
        assert_eq!(
            DiscardReason::EmptySession.to_string(),
            "session has no vertices"
        );
    }

    #[test]
    fn test_discard_reason_all_variants() {
        assert!(DiscardReason::ExploratoryOnly
            .to_string()
            .contains("exploratory"));
        assert!(DiscardReason::BelowThreshold
            .to_string()
            .contains("threshold"));
        assert!(DiscardReason::Duplicate.to_string().contains("duplicate"));
        assert!(DiscardReason::Ephemeral.to_string().contains("ephemeral"));
    }

    #[test]
    fn test_branch_spec() {
        let spec = BranchSpec::new("branch-1", "root-v")
            .with_tip("tip-1")
            .with_tip("tip-2")
            .with_vertices(vec!["v1".to_string(), "v2".to_string()]);

        assert_eq!(spec.id, "branch-1");
        assert_eq!(spec.root, "root-v");
        assert_eq!(spec.tips.len(), 2);
        assert_eq!(spec.vertices.len(), 2);
    }

    #[test]
    fn test_branch_spec_empty_vertices() {
        let spec = BranchSpec::new("empty-branch", "root").with_vertices(vec![]);
        assert!(spec.vertices.is_empty());
        assert_eq!(spec.tips.len(), 0);
    }

    #[test]
    fn test_compression_level() {
        let level = CompressionLevel::new(1, GroupingStrategy::Branch).with_max_size(20);

        assert_eq!(level.level, 1);
        assert_eq!(level.max_group_size, 20);
    }

    #[test]
    fn test_compression_level_default_max_size() {
        let level = CompressionLevel::new(0, GroupingStrategy::ActivityType);
        assert_eq!(level.max_group_size, 10);
    }

    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();

        assert_eq!(config.min_vertices, 1);
        assert_eq!(config.split_threshold, 100);
        assert!(config.generate_summaries);
    }

    #[test]
    fn test_config_all_fields() {
        let config = CompressionConfig::default();
        assert_eq!(config.hierarchical_threshold, 1000);
        assert!((config.coherence_threshold - 0.7).abs() < 1e-9);
        assert_eq!(config.max_braids_per_session, 100);
        assert_eq!(config.max_summary_depth, 3);
        assert!(config.honor_hints);
    }

    #[test]
    fn test_grouping_strategy_temporal() {
        let g = GroupingStrategy::temporal(std::time::Duration::from_secs(3600));
        match &g {
            GroupingStrategy::Temporal { window_secs } => assert_eq!(*window_secs, 3600),
            _ => panic!("expected Temporal"),
        }
    }

    #[test]
    fn test_grouping_strategy_fixed_size() {
        let g = GroupingStrategy::fixed_size(25);
        match &g {
            GroupingStrategy::FixedSize { size } => assert_eq!(*size, 25),
            _ => panic!("expected FixedSize"),
        }
    }

    #[test]
    fn test_grouping_strategy_variants() {
        let _ = GroupingStrategy::ActivityType;
        let _ = GroupingStrategy::Contributor;
        let _ = GroupingStrategy::Branch;
    }

    #[test]
    fn test_compression_strategy_variants() {
        let _ = CompressionStrategy::Discard(DiscardReason::Rollback);
        let _ = CompressionStrategy::Single;
        let _ = CompressionStrategy::Split(vec![BranchSpec::new("b1", "r1")]);
        let _ = CompressionStrategy::Hierarchical(vec![CompressionLevel::new(
            0,
            GroupingStrategy::Branch,
        )]);
    }

    #[test]
    fn test_discard_reason_equality() {
        assert_eq!(DiscardReason::Rollback, DiscardReason::Rollback);
        assert_ne!(DiscardReason::Rollback, DiscardReason::EmptySession);
    }
}
