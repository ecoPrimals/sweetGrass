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

    /// Resolve the default socket directory (respects `$TMPDIR` when present).
    #[must_use]
    pub fn default_socket_dir() -> std::path::PathBuf {
        std::env::var(super::env_vars::TMPDIR).map_or_else(
            |_| std::path::PathBuf::from(DEFAULT_SOCKET_DIR),
            |tmpdir| std::path::PathBuf::from(tmpdir).join(BIOMEOS_DIR),
        )
    }
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

    /// Capability-based security provider socket override.
    ///
    /// Points at whichever primal provides the `crypto.*` capability
    /// domain (currently `BearDog`).  Used by the BTSP relay in
    /// `btsp/server.rs` for handshake delegation.
    pub const SECURITY_PROVIDER_SOCKET: &str = "SECURITY_PROVIDER_SOCKET";

    /// Explicit sweetGrass UDS socket path override.
    ///
    /// When set, bypasses all socket discovery logic in `uds.rs`.
    pub const SWEETGRASS_SOCKET: &str = "SWEETGRASS_SOCKET";

    /// Override for this primal's advertised name in socket filenames.
    ///
    /// Falls back to `identity::PRIMAL_NAME` (`"sweetgrass"`) when absent.
    pub const PRIMAL_NAME: &str = "PRIMAL_NAME";

    /// POSIX temporary directory override.
    ///
    /// Standard POSIX variable; used in socket directory resolution
    /// as a fallback before `DEFAULT_SOCKET_DIR`.
    pub const TMPDIR: &str = "TMPDIR";

    /// `DATABASE_URL` — Postgres connection string.
    ///
    /// Used by `sweet-grass-store-postgres` for database connectivity.
    pub const DATABASE_URL: &str = "DATABASE_URL";

    /// `TARPC_MAX_CONCURRENT_REQUESTS` — tarpc server concurrency limit.
    ///
    /// Configures max in-flight requests for the tarpc server.
    pub const TARPC_MAX_CONCURRENT_REQUESTS: &str = "TARPC_MAX_CONCURRENT_REQUESTS";

    /// `BEARDOG_SOCKET` — path to the `BearDog` Tower crypto provider socket.
    ///
    /// Per `NUCLEUS_TWO_TIER_CRYPTO_MODEL`, all primals delegate signing and
    /// encryption to `BearDog` through this socket.
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// `DISCOVERY_SOCKET` — path to the Songbird discovery service socket.
    ///
    /// Used for capability-based resolution (e.g. resolve `"crypto"` to
    /// `BearDog`'s socket path at runtime).
    pub const DISCOVERY_SOCKET: &str = "DISCOVERY_SOCKET";

    /// `DISCOVERY_ADDRESS` — TCP address for Songbird discovery bootstrap.
    ///
    /// Primary env var for locating the discovery service when no UDS socket
    /// is available (e.g. cross-gate federation).
    pub const DISCOVERY_ADDRESS: &str = "DISCOVERY_ADDRESS";

    /// `UNIVERSAL_ADAPTER_ADDRESS` — legacy/fallback name for the discovery
    /// bootstrap address. Checked after `DISCOVERY_ADDRESS`.
    pub const UNIVERSAL_ADAPTER_ADDRESS: &str = "UNIVERSAL_ADAPTER_ADDRESS";

    /// `DISCOVERY_BOOTSTRAP` — tertiary fallback for discovery service address.
    pub const DISCOVERY_BOOTSTRAP: &str = "DISCOVERY_BOOTSTRAP";

    /// POSIX login name for the current user.
    ///
    /// Used in UDS socket path resolution when `$TMPDIR/biomeos-{user}/` is
    /// the fallback directory.
    pub const USER: &str = "USER";

    /// TCP listen port override for sweetGrass JSON-RPC.
    ///
    /// When set, `capabilities.list` advertises `"tcp"` as an available
    /// transport alongside HTTP and UDS.
    pub const SWEETGRASS_PORT: &str = "SWEETGRASS_PORT";

    /// JSON-RPC method gate enforcement mode.
    ///
    /// Values: `enforced` / `enforce` / `strict` enable token checks;
    /// unset or any other value defaults to permissive mode.
    pub const SWEETGRASS_AUTH_MODE: &str = "SWEETGRASS_AUTH_MODE";

    /// Unique instance identifier for this primal process.
    pub const PRIMAL_INSTANCE_ID: &str = "PRIMAL_INSTANCE_ID";

    /// tarpc listen port override.
    pub const TARPC_PORT: &str = "TARPC_PORT";

    /// REST/HTTP listen port override.
    pub const REST_PORT: &str = "REST_PORT";

    /// Comma-separated list of capability strings for self-knowledge.
    pub const PRIMAL_CAPABILITIES: &str = "PRIMAL_CAPABILITIES";

    /// Path to a sweetGrass TOML configuration file.
    pub const SWEETGRASS_CONFIG: &str = "SWEETGRASS_CONFIG";

    /// XDG base directory for user configuration.
    pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";

    /// Override primal name in configuration.
    pub const SWEETGRASS_NAME: &str = "SWEETGRASS_NAME";

    /// Override tarpc listen address in configuration.
    pub const SWEETGRASS_TARPC_LISTEN: &str = "SWEETGRASS_TARPC_LISTEN";

    /// Override REST listen address in configuration.
    pub const SWEETGRASS_REST_LISTEN: &str = "SWEETGRASS_REST_LISTEN";

    /// Discovery bootstrap address override in configuration.
    pub const SWEETGRASS_DISCOVERY_BOOTSTRAP: &str = "SWEETGRASS_DISCOVERY_BOOTSTRAP";

    /// ecoPrimals JSON-LD vocabulary namespace URI override.
    pub const ECOP_VOCAB_URI: &str = "ECOP_VOCAB_URI";

    /// ecoPrimals JSON-LD base URI override.
    pub const ECOP_BASE_URI: &str = "ECOP_BASE_URI";

    /// Explicit path to the biomeOS neural-api socket.
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

    /// ecoPrimals family identifier (used for socket name resolution).
    pub const ECOPRIMALS_FAMILY_ID: &str = "ECOPRIMALS_FAMILY_ID";

    /// Redb storage directory path override.
    pub const STORAGE_PATH: &str = "STORAGE_PATH";

    /// Maximum retry attempts for inter-primal IPC.
    pub const SWEETGRASS_RETRY_MAX: &str = "SWEETGRASS_RETRY_MAX";

    /// Initial delay (ms) for exponential backoff retries.
    pub const SWEETGRASS_RETRY_INITIAL_MS: &str = "SWEETGRASS_RETRY_INITIAL_MS";

    /// Maximum delay (ms) cap for exponential backoff retries.
    pub const SWEETGRASS_RETRY_MAX_MS: &str = "SWEETGRASS_RETRY_MAX_MS";

    /// Override the default agent DID for sweetGrass attribution.
    pub const SWEETGRASS_AGENT_DID: &str = "SWEETGRASS_AGENT_DID";

    /// HTTP listen port for sweetGrass REST API.
    pub const SWEETGRASS_HTTP_PORT: &str = "SWEETGRASS_HTTP_PORT";

    /// HTTP listen address for sweetGrass REST API.
    pub const SWEETGRASS_HTTP_ADDRESS: &str = "SWEETGRASS_HTTP_ADDRESS";

    /// tarpc listen address override.
    pub const SWEETGRASS_TARPC_ADDRESS: &str = "SWEETGRASS_TARPC_ADDRESS";

    /// Storage backend selector (e.g. `memory`, `redb`, `postgres`).
    pub const STORAGE_BACKEND: &str = "STORAGE_BACKEND";
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
            env_vars::SECURITY_PROVIDER_SOCKET,
            env_vars::SWEETGRASS_SOCKET,
            env_vars::PRIMAL_NAME,
            env_vars::TMPDIR,
            env_vars::DATABASE_URL,
            env_vars::TARPC_MAX_CONCURRENT_REQUESTS,
            env_vars::BEARDOG_SOCKET,
            env_vars::DISCOVERY_SOCKET,
            env_vars::USER,
            env_vars::SWEETGRASS_PORT,
            env_vars::SWEETGRASS_AUTH_MODE,
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
