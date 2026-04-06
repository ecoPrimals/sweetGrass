// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_put_batch() {
    let (store, _temp) = create_test_store();

    let braids: Vec<Braid> = (0..5)
        .map(|i| create_test_braid(&format!("sha256:batch{i}")))
        .collect();

    let (succeeded, errors) = store.put_batch(&braids, Some(5)).await;
    assert_eq!(succeeded, 5);
    assert!(errors.is_empty());

    for braid in &braids {
        let retrieved = store.get(&braid.id).await.expect("get");
        assert!(retrieved.is_some());
    }
}

#[tokio::test]
async fn test_get_batch() {
    let (store, _temp) = create_test_store();

    let braids: Vec<Braid> = (0..4)
        .map(|i| create_test_braid(&format!("sha256:getbatch{i}")))
        .collect();

    for braid in &braids {
        store.put(braid).await.expect("put");
    }

    let ids: Vec<BraidId> = braids.iter().map(|b| b.id.clone()).collect();
    let (results, errors) = store.get_batch(&ids, Some(10)).await;

    assert!(errors.is_empty());
    assert_eq!(results.len(), 4);
    let expected_hashes: std::collections::HashSet<_> =
        (0..4).map(|i| format!("sha256:getbatch{i}")).collect();
    let found_hashes: std::collections::HashSet<_> = results
        .iter()
        .filter_map(|opt| opt.as_ref())
        .map(|b| b.data_hash.as_str().to_string())
        .collect();
    assert_eq!(found_hashes, expected_hashes);
}

#[tokio::test]
async fn test_get_batch_mixed_existent_nonexistent() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:batch_mixed");
    store.put(&braid).await.expect("put");

    let ids = vec![
        braid.id.clone(),
        BraidId::from_string("nonexistent-1".to_string()),
        braid.id.clone(),
    ];
    let (results, errors) = store.get_batch(&ids, Some(5)).await;

    assert!(errors.is_empty());
    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}
