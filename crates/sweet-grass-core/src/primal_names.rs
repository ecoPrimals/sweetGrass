// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Centralized primal name constants for IPC identifiers.
//!
//! Single source of truth for external primal names used in socket paths,
//! capability discovery, and environment variable lookups. Follows the
//! groundSpring V106 / wetSpring V119 pattern.
//!
//! Production code MUST use capability-based discovery rather than
//! hardcoding connections to specific primals. These constants exist
//! for socket path construction and environment variable naming only.

/// Socket and env var names for primals sweetGrass may interact with.
///
/// These are used exclusively for:
/// - Constructing socket paths during discovery fallback
/// - Environment variable naming (`{NAME}_SOCKET`, `{NAME}_ADDRESS`)
/// - Logging and diagnostics
///
/// **Never** use these to hardcode a connection target. Use
/// `capability.list` / `capability.call` for runtime routing.
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

/// Environment variable names for discovering primal sockets.
pub mod env_vars {
    /// Override socket path for `rhizoCrypt`.
    pub const RHIZOCRYPT_SOCKET: &str = "RHIZOCRYPT_SOCKET";
    /// Override socket path for `LoamSpine`.
    pub const LOAMSPINE_SOCKET: &str = "LOAMSPINE_SOCKET";
    /// Override socket path for `BearDog` (crypto).
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";
    /// Override socket path for `NestGate` (storage).
    pub const NESTGATE_SOCKET: &str = "NESTGATE_SOCKET";
    /// Override socket path for `Songbird` (discovery).
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";
    /// `biomeOS` socket directory.
    pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
    /// `biomeOS` family ID for multi-instance.
    pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
    /// XDG runtime directory.
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
    fn env_vars_are_uppercase() {
        let all = [
            env_vars::RHIZOCRYPT_SOCKET,
            env_vars::LOAMSPINE_SOCKET,
            env_vars::BEARDOG_SOCKET,
            env_vars::NESTGATE_SOCKET,
            env_vars::SONGBIRD_SOCKET,
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
