// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for redb store.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

use std::sync::Arc;

use super::*;
use sweet_grass_core::braid::BraidBuilder;
use tempfile::TempDir;

fn create_test_store() -> (RedbStore, TempDir) {
    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("sweetgrass.redb");
    let store = RedbStore::open_path(&db_path).expect("open store");
    (store, temp)
}

fn create_test_braid(hash: &str) -> Braid {
    BraidBuilder::default()
        .data_hash(hash)
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("build braid")
}

mod activities;
mod batch;
mod config;
mod crud;
mod edge;
mod query;
mod query_ecop;
