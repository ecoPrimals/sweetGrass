// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Socket lifecycle management: PID files, capability symlinks, cleanup.
//!
//! Handles filesystem artifacts alongside the UDS socket:
//! - **PID file** (`sweetgrass.pid`) for instant `kill(pid, 0)` liveness checks
//! - **Capability symlink** (`provenance.sock -> sweetgrass.sock`) for Tier 3 discovery
//! - **Cleanup** on shutdown (socket + PID + symlink)

use std::path::PathBuf;

use tracing::{debug, info, warn};

/// Primary capability domain for filesystem-based discovery.
///
/// Per `CAPABILITY_BASED_DISCOVERY_STANDARD.md` Tier 3, primals SHOULD create
/// a symlink named after their capability domain alongside the primal-named
/// socket: `provenance.sock -> sweetgrass.sock`.
pub const CAPABILITY_DOMAIN: &str = "provenance";

/// Derive the PID file path from a socket path: `sweetgrass.sock` → `sweetgrass.pid`.
pub fn pid_path(socket_path: &std::path::Path) -> PathBuf {
    socket_path.with_extension("pid")
}

/// Write a PID file alongside the socket for instant liveness checks.
///
/// Consumers can `kill(pid, 0)` instead of `connect()` to determine whether
/// the socket has a listening process (50ms → 0ms per
/// `CAPABILITY_BASED_DISCOVERY_STANDARD.md` v1.3.0 §5).
pub fn write_pid_file(socket_path: &std::path::Path) {
    let pid = std::process::id();
    let path = pid_path(socket_path);
    match std::fs::write(&path, pid.to_string()) {
        Ok(()) => info!("PID file written: {} (pid {pid})", path.display()),
        Err(e) => warn!("Failed to write PID file {}: {e} (non-fatal)", path.display()),
    }
}

/// Remove the PID file alongside a socket on shutdown.
pub fn cleanup_pid_file(socket_path: &std::path::Path) {
    let path = pid_path(socket_path);
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            warn!("Failed to clean up PID file {}: {e}", path.display());
        } else {
            debug!("Cleaned up PID file {}", path.display());
        }
    }
}

/// Create a capability-domain symlink alongside the primal socket.
///
/// Per `CAPABILITY_BASED_DISCOVERY_STANDARD.md` v1.1, primals SHOULD create
/// a symlink named `{domain}.sock` pointing at the primal-named socket in
/// the same directory, enabling Tier 3 filesystem-based capability discovery
/// without Songbird or Neural API.
///
/// For sweetGrass the symlink is `provenance.sock -> sweetgrass.sock`.
pub fn create_capability_symlink(socket_path: &std::path::Path) {
    let Some(parent) = socket_path.parent() else {
        return;
    };
    let Some(socket_filename) = socket_path.file_name() else {
        return;
    };

    let symlink_path = parent.join(format!("{CAPABILITY_DOMAIN}.sock"));

    if (symlink_path.exists() || symlink_path.is_symlink())
        && let Err(e) = std::fs::remove_file(&symlink_path)
    {
        warn!(
            "Failed to remove stale capability symlink {}: {e}",
            symlink_path.display()
        );
        return;
    }

    if let Err(e) = std::os::unix::fs::symlink(socket_filename, &symlink_path) {
        warn!(
            "Failed to create capability symlink {} -> {}: {e}",
            symlink_path.display(),
            socket_filename.to_string_lossy(),
        );
    } else {
        info!(
            "Capability symlink: {} -> {}",
            symlink_path.display(),
            socket_filename.to_string_lossy(),
        );
    }
}

/// Remove the capability-domain symlink for a socket.
pub fn cleanup_capability_symlink(socket_path: &std::path::Path) {
    let Some(parent) = socket_path.parent() else {
        return;
    };
    let symlink_path = parent.join(format!("{CAPABILITY_DOMAIN}.sock"));
    if symlink_path.is_symlink() || symlink_path.exists() {
        if let Err(e) = std::fs::remove_file(&symlink_path) {
            warn!(
                "Failed to clean up capability symlink {}: {e}",
                symlink_path.display()
            );
        } else {
            debug!("Cleaned up capability symlink {}", symlink_path.display());
        }
    }
}

/// Remove a specific socket file, its capability symlink, and PID file.
pub fn cleanup_socket_at(path: &std::path::Path) {
    cleanup_pid_file(path);
    cleanup_capability_symlink(path);
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
            warn!("Failed to clean up UDS socket {}: {e}", path.display());
        } else {
            debug!("Cleaned up UDS socket {}", path.display());
        }
    }
}
