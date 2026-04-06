// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Index management for the in-memory store.
//!
//! This module handles maintaining secondary indexes for efficient queries.
//! Indexes are maintained on:
//! - Content hash → Braid ID
//! - Agent DID → Braid IDs
//! - Derivation source → Braid IDs
//! - Tags → Braid IDs
//! - MIME type → Braid IDs

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use parking_lot::RwLock;
use sweet_grass_core::{Braid, BraidId, ContentHash};

/// Collection of secondary indexes for efficient queries.
///
/// Uses `parking_lot::RwLock` for lock-free, panic-safe synchronization.
/// All operations are infallible since `parking_lot` does not poison locks.
///
/// ## Content Convergence
///
/// The hash index preserves 1:many mappings — when independent agents
/// produce braids with identical content hashes, all provenance paths
/// are retained rather than silently overwriting.
/// See `specs/CONTENT_CONVERGENCE.md`.
pub(super) struct Indexes {
    /// Index: content hash → Braid IDs (collision-preserving).
    ///
    /// Multiple braids may share the same content hash when independent
    /// provenance paths converge on identical content.
    pub hash: RwLock<HashMap<ContentHash, HashSet<BraidId>>>,

    /// Index: agent DID → Braid IDs.
    pub agent: RwLock<HashMap<String, HashSet<BraidId>>>,

    /// Index: derivation source hash → Braid IDs.
    pub derivation: RwLock<HashMap<ContentHash, HashSet<BraidId>>>,

    /// Index: tag → Braid IDs.
    ///
    /// Keyed on `Arc<str>` to share allocations with `BraidMetadata.tags` (O(1) clone).
    pub tag: RwLock<HashMap<Arc<str>, HashSet<BraidId>>>,

    /// Index: MIME type → Braid IDs.
    ///
    /// Keyed on `Arc<str>` to share allocations with `Braid.mime_type` (O(1) clone).
    pub mime: RwLock<HashMap<Arc<str>, HashSet<BraidId>>>,
}

impl Indexes {
    /// Create a new set of empty indexes.
    pub fn new() -> Self {
        Self {
            hash: RwLock::new(HashMap::new()),
            agent: RwLock::new(HashMap::new()),
            derivation: RwLock::new(HashMap::new()),
            tag: RwLock::new(HashMap::new()),
            mime: RwLock::new(HashMap::new()),
        }
    }

    /// Clear all indexes.
    pub fn clear(&self) {
        self.hash.write().clear();
        self.agent.write().clear();
        self.derivation.write().clear();
        self.tag.write().clear();
        self.mime.write().clear();
    }

    /// Add a Braid to all secondary indexes.
    pub fn add(&self, braid: &Braid) {
        let id = braid.id.clone();

        self.hash
            .write()
            .entry(braid.data_hash.clone())
            .or_default()
            .insert(id.clone());

        self.agent
            .write()
            .entry(braid.was_attributed_to.as_str().to_string())
            .or_default()
            .insert(id.clone());

        {
            let mut index = self.derivation.write();
            for derived in &braid.was_derived_from {
                if let Some(hash) = derived.content_hash() {
                    index.entry((*hash).clone()).or_default().insert(id.clone());
                }
            }
        }

        {
            let mut index = self.tag.write();
            for tag in &braid.metadata.tags {
                index.entry(Arc::clone(tag)).or_default().insert(id.clone());
            }
        }

        self.mime
            .write()
            .entry(Arc::clone(&braid.mime_type))
            .or_default()
            .insert(id);
    }

    /// Remove a Braid from all secondary indexes.
    pub fn remove(&self, braid: &Braid) {
        {
            let mut hash_idx = self.hash.write();
            if let Some(set) = hash_idx.get_mut(&braid.data_hash) {
                set.remove(&braid.id);
                if set.is_empty() {
                    hash_idx.remove(&braid.data_hash);
                }
            }
        }

        if let Some(set) = self.agent.write().get_mut(braid.was_attributed_to.as_str()) {
            set.remove(&braid.id);
        }

        {
            let mut index = self.derivation.write();
            for derived in &braid.was_derived_from {
                if let Some(hash) = derived.content_hash()
                    && let Some(set) = index.get_mut(hash)
                {
                    set.remove(&braid.id);
                }
            }
        }

        {
            let mut index = self.tag.write();
            for tag in &braid.metadata.tags {
                if let Some(set) = index.get_mut(tag) {
                    set.remove(&braid.id);
                }
            }
        }

        if let Some(set) = self.mime.write().get_mut(&braid.mime_type) {
            set.remove(&braid.id);
        }
    }

