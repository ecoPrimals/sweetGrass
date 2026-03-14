// SPDX-License-Identifier: AGPL-3.0-only
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

use parking_lot::RwLock;
use sweet_grass_core::{Braid, ContentHash};

/// Collection of secondary indexes for efficient queries.
///
/// Uses `parking_lot::RwLock` for lock-free, panic-safe synchronization.
/// All operations are infallible since `parking_lot` does not poison locks.
pub(super) struct Indexes {
    /// Index: content hash → Braid ID.
    pub hash: RwLock<HashMap<ContentHash, String>>,

    /// Index: agent DID → Braid IDs.
    pub agent: RwLock<HashMap<String, HashSet<String>>>,

    /// Index: derivation source hash → Braid IDs.
    pub derivation: RwLock<HashMap<ContentHash, HashSet<String>>>,

    /// Index: tag → Braid IDs.
    pub tag: RwLock<HashMap<String, HashSet<String>>>,

    /// Index: MIME type → Braid IDs.
    pub mime: RwLock<HashMap<String, HashSet<String>>>,
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
        let id = braid.id.as_str().to_string();

        self.hash
            .write()
            .insert(braid.data_hash.clone(), id.clone());

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
                index.entry(tag.clone()).or_default().insert(id.clone());
            }
        }

        self.mime
            .write()
            .entry(braid.mime_type.clone())
            .or_default()
            .insert(id);
    }

    /// Remove a Braid from all secondary indexes.
    pub fn remove(&self, braid: &Braid) {
        let id = braid.id.as_str().to_string();

        self.hash.write().remove(&braid.data_hash);

        if let Some(set) = self.agent.write().get_mut(braid.was_attributed_to.as_str()) {
            set.remove(&id);
        }

        {
            let mut index = self.derivation.write();
            for derived in &braid.was_derived_from {
                if let Some(hash) = derived.content_hash() {
                    if let Some(set) = index.get_mut(hash) {
                        set.remove(&id);
                    }
                }
            }
        }

        {
            let mut index = self.tag.write();
            for tag in &braid.metadata.tags {
                if let Some(set) = index.get_mut(tag) {
                    set.remove(&id);
                }
            }
        }

        if let Some(set) = self.mime.write().get_mut(&braid.mime_type) {
            set.remove(&id);
        }
    }

    /// Get Braid ID by content hash.
    pub fn get_by_hash(&self, hash: &str) -> Option<String> {
        self.hash.read().get(hash).cloned()
    }

    /// Get Braid IDs by agent DID.
    pub fn get_by_agent(&self, agent: &str) -> HashSet<String> {
        self.agent.read().get(agent).cloned().unwrap_or_default()
    }

    /// Get Braid IDs by derivation source hash.
    pub fn get_by_derivation(&self, hash: &str) -> HashSet<String> {
        self.derivation
            .read()
            .get(hash)
            .cloned()
            .unwrap_or_default()
    }

    /// Get Braid IDs by tag.
    pub fn get_by_tag(&self, tag: &str) -> HashSet<String> {
        self.tag.read().get(tag).cloned().unwrap_or_default()
    }

    /// Get Braid IDs by MIME type.
    pub fn get_by_mime(&self, mime: &str) -> HashSet<String> {
        self.mime.read().get(mime).cloned().unwrap_or_default()
    }
}

impl Default for Indexes {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
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
        assert_eq!(id.unwrap(), braid.id.as_str());
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
        assert!(ids.contains(braid.id.as_str()));
    }

    #[test]
    fn test_get_by_tag() {
        let indexes = Indexes::new();
        let mut braid = make_test_braid("sha256:tagged", "did:key:z6MkTest");
        braid.metadata.tags.push("my-tag".to_string());

        indexes.add(&braid);

        let ids = indexes.get_by_tag("my-tag");
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(braid.id.as_str()));

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
        assert!(ids.contains(braid.id.as_str()));

        let empty = indexes.get_by_derivation("sha256:nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_remove_with_tags_and_derivations() {
        use sweet_grass_core::entity::EntityReference;

        let indexes = Indexes::new();
        let mut braid = make_test_braid("sha256:complex", "did:key:z6MkTest");
        braid.metadata.tags.push("tag1".to_string());
        braid.metadata.tags.push("tag2".to_string());
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
