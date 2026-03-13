// SPDX-License-Identifier: AGPL-3.0-only
//! `PostgreSQL` store error types.

use thiserror::Error;

/// Result type for `PostgreSQL` operations.
pub type Result<T> = std::result::Result<T, PostgresError>;

/// `PostgreSQL` store error types.
#[derive(Debug, Error)]
pub enum PostgresError {
    /// Database connection error.
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Query execution error.
    #[error("Query error: {0}")]
    Query(String),

    /// Migration error.
    #[error("Migration error: {0}")]
    Migration(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Record not found.
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Constraint violation.
    #[error("Constraint violation: {0}")]
    Constraint(String),

    /// Pool error.
    #[error("Connection pool error: {0}")]
    Pool(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<sqlx::Error> for PostgresError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Row not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    // PostgreSQL constraint violation codes
                    if code.starts_with("23") {
                        return Self::Constraint(db_err.message().to_string());
                    }
                }
                Self::Query(db_err.message().to_string())
            },
            sqlx::Error::PoolTimedOut => Self::Pool("Connection pool timed out".to_string()),
            sqlx::Error::PoolClosed => Self::Pool("Connection pool closed".to_string()),
            sqlx::Error::Configuration(msg) => Self::Config(msg.to_string()),
            _ => Self::Query(err.to_string()),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for PostgresError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::Migration(err.to_string())
    }
}

impl From<serde_json::Error> for PostgresError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<PostgresError> for sweet_grass_store::StoreError {
    fn from(err: PostgresError) -> Self {
        match err {
            PostgresError::NotFound(msg) => Self::NotFound(msg),
            PostgresError::Constraint(msg) => Self::Duplicate(msg),
            _ => Self::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_error_display() {
        let err = PostgresError::Connection("failed to connect".to_string());
        assert!(err.to_string().contains("Database connection error"));

        let err = PostgresError::Query("invalid sql".to_string());
        assert!(err.to_string().contains("Query error"));

        let err = PostgresError::Migration("version mismatch".to_string());
        assert!(err.to_string().contains("Migration error"));

        let err = PostgresError::Serialization("json error".to_string());
        assert!(err.to_string().contains("Serialization error"));

        let err = PostgresError::NotFound("braid-123".to_string());
        assert!(err.to_string().contains("Record not found"));

        let err = PostgresError::Constraint("unique violation".to_string());
        assert!(err.to_string().contains("Constraint violation"));

        let err = PostgresError::Pool("timeout".to_string());
        assert!(err.to_string().contains("Connection pool error"));

        let err = PostgresError::Config("bad url".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_postgres_error_to_store_error_not_found() {
        let pg_err = PostgresError::NotFound("missing".to_string());
        let store_err: sweet_grass_store::StoreError = pg_err.into();
        assert!(matches!(
            store_err,
            sweet_grass_store::StoreError::NotFound(_)
        ));
    }

    #[test]
    fn test_postgres_error_to_store_error_constraint() {
        let pg_err = PostgresError::Constraint("duplicate".to_string());
        let store_err: sweet_grass_store::StoreError = pg_err.into();
        assert!(matches!(
            store_err,
            sweet_grass_store::StoreError::Duplicate(_)
        ));
    }

    #[test]
    fn test_postgres_error_to_store_error_internal() {
        let pg_err = PostgresError::Connection("timeout".to_string());
        let store_err: sweet_grass_store::StoreError = pg_err.into();
        assert!(matches!(
            store_err,
            sweet_grass_store::StoreError::Internal(_)
        ));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: std::result::Result<serde_json::Value, _> = serde_json::from_str("{bad}");
        let err: PostgresError = json_err.unwrap_err().into();
        assert!(matches!(err, PostgresError::Serialization(_)));
    }
}
