// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Zero-panic exit helpers for `UniBin` validation.
//!
//! The `OrExit` trait replaces `unwrap()`/`expect()` in binary entrypoints
//! with structured logging and clean exit codes. Aligned with the biomeOS
//! `OrExit` pattern and wateringHole `UNIBIN_ARCHITECTURE_STANDARD`.

use std::fmt::Display;

/// Exit codes per wateringHole `UNIBIN_ARCHITECTURE_STANDARD`.
pub mod exit_code {
    /// Normal termination.
    pub const SUCCESS: i32 = 0;
    /// Unclassified error.
    pub const GENERAL_ERROR: i32 = 1;
    /// Invalid configuration, bad CLI args, missing env var.
    pub const CONFIG_ERROR: i32 = 2;
    /// Network bind/connect failure.
    pub const NETWORK_ERROR: i32 = 3;
}

/// Extension trait for fallible values in binary entrypoints.
///
/// Replaces `unwrap()`/`expect()` with structured exit: logs the error
/// via `tracing::error!` and returns the exit code instead of panicking.
///
/// # Errors
///
/// Returns the given exit code when the inner value represents failure.
///
/// # Examples
///
/// ```rust,ignore
/// let addr: SocketAddr = "0.0.0.0:3000"
///     .parse()
///     .or_exit(exit_code::CONFIG_ERROR, "invalid HTTP address");
/// ```
pub trait OrExit<T> {
    /// Unwrap the value or log and return the given exit code.
    ///
    /// # Errors
    ///
    /// Returns `Err(code)` when the value is an error or absent.
    fn or_exit(self, code: i32, context: &str) -> Result<T, i32>;
}

impl<T, E: Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, code: i32, context: &str) -> Result<T, i32> {
        self.map_err(|e| {
            tracing::error!("{context}: {e}");
            code
        })
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, code: i32, context: &str) -> Result<T, i32> {
        self.ok_or_else(|| {
            tracing::error!("{context}");
            code
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_ok_passes_through() {
        let val: Result<i32, String> = Ok(42);
        assert_eq!(val.or_exit(1, "should not fail"), Ok(42));
    }

    #[test]
    fn result_err_returns_exit_code() {
        let val: Result<i32, String> = Err("boom".into());
        assert_eq!(val.or_exit(2, "config"), Err(2));
    }

    #[test]
    fn option_some_passes_through() {
        let val: Option<i32> = Some(42);
        assert_eq!(val.or_exit(1, "missing"), Ok(42));
    }

    #[test]
    fn option_none_returns_exit_code() {
        let val: Option<i32> = None;
        assert_eq!(val.or_exit(3, "not found"), Err(3));
    }
}