    /// Get a single Braid ID by content hash (backward-compatible).
    ///
    /// When multiple braids share a content hash (provenance convergence),
    /// returns one arbitrarily. Use [`get_all_by_hash`](Self::get_all_by_hash)
    /// to retrieve all convergent braids.
    pub fn get_by_hash(&self, hash: &str) -> Option<BraidId> {
        self.hash
            .read()
            .get(hash)
            .and_then(|set| set.iter().next().cloned())
    }

    /// Get all Braid IDs sharing a content hash (content convergence).
    ///
    /// Returns all braids that produced identical content via independent
    /// provenance paths. Empty set if no braids match.
    pub fn get_all_by_hash(&self, hash: &str) -> HashSet<BraidId> {
        self.hash.read().get(hash).cloned().unwrap_or_default()
    }

    /// Get Braid IDs by agent DID.
    pub fn get_by_agent(&self, agent: &str) -> HashSet<BraidId> {
        self.agent.read().get(agent).cloned().unwrap_or_default()
    }

    /// Get Braid IDs by derivation source hash.
    pub fn get_by_derivation(&self, hash: &str) -> HashSet<BraidId> {
        self.derivation
            .read()
            .get(hash)
            .cloned()
            .unwrap_or_default()
    }

    /// Get Braid IDs by tag.
    pub fn get_by_tag(&self, tag: &str) -> HashSet<BraidId> {
        self.tag.read().get(tag).cloned().unwrap_or_default()
    }

    /// Get Braid IDs by MIME type.
    pub fn get_by_mime(&self, mime: &str) -> HashSet<BraidId> {
        self.mime.read().get(mime).cloned().unwrap_or_default()
    }
}

