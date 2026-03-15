// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Session data types.
//!
//! These types represent session events data that gets compressed
//! into Braids. They are protocol-agnostic representations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use sweet_grass_core::{agent::Did, braid::Timestamp, ActivityType, ContentHash};

/// A session vertex (node in the DAG).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionVertex {
    /// Vertex identifier.
    pub id: String,

    /// Content hash of the data at this vertex.
    pub data_hash: ContentHash,

    /// MIME type of the data.
    pub mime_type: String,

    /// Size in bytes.
    pub size: u64,

    /// Parent vertex IDs (derivation).
    pub parents: Vec<String>,

    /// When this vertex was created.
    pub timestamp: Timestamp,

    /// Who created this vertex.
    pub agent: Did,

    /// Activity type.
    pub activity_type: ActivityType,

    /// Whether this vertex was committed (vs exploratory).
    pub committed: bool,

    /// Custom metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SessionVertex {
    /// Create a new vertex.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        data_hash: impl Into<ContentHash>,
        mime_type: impl Into<String>,
        agent: Did,
    ) -> Self {
        Self {
            id: id.into(),
            data_hash: data_hash.into(),
            mime_type: mime_type.into(),
            size: 0,
            parents: Vec::new(),
            timestamp: sweet_grass_core::braid::current_timestamp_nanos(),
            agent,
            activity_type: ActivityType::Creation,
            committed: false,
            metadata: HashMap::new(),
        }
    }

    /// Set size.
    #[must_use]
    pub const fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Add parent.
    #[must_use]
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parents.push(parent.into());
        self
    }

    /// Set activity type.
    #[must_use]
    pub fn with_activity_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = activity_type;
        self
    }

    /// Mark as committed.
    #[must_use]
    pub const fn committed(mut self) -> Self {
        self.committed = true;
        self
    }

    /// Check if this is a root vertex (no parents).
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.parents.is_empty()
    }
}

/// Session outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionOutcome {
    /// Session completed successfully with commits.
    Committed,

    /// Session was rolled back.
    Rollback,

    /// Session is still in progress.
    #[default]
    InProgress,

    /// Session had no changes.
    NoOp,
}

/// Compression hint from the session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionHint {
    /// Force single Braid.
    Single,

    /// Allow any compression.
    #[default]
    Auto,

    /// Treat as atomic unit.
    Atomic,

    /// No Braid needed (ephemeral).
    Ephemeral,

    /// Important: prioritize preservation.
    Important,
}

