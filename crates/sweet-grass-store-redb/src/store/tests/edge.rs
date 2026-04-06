// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use crate::tables::BRAIDS;
use redb::Database;

use super::*;

#[tokio::test]
async fn test_store_empty_metadata_braid() {
    let (store, _temp) = create_test_store();

    let braid = BraidBuilder::default()
        .data_hash("sha256:empty_meta")
        .mime_type("text/plain")
        .size(0)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .metadata(sweet_grass_core::braid::BraidMetadata::default())
        .build()
        .expect("build braid");

    store.put(&braid).await.expect("put");
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().metadata.tags.is_empty());
}

#[tokio::test]
async fn test_store_large_braid() {
    let (store, _temp) = create_test_store();

    let mut braid = create_test_braid("sha256:large");
    braid.size = 10 * 1024 * 1024;
    braid.metadata.description = Some("x".repeat(100_000).into());

    store.put(&braid).await.expect("put");
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().size, 10 * 1024 * 1024);
}

#[tokio::test]
async fn test_concurrent_put_and_get() {
    let (store, _temp) = create_test_store();
    let store = std::sync::Arc::new(store);

    let mut handles = vec![];
    for i in 0..10 {
        let s = std::sync::Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let braid = create_test_braid(&format!("sha256:conc_{i}"));
            s.put(&braid).await.expect("put");
            let retrieved = s.get(&braid.id).await.expect("get");
            assert!(retrieved.is_some());
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("join");
    }
}

#[tokio::test]
async fn test_corrupted_data() {
    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("corrupt_db.redb");

    let braid = create_test_braid("sha256:corrupt_test");
    let braid_id = braid.id.clone();

    // Put a valid braid first
    {
        let store = RedbStore::open_path(&db_path).expect("open");
        store.put(&braid).await.expect("put");
    }

    // Corrupt the braid data directly in redb
    {
        let db = Database::create(&db_path).expect("open redb");
        let write_txn = db.begin_write().expect("begin write");
        {
            let mut braids = write_txn.open_table(BRAIDS).expect("open braids");
            braids
                .insert(braid_id.as_str().as_bytes(), b"invalid json {{{".as_slice())
                .expect("insert corrupt");
        }
        write_txn.commit().expect("commit");
    }

    // Reopen and try to get - should fail with deserialization error
    let store = RedbStore::open_path(&db_path).expect("open");
    let result = store.get(&braid_id).await;
    assert!(result.is_err());
}
