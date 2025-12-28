//! `PostgreSQL` integration tests using testcontainers.
//!
//! These tests require Docker to be running and will spin up a real `PostgreSQL`
//! instance for testing. Run with:
//!

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code may use unwrap/expect for clarity
//! ```bash
//! cargo test -p sweet-grass-store-postgres --features integration-tests -- --ignored
//! ```

#![cfg(feature = "integration-tests")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;

use sweet_grass_core::{
    activity::{Activity, ActivityType},
    agent::{AgentAssociation, AgentRole, Did},
    braid::BraidMetadata,
    entity::EntityReference,
    Braid,
};
use sweet_grass_store::{BraidStore, QueryFilter, QueryOrder};
use sweet_grass_store_postgres::{PostgresConfig, PostgresStore};
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;

/// Helper to spin up a `PostgreSQL` container and return a connected store.
async fn setup_postgres() -> (ContainerAsync<Postgres>, PostgresStore) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get PostgreSQL port");

    let connection_string =
        format!("postgresql://postgres:postgres@127.0.0.1:{host_port}/postgres");

    let config = PostgresConfig::new(&connection_string)
        .max_connections(5)
        .min_connections(1);

    let store = PostgresStore::connect(&config)
        .await
        .expect("Failed to connect to PostgreSQL");

    store
        .run_migrations()
        .await
        .expect("Failed to run migrations");

    (container, store)
}

/// Create a test braid with the given hash.
fn create_test_braid(hash_suffix: &str) -> Braid {
    Braid::builder()
        .data_hash(format!("sha256:{hash_suffix}"))
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTestAgent"))
        .build()
        .expect("Failed to create test braid")
}

/// Create a test braid with metadata.
fn create_braid_with_metadata(hash_suffix: &str, tags: Vec<&str>) -> Braid {
    let metadata = BraidMetadata {
        title: Some(format!("Test Braid {hash_suffix}")),
        description: Some("A test braid for integration testing".to_string()),
        tags: tags.into_iter().map(String::from).collect(),
        ..Default::default()
    };

    Braid::builder()
        .data_hash(format!("sha256:{hash_suffix}"))
        .mime_type("application/json")
        .size(256)
        .attributed_to(Did::new("did:key:z6MkTestAgent"))
        .metadata(metadata)
        .build()
        .expect("Failed to create test braid")
}

