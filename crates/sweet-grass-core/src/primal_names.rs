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

/// Ecosystem-wide filesystem constants.
pub mod paths {
    /// Directory name used by `biomeOS` under `$XDG_RUNTIME_DIR` and
    /// `$TMPDIR` for primal sockets. Not a primal reference — this is a
    /// filesystem namespace convention.
    pub const BIOMEOS_DIR: &str = "biomeos";

    /// Last-resort fallback socket directory when no env vars are set.
    ///
    /// Used by `NestGate` discovery, composition health probes, and UDS
    /// resolution when `BIOMEOS_SOCKET_DIR`, `XDG_RUNTIME_DIR`, and
    /// `TMPDIR` are all absent.
    pub const DEFAULT_SOCKET_DIR: &str = "/tmp/biomeos";
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
    /// Generic family ID (`BTSP_PROTOCOL_STANDARD` §Phase 1).
    pub const FAMILY_ID: &str = "FAMILY_ID";
    /// Primal-specific family ID override for sweetGrass.
    pub const SWEETGRASS_FAMILY_ID: &str = "SWEETGRASS_FAMILY_ID";
    /// Development-mode flag — skips BTSP handshake when no `FAMILY_ID` is set.
    ///
    /// Per `BTSP_PROTOCOL_STANDARD` §Security Model, setting `BIOMEOS_INSECURE=1`
    /// alongside a non-default `FAMILY_ID` is a configuration error: a primal
    /// MUST refuse to start.
    pub const BIOMEOS_INSECURE: &str = "BIOMEOS_INSECURE";

    /// Advertise address override for this primal's network identity.
    ///
    /// Takes precedence over system hostname when resolving the advertise host
    /// for tarpc / TCP listeners.
    pub const PRIMAL_ADVERTISE_ADDRESS: &str = "PRIMAL_ADVERTISE_ADDRESS";

    /// Capability-based storage provider socket override.
    ///
    /// Capability-domain env var: any primal offering storage (currently
    /// `NestGate`) can be targeted via this override.
    pub const STORAGE_PROVIDER_SOCKET: &str = "STORAGE_PROVIDER_SOCKET";

    /// Explicit `NestGate` socket path override.
    ///
    /// Per-primal override, derived from `socket_env_var("nestgate")`. Takes
    /// precedence over `STORAGE_PROVIDER_SOCKET` and filesystem discovery.
    pub const NESTGATE_SOCKET: &str = "NESTGATE_SOCKET";

    /// Family seed for BTSP handshake key derivation.
    ///
    /// Set by `primalSpring` guidestone / harness as a hex-encoded 32-byte
    /// seed. Read by the BTSP relay to forward to the crypto provider.
    pub const FAMILY_SEED: &str = "FAMILY_SEED";

    /// Alias for [`FAMILY_SEED`] — some deployments set the seed under
    /// this name when `BearDog` is the explicit crypto provider.
    pub const BEARDOG_FAMILY_SEED: &str = "BEARDOG_FAMILY_SEED";
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
            env_vars::FAMILY_ID,
            env_vars::SWEETGRASS_FAMILY_ID,
            env_vars::BIOMEOS_INSECURE,
            env_vars::PRIMAL_ADVERTISE_ADDRESS,
            env_vars::STORAGE_PROVIDER_SOCKET,
            env_vars::NESTGATE_SOCKET,
            env_vars::FAMILY_SEED,
            env_vars::BEARDOG_FAMILY_SEED,
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
