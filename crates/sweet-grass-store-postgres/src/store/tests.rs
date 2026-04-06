// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;

// ========================================================================
// Configuration Tests
// ========================================================================

#[test]
fn test_postgres_config_default() {
    let config = PostgresConfig::default();
    assert_eq!(config.max_connections, crate::DEFAULT_MAX_CONNECTIONS);
    assert_eq!(config.min_connections, 1);
    assert_eq!(
        config.connect_timeout_secs,
        crate::DEFAULT_CONNECT_TIMEOUT_SECS
    );
    assert_eq!(config.idle_timeout_secs, crate::DEFAULT_IDLE_TIMEOUT_SECS);
}

#[test]
fn test_postgres_config_builder() {
    let config = PostgresConfig::new("postgresql://test")
        .max_connections(20)
        .min_connections(5);

    assert_eq!(config.database_url, "postgresql://test");
    assert_eq!(config.max_connections, 20);
    assert_eq!(config.min_connections, 5);
}

#[test]
fn test_postgres_config_from_reader_missing() {
    let config = PostgresConfig::from_reader(|_| None);
    assert!(config.is_none());
}

#[test]
fn test_postgres_config_from_reader_present() {
    let config = PostgresConfig::from_reader(|key| {
        (key == "DATABASE_URL").then(|| "postgresql://envtest".to_string())
    });
    assert!(config.is_some());
    assert_eq!(config.unwrap().database_url, "postgresql://envtest");
}

// ========================================================================
// Integer Conversion Tests
// ========================================================================

#[test]
fn test_u64_to_i64_valid() {
    assert_eq!(u64_to_i64(0).unwrap(), 0);
    assert_eq!(u64_to_i64(1000).unwrap(), 1000);
    assert_eq!(u64_to_i64(i64::MAX as u64).unwrap(), i64::MAX);
}

#[test]
fn test_u64_to_i64_overflow() {
    let result = u64_to_i64(u64::MAX);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, StoreError::Internal(_)));
}

#[test]
fn test_i64_to_u64_positive() {
    assert_eq!(i64_to_u64(0), 0);
    assert_eq!(i64_to_u64(1000), 1000);
    assert_eq!(i64_to_u64(i64::MAX), i64::MAX as u64);
}

#[test]
fn test_i64_to_u64_negative_clamped() {
    // Negative values should clamp to 0
    assert_eq!(i64_to_u64(-1), 0);
    assert_eq!(i64_to_u64(-1000), 0);
    assert_eq!(i64_to_u64(i64::MIN), 0);
}

#[test]
fn test_i64_to_usize_positive() {
    assert_eq!(i64_to_usize(0), 0);
    assert_eq!(i64_to_usize(100), 100);
}

#[test]
fn test_i64_to_usize_negative_clamped() {
    assert_eq!(i64_to_usize(-1), 0);
    assert_eq!(i64_to_usize(-100), 0);
}

// ========================================================================
// Boundary Tests
// ========================================================================

#[test]
fn test_i64_max_boundary() {
    let max: u64 = i64::MAX as u64;
    assert_eq!(u64_to_i64(max).unwrap(), i64::MAX);

    // One over should fail
    assert!(u64_to_i64(max + 1).is_err());
}

#[test]
fn test_config_chain() {
    let config = PostgresConfig::new("postgresql://chain")
        .max_connections(5)
        .min_connections(2)
        .max_connections(10); // Override

    assert_eq!(config.max_connections, 10);
    assert_eq!(config.min_connections, 2);
}
