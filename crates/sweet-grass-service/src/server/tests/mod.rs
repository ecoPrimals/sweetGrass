// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::clone_on_ref_ptr,
    reason = "test file: expect/unwrap are standard in tests"
)]

mod attribution;
mod braid_crud;
mod compression;
mod provenance;
mod tarpc_roundtrip;

use super::*;
use crate::backend::BraidBackend;
use crate::rpc::{CreateBraidRequest, SweetGrassRpc};
use crate::state::AppState;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use sweet_grass_compression::{CompressionEngine, Session, SessionOutcome, SessionVertex};
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::{BraidId, SummaryType};
use sweet_grass_core::entity::EntityReference;
use sweet_grass_factory::{AttributionCalculator, AttributionConfig, BraidFactory};
use sweet_grass_query::QueryEngine;
use sweet_grass_store::{MemoryStore, QueryFilter, QueryOrder};
use tarpc::context;

/// Test bind address (OS-allocated port).
const TEST_BIND_ADDR: &str = "127.0.0.1:0";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn make_server() -> SweetGrassServer {
    let store = Arc::new(BraidBackend::Memory(MemoryStore::new()));
    let did = Did::new("did:key:z6MkTest");
    let factory = Arc::new(BraidFactory::new(did));
    let query = Arc::new(QueryEngine::new(Arc::clone(&store)));
    let compression = Arc::new(CompressionEngine::new(Arc::clone(&factory)));
    let attribution = Arc::new(AttributionCalculator::new());

    SweetGrassServer::new(store, factory, query, compression, attribution)
        .with_store_backend("memory")
}

async fn create_test_braid(server: &SweetGrassServer) -> Braid {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let request = CreateBraidRequest {
        data_hash: format!("sha256:test{id}").into(),
        mime_type: "text/plain".to_string(),
        size: 1024,
        attributed_to: Did::new("did:key:z6MkTest"),
        activity: None,
        derived_from: vec![],
        metadata: None,
    };
    server
        .clone()
        .create_braid(context::current(), request)
        .await
        .unwrap()
}
