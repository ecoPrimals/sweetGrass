// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Application state for the service.

use std::sync::Arc;

use sweet_grass_compression::CompressionEngine;
use sweet_grass_core::SelfKnowledge;
use sweet_grass_core::agent::Did;
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_store::MemoryStore;

use std::path::PathBuf;

use crate::backend::BraidBackend;
#[cfg(unix)]
use crate::crypto_delegate::CryptoDelegate;
use crate::method_gate::MethodGate;
use sweet_grass_core::primal_names::env_vars;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Braid storage.
    pub store: Arc<BraidBackend>,

    /// Query engine.
    pub query: Arc<QueryEngine<BraidBackend>>,

    /// Braid factory.
    pub factory: Arc<BraidFactory>,

    /// Compression engine.
    pub compression: Arc<CompressionEngine>,

    /// Self-knowledge (for health checks, uptime, etc.).
    pub self_knowledge: Option<Arc<SelfKnowledge>>,

    /// Store backend type (for health reporting).
    pub store_backend: &'static str,

    /// Crypto delegation to `BearDog` Tower for braid signing.
    #[cfg(unix)]
    pub crypto: Option<Arc<CryptoDelegate>>,

    /// Pre-dispatch method gate (JH-0).
    pub method_gate: Arc<MethodGate>,

    /// Whether TCP transport is active (snapshotted at startup from
    /// `SWEETGRASS_PORT`). Used by `capability.list` to avoid env reads
    /// at handler call time.
    pub tcp_transport_active: bool,

    /// Whether BTSP handshake is required (snapshotted at startup from
    /// `FAMILY_ID`). Used by `capability.list` to avoid env reads at
    /// handler call time.
    pub btsp_required: bool,

    /// Resolved `biomeOS` socket directory (snapshotted at startup).
    /// Used by `composition.*_health` handlers to probe capability sockets
    /// without re-reading env vars on every request.
    pub socket_dir: PathBuf,

    /// Snapshotted `DISCOVERY_ADDRESS` for health integration reporting.
    pub discovery_address: Option<String>,
}

impl AppState {
    /// Create a new application state with in-memory store.
    #[must_use]
    pub fn new_memory(default_agent: Did) -> Self {
        let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
        let factory = Arc::new(BraidFactory::new(default_agent));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: None,
            store_backend: "memory",
            #[cfg(unix)]
            crypto: None,
            method_gate: Arc::new(MethodGate::from_env()),
            tcp_transport_active: false,
            btsp_required: false,
            socket_dir: Self::snapshot_socket_dir(),
            discovery_address: None,
        }
    }

    /// Create with custom store.
    #[must_use]
    pub fn with_store(store: Arc<BraidBackend>, default_agent: Did) -> Self {
        let factory = Arc::new(BraidFactory::new(default_agent));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: None,
            store_backend: "unknown",
            #[cfg(unix)]
            crypto: None,
            method_gate: Arc::new(MethodGate::from_env()),
            tcp_transport_active: std::env::var(env_vars::SWEETGRASS_PORT).is_ok(),
            btsp_required: Self::snapshot_btsp_required(),
            socket_dir: Self::snapshot_socket_dir(),
            discovery_address: std::env::var("DISCOVERY_ADDRESS").ok(),
        }
    }

    /// Create with self-knowledge and store backend type.
    ///
    /// Uses the primal's self-knowledge for source attribution (factory and
    /// compression engine) instead of hardcoded defaults.
    #[must_use]
    pub fn with_self_knowledge(
        store: Arc<BraidBackend>,
        default_agent: Did,
        self_knowledge: SelfKnowledge,
        store_backend: &'static str,
    ) -> Self {
        let factory = Arc::new(BraidFactory::from_self_knowledge(
            default_agent,
            &self_knowledge,
        ));
        let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
        let compression = Arc::new(
            CompressionEngine::new(Arc::clone(&factory)).with_source(self_knowledge.name.as_str()),
        );

        Self {
            store,
            query,
            factory,
            compression,
            self_knowledge: Some(Arc::new(self_knowledge)),
            store_backend,
            #[cfg(unix)]
            crypto: None,
            method_gate: Arc::new(MethodGate::from_env()),
            tcp_transport_active: std::env::var(env_vars::SWEETGRASS_PORT).is_ok(),
            btsp_required: Self::snapshot_btsp_required(),
            socket_dir: Self::snapshot_socket_dir(),
            discovery_address: std::env::var("DISCOVERY_ADDRESS").ok(),
        }
    }

    /// Snapshot the BTSP requirement from the current env.
    fn snapshot_btsp_required() -> bool {
        #[cfg(unix)]
        { crate::btsp::is_btsp_required() }
        #[cfg(not(unix))]
        { false }
    }

    /// Snapshot the `biomeOS` socket directory from the current env.
    ///
    /// Follows the ecosystem standard fallback chain:
    /// 1. `BIOMEOS_SOCKET_DIR`
    /// 2. `{XDG_RUNTIME_DIR}/biomeos`
    /// 3. `{TMPDIR}/biomeos`
    /// 4. `{temp_dir}/biomeos`
    fn snapshot_socket_dir() -> PathBuf {
        use sweet_grass_core::primal_names::{env_vars as ev, paths};
        if let Ok(dir) = std::env::var(ev::BIOMEOS_SOCKET_DIR) {
            return PathBuf::from(dir);
        }
        if let Ok(xdg) = std::env::var(ev::XDG_RUNTIME_DIR) {
            return PathBuf::from(xdg).join(paths::BIOMEOS_DIR);
        }
        if let Ok(tmpdir) = std::env::var(ev::TMPDIR) {
            return PathBuf::from(tmpdir).join(paths::BIOMEOS_DIR);
        }
        paths::default_socket_dir()
    }

    /// Attach a crypto delegate for Tower-delegated braid signing.
    #[cfg(unix)]
    #[must_use]
    pub fn with_crypto(mut self, crypto: CryptoDelegate) -> Self {
        self.crypto = Some(Arc::new(crypto));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new_memory() {
        let state = AppState::new_memory(Did::new("did:key:z6MkTestAgent"));

        assert!(Arc::strong_count(&state.store) >= 1);
        assert!(Arc::strong_count(&state.query) >= 1);
        assert!(Arc::strong_count(&state.factory) >= 1);
        assert!(Arc::strong_count(&state.compression) >= 1);
    }

    #[test]
    fn test_app_state_with_store() {
        let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
        let state = AppState::with_store(store, Did::new("did:key:z6MkTestAgent"));

        assert!(Arc::strong_count(&state.store) >= 1);
        assert!(Arc::strong_count(&state.query) >= 1);
    }

    #[test]
    fn test_app_state_clone() {
        let original = AppState::new_memory(Did::new("did:key:z6MkTestAgent"));
        let cloned = original.clone();

        assert!(Arc::ptr_eq(&original.store, &cloned.store));
        assert!(Arc::ptr_eq(&original.query, &cloned.query));
        assert!(Arc::ptr_eq(&original.factory, &cloned.factory));
        assert!(Arc::ptr_eq(&original.compression, &cloned.compression));
    }

    #[test]
    fn test_app_state_with_self_knowledge() {
        let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
        let sk = SelfKnowledge::default();
        let state = AppState::with_self_knowledge(store, Did::new("did:key:z6MkSK"), sk, "memory");
        assert!(state.self_knowledge.is_some());
        assert_eq!(state.store_backend, "memory");
    }

    #[test]
    fn test_app_state_default_store_backend() {
        let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
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