/// A session from session events provider ready for compression.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Session {
    /// Session identifier.
    pub id: String,

    /// Vertices in the session DAG.
    pub vertices: Vec<SessionVertex>,

    /// When the session started.
    pub started_at: Timestamp,

    /// When the session ended.
    pub ended_at: Option<Timestamp>,

    /// Session outcome.
    pub outcome: SessionOutcome,

    /// Compression hint.
    pub compression_hint: CompressionHint,

    /// Total compute units consumed.
    pub compute_units: f64,

    /// Session metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            vertices: Vec::new(),
            started_at: sweet_grass_core::braid::current_timestamp_nanos(),
            ended_at: None,
            outcome: SessionOutcome::InProgress,
            compression_hint: CompressionHint::Auto,
            compute_units: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Add a vertex.
    pub fn add_vertex(&mut self, vertex: SessionVertex) {
        self.vertices.push(vertex);
    }

    /// Get vertex count.
    #[must_use]
    pub const fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get committed vertices.
    #[must_use]
    pub fn committed_vertices(&self) -> Vec<&SessionVertex> {
        self.vertices.iter().filter(|v| v.committed).collect()
    }

    /// Get unique output hashes (committed vertices with no children).
    #[must_use]
    pub fn unique_outputs(&self) -> Vec<&ContentHash> {
        let has_children: HashSet<_> = self.vertices.iter().flat_map(|v| &v.parents).collect();

        self.vertices
            .iter()
            .filter(|v| v.committed && !has_children.contains(&v.id))
            .map(|v| &v.data_hash)
            .collect()
    }

    /// Get all unique contributors.
    #[must_use]
    pub fn contributors(&self) -> HashSet<Did> {
        self.vertices.iter().map(|v| v.agent.clone()).collect()
    }

    /// Count branch points (vertices with multiple children).
    #[must_use]
    pub fn branch_count(&self) -> usize {
        let mut child_counts: HashMap<&str, usize> = HashMap::new();

        for vertex in &self.vertices {
            for parent in &vertex.parents {
                *child_counts.entry(parent.as_str()).or_insert(0) += 1;
            }
        }

        child_counts.values().filter(|&&c| c > 1).count()
    }

    /// Get root vertices (no parents).
    #[must_use]
    pub fn roots(&self) -> Vec<&SessionVertex> {
        self.vertices.iter().filter(|v| v.is_root()).collect()
    }

    /// Get tip vertices (no children).
    #[must_use]
    pub fn tips(&self) -> Vec<&SessionVertex> {
        let has_children: HashSet<_> = self.vertices.iter().flat_map(|v| &v.parents).collect();

        self.vertices
            .iter()
            .filter(|v| !has_children.contains(&v.id))
            .collect()
    }

    /// Calculate max depth of the DAG.
    #[must_use]
    pub fn max_depth(&self) -> usize {
        if self.vertices.is_empty() {
            return 0;
        }

        // Build parent lookup
        let vertex_map: HashMap<&str, &SessionVertex> =
            self.vertices.iter().map(|v| (v.id.as_str(), v)).collect();

        let mut cache = HashMap::new();
        self.tips()
            .iter()
            .map(|t| Self::depth_of(t, &vertex_map, &mut cache))
            .max()
            .unwrap_or(0)
    }

    fn depth_of(
        vertex: &SessionVertex,
        map: &HashMap<&str, &SessionVertex>,
        cache: &mut HashMap<String, usize>,
    ) -> usize {
        if let Some(&d) = cache.get(&vertex.id) {
            return d;
        }

        let d = if vertex.parents.is_empty() {
            0
        } else {
            vertex
                .parents
                .iter()
                .filter_map(|p| map.get(p.as_str()))
                .map(|p| Self::depth_of(p, map, cache))
                .max()
                .unwrap_or(0)
                + 1
        };

        cache.insert(vertex.id.clone(), d);
        d
    }

    /// Get temporal span in nanoseconds.
    #[must_use]
    pub fn temporal_span(&self) -> u64 {
        if self.vertices.is_empty() {
            return 0;
        }

        let min = self.vertices.iter().map(|v| v.timestamp).min().unwrap_or(0);
        let max = self.vertices.iter().map(|v| v.timestamp).max().unwrap_or(0);

        max.saturating_sub(min)
    }

    /// Check if session is atomic (marked or single vertex).
    #[must_use]
    pub fn is_atomic(&self) -> bool {
        self.compression_hint == CompressionHint::Atomic || self.vertex_count() == 1
    }

    /// Check if session has a single outcome.
    #[must_use]
    pub fn has_single_outcome(&self) -> bool {
        self.unique_outputs().len() <= 1
    }

    /// Finalize the session.
    pub fn finalize(&mut self, outcome: SessionOutcome) {
        self.outcome = outcome;
        self.ended_at = Some(sweet_grass_core::braid::current_timestamp_nanos());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vertex(id: &str, hash: &str) -> SessionVertex {
        SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkTest"))
    }

    #[test]
    fn test_empty_session() {
        let session = Session::new("test-session");
        assert_eq!(session.vertex_count(), 0);
        assert_eq!(session.branch_count(), 0);
        assert_eq!(session.max_depth(), 0);
    }

    #[test]
    fn test_linear_session() {
        let mut session = Session::new("linear");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());
        session.add_vertex(make_vertex("v3", "sha256:c").with_parent("v2").committed());

        assert_eq!(session.vertex_count(), 3);
        assert_eq!(session.branch_count(), 0);
        assert_eq!(session.max_depth(), 2);
        assert_eq!(session.roots().len(), 1);
        assert_eq!(session.tips().len(), 1);
    }

    #[test]
    fn test_branching_session() {
        let mut session = Session::new("branching");
        session.add_vertex(make_vertex("root", "sha256:root").committed());
        session.add_vertex(
            make_vertex("branch1", "sha256:b1")
                .with_parent("root")
                .committed(),
        );
        session.add_vertex(
            make_vertex("branch2", "sha256:b2")
                .with_parent("root")
                .committed(),
        );

        assert_eq!(session.vertex_count(), 3);
        assert_eq!(session.branch_count(), 1);
        assert_eq!(session.tips().len(), 2);
        assert_eq!(session.unique_outputs().len(), 2);
    }

    #[test]
    fn test_contributors() {
        let mut session = Session::new("collab");
        let agent1 = Did::new("did:key:z6MkAgent1");
        let agent2 = Did::new("did:key:z6MkAgent2");

        session.add_vertex(
            SessionVertex::new("v1", "sha256:a", "text/plain", agent1.clone()).committed(),
        );
        session.add_vertex(
            SessionVertex::new("v2", "sha256:b", "text/plain", agent2.clone())
                .with_parent("v1")
                .committed(),
        );

        let contributors = session.contributors();
        assert_eq!(contributors.len(), 2);
        assert!(contributors.contains(&agent1));
        assert!(contributors.contains(&agent2));
    }

    #[test]
    fn test_committed_filter() {
        let mut session = Session::new("mixed");
        session.add_vertex(make_vertex("v1", "sha256:a").committed());
        session.add_vertex(make_vertex("v2", "sha256:b")); // Not committed
        session.add_vertex(make_vertex("v3", "sha256:c").committed());

        assert_eq!(session.vertex_count(), 3);
        assert_eq!(session.committed_vertices().len(), 2);
    }

    #[test]
    fn test_session_finalize() {
        let mut session = Session::new("finalize");
        assert_eq!(session.outcome, SessionOutcome::InProgress);
        assert!(session.ended_at.is_none());

        session.finalize(SessionOutcome::Committed);

        assert_eq!(session.outcome, SessionOutcome::Committed);
        assert!(session.ended_at.is_some());
    }
}