impl Default for Indexes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;

    fn make_test_braid(hash: &str, agent: &str) -> Braid {
        Braid::builder()
            .data_hash(hash)
            .mime_type("application/json")
            .size(1024)
            .attributed_to(Did::new(agent))
            .build()
            .expect("should build")
    }

    #[test]
    fn test_add_and_get_by_hash() {
        let indexes = Indexes::new();
        let braid = make_test_braid("sha256:test1", "did:key:z6MkTest");

        indexes.add(&braid);

        let id = indexes.get_by_hash("sha256:test1");
        assert!(id.is_some());
        assert_eq!(id.unwrap(), braid.id);
    }

    #[test]
    fn test_add_and_get_by_agent() {
        let indexes = Indexes::new();
        let braid1 = make_test_braid("sha256:a1", "did:key:z6MkAgent");
        let braid2 = make_test_braid("sha256:a2", "did:key:z6MkAgent");

        indexes.add(&braid1);
        indexes.add(&braid2);

        let ids = indexes.get_by_agent("did:key:z6MkAgent");
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_remove() {
        let indexes = Indexes::new();
        let braid = make_test_braid("sha256:remove", "did:key:z6MkTest");

        indexes.add(&braid);
        assert!(indexes.get_by_hash("sha256:remove").is_some());

        indexes.remove(&braid);
        assert!(indexes.get_by_hash("sha256:remove").is_none());
    }

    #[test]
    fn test_clear() {
        let indexes = Indexes::new();
        let braid = make_test_braid("sha256:clear", "did:key:z6MkTest");

        indexes.add(&braid);
        indexes.clear();

        assert!(indexes.get_by_hash("sha256:clear").is_none());
    }

    #[test]
    fn test_get_by_mime() {
        let indexes = Indexes::new();
        let braid = make_test_braid("sha256:mime", "did:key:z6MkTest");

        indexes.add(&braid);

        let ids = indexes.get_by_mime("application/json");
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&braid.id));
    }

    #[test]
    fn test_get_by_tag() {
        let indexes = Indexes::new();
        let mut braid = make_test_braid("sha256:tagged", "did:key:z6MkTest");
        braid.metadata.tags.push("my-tag".into());

        indexes.add(&braid);

        let ids = indexes.get_by_tag("my-tag");
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&braid.id));

        let empty = indexes.get_by_tag("nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_get_by_derivation() {
        use sweet_grass_core::entity::EntityReference;

        let indexes = Indexes::new();
        let mut braid = make_test_braid("sha256:derived", "did:key:z6MkTest");
        braid
            .was_derived_from
            .push(EntityReference::by_hash("sha256:source"));

        indexes.add(&braid);

        let ids = indexes.get_by_derivation("sha256:source");
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&braid.id));

        let empty = indexes.get_by_derivation("sha256:nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_remove_with_tags_and_derivations() {
        use sweet_grass_core::entity::EntityReference;

        let indexes = Indexes::new();
        let mut braid = make_test_braid("sha256:complex", "did:key:z6MkTest");
        braid.metadata.tags.push("tag1".into());
        braid.metadata.tags.push("tag2".into());
        braid
            .was_derived_from
            .push(EntityReference::by_hash("sha256:parent"));

        indexes.add(&braid);

        assert!(!indexes.get_by_tag("tag1").is_empty());
        assert!(!indexes.get_by_tag("tag2").is_empty());
        assert!(!indexes.get_by_derivation("sha256:parent").is_empty());
        assert!(!indexes.get_by_mime("application/json").is_empty());

        indexes.remove(&braid);

        assert!(indexes.get_by_tag("tag1").is_empty());
        assert!(indexes.get_by_tag("tag2").is_empty());
        assert!(indexes.get_by_derivation("sha256:parent").is_empty());
        assert!(indexes.get_by_mime("application/json").is_empty());
    }

    #[test]
    fn test_content_convergence_multiple_braids_same_hash() {
        let indexes = Indexes::new();
        let mut braid1 = make_test_braid("sha256:converged", "did:key:z6MkAlice");
        let mut braid2 = make_test_braid("sha256:converged", "did:key:z6MkBob");
        braid1.id = BraidId::new();
        braid2.id = BraidId::new();

        indexes.add(&braid1);
        indexes.add(&braid2);

        let all = indexes.get_all_by_hash("sha256:converged");
        assert_eq!(all.len(), 2);
        assert!(all.contains(&braid1.id));
        assert!(all.contains(&braid2.id));

        let single = indexes.get_by_hash("sha256:converged");
        assert!(single.is_some());
    }

    #[test]
    fn test_content_convergence_remove_preserves_others() {
        let indexes = Indexes::new();
        let mut braid1 = make_test_braid("sha256:converged", "did:key:z6MkAlice");
        let mut braid2 = make_test_braid("sha256:converged", "did:key:z6MkBob");
        braid1.id = BraidId::new();
        braid2.id = BraidId::new();

        indexes.add(&braid1);
        indexes.add(&braid2);
        assert_eq!(indexes.get_all_by_hash("sha256:converged").len(), 2);

        indexes.remove(&braid1);

        let remaining = indexes.get_all_by_hash("sha256:converged");
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains(&braid2.id));

        indexes.remove(&braid2);
        assert!(indexes.get_all_by_hash("sha256:converged").is_empty());
        assert!(indexes.get_by_hash("sha256:converged").is_none());
    }

    #[test]
    fn test_get_all_by_hash_empty() {
        let indexes = Indexes::new();
        assert!(indexes.get_all_by_hash("sha256:nonexistent").is_empty());
    }

    #[test]
    fn test_default_trait() {
        let indexes = Indexes::default();
        assert!(indexes.get_by_hash("any").is_none());
    }

    #[test]
    fn test_multiple_agents() {
        let indexes = Indexes::new();
        let braid1 = make_test_braid("sha256:m1", "did:key:z6MkAlice");
        let braid2 = make_test_braid("sha256:m2", "did:key:z6MkBob");
        let braid3 = make_test_braid("sha256:m3", "did:key:z6MkAlice");

        indexes.add(&braid1);
        indexes.add(&braid2);
        indexes.add(&braid3);

        let alice_ids = indexes.get_by_agent("did:key:z6MkAlice");
        assert_eq!(alice_ids.len(), 2);

        let bob_ids = indexes.get_by_agent("did:key:z6MkBob");
        assert_eq!(bob_ids.len(), 1);

        let empty = indexes.get_by_agent("did:key:z6MkNobody");
        assert!(empty.is_empty());
    }
}