/// Create a test activity.
fn create_test_activity() -> Activity {
    Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkTestAgent"),
            AgentRole::Creator,
        ))
        .compute_units(1.5)
        .build()
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_basic_crud() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("crud001");

    // Create
    store.put(&braid).await.expect("Failed to store braid");

    // Read
    let retrieved = store.get(&braid.id).await.expect("Failed to get braid");
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.data_hash, braid.data_hash);
    assert_eq!(retrieved.mime_type, braid.mime_type);

    // Exists
    assert!(store
        .exists(&braid.id)
        .await
        .expect("Failed to check exists"));

    // Delete
    let deleted = store.delete(&braid.id).await.expect("Failed to delete");
    assert!(deleted);

    // Verify deleted
    assert!(!store
        .exists(&braid.id)
        .await
        .expect("Failed to check exists"));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_get_by_hash() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("hash001");
    store.put(&braid).await.expect("Failed to store braid");

    let retrieved = store
        .get_by_hash(&braid.data_hash)
        .await
        .expect("Failed to get by hash");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_with_filter() {
    let (_container, store) = setup_postgres().await;

    // Store multiple braids
    for i in 0..5 {
        let braid = create_test_braid(&format!("query{i:03}"));
        store.put(&braid).await.expect("Failed to store braid");
    }

    // Query all
    let result = store
        .query(&QueryFilter::new().with_limit(10), QueryOrder::NewestFirst)
        .await
        .expect("Failed to query");

    assert_eq!(result.braids.len(), 5);
    assert_eq!(result.total_count, 5);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_by_agent() {
    let (_container, store) = setup_postgres().await;

    let agent1 = Did::new("did:key:z6MkAgent1");
    let agent2 = Did::new("did:key:z6MkAgent2");

    // Create braids for different agents
    let braid1 = Braid::builder()
        .data_hash("sha256:agent1_data")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(agent1.clone())
        .build()
        .expect("build");

    let braid2 = Braid::builder()
        .data_hash("sha256:agent2_data")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(agent2.clone())
        .build()
        .expect("build");

    store.put(&braid1).await.expect("store");
    store.put(&braid2).await.expect("store");

    // Query by agent
    let agent1_braids = store.by_agent(&agent1).await.expect("by_agent");
    assert_eq!(agent1_braids.len(), 1);
    assert_eq!(agent1_braids[0].was_attributed_to, agent1);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_tags() {
    let (_container, store) = setup_postgres().await;

    let braid = create_braid_with_metadata("tags001", vec!["rust", "provenance", "test"]);
    store.put(&braid).await.expect("Failed to store braid");

    // Query by tag
    let filter = QueryFilter::new().with_tag("provenance");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("Failed to query");

    assert_eq!(result.braids.len(), 1);
    assert_eq!(result.braids[0].id, braid.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_activity_storage() {
    let (_container, store) = setup_postgres().await;

    let activity = create_test_activity();

    // Store activity
    store
        .put_activity(&activity)
        .await
        .expect("Failed to store activity");

    // Retrieve activity
    let retrieved = store
        .get_activity(&activity.id)
        .await
        .expect("Failed to get activity");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.activity_type, ActivityType::Computation);
    assert_eq!(retrieved.ecop.compute_units, Some(1.5));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_braid_activity_relationship() {
    let (_container, store) = setup_postgres().await;

    // Create an activity
    let activity = create_test_activity();
    store
        .put_activity(&activity)
        .await
        .expect("Failed to store activity");

    // Create a braid generated by the activity
    let mut braid = create_test_braid("withactivity001");
    braid.was_generated_by = Some(activity.clone());
    store.put(&braid).await.expect("Failed to store braid");

    // Query activities for braid
    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("Failed to get activities");

    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].id, activity.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_derived_from() {
    let (_container, store) = setup_postgres().await;

    // Create parent braid
    let parent = create_test_braid("parent001");
    store.put(&parent).await.expect("store parent");

    // Create child braid derived from parent
    let mut child = create_test_braid("child001");
    child.was_derived_from = vec![EntityReference::by_hash(&parent.data_hash)];
    store.put(&child).await.expect("store child");

    // Query derived braids
    let derived = store
        .derived_from(&parent.data_hash)
        .await
        .expect("derived_from");

    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].id, child.id);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_upsert_behavior() {
    let (_container, store) = setup_postgres().await;

    let mut braid = create_test_braid("upsert001");
    store.put(&braid).await.expect("first put");

    // Modify and re-put (should upsert)
    braid.mime_type = "application/octet-stream".to_string();
    store.put(&braid).await.expect("second put");

    // Verify update
    let retrieved = store.get(&braid.id).await.expect("get").unwrap();
    assert_eq!(retrieved.mime_type, "application/octet-stream");

    // Should still be one braid
    let count = store.count(&QueryFilter::default()).await.expect("count");
    assert_eq!(count, 1);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_health_check() {
    let (_container, store) = setup_postgres().await;

    let health = store.health().await.expect("health check");
    assert!(health);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migrations_check() {
    let (_container, store) = setup_postgres().await;

    let up_to_date = store.check_migrations().await.expect("migrations check");
    assert!(up_to_date);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering() {
    let (_container, store) = setup_postgres().await;

    // Create braids with different sizes
    for (i, size) in [(1, 100u64), (2, 500u64), (3, 250u64)] {
        let mut braid = create_test_braid(&format!("order{i:03}"));
        braid.size = size;
        store.put(&braid).await.expect("store");
    }

    // Query largest first
    let result = store
        .query(&QueryFilter::new(), QueryOrder::LargestFirst)
        .await
        .expect("query");

    assert_eq!(result.braids.len(), 3);
    assert_eq!(result.braids[0].size, 500);
    assert_eq!(result.braids[1].size, 250);
    assert_eq!(result.braids[2].size, 100);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_pagination() {
    let (_container, store) = setup_postgres().await;

    // Store 10 braids
    for i in 0..10 {
        let braid = create_test_braid(&format!("page{i:03}"));
        store.put(&braid).await.expect("store");
    }

    // Query first page
    let page1 = store
        .query(
            &QueryFilter::new().with_limit(3).with_offset(0),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("page 1");

    assert_eq!(page1.braids.len(), 3);
    assert_eq!(page1.total_count, 10);
    assert!(page1.has_more);

    // Query second page
    let page2 = store
        .query(
            &QueryFilter::new().with_limit(3).with_offset(3),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("page 2");

    assert_eq!(page2.braids.len(), 3);
    assert!(page2.has_more);

    // Query last page
    let last = store
        .query(
            &QueryFilter::new().with_limit(3).with_offset(9),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("last page");

    assert_eq!(last.braids.len(), 1);
    assert!(!last.has_more);
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_concurrent_writes() {
    let (_container, store) = setup_postgres().await;
    let store = Arc::new(store);

    let mut handles = vec![];

    for i in 0..10 {
        let store = Arc::clone(&store);
        handles.push(tokio::spawn(async move {
            let braid = create_test_braid(&format!("concurrent{i:03}"));
            store.put(&braid).await
        }));
    }

    for handle in handles {
        handle.await.expect("join").expect("put");
    }

    let count = store.count(&QueryFilter::default()).await.expect("count");
    assert_eq!(count, 10);
}

// ============================================================================
// Migration Tests (comprehensive coverage)
// ============================================================================

/// Setup PostgreSQL without running migrations (for migration tests).
async fn setup_postgres_no_migrations() -> (ContainerAsync<Postgres>, PostgresStore) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get PostgreSQL port");

    let connection_string =
        format!("postgresql://postgres:postgres@127.0.0.1:{host_port}/postgres");

    let config = PostgresConfig::new(&connection_string)
        .max_connections(5)
        .min_connections(1);

    let store = PostgresStore::connect(&config)
        .await
        .expect("Failed to connect to PostgreSQL");

    (container, store)
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_all_tables() {
    let (_container, store) = setup_postgres_no_migrations().await;

    // Run migrations
    store.run_migrations().await.expect("migrations");

    // Verify tables exist
    let pool = store.pool();

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' 
         ORDER BY table_name",
    )
    .fetch_all(pool)
    .await
    .expect("query tables");

    assert!(tables.contains(&"braids".to_string()));
    assert!(tables.contains(&"activities".to_string()));
    assert!(tables.contains(&"braid_activities".to_string()));
    assert!(tables.contains(&"braid_tags".to_string()));
    assert!(tables.contains(&"_sweetgrass_migrations".to_string()));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_idempotency() {
    let (_container, store) = setup_postgres_no_migrations().await;

    // Run migrations twice
    store.run_migrations().await.expect("first run");
    store
        .run_migrations()
        .await
        .expect("second run should not fail");

    // Verify only one migration version is recorded
    let pool = store.pool();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sweetgrass_migrations")
        .fetch_one(pool)
        .await
        .expect("count");

    assert_eq!(count, 1, "Only one migration record should exist");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_version_tracking() {
    let (_container, store) = setup_postgres_no_migrations().await;

    // Run migrations
    store.run_migrations().await.expect("migrations");

    // Check version tracking
    let pool = store.pool();
    let version: i32 = sqlx::query_scalar(
        "SELECT version FROM _sweetgrass_migrations ORDER BY version DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .expect("version");

    assert_eq!(version, 1, "Current migration version should be 1");

    let name: String =
        sqlx::query_scalar("SELECT name FROM _sweetgrass_migrations WHERE version = 1")
            .fetch_one(pool)
            .await
            .expect("name");

    assert_eq!(name, "init", "First migration should be named 'init'");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_braids_columns() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'braids' 
         ORDER BY column_name",
    )
    .fetch_all(pool)
    .await
    .expect("columns");

    // Verify essential columns exist
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"braid_id".to_string()));
    assert!(columns.contains(&"data_hash".to_string()));
    assert!(columns.contains(&"mime_type".to_string()));
    assert!(columns.contains(&"size".to_string()));
    assert!(columns.contains(&"attributed_to".to_string()));
    assert!(columns.contains(&"generated_at_time".to_string()));
    assert!(columns.contains(&"braid_type".to_string()));
    assert!(columns.contains(&"metadata".to_string()));
    assert!(columns.contains(&"ecop".to_string()));
    assert!(columns.contains(&"was_derived_from".to_string()));
    assert!(columns.contains(&"was_generated_by".to_string()));
    assert!(columns.contains(&"signature".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
    assert!(columns.contains(&"updated_at".to_string()));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_activities_columns() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name FROM information_schema.columns 
         WHERE table_name = 'activities' 
         ORDER BY column_name",
    )
    .fetch_all(pool)
    .await
    .expect("columns");

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"activity_id".to_string()));
    assert!(columns.contains(&"activity_type".to_string()));
    assert!(columns.contains(&"started_at_time".to_string()));
    assert!(columns.contains(&"ended_at_time".to_string()));
    assert!(columns.contains(&"used_entities".to_string()));
    assert!(columns.contains(&"was_associated_with".to_string()));
    assert!(columns.contains(&"metadata".to_string()));
    assert!(columns.contains(&"ecop".to_string()));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_indexes() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT indexname FROM pg_indexes 
         WHERE schemaname = 'public' AND tablename = 'braids'
         ORDER BY indexname",
    )
    .fetch_all(pool)
    .await
    .expect("indexes");

    // Verify essential indexes exist
    assert!(indexes.contains(&"braids_pkey".to_string()));
    assert!(indexes.contains(&"braids_braid_id_key".to_string()));
    assert!(indexes.contains(&"idx_braids_data_hash".to_string()));
    assert!(indexes.contains(&"idx_braids_attributed_to".to_string()));
    assert!(indexes.contains(&"idx_braids_mime_type".to_string()));
    assert!(indexes.contains(&"idx_braids_generated_at".to_string()));
    assert!(indexes.contains(&"idx_braids_braid_type".to_string()));
    assert!(indexes.contains(&"idx_braids_metadata".to_string()));
    assert!(indexes.contains(&"idx_braids_derived_from".to_string()));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_gin_indexes() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();

    // Verify GIN indexes for JSONB columns
    let gin_indexes: Vec<(String, String)> = sqlx::query_as(
        "SELECT indexname, indexdef FROM pg_indexes 
         WHERE schemaname = 'public' 
         AND indexdef LIKE '%USING gin%'
         ORDER BY indexname",
    )
    .fetch_all(pool)
    .await
    .expect("gin indexes");

    assert!(!gin_indexes.is_empty(), "GIN indexes should exist");

    let gin_index_names: Vec<String> = gin_indexes.iter().map(|(name, _)| name.clone()).collect();

    assert!(gin_index_names.contains(&"idx_braids_metadata".to_string()));
    assert!(gin_index_names.contains(&"idx_braids_derived_from".to_string()));
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_foreign_keys() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();
    let fks: Vec<String> = sqlx::query_scalar(
        "SELECT constraint_name FROM information_schema.table_constraints 
         WHERE table_name = 'braid_activities' 
         AND constraint_type = 'FOREIGN KEY'",
    )
    .fetch_all(pool)
    .await
    .expect("foreign keys");

    assert!(
        fks.len() >= 2,
        "braid_activities should have 2 foreign keys"
    );
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_creates_trigger() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();

    // Check if the trigger function exists
    let function_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM pg_proc 
            WHERE proname = 'update_updated_at_column'
        )",
    )
    .fetch_one(pool)
    .await
    .expect("check function");

    assert!(
        function_exists,
        "update_updated_at_column function should exist"
    );

    // Check if the trigger exists
    let trigger_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM pg_trigger 
            WHERE tgname = 'update_braids_updated_at'
        )",
    )
    .fetch_one(pool)
    .await
    .expect("check trigger");

    assert!(
        trigger_exists,
        "update_braids_updated_at trigger should exist"
    );
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_trigger_functionality() {
    let (_container, store) = setup_postgres().await;

    // Create a test braid
    let braid = create_test_braid("trigger_test");
    store.put(&braid).await.expect("put");

    let pool = store.pool();

    // Get initial timestamps
    let (created_at, updated_at): (String, String) =
        sqlx::query_as("SELECT created_at::text, updated_at::text FROM braids WHERE braid_id = $1")
            .bind(braid.id.as_str())
            .fetch_one(pool)
            .await
            .expect("fetch timestamps");

    assert_eq!(created_at, updated_at, "Initially timestamps should match");

    // Update the braid (PostgreSQL trigger will update updated_at automatically)
    sqlx::query("UPDATE braids SET size = size + 1 WHERE braid_id = $1")
        .bind(braid.id.as_str())
        .execute(pool)
        .await
        .expect("update");

    // Check updated_at changed
    let (new_created_at, new_updated_at): (String, String) =
        sqlx::query_as("SELECT created_at::text, updated_at::text FROM braids WHERE braid_id = $1")
            .bind(braid.id.as_str())
            .fetch_one(pool)
            .await
            .expect("fetch new timestamps");

    assert_eq!(created_at, new_created_at, "created_at should not change");
    assert_ne!(updated_at, new_updated_at, "updated_at should change");
}

#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_migration_uuid_extension() {
    let (_container, store) = setup_postgres_no_migrations().await;
    store.run_migrations().await.expect("migrations");

    let pool = store.pool();

    // Check if uuid-ossp extension is installed
    let extension_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM pg_extension 
            WHERE extname = 'uuid-ossp'
        )",
    )
    .fetch_one(pool)
    .await
    .expect("check extension");

    assert!(extension_exists, "uuid-ossp extension should be installed");

    // Test that uuid_generate_v4() works
    let uuid: String = sqlx::query_scalar("SELECT uuid_generate_v4()::text")
        .fetch_one(pool)
        .await
        .expect("generate uuid");

    assert!(
        uuid.len() == 36,
        "UUID should be 36 characters (with hyphens)"
    );
}

// ============================================================================
// NEW COMPREHENSIVE TESTS (Dec 27, 2025) - Coverage Expansion
// ============================================================================

/// Test concurrent writes don't cause deadlocks or data corruption
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_concurrent_writes() {
    let (_container, store) = setup_postgres().await;
    let store = Arc::new(store);

    // Spawn 20 concurrent write operations
    let mut handles = vec![];
    for i in 0..20 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let braid = create_test_braid(&format!("concurrent_{i:03}"));
            store_clone.put(&braid).await
        });
        handles.push(handle);
    }

    // Wait for all writes to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify count
    let count = store.count().await.expect("count");
    assert_eq!(count, 20);
}

