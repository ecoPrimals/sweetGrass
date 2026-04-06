// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Activity storage tests for `PostgreSQL` backend.
//!
//! Tests `put_activity`, `get_activity`, `activities_for_braid`.

use super::common::{create_test_activity, create_test_braid, setup_postgres};
use sweet_grass_core::activity::ActivityType;
use sweet_grass_store::BraidStore;

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_put_activity_and_get_activity() {
    let (_container, store) = setup_postgres().await;

    let activity = create_test_activity();
    store.put_activity(&activity).await.expect("put_activity");

    let retrieved = store
        .get_activity(&activity.id)
        .await
        .expect("get_activity");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, activity.id);
    assert_eq!(retrieved.activity_type, ActivityType::Computation);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_get_activity_nonexistent_returns_none() {
    let (_container, store) = setup_postgres().await;

    let activity_id = sweet_grass_core::ActivityId::new();
    let retrieved = store
        .get_activity(&activity_id)
        .await
        .expect("get_activity");

    assert!(retrieved.is_none());
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_activities_for_braid_returns_associated_activities() {
    let (_container, store) = setup_postgres().await;

    // Create activity and braid; link via was_generated_by
    let activity = create_test_activity();
    store.put_activity(&activity).await.expect("put_activity");

    let braid = sweet_grass_core::Braid::builder()
        .data_hash("sha256:activity_braid")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTestAgent"))
        .generated_by(activity.clone())
        .build()
        .expect("braid");

    store.put(&braid).await.expect("put braid");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities_for_braid");

    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, activity.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_activities_for_braid_empty_when_no_association() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("no_activities");
    store.put(&braid).await.expect("put braid");

    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities_for_braid");

    assert!(activities.is_empty());
}
