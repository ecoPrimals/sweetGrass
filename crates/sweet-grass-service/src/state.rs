// SPDX-License-Identifier: AGPL-3.0-only
//! Application state for the service.

use std::sync::Arc;

use sweet_grass_compression::CompressionEngine;
use sweet_grass_core::agent::Did;
use sweet_grass_core::SelfKnowledge;
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_store::{BraidStore, MemoryStore};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Braid storage.
    pub store: Arc<dyn BraidStore>,

    /// Query engine.
    pub query: Arc<QueryEngine>,

    /// Braid factory.
    pub factory: Arc<BraidFactory>,

    /// Compression engine.
    pub compression: Arc<CompressionEngine>,

    /// Self-knowledge (for health checks, uptime, etc.).
    pub self_knowledge: Option<Arc<SelfKnowledge>>,

    /// Store backend type (for health reporting).
    pub store_backend: String,
}

impl AppState {
    /// Create a new application state with in-memory store.
    #[must_use]
    pub fn new_memory(default_agent: Did) -> Self {
        let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
        let factory = Arc::new(BraidFactory::new(default_agent));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: None,
            store_backend: "memory".to_string(),
        }
    }

    /// Create with custom store.
    #[must_use]
    pub fn with_store(store: Arc<dyn BraidStore>, default_agent: Did) -> Self {
        let factory = Arc::new(BraidFactory::new(default_agent));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: None,
            store_backend: "unknown".to_string(),
        }
    }

    /// Create with self-knowledge and store backend type.
    #[must_use]
    pub fn with_self_knowledge(
        store: Arc<dyn BraidStore>,
        default_agent: Did,
        self_knowledge: SelfKnowledge,
        store_backend: impl Into<String>,
    ) -> Self {
        let factory = Arc::new(BraidFactory::new(default_agent));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: Some(Arc::new(self_knowledge)),
            store_backend: store_backend.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new_memory() {
        let state = AppState::new_memory(Did::new("did:key:z6MkTestAgent"));

        // Verify all components are initialized
        assert!(Arc::strong_count(&state.store) >= 1);
        assert!(Arc::strong_count(&state.query) >= 1);
        assert!(Arc::strong_count(&state.factory) >= 1);
        assert!(Arc::strong_count(&state.compression) >= 1);
    }

    #[test]
    fn test_app_state_with_store() {
        let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
        let state = AppState::with_store(store, Did::new("did:key:z6MkTestAgent"));

        // Verify all components are initialized
        assert!(Arc::strong_count(&state.store) >= 1);
        assert!(Arc::strong_count(&state.query) >= 1);
    }

    #[test]
    fn test_app_state_clone() {
        let original = AppState::new_memory(Did::new("did:key:z6MkTestAgent"));

        // Explicitly test Clone trait - clippy's redundant_clone is intentional here
        #[allow(clippy::redundant_clone)]
        let cloned = original.clone();

        // Cloned state should share the same Arc references
        assert!(Arc::ptr_eq(&original.store, &cloned.store));
        assert!(Arc::ptr_eq(&original.query, &cloned.query));
        assert!(Arc::ptr_eq(&original.factory, &cloned.factory));
        assert!(Arc::ptr_eq(&original.compression, &cloned.compression));
    }

    #[test]
    fn test_app_state_with_self_knowledge() {
        let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
        let sk = SelfKnowledge::default();
        let state = AppState::with_self_knowledge(store, Did::new("did:key:z6MkSK"), sk, "memory");
        assert!(state.self_knowledge.is_some());
        assert_eq!(state.store_backend, "memory");
    }

    #[test]
    fn test_app_state_default_store_backend() {
        let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
        let state = AppState::with_store(store, Did::new("did:key:z6MkTest"));
        assert_eq!(state.store_backend, "unknown");
        assert!(state.self_knowledge.is_none());
    }

    #[test]
    fn test_app_state_new_memory_backend() {
        let state = AppState::new_memory(Did::new("did:key:z6MkMemTest"));
        assert_eq!(state.store_backend, "memory");
    }
}
