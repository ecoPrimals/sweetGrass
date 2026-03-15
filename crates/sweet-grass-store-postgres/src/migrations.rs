// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Database migrations for the `PostgreSQL` store.
//!
//! Migrations are embedded and run automatically on startup.

use crate::{PostgresError, Result};
use sqlx::PgPool;
use tracing::{debug, info};

/// Initial schema migration.
/// Raw string preserves SQL readability; r#" avoids escaping inner quotes in SQL.
const MIGRATION_001_INIT: &str = r#"
-- SweetGrass Schema v1
-- Braid storage with indexes for efficient queries

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Main braids table
CREATE TABLE IF NOT EXISTS braids (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    braid_id VARCHAR(255) UNIQUE NOT NULL,
    data_hash VARCHAR(255) NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    attributed_to VARCHAR(255) NOT NULL,
    generated_at_time BIGINT NOT NULL,
    braid_type VARCHAR(50) NOT NULL DEFAULT 'atomic',
    
    -- JSON columns for complex data
    metadata JSONB NOT NULL DEFAULT '{}',
    ecop JSONB NOT NULL DEFAULT '{}',
    was_derived_from JSONB NOT NULL DEFAULT '[]',
    was_generated_by JSONB,
    signature JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Activities table
CREATE TABLE IF NOT EXISTS activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    activity_id VARCHAR(255) UNIQUE NOT NULL,
    activity_type VARCHAR(100) NOT NULL,
    started_at_time BIGINT NOT NULL,
    ended_at_time BIGINT,
    
    -- JSON columns
    used_entities JSONB NOT NULL DEFAULT '[]',
    was_associated_with JSONB NOT NULL DEFAULT '[]',
    metadata JSONB NOT NULL DEFAULT '{}',
    ecop JSONB NOT NULL DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Braid-Activity relationship
CREATE TABLE IF NOT EXISTS braid_activities (
    braid_id VARCHAR(255) NOT NULL REFERENCES braids(braid_id) ON DELETE CASCADE,
    activity_id VARCHAR(255) NOT NULL REFERENCES activities(activity_id) ON DELETE CASCADE,
    PRIMARY KEY (braid_id, activity_id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_braids_data_hash ON braids(data_hash);
CREATE INDEX IF NOT EXISTS idx_braids_attributed_to ON braids(attributed_to);
CREATE INDEX IF NOT EXISTS idx_braids_mime_type ON braids(mime_type);
CREATE INDEX IF NOT EXISTS idx_braids_generated_at ON braids(generated_at_time DESC);
CREATE INDEX IF NOT EXISTS idx_braids_braid_type ON braids(braid_type);

-- GIN indexes for JSONB queries
CREATE INDEX IF NOT EXISTS idx_braids_metadata ON braids USING GIN (metadata);
CREATE INDEX IF NOT EXISTS idx_braids_derived_from ON braids USING GIN (was_derived_from);

-- Tags table for efficient tag queries
CREATE TABLE IF NOT EXISTS braid_tags (
    braid_id VARCHAR(255) NOT NULL REFERENCES braids(braid_id) ON DELETE CASCADE,
    tag VARCHAR(255) NOT NULL,
    PRIMARY KEY (braid_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_braid_tags_tag ON braid_tags(tag);

-- Updated at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_braids_updated_at ON braids;
CREATE TRIGGER update_braids_updated_at
    BEFORE UPDATE ON braids
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
"#;

/// Migration version tracking table.
const MIGRATION_VERSION_TABLE: &str = r"
CREATE TABLE IF NOT EXISTS _sweetgrass_migrations (
    version INT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
";

/// Run all pending migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations");

    // Ensure migration tracking table exists
    sqlx::query(MIGRATION_VERSION_TABLE)
        .execute(pool)
        .await
        .map_err(|e| PostgresError::Migration(e.to_string()))?;

    // Get current version
    let current_version: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sweetgrass_migrations")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    debug!("Current migration version: {}", current_version);

    // Run migrations
    let migrations = [(1, "init", MIGRATION_001_INIT)];

    for (version, name, sql) in migrations {
        if version > current_version {
            info!("Applying migration {}: {}", version, name);

            // Run migration in transaction
            let mut tx = pool.begin().await.map_err(PostgresError::from)?;

            sqlx::query(sql).execute(&mut *tx).await.map_err(|e| {
                PostgresError::Migration(format!("Migration {version} failed: {e}"))
            })?;

            sqlx::query("INSERT INTO _sweetgrass_migrations (version, name) VALUES ($1, $2)")
                .bind(version)
                .bind(name)
                .execute(&mut *tx)
                .await
                .map_err(PostgresError::from)?;

            tx.commit().await.map_err(PostgresError::from)?;

            info!("Migration {} applied successfully", version);
        }
    }

    info!("Database migrations complete");
    Ok(())
}

/// Check if migrations are up to date.
pub async fn check_migrations(pool: &PgPool) -> Result<bool> {
    let current_version: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sweetgrass_migrations")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    // Latest migration version
    let latest_version = 1;

    Ok(current_version >= latest_version)
}
