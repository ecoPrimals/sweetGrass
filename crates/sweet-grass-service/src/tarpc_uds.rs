// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! tarpc UDS transport — dual-socket pattern (G64 Cephalization C2).
//!
//! **NOTE:** G65 protocol negotiation (single-socket) supersedes this as the
//! canonical tarpc entry point. This module remains for backward compatibility
//! with clients that connect directly to `.tarpc.sock`.
//!
//! Provides intra-gate sub-ms binary RPC over Unix domain sockets.
//! JSON-RPC stays on `sweetgrass.sock`; tarpc serves on `sweetgrass.tarpc.sock`.
//!
//! ## Socket Resolution Order
//!
//! 1. `SWEETGRASS_TARPC_SOCKET` — explicit override
//! 2. `{BIOMEOS_SOCKET_DIR}/sweetgrass-{family_id}.tarpc.sock` — family-scoped
//! 3. `{BIOMEOS_SOCKET_DIR}/sweetgrass.tarpc.sock` — standalone

use std::path::{Path, PathBuf};

use futures::prelude::*;
use tarpc::server::{BaseChannel, Channel};
use tarpc::tokio_serde::formats::Bincode;
use tracing::{debug, info, warn};

use sweet_grass_core::primal_names::env_vars;

use crate::rpc::SweetGrassRpc;
use crate::server::SweetGrassServer;

/// Environment variable for explicit tarpc socket path override.
const TARPC_SOCKET_ENV: &str = "SWEETGRASS_TARPC_SOCKET";

/// Suffix appended to the primal socket name for the tarpc channel.
const TARPC_SOCKET_SUFFIX: &str = ".tarpc.sock";

/// Resolve the tarpc UDS socket path.
///
/// Resolution follows the same tier pattern as the JSON-RPC socket but with
/// the `.tarpc.sock` suffix instead of `.sock`.
#[must_use]
pub fn resolve_tarpc_socket_path(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }

    if let Ok(p) = std::env::var(TARPC_SOCKET_ENV) {
        return PathBuf::from(p);
    }

    let socket_dir = resolve_socket_dir();
    let primal_name = sweet_grass_core::identity::PRIMAL_NAME;

    if let Ok(family_id) = std::env::var(env_vars::FAMILY_ID) {
        return socket_dir.join(format!("{primal_name}-{family_id}{TARPC_SOCKET_SUFFIX}"));
    }

    socket_dir.join(format!("{primal_name}{TARPC_SOCKET_SUFFIX}"))
}

/// Start a tarpc server listening on a Unix domain socket.
///
/// Uses `tarpc::serde_transport::unix::listen` for length-delimited binary
/// framing over UDS — the canonical G64 cephalization pattern.
///
/// # Errors
///
/// Returns an error if binding to the socket fails.
pub async fn start_tarpc_uds_server(
    server: SweetGrassServer,
    socket_path: &Path,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> std::result::Result<(), crate::ServiceError> {
    if socket_path.exists() {
        debug!(path = %socket_path.display(), "removing stale tarpc socket");
        let _ = std::fs::remove_file(socket_path);
    }

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            crate::ServiceError::Internal(format!("tarpc UDS mkdir {}: {e}", parent.display()))
        })?;
    }

    let mut listener = tarpc::serde_transport::unix::listen(socket_path, Bincode::default)
        .await
        .map_err(|e| {
            crate::ServiceError::Internal(format!("tarpc UDS bind {}: {e}", socket_path.display()))
        })?;

    info!(path = %socket_path.display(), "tarpc UDS server listening");

    loop {
        tokio::select! {
            result = listener.next() => {
                match result {
                    Some(Ok(transport)) => {
                        let server = server.clone();
                        tokio::spawn(async move {
                            let channel = BaseChannel::with_defaults(transport);
                            let () = channel.execute(server.serve()).for_each(|f| f).await;
                        });
                    },
                    Some(Err(e)) => {
                        warn!("tarpc UDS accept error: {e}");
                    },
                    None => break,
                }
            }
            _ = shutdown.changed() => {
                info!("tarpc UDS server shutting down");
                break;
            }
        }
    }

    Ok(())
}

/// Clean up the tarpc socket file.
pub fn cleanup_tarpc_socket(path: &Path) {
    if path.exists() {
        debug!(path = %path.display(), "cleaning up tarpc socket");
        let _ = std::fs::remove_file(path);
    }
}

/// Resolve the `biomeOS` socket directory (shared with JSON-RPC UDS resolver).
fn resolve_socket_dir() -> PathBuf {
    use sweet_grass_core::primal_names::{env_vars as ev, paths};
    if let Ok(dir) = std::env::var(ev::BIOMEOS_SOCKET_DIR) {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var(ev::XDG_RUNTIME_DIR) {
        return PathBuf::from(xdg).join(paths::BIOMEOS_DIR);
    }
    if let Ok(tmpdir) = std::env::var(ev::TMPDIR) {
        return PathBuf::from(tmpdir).join(paths::BIOMEOS_DIR);
    }
    paths::default_socket_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_tarpc_socket_explicit_override() {
        let path = PathBuf::from("/tmp/test/custom.tarpc.sock");
        let resolved = resolve_tarpc_socket_path(Some(&path));
        assert_eq!(resolved, path);
    }

    #[test]
    fn resolve_tarpc_socket_contains_tarpc_suffix() {
        temp_env::with_vars(
            [
                (TARPC_SOCKET_ENV, None::<&str>),
                (env_vars::FAMILY_ID, None::<&str>),
                (env_vars::BIOMEOS_SOCKET_DIR, Some("/run/biomeos")),
            ],
            || {
                let resolved = resolve_tarpc_socket_path(None);
                assert!(
                    resolved.to_string_lossy().ends_with(".tarpc.sock"),
                    "expected .tarpc.sock suffix, got: {resolved:?}"
                );
                assert!(
                    resolved.to_string_lossy().contains("sweetgrass"),
                    "expected primal name in path, got: {resolved:?}"
                );
            },
        );
    }

    #[test]
    fn resolve_tarpc_socket_from_env() {
        temp_env::with_vars(
            [(TARPC_SOCKET_ENV, Some("/custom/path.tarpc.sock"))],
            || {
                let resolved = resolve_tarpc_socket_path(None);
                assert_eq!(resolved, PathBuf::from("/custom/path.tarpc.sock"));
            },
        );
    }

    #[test]
    fn resolve_tarpc_socket_family_scoped() {
        temp_env::with_vars(
            [
                (TARPC_SOCKET_ENV, None::<&str>),
                (env_vars::FAMILY_ID, Some("test-family")),
                (env_vars::BIOMEOS_SOCKET_DIR, Some("/run/biomeos")),
            ],
            || {
                let resolved = resolve_tarpc_socket_path(None);
                assert_eq!(
                    resolved,
                    PathBuf::from("/run/biomeos/sweetgrass-test-family.tarpc.sock")
                );
            },
        );
    }
}