/// Test query with invalid filter doesn't crash
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_empty_results() {
    let (_container, store) = setup_postgres().await;

    // Query with filter that matches nothing
    let filter = QueryFilter::new().with_tag("nonexistent_tag_xyz");
    let result = store
        .query(&filter, QueryOrder::NewestFirst)
        .await
        .expect("query should succeed");

    assert_eq!(result.braids.len(), 0);
    assert_eq!(result.total_count, 0);
}

/// Test delete nonexistent braid returns false
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_delete_nonexistent() {
    let (_container, store) = setup_postgres().await;

    let nonexistent_id = sweet_grass_core::braid::BraidId::new();
    let deleted = store.delete(&nonexistent_id).await.expect("delete");

    assert!(!deleted, "Deleting nonexistent should return false");
}

/// Test get_by_hash with nonexistent hash
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_get_by_hash_nonexistent() {
    let (_container, store) = setup_postgres().await;

    let result = store
        .get_by_hash(&"sha256:nonexistent".parse().unwrap())
        .await
        .expect("get_by_hash");

    assert!(result.is_none());
}

/// Test query pagination works correctly
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_pagination() {
    let (_container, store) = setup_postgres().await;

    // Create 15 braids
    for i in 0..15 {
        let braid = create_test_braid(&format!("page_{i:03}"));
        store.put(&braid).await.expect("put");
    }

    // Page 1: limit 5, offset 0
    let page1 = store
        .query(
            &QueryFilter::new().with_limit(5).with_offset(0),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("query page 1");

    assert_eq!(page1.braids.len(), 5);
    assert_eq!(page1.total_count, 15);

    // Page 2: limit 5, offset 5
    let page2 = store
        .query(
            &QueryFilter::new().with_limit(5).with_offset(5),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("query page 2");

    assert_eq!(page2.braids.len(), 5);
    assert_eq!(page2.total_count, 15);

    // Page 3: limit 5, offset 10
    let page3 = store
        .query(
            &QueryFilter::new().with_limit(5).with_offset(10),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("query page 3");

    assert_eq!(page3.braids.len(), 5);
    assert_eq!(page3.total_count, 15);

    // Verify no duplicates across pages
    let mut all_ids = std::collections::HashSet::new();
    all_ids.extend(page1.braids.iter().map(|b| &b.id));
    all_ids.extend(page2.braids.iter().map(|b| &b.id));
    all_ids.extend(page3.braids.iter().map(|b| &b.id));
    assert_eq!(all_ids.len(), 15, "Should have no duplicates");
}

/// Test count method accuracy
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_count_accuracy() {
    let (_container, store) = setup_postgres().await;

    // Initially empty
    assert_eq!(store.count().await.expect("count"), 0);

    // Add 3 braids
    for i in 0..3 {
        let braid = create_test_braid(&format!("count_{i}"));
        store.put(&braid).await.expect("put");
    }

    assert_eq!(store.count().await.expect("count"), 3);

    // Delete 1
    let braid = create_test_braid("count_0");
    store.delete(&braid.id).await.expect("delete");

    assert_eq!(store.count().await.expect("count"), 2);
}

/// Test exists returns correct boolean
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_exists_correctness() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("exists_test");

    // Should not exist initially
    assert!(!store.exists(&braid.id).await.expect("exists"));

    // Store it
    store.put(&braid).await.expect("put");

    // Should exist now
    assert!(store.exists(&braid.id).await.expect("exists"));

    // Delete it
    store.delete(&braid.id).await.expect("delete");

    // Should not exist anymore
    assert!(!store.exists(&braid.id).await.expect("exists"));
}

/// Test derived_from relationships
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_derived_from_relationships() {
    let (_container, store) = setup_postgres().await;

    // Create parent braid
    let parent = create_test_braid("parent_001");
    store.put(&parent).await.expect("put parent");

    // Create child derived from parent
    let child = Braid::builder()
        .data_hash("sha256:child_001")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTestAgent"))
        .was_derived_from(vec![EntityReference::by_hash(&parent.data_hash)])
        .build()
        .expect("build child");

    store.put(&child).await.expect("put child");

    // Query derived_from
    let derived = store
        .derived_from(&parent.data_hash)
        .await
        .expect("derived_from");

    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0].id, child.id);
}

