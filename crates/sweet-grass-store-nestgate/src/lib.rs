// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `NestGate` `JSON-RPC` storage backend for `SweetGrass`.
//!
//! This crate implements [`sweet_grass_store::BraidStore`] by delegating persistence to `NestGate`
//! via `storage.*` `JSON-RPC` methods over Unix Domain Sockets. This aligns with
//! the ecosystem pattern where `NestGate` owns durable persistence and primals
//! discover it at runtime via capability-based socket resolution.
//!
//! # Architecture
//!
//! ```text
//! `sweetGrass` ──`JSON-RPC`──▶ `NestGate` (storage.store/retrieve/list/delete)
//!                             │
//!                        filesystem / ZFS
//! ```
//!
//! Braids and activities are serialized as JSON values keyed by type-prefixed
//! identifiers (`braid:{id}`, `activity:{id}`). Indices for agent lookups
//! and derivation queries are maintained as separate KV entries.
//!
//! # Socket Discovery
//!
//! `NestGate` socket resolution follows the ecosystem standard:
//! 1. `NESTGATE_SOCKET` environment variable
//! 2. `STORAGE_PROVIDER_SOCKET` environment variable
//! 3. `{BIOMEOS_SOCKET_DIR}/nestgate.sock`
//! 4. `{XDG_RUNTIME_DIR}/biomeos/nestgate.sock`
//! 5. `/tmp/biomeos/nestgate.sock`

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod discovery;
mod error;
mod store;

pub use error::NestGateStoreError;
pub use store::NestGateStore;

/// Configuration for the `NestGate` store backend.
#[derive(Clone, Debug)]
pub struct NestGateConfig {
    /// Explicit socket path override. If `None`, uses discovery.
    pub socket_path: Option<String>,

    /// Family ID for multi-instance scoping.
    pub family_id: Option<String>,

    /// Key prefix for all `sweetGrass` data in `NestGate`.
    pub key_prefix: String,
}

impl Default for NestGateConfig {
    fn default() -> Self {
        Self {
            socket_path: None,
            family_id: None,
            key_prefix: sweet_grass_core::identity::PRIMAL_NAME.to_string(),
        }
    }
}
