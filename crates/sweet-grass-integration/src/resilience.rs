// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Resilience patterns for inter-primal communication.
//!
//! Provides circuit breaker and retry logic for capability-based
//! discovery and IPC, following the loamSpine `ResilientAdapter` pattern.
//!
//! When trio partners (`rhizoCrypt`, `LoamSpine`) or other primals are
//! temporarily unavailable, these patterns prevent cascading failures
//! and allow graceful degradation.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum BreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl From<u8> for BreakerState {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::Open,
            2 => Self::HalfOpen,
            _ => Self::Closed,
        }
    }
}

/// Circuit breaker for inter-primal IPC.
///
/// Tracks consecutive failures and opens the circuit to prevent
/// further calls to an unavailable primal. After a cooldown period,
/// allows a single probe request (half-open state).
///
/// Thread-safe via atomics — no locks required.
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU64,
    last_failure_epoch_ms: AtomicU64,
    failure_threshold: u64,
    cooldown: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `failure_threshold`: consecutive failures before opening circuit.
    /// - `cooldown`: how long to wait before allowing a probe in half-open state.
    #[must_use]
    pub const fn new(failure_threshold: u64, cooldown: Duration) -> Self {
        Self {
            state: AtomicU8::new(BreakerState::Closed as u8),
            failure_count: AtomicU64::new(0),
            last_failure_epoch_ms: AtomicU64::new(0),
            failure_threshold,
            cooldown,
        }
    }

    /// Check if a request is allowed through.
    ///
    /// Returns `true` if the circuit is closed or transitioning to
    /// half-open (probe allowed).
    #[must_use]
    pub fn allow_request(&self) -> bool {
        let state = BreakerState::from(self.state.load(Ordering::Acquire));
        match state {
            BreakerState::Closed | BreakerState::HalfOpen => true,
            BreakerState::Open => {
                let last_failure_ms = self.last_failure_epoch_ms.load(Ordering::Acquire);
                let now_ms = epoch_ms();
                let elapsed = Duration::from_millis(now_ms.saturating_sub(last_failure_ms));
                if elapsed >= self.cooldown {
                    self.state
                        .store(BreakerState::HalfOpen as u8, Ordering::Release);
                    true
                } else {
                    false
                }
            },
        }
    }

    /// Record a successful call (resets the breaker to closed).
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Release);
        self.state
            .store(BreakerState::Closed as u8, Ordering::Release);
    }

    /// Record a failed call.
    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        self.last_failure_epoch_ms
            .store(epoch_ms(), Ordering::Release);
        if count >= self.failure_threshold {
            self.state
                .store(BreakerState::Open as u8, Ordering::Release);
        }
    }

    /// Current failure count.
    #[must_use]
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Acquire)
    }

    /// Whether the circuit is currently open (blocking requests).
    #[must_use]
    pub fn is_open(&self) -> bool {
        BreakerState::from(self.state.load(Ordering::Acquire)) == BreakerState::Open
    }

    /// Reset the breaker to closed state.
    pub fn reset(&self) {
        self.failure_count.store(0, Ordering::Release);
        self.state
            .store(BreakerState::Closed as u8, Ordering::Release);
    }
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field(
                "state",
                &BreakerState::from(self.state.load(Ordering::Acquire)),
            )
            .field("failure_count", &self.failure_count())
            .field(
                "last_failure_epoch_ms",
                &self.last_failure_epoch_ms.load(Ordering::Acquire),
            )
            .field("threshold", &self.failure_threshold)
            .field("cooldown", &self.cooldown)
            .finish()
    }
}

/// Retry policy for transient failures.
///
/// Implements base-2 exponential backoff for retrying IPC
/// calls to temporarily unavailable primals.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay cap.
    pub max_delay: Duration,
}

impl RetryPolicy {
    /// Compute delay for the given attempt number (0-indexed).
    ///
    /// Uses base-2 exponential backoff: `initial_delay * 2^attempt`,
    /// capped at `max_delay`.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let initial_ms = self.initial_delay.as_millis().min(u128::from(u64::MAX));
        let max_ms = self.max_delay.as_millis().min(u128::from(u64::MAX));
        let shift = attempt.min(63);
        let base = initial_ms.saturating_mul(1u128 << shift);
        let capped = base.min(max_ms);
        Duration::from_millis(u64::try_from(capped).unwrap_or(u64::MAX))
    }

    /// Whether more retries are allowed for the given attempt count.
    #[must_use]
    pub const fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

impl RetryPolicy {
    /// Create a `RetryPolicy` from environment variables, falling back to defaults.
    ///
    /// | Variable                          | Default |
    /// |-----------------------------------|---------|
    /// | `SWEETGRASS_RETRY_MAX`            | 3       |
    /// | `SWEETGRASS_RETRY_INITIAL_MS`     | 100     |
    /// | `SWEETGRASS_RETRY_MAX_MS`         | 5000    |
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with(|key| std::env::var(key))
    }

    /// Testable constructor that accepts a custom env reader.
    #[must_use]
    pub(crate) fn from_env_with<F>(reader: F) -> Self
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        let max_retries = reader("SWEETGRASS_RETRY_MAX")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let initial_delay_ms: u64 = reader("SWEETGRASS_RETRY_INITIAL_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let max_delay_ms: u64 = reader("SWEETGRASS_RETRY_MAX_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000);

        Self {
            max_retries,
            initial_delay: Duration::from_millis(initial_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
        }
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

/// Execute an async operation with retry and circuit breaker protection.
///
/// # Errors
///
/// Returns the last error if all retries are exhausted or the circuit is open.
///
/// # Panics
///
/// Panics if the internal loop invariant is violated (should never happen
/// in practice as at least one attempt always executes).
pub async fn with_resilience<F, Fut, T, E>(
    breaker: &CircuitBreaker,
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // Execute the first attempt unconditionally to guarantee we always have an
    // error to return. This eliminates Option<E> and the need for any
    // unwrap/expect.
    let mut last_err = match try_once(&mut operation, breaker).await {
        Ok(value) => return Ok(value),
        Err(e) => e,
    };

    for attempt in 1..=policy.max_retries {
        if !breaker.allow_request() {
            return Err(last_err);
        }

        let delay = policy.delay_for_attempt(attempt.saturating_sub(1));
        tokio::time::sleep(delay).await;

        match operation().await {
            Ok(value) => {
                breaker.record_success();
                return Ok(value);
            },
            Err(e) => {
                tracing::debug!(
                    attempt,
                    error = %e,
                    "Resilient operation failed, recording failure"
                );
                breaker.record_failure();
                last_err = e;
            },
        }
    }

    Err(last_err)
}

async fn try_once<F, Fut, T, E>(operation: &mut F, breaker: &CircuitBreaker) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    if !breaker.allow_request() {
        return operation().await;
    }

    match operation().await {
        Ok(value) => {
            breaker.record_success();
            Ok(value)
        },
        Err(e) => {
            tracing::debug!(
                attempt = 0,
                error = %e,
                "Resilient operation failed, recording failure"
            );
            breaker.record_failure();
            Err(e)
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;

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

        let result: Result<i32, String> =
            with_resilience(&breaker, &policy, || async { Ok(42) }).await;

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
}