/// Test by_agent filters correctly
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_by_agent_filtering() {
    let (_container, store) = setup_postgres().await;

    let agent1 = Did::new("did:key:z6MkAgent1");
    let agent2 = Did::new("did:key:z6MkAgent2");

    // Create 3 braids for agent1, 2 for agent2
    for i in 0..3 {
        let braid = Braid::builder()
            .data_hash(format!("sha256:agent1_{i}"))
            .mime_type("text/plain")
            .size(100)
            .attributed_to(agent1.clone())
            .build()
            .expect("build");
        store.put(&braid).await.expect("put");
    }

    for i in 0..2 {
        let braid = Braid::builder()
            .data_hash(format!("sha256:agent2_{i}"))
            .mime_type("text/plain")
            .size(100)
            .attributed_to(agent2.clone())
            .build()
            .expect("build");
        store.put(&braid).await.expect("put");
    }

    // Query agent1
    let agent1_braids = store.by_agent(&agent1).await.expect("by_agent");
    assert_eq!(agent1_braids.len(), 3);

    // Query agent2
    let agent2_braids = store.by_agent(&agent2).await.expect("by_agent");
    assert_eq!(agent2_braids.len(), 2);

    // Query nonexistent agent
    let nonexistent = Did::new("did:key:z6MkNonexistent");
    let no_braids = store.by_agent(&nonexistent).await.expect("by_agent");
    assert_eq!(no_braids.len(), 0);
}

