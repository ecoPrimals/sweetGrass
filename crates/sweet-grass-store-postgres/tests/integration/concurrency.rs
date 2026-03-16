// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Concurrency tests for `PostgreSQL` backend.
//!
//! Tests parallel puts, concurrent reads while writing, batch operations.

use super::common::{create_test_braid, setup_postgres};
use sweet_grass_store::BraidStore;
use tokio::task::JoinSet;

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_parallel_puts_different_braids() {
    let (_container, store) = setup_postgres().await;

    let mut handles = JoinSet::new();
    for i in 0..10 {
        let store = store.clone();
        handles.spawn(async move {
            let braid = create_test_braid(&format!("parallel{i}"));
            store.put(&braid).await
        });
    }

    let mut success_count = 0;
    while let Some(res) = handles.join_next().await {
        let result = res.expect("task panicked");
        if result.is_ok() {
            success_count += 1;
        }
    }
    assert_eq!(success_count, 10);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_concurrent_reads_while_writing() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("concurrent");
    store.put(&braid).await.expect("initial put");

    let store_clone = store.clone();
    let write_handle = tokio::spawn(async move {
        for i in 0u64..5 {
            let mut b = create_test_braid(&format!("concurrent_write{i}"));
            b.size = 100 + i;
            store_clone.put(&b).await.expect("put");
        }
    });

    let store_clone = store.clone();
    let read_handle = tokio::spawn(async move {
        for _ in 0..20 {
            let result = store_clone.get(&braid.id).await.expect("get");
            assert!(result.is_some());
        }
    });

    write_handle.await.expect("write task");
    read_handle.await.expect("read task");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_batch_operations() {
    let (_container, store) = setup_postgres().await;

    let braids: Vec<_> = (0..10)
        .map(|i| create_test_braid(&format!("batch{i}")))
        .collect();

    let (succeeded, errors) = store.put_batch(&braids, Some(5)).await;
    assert_eq!(succeeded, 10);
    assert!(errors.is_empty());

    let ids: Vec<_> = braids.iter().map(|b| b.id.clone()).collect();
    let retrieved = store.get_batch(&ids, Some(5)).await;
    assert_eq!(retrieved.len(), 10);
    assert!(retrieved.iter().all(Option::is_some));
}
