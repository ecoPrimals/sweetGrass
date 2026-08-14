// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unix domain socket path resolution and BTSP guard validation.
//!
//! Resolution order per `UNIVERSAL_IPC_STANDARD_V3`:
//!
//! 1. `SWEETGRASS_SOCKET` — explicit override
//! 2. `BIOMEOS_SOCKET_DIR` + `/sweetgrass-{family_id}.sock`
//! 3. `$XDG_RUNTIME_DIR/biomeos/sweetgrass-{family_id}.sock`
//! 4. `$TMPDIR/biomeos-{user}/sweetgrass-{family_id}.sock`
//! 5. `$TMPDIR/biomeos/sweetgrass-{family_id}.sock`

use std::path::PathBuf;

use sweet_grass_core::identity;
use sweet_grass_core::primal_names::env_vars;

/// BTSP Phase 1 configuration error: `FAMILY_ID` and `BIOMEOS_INSECURE=1`
/// are mutually exclusive.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Security Model, a primal that claims family
/// membership MUST authenticate via BTSP handshake. Setting `BIOMEOS_INSECURE`
/// while a family is configured is contradictory and the primal MUST refuse
/// to start.
#[derive(Debug, thiserror::Error)]
#[error(
    "BTSP guard violation: FAMILY_ID=\"{family_id}\" and BIOMEOS_INSECURE=1 \
     are mutually exclusive — cannot claim a family and skip authentication \
     (BTSP_PROTOCOL_STANDARD §Phase 1)"
)]
pub struct BtspGuardViolation {
    family_id: String,
}

/// Default primal name when `SelfKnowledge` is unavailable.
const DEFAULT_PRIMAL_NAME: &str = identity::PRIMAL_NAME;

/// Injected socket resolution configuration.
///
/// Follows the airSpring / biomeOS `_with_config` DI pattern so tests
/// can resolve socket paths without mutating process environment.
#[derive(Debug, Clone, Default)]
pub struct SocketConfig {
    /// Explicit socket path override (like `SWEETGRASS_SOCKET` env var).
    pub explicit_socket: Option<String>,
    /// biomeOS socket directory (like `BIOMEOS_SOCKET_DIR` env var).
    pub biomeos_socket_dir: Option<String>,
    /// biomeOS family ID (like `BIOMEOS_FAMILY_ID` env var).
    pub family_id: Option<String>,
    /// XDG runtime directory (like `XDG_RUNTIME_DIR` env var).
    pub xdg_runtime_dir: Option<String>,
    /// System user (like `USER` env var).
    pub user: Option<String>,
    /// Override primal name (otherwise uses default).
    pub primal_name: Option<String>,
}

/// Resolve the effective `FAMILY_ID` from the environment.
///
/// Resolution order per `BTSP_PROTOCOL_STANDARD` §Phase 1:
/// 1. `SWEETGRASS_FAMILY_ID` (primal-specific override)
/// 2. `BIOMEOS_FAMILY_ID` (ecosystem-wide)
/// 3. `FAMILY_ID` (generic)
///
/// Empty strings and `"default"` are treated as absent.
#[must_use]
pub fn resolve_family_id_from_env() -> Option<String> {
    std::env::var(env_vars::SWEETGRASS_FAMILY_ID)
        .or_else(|_| std::env::var(env_vars::BIOMEOS_FAMILY_ID))
        .or_else(|_| std::env::var(env_vars::FAMILY_ID))
        .ok()
        .filter(|s| !s.is_empty() && s != "default")
}

/// Validate the BTSP insecure guard by reading environment variables.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Security Model: if `FAMILY_ID` is set
/// (non-empty, not `"default"`) AND `BIOMEOS_INSECURE=1`, the primal MUST
/// refuse to start. Delegates to [`validate_insecure_guard_with`] for
/// DI-testable logic.
///
/// # Errors
///
/// Returns [`BtspGuardViolation`] when the conflicting configuration is
/// detected.
pub fn validate_insecure_guard() -> Result<(), BtspGuardViolation> {
    let family_id = resolve_family_id_from_env();
    let insecure = std::env::var(env_vars::BIOMEOS_INSECURE).is_ok_and(|v| v == "1");

    validate_insecure_guard_with(family_id.as_deref(), insecure)
}

/// DI-friendly BTSP insecure guard validation (no env var reads).
///
/// # Errors
///
/// Returns [`BtspGuardViolation`] when `family_id` is `Some` and
/// `biomeos_insecure` is `true`.
pub fn validate_insecure_guard_with(
    family_id: Option<&str>,
    biomeos_insecure: bool,
) -> Result<(), BtspGuardViolation> {
    if let Some(fid) = family_id
        && biomeos_insecure
    {
        return Err(BtspGuardViolation {
            family_id: fid.to_owned(),
        });
    }
    Ok(())
}

/// Resolve the Unix domain socket path using XDG-compliant resolution.
///
/// The primal name is derived from `SelfKnowledge` when available (e.g. via
/// `state.self_knowledge`). When `primal_name` is `None`, falls back to
/// `PRIMAL_NAME` env var or `"sweetgrass"`.
///
/// Family ID resolution follows the BTSP standard chain:
/// `SWEETGRASS_FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `FAMILY_ID`.
///
/// Delegates to [`resolve_socket_path_with`] after reading env vars.
#[must_use]
pub fn resolve_socket_path(primal_name: Option<&str>) -> PathBuf {
    let config = SocketConfig {
        explicit_socket: std::env::var(env_vars::SWEETGRASS_SOCKET).ok(),
        biomeos_socket_dir: std::env::var(env_vars::BIOMEOS_SOCKET_DIR).ok(),
        family_id: resolve_family_id_from_env(),
        xdg_runtime_dir: std::env::var(env_vars::XDG_RUNTIME_DIR).ok(),
        user: std::env::var(env_vars::USER).ok(),
        primal_name: primal_name
            .map(String::from)
            .or_else(|| std::env::var(env_vars::PRIMAL_NAME).ok()),
    };
    resolve_socket_path_with(&config)
}

/// Resolve socket path with injected configuration (no env var reads).
///
/// DI-friendly variant for tests and embedded contexts. Follows the
/// airSpring `_with` pattern adopted ecosystem-wide per biomeOS V239.
#[must_use]
pub fn resolve_socket_path_with(config: &SocketConfig) -> PathBuf {
    let name = config.primal_name.as_deref().unwrap_or(DEFAULT_PRIMAL_NAME);
    let family_id = config.family_id.as_deref().unwrap_or("");
    let sock_name = if family_id.is_empty() {
        format!("{name}.sock")
    } else {
        format!("{name}-{family_id}.sock")
    };

    if let Some(ref path) = config.explicit_socket {
        return PathBuf::from(path);
    }

    if let Some(ref dir) = config.biomeos_socket_dir {
        return PathBuf::from(dir).join(&sock_name);
    }

    if let Some(ref xdg) = config.xdg_runtime_dir {
        return PathBuf::from(xdg)
            .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
            .join(&sock_name);
    }

    if let Some(ref user) = config.user {
        return std::env::temp_dir()
            .join(format!("biomeos-{user}"))
            .join(&sock_name);
    }

    std::env::temp_dir()
        .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
        .join(&sock_name)
}
