// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_config_builder() {
    let config = RedbConfig::new("/tmp/test.redb");
    assert!(config.path.contains("test"));
}

#[tokio::test]
async fn test_redb_config_default() {
    let config = RedbConfig::default();
    assert!(config.path.contains("sweetgrass"));
}

#[tokio::test]
async fn test_open_path_with_config() {
    let temp = TempDir::new().expect("create temp dir");
    let db_path = temp.path().join("config_test.redb");
    let config = RedbConfig::new(db_path.to_string_lossy().to_string());

    let store = RedbStore::open(&config).expect("open");
    let braid = create_test_braid("sha256:config_test");
    store.put(&braid).await.expect("put");

    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
}
