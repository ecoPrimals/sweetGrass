// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Primal name constants and generic env var helpers.
//!
//! Follows the capability-based discovery principle: production code
//! discovers primals at runtime via `capability.list` / `capability.call`.
//! These constants serve as a canonical registry for logging, diagnostics,
//! and environment variable naming.
//!
//! ## Generic socket env var pattern
//!
//! Instead of per-primal `RHIZOCRYPT_SOCKET`, `LOAMSPINE_SOCKET` constants,
//! use [`socket_env_var`] to derive `{NAME}_SOCKET` from any primal name
//! at runtime — no code change needed when new primals join the ecosystem.

/// Derive the `{NAME}_SOCKET` environment variable name for any primal.
///
/// This replaces per-primal constants (`RHIZOCRYPT_SOCKET`, etc.) with a
/// generic pattern that works for any primal discovered at runtime.
///
/// # Example
///
/// ```
/// use sweet_grass_core::primal_names::socket_env_var;
///
/// // Use with runtime-discovered primal names, not hardcoded constants:
/// let discovered_name = "rhizocrypt"; // from capabilities.list / mDNS
/// assert_eq!(socket_env_var(discovered_name), "RHIZOCRYPT_SOCKET");
/// assert_eq!(socket_env_var("newprimal"), "NEWPRIMAL_SOCKET");
/// ```
#[must_use]
pub fn socket_env_var(primal_name: &str) -> String {
    format!("{}_SOCKET", primal_name.to_uppercase())
}

/// Derive the `{NAME}_ADDRESS` environment variable name for any primal.
///
/// Used for TCP/tarpc address override lookup during discovery.
#[must_use]
pub fn address_env_var(primal_name: &str) -> String {
    format!("{}_ADDRESS", primal_name.to_uppercase())
}

/// Infrastructure environment variable names.
///
/// These are ecosystem-wide configuration, not primal-specific connections.
pub mod env_vars {
    /// `biomeOS` socket directory (where all primal sockets live).
    pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
    /// `biomeOS` family ID for multi-instance separation.
    pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
    /// XDG runtime directory (standard Linux convention).
    pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_env_var_pattern() {
        assert_eq!(socket_env_var("rhizocrypt"), "RHIZOCRYPT_SOCKET");
        assert_eq!(socket_env_var("loamspine"), "LOAMSPINE_SOCKET");
        assert_eq!(socket_env_var("beardog"), "BEARDOG_SOCKET");
        assert_eq!(socket_env_var("newprimal"), "NEWPRIMAL_SOCKET");
    }

    #[test]
    fn address_env_var_pattern() {
        assert_eq!(address_env_var("rhizocrypt"), "RHIZOCRYPT_ADDRESS");
        assert_eq!(address_env_var("newprimal"), "NEWPRIMAL_ADDRESS");
    }

    #[test]
    fn env_vars_are_uppercase() {
        let all = [
            env_vars::BIOMEOS_SOCKET_DIR,
            env_vars::BIOMEOS_FAMILY_ID,
            env_vars::XDG_RUNTIME_DIR,
        ];
        for var in &all {
            assert_eq!(
                *var,
                var.to_uppercase(),
                "env var should be uppercase: {var}"
            );
        }
    }
}
