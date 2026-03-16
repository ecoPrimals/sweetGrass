// SPDX-License-Identifier: AGPL-3.0-only
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

/// Canonical primal names (lowercase, kebab-free).
///
/// Used for logging, diagnostics, and constructing env var names via
/// [`socket_env_var`]. **Never** use these to hardcode connection targets.
pub mod names {
    /// `rhizoCrypt` — ephemeral DAG engine (provenance trio partner).
    pub const RHIZOCRYPT: &str = "rhizocrypt";
    /// `LoamSpine` — permanence layer (provenance trio partner).
    pub const LOAMSPINE: &str = "loamspine";
    /// `BearDog` — cryptographic service provider.
    pub const BEARDOG: &str = "beardog";
    /// `NestGate` — storage and discovery.
    pub const NESTGATE: &str = "nestgate";
    /// `Songbird` — network orchestration and discovery.
    pub const SONGBIRD: &str = "songbird";
    /// `ToadStool` — universal compute platform.
    pub const TOADSTOOL: &str = "toadstool";
    /// `Squirrel` — AI coordination.
    pub const SQUIRREL: &str = "squirrel";
    /// `biomeOS` — orchestration layer.
    pub const BIOMEOS: &str = "biomeos";
}

/// Derive the `{NAME}_SOCKET` environment variable name for any primal.
///
/// This replaces per-primal constants (`RHIZOCRYPT_SOCKET`, etc.) with a
/// generic pattern that works for any primal discovered at runtime.
///
/// # Example
///
/// ```
/// use sweet_grass_core::primal_names::{socket_env_var, names};
///
/// assert_eq!(socket_env_var(names::RHIZOCRYPT), "RHIZOCRYPT_SOCKET");
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
    fn primal_names_are_lowercase() {
        let all = [
            names::RHIZOCRYPT,
            names::LOAMSPINE,
            names::BEARDOG,
            names::NESTGATE,
            names::SONGBIRD,
            names::TOADSTOOL,
            names::SQUIRREL,
            names::BIOMEOS,
        ];
        for name in &all {
            assert_eq!(
                *name,
                name.to_lowercase(),
                "primal name should be lowercase: {name}"
            );
        }
    }

    #[test]
    fn socket_env_var_pattern() {
        assert_eq!(socket_env_var(names::RHIZOCRYPT), "RHIZOCRYPT_SOCKET");
        assert_eq!(socket_env_var(names::LOAMSPINE), "LOAMSPINE_SOCKET");
        assert_eq!(socket_env_var(names::BEARDOG), "BEARDOG_SOCKET");
        assert_eq!(socket_env_var(names::NESTGATE), "NESTGATE_SOCKET");
        assert_eq!(socket_env_var(names::SONGBIRD), "SONGBIRD_SOCKET");
        assert_eq!(socket_env_var("newprimal"), "NEWPRIMAL_SOCKET");
    }

    #[test]
    fn address_env_var_pattern() {
        assert_eq!(address_env_var(names::RHIZOCRYPT), "RHIZOCRYPT_ADDRESS");
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

    #[test]
    fn no_duplicate_names() {
        let all = [
            names::RHIZOCRYPT,
            names::LOAMSPINE,
            names::BEARDOG,
            names::NESTGATE,
            names::SONGBIRD,
            names::TOADSTOOL,
            names::SQUIRREL,
            names::BIOMEOS,
        ];
        let mut seen = std::collections::HashSet::new();
        for name in &all {
            assert!(seen.insert(name), "duplicate primal name: {name}");
        }
    }
}
