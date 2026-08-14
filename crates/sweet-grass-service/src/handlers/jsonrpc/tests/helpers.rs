// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Shared fixtures for JSON-RPC core dispatch tests.

use crate::state::AppState;
use sweet_grass_core::agent::Did;

/// In-memory application state for dispatch tests.
pub(super) fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}
