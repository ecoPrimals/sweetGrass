// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_activity_storage() {
    use sweet_grass_core::activity::{Activity, ActivityType};

    let (store, _temp) = create_test_store();
    let activity = Activity::builder(ActivityType::Creation).build();

    store.put_activity(&activity).await.expect("put");

    let retrieved = store.get_activity(&activity.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, activity.id);
}

#[tokio::test]
async fn test_get_activity_nonexistent() {
    use sweet_grass_core::activity::ActivityId;

    let (store, _temp) = create_test_store();

    let result = store
        .get_activity(&ActivityId::from_task("nonexistent"))
        .await
        .expect("get");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_activities_for_braid() {
    let (store, _temp) = create_test_store();

    let braid = create_test_braid("sha256:activities_test");
    store.put(&braid).await.expect("put");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities");
    assert!(activities.is_empty());
}

#[tokio::test]
async fn test_activities_for_braid_returns_embedded_activity() {
    use sweet_grass_core::activity::{Activity, ActivityType};

    let (store, _temp) = create_test_store();

    let activity = Activity::builder(ActivityType::Creation).build();
    let braid = BraidBuilder::default()
        .data_hash("sha256:with_activity")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .generated_by(activity.clone())
        .build()
        .expect("build braid");

    store.put(&braid).await.expect("put");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, activity.id);
}

#[tokio::test]
async fn test_activities_for_braid_unknown_returns_empty() {
    let (store, _temp) = create_test_store();

    let unknown = BraidId::from_string("nonexistent-braid-id".to_string());
    let activities = store
        .activities_for_braid(&unknown)
        .await
        .expect("activities");
    assert!(activities.is_empty());
}