/// Test activity CRUD operations
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_activity_crud_complete() {
    let (_container, store) = setup_postgres().await;

    let activity = create_test_activity();

    // Create
    store.put_activity(&activity).await.expect("put_activity");

    // Read
    let retrieved = store
        .get_activity(&activity.id)
        .await
        .expect("get_activity");
    assert!(retrieved.is_some());

    // Get nonexistent
    let nonexistent_id = sweet_grass_core::activity::ActivityId::new();
    let none = store
        .get_activity(&nonexistent_id)
        .await
        .expect("get_activity");
    assert!(none.is_none());
}

/// Test activities_for_braid
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_activities_for_braid() {
    let (_container, store) = setup_postgres().await;

    let braid = create_test_braid("activities_test");
    store.put(&braid).await.expect("put");

    // Create 3 activities for this braid
    for i in 0..3 {
        let activity = Activity::builder(ActivityType::Computation)
            .associated_with(AgentAssociation::new(
                Did::new(&format!("did:key:z6MkAgent{i}")),
                AgentRole::Creator,
            ))
            .used(EntityReference::by_hash(&braid.data_hash))
            .compute_units(1.0 + i as f64)
            .build();

        store.put_activity(&activity).await.expect("put_activity");
    }

    // Query activities for this braid
    let activities = store
        .activities_for_braid(&braid.id)
        .await
        .expect("activities_for_braid");

    assert_eq!(activities.len(), 3);
}

