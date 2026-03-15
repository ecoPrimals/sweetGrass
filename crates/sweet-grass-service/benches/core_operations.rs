// SPDX-License-Identifier: AGPL-3.0-only
//! Core operations benchmarks for SweetGrass.
//!
//! Benchmarks braid creation, store operations, hashing, attribution,
//! compression, and provenance graph traversal.

#![expect(
    clippy::expect_used,
    reason = "benchmark harness requires direct assertions"
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use sweet_grass_compression::{
    session::{Session, SessionOutcome, SessionVertex},
    CompressionEngine,
};
use sweet_grass_core::{agent::Did, entity::EntityReference, hash::sha256};
use sweet_grass_factory::{AttributionCalculator, BraidFactory};
use sweet_grass_query::ProvenanceGraphBuilder;
use sweet_grass_store::{BraidStore, MemoryStore, QueryFilter, QueryOrder};
use tokio::runtime::Runtime;

fn rt() -> Runtime {
    Runtime::new().expect("create runtime")
}

fn make_factory() -> BraidFactory {
    BraidFactory::new(Did::new("did:key:z6MkBenchmark"))
}

fn make_vertex(id: &str, hash: &str) -> SessionVertex {
    SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkBench")).with_size(1024)
}

/// Braid creation via BraidFactory::from_data at various data sizes.
fn bench_braid_creation(c: &mut Criterion) {
    let factory = make_factory();
    let mut group = c.benchmark_group("braid_creation");
    for size in [1024_usize, 10 * 1024, 100 * 1024] {
        #[allow(clippy::cast_possible_truncation)]
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.bench_with_input(
            BenchmarkId::new("from_data", format!("{}KB", size / 1024)),
            &data,
            |b, data| {
                b.iter(|| {
                    black_box(
                        factory
                            .from_data(data, "application/octet-stream", None)
                            .expect("from_data"),
                    );
                });
            },
        );
    }
    group.finish();
}

/// Store put and get operations on MemoryStore.
fn bench_store_put_get(c: &mut Criterion) {
    let rt = rt();
    let factory = make_factory();
    let data = vec![0u8; 1024];
    let braid = factory
        .from_data(&data, "application/octet-stream", None)
        .expect("create braid");

    let mut group = c.benchmark_group("store_operations");
    group.bench_function("put", |b| {
        b.iter(|| {
            let store = MemoryStore::new();
            rt.block_on(async {
                let _: () = store.put(&braid).await.expect("put");
                black_box(());
            });
        });
    });
    group.bench_function("get", |b| {
        let store = MemoryStore::new();
        rt.block_on(store.put(&braid)).expect("setup");
        let id = braid.id.clone();
        b.iter(|| {
            rt.block_on(async {
                black_box(store.get(&id).await.expect("get"));
            });
        });
    });
    group.finish();
}

/// Content hashing: sha256 and Braid::compute_signing_hash.
fn bench_content_hashing(c: &mut Criterion) {
    let factory = make_factory();
    let data = vec![0u8; 1024];
    let braid = factory
        .from_data(&data, "application/octet-stream", None)
        .expect("create braid");

    let mut group = c.benchmark_group("content_hashing");
    group.bench_with_input(BenchmarkId::new("sha256", "1KB"), &data, |b, data| {
        b.iter(|| black_box(sha256(data)));
    });
    group.bench_function("compute_signing_hash", |b| {
        b.iter(|| black_box(braid.compute_signing_hash()));
    });
    group.finish();
}

/// Store query on a pre-populated store with 100 braids.
fn bench_store_query(c: &mut Criterion) {
    let rt = rt();
    let factory = make_factory();
    let store = MemoryStore::new();
    let data = vec![0u8; 256];

    // Pre-populate with 100 braids
    for i in 0..100_usize {
        let mut d = data.clone();
        d.extend_from_slice(&i.to_le_bytes());
        let braid = factory
            .from_data(&d, "application/octet-stream", None)
            .expect("create braid");
        rt.block_on(store.put(&braid)).expect("put");
    }

    let filter = QueryFilter::new().with_limit(50);
    c.bench_function("store_query_100_braids", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    store
                        .query(&filter, QueryOrder::NewestFirst)
                        .await
                        .expect("query"),
                );
            });
        });
    });
}

/// Attribution chain computation.
fn bench_attribution(c: &mut Criterion) {
    let factory = make_factory();
    let braid = factory
        .from_data(&[0u8; 256], "application/octet-stream", None)
        .expect("create braid");
    let calculator = AttributionCalculator::new();

    c.bench_function("attribution_calculate_single", |b| {
        b.iter(|| black_box(calculator.calculate_single(&braid)));
    });
}

/// Session compression with 0/1/Many model.
fn bench_compression(c: &mut Criterion) {
    let factory = Arc::new(make_factory());
    let engine = CompressionEngine::new(factory);

    let mut session = Session::new("bench-session");
    session.add_vertex(make_vertex("v1", "sha256:root").committed());
    session.add_vertex(
        make_vertex("v2", "sha256:derived")
            .with_parent("v1")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    c.bench_function("compression_single_braid", |b| {
        b.iter(|| black_box(engine.compress(&session).expect("compress")));
    });
}

/// Provenance graph traversal.
fn bench_query_traversal(c: &mut Criterion) {
    let rt = rt();
    let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
    let factory = make_factory();

    // Build a chain: root -> child -> grandchild
    let root_data = vec![0u8; 64];
    let root = factory
        .from_data(&root_data, "application/octet-stream", None)
        .expect("root");
    rt.block_on(store.put(&root)).expect("put root");

    let child_data = vec![1u8; 64];
    let mut child = factory
        .from_data(&child_data, "application/octet-stream", None)
        .expect("child");
    child.was_derived_from = vec![EntityReference::by_hash(root.data_hash.as_str())];
    rt.block_on(store.put(&child)).expect("put child");

    let gc_data = vec![2u8; 64];
    let mut grandchild = factory
        .from_data(&gc_data, "application/octet-stream", None)
        .expect("grandchild");
    grandchild.was_derived_from = vec![EntityReference::by_hash(child.data_hash.as_str())];
    rt.block_on(store.put(&grandchild)).expect("put grandchild");

    let root_ref = EntityReference::by_hash(grandchild.data_hash.as_str());
    let builder = ProvenanceGraphBuilder::new();

    c.bench_function("provenance_graph_traversal", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    builder
                        .build(root_ref.clone(), &store)
                        .await
                        .expect("build graph"),
                );
            });
        });
    });
}

criterion_group!(
    benches,
    bench_braid_creation,
    bench_store_put_get,
    bench_content_hashing,
    bench_store_query,
    bench_attribution,
    bench_compression,
    bench_query_traversal
);
criterion_main!(benches);
