// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
#![expect(clippy::unwrap_used, reason = "test file: unwrap is standard in tests")]

use super::*;
use std::sync::atomic::Ordering;

#[test]
fn circuit_breaker_starts_closed() {
    let cb = CircuitBreaker::new(3, Duration::from_secs(5));
    assert!(cb.allow_request());
    assert!(!cb.is_open());
    assert_eq!(cb.failure_count(), 0);
}

#[test]
fn circuit_opens_after_threshold() {
    let cb = CircuitBreaker::new(3, Duration::from_secs(60));
    cb.record_failure();
    cb.record_failure();
    assert!(cb.allow_request());
    cb.record_failure();
    assert!(cb.is_open());
    assert!(!cb.allow_request());
}

#[test]
fn circuit_closes_on_success() {
    let cb = CircuitBreaker::new(2, Duration::from_secs(60));
    cb.record_failure();
    cb.record_failure();
    assert!(cb.is_open());
    cb.record_success();
    assert!(!cb.is_open());
    assert!(cb.allow_request());
    assert_eq!(cb.failure_count(), 0);
}

#[test]
fn circuit_breaker_half_open_after_cooldown() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(0));
    cb.record_failure();
    assert!(cb.is_open());
    // Cooldown is 0ms, so should immediately transition to half-open
    assert!(cb.allow_request());
}

#[test]
fn circuit_breaker_reset() {
    let cb = CircuitBreaker::new(1, Duration::from_secs(60));
    cb.record_failure();
    assert!(cb.is_open());
    cb.reset();
    assert!(!cb.is_open());
    assert_eq!(cb.failure_count(), 0);
}

#[test]
fn circuit_breaker_debug_impl() {
    let cb = CircuitBreaker::new(5, Duration::from_secs(30));
    let debug_str = format!("{cb:?}");
    assert!(debug_str.contains("CircuitBreaker"));
    assert!(debug_str.contains("threshold"));
}

#[test]
fn retry_policy_default() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert!(policy.should_retry(0));
    assert!(policy.should_retry(2));
    assert!(!policy.should_retry(3));
}

#[test]
fn retry_policy_exponential_backoff() {
    let policy = RetryPolicy {
        max_retries: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
    };

    let d0 = policy.delay_for_attempt(0);
    let d1 = policy.delay_for_attempt(1);
    let d2 = policy.delay_for_attempt(2);

    assert_eq!(d0, Duration::from_millis(100));
    assert_eq!(d1, Duration::from_millis(200));
    assert_eq!(d2, Duration::from_millis(400));
}

#[test]
fn retry_policy_caps_at_max_delay() {
    let policy = RetryPolicy {
        max_retries: 10,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
    };

    let d10 = policy.delay_for_attempt(10);
    assert!(d10 <= Duration::from_millis(500));
}

#[tokio::test]
async fn with_resilience_succeeds_first_try() {
    let breaker = CircuitBreaker::new(3, Duration::from_secs(5));
    let policy = RetryPolicy {
        max_retries: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
    };

    let result: Result<i32, String> = with_resilience(&breaker, &policy, || async { Ok(42) }).await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(breaker.failure_count(), 0);
}

#[tokio::test]
async fn with_resilience_retries_on_failure() {
    let breaker = CircuitBreaker::new(10, Duration::from_secs(60));
    let policy = RetryPolicy {
        max_retries: 2,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
    };

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let cc = std::sync::Arc::clone(&call_count);

    let result: Result<i32, String> = with_resilience(&breaker, &policy, || {
        let cc = std::sync::Arc::clone(&cc);
        async move {
            let count = cc.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(format!("transient error #{count}"))
            } else {
                Ok(42)
            }
        }
    })
    .await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn with_resilience_exhausts_retries() {
    let breaker = CircuitBreaker::new(10, Duration::from_secs(60));
    let policy = RetryPolicy {
        max_retries: 2,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
    };

    let result: Result<i32, String> = with_resilience(&breaker, &policy, || async {
        Err("always fails".to_string())
    })
    .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "always fails");
}

#[tokio::test]
async fn with_resilience_respects_open_circuit() {
    let breaker = CircuitBreaker::new(1, Duration::from_secs(60));
    let policy = RetryPolicy {
        max_retries: 5,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
    };

    // Pre-open the circuit
    breaker.record_failure();
    assert!(breaker.is_open());

    let result: Result<i32, String> = with_resilience(&breaker, &policy, || async {
        Err("should not execute many times".to_string())
    })
    .await;

    assert!(result.is_err());
}

#[test]
fn breaker_state_from_u8() {
    assert_eq!(BreakerState::from(0), BreakerState::Closed);
    assert_eq!(BreakerState::from(1), BreakerState::Open);
    assert_eq!(BreakerState::from(2), BreakerState::HalfOpen);
    assert_eq!(BreakerState::from(255), BreakerState::Closed);
}

#[test]
fn retry_policy_from_env_defaults() {
    let policy = RetryPolicy::from_env_with(|_| Err(std::env::VarError::NotPresent));
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.initial_delay, Duration::from_millis(100));
    assert_eq!(policy.max_delay, Duration::from_secs(5));
}

#[test]
fn retry_policy_from_env_custom() {
    let policy = RetryPolicy::from_env_with(|key| match key {
        "SWEETGRASS_RETRY_MAX" => Ok("5".to_string()),
        "SWEETGRASS_RETRY_INITIAL_MS" => Ok("200".to_string()),
        "SWEETGRASS_RETRY_MAX_MS" => Ok("10000".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert_eq!(policy.max_retries, 5);
    assert_eq!(policy.initial_delay, Duration::from_millis(200));
    assert_eq!(policy.max_delay, Duration::from_secs(10));
}

#[test]
fn retry_policy_from_env_partial_override() {
    let policy = RetryPolicy::from_env_with(|key| match key {
        "SWEETGRASS_RETRY_MAX" => Ok("7".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert_eq!(policy.max_retries, 7);
    assert_eq!(policy.initial_delay, Duration::from_millis(100));
    assert_eq!(policy.max_delay, Duration::from_secs(5));
}

#[test]
fn retry_policy_from_env_invalid_values() {
    let policy = RetryPolicy::from_env_with(|key| match key {
        "SWEETGRASS_RETRY_MAX" => Ok("not_a_number".to_string()),
        "SWEETGRASS_RETRY_INITIAL_MS" => Ok("-1".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.initial_delay, Duration::from_millis(100));
}