/// Test update existing braid (put with same ID)
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_update_existing_braid() {
    let (_container, store) = setup_postgres().await;

    let mut braid = create_test_braid("update_test");
    braid.size = 100;

    // Initial put
    store.put(&braid).await.expect("initial put");

    // Update size
    braid.size = 200;
    store.put(&braid).await.expect("update put");

    // Verify updated
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().size, 200);
}

/// Test query with multiple tags
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_multiple_tags() {
    let (_container, store) = setup_postgres().await;

    // Braid with tags: [rust, test]
    let braid1 = create_braid_with_metadata("multi_tag_1", vec!["rust", "test"]);
    store.put(&braid1).await.expect("put");

    // Braid with tags: [rust, production]
    let braid2 = create_braid_with_metadata("multi_tag_2", vec!["rust", "production"]);
    store.put(&braid2).await.expect("put");

    // Query for "rust" tag
    let rust_results = store
        .query(
            &QueryFilter::new().with_tag("rust"),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("query");

    assert_eq!(rust_results.braids.len(), 2);

    // Query for "test" tag
    let test_results = store
        .query(
            &QueryFilter::new().with_tag("test"),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("query");

    assert_eq!(test_results.braids.len(), 1);
}

/// Test query ordering (OldestFirst vs NewestFirst)
#[tokio::test]
#[ignore = "Requires Docker"]
async fn test_query_ordering() {
    let (_container, store) = setup_postgres().await;

    // Create 3 braids with delays to ensure different timestamps
    let braid1 = create_test_braid("order_1");
    store.put(&braid1).await.expect("put 1");

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let braid2 = create_test_braid("order_2");
    store.put(&braid2).await.expect("put 2");

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let braid3 = create_test_braid("order_3");
    store.put(&braid3).await.expect("put 3");

    // Query newest first
    let newest = store
        .query(&QueryFilter::new(), QueryOrder::NewestFirst)
        .await
        .expect("query newest");

    assert_eq!(newest.braids[0].id, braid3.id);
    assert_eq!(newest.braids[2].id, braid1.id);

    // Query oldest first
    let oldest = store
        .query(&QueryFilter::new(), QueryOrder::OldestFirst)
        .await
        .expect("query oldest");

    assert_eq!(oldest.braids[0].id, braid1.id);
    assert_eq!(oldest.braids[2].id, braid3.id);
}
