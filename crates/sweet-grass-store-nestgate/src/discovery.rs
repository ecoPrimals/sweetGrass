// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `NestGate` socket discovery following the ecosystem standard.
//!
//! Resolution order:
//! 1. `NESTGATE_SOCKET` env var
//! 2. `STORAGE_PROVIDER_SOCKET` env var
//! 3. `{BIOMEOS_SOCKET_DIR}/nestgate.sock`
//! 4. `{XDG_RUNTIME_DIR}/biomeos/nestgate.sock`
//! 5. `/tmp/biomeos/nestgate.sock`

use std::path::PathBuf;

/// Socket filename for `NestGate`.
const NESTGATE_SOCK: &str = "nestgate.sock";

/// Default fallback socket directory.
const DEFAULT_SOCKET_DIR: &str = "/tmp/biomeos";

/// Discover the `NestGate` UDS socket path.
///
/// Uses the provided reader function for environment variable lookup
/// (DI-friendly, no direct `std::env::var` calls).
pub fn discover_socket(reader: &impl Fn(&str) -> Option<String>) -> PathBuf {
    if let Some(path) = reader("NESTGATE_SOCKET") {
        return PathBuf::from(path);
    }

    if let Some(path) = reader("STORAGE_PROVIDER_SOCKET") {
        return PathBuf::from(path);
    }

    if let Some(dir) = reader("BIOMEOS_SOCKET_DIR") {
        return PathBuf::from(dir).join(NESTGATE_SOCK);
    }

    if let Some(xdg) = reader("XDG_RUNTIME_DIR") {
        return PathBuf::from(xdg).join("biomeos").join(NESTGATE_SOCK);
    }

    PathBuf::from(DEFAULT_SOCKET_DIR).join(NESTGATE_SOCK)
}

/// Discover with family-scoped socket naming.
///
/// When `family_id` is provided, tries `nestgate-{family_id}.sock` first,
/// then falls back to the standard discovery chain.
pub fn discover_socket_with_family(
    reader: &impl Fn(&str) -> Option<String>,
    family_id: Option<&str>,
) -> PathBuf {
    if let Some(fid) = family_id
        && let Some(dir) = reader("BIOMEOS_SOCKET_DIR")
    {
        let family_sock = format!("nestgate-{fid}.sock");
        let path = PathBuf::from(&dir).join(&family_sock);
        if path.exists() {
            return path;
        }
    }

    discover_socket(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_explicit_nestgate_socket() {
        let path = discover_socket(&|key| match key {
            "NESTGATE_SOCKET" => Some("/custom/nestgate.sock".to_string()),
            _ => None,
        });
        assert_eq!(path.to_str(), Some("/custom/nestgate.sock"));
    }

    #[test]
    fn test_discover_storage_provider_socket() {
        let path = discover_socket(&|key| match key {
            "STORAGE_PROVIDER_SOCKET" => Some("/alt/storage.sock".to_string()),
            _ => None,
        });
        assert_eq!(path.to_str(), Some("/alt/storage.sock"));
    }

    #[test]
    fn test_discover_biomeos_socket_dir() {
        let path = discover_socket(&|key| match key {
            "BIOMEOS_SOCKET_DIR" => Some("/run/biomeos".to_string()),
            _ => None,
        });
        assert_eq!(path.to_str(), Some("/run/biomeos/nestgate.sock"));
    }

    #[test]
    fn test_discover_xdg_runtime() {
        let path = discover_socket(&|key| match key {
            "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
            _ => None,
        });
        assert_eq!(path.to_str(), Some("/run/user/1000/biomeos/nestgate.sock"));
    }

    #[test]
    fn test_discover_fallback() {
        let path = discover_socket(&|_| None);
        assert_eq!(path.to_str(), Some("/tmp/biomeos/nestgate.sock"));
    }

    #[test]
    fn test_discover_priority_order() {
        let path = discover_socket(&|key| match key {
            "NESTGATE_SOCKET" => Some("/first.sock".to_string()),
            "STORAGE_PROVIDER_SOCKET" => Some("/second.sock".to_string()),
            _ => None,
        });
        assert_eq!(
            path.to_str(),
            Some("/first.sock"),
            "NESTGATE_SOCKET should take priority"
        );
    }

    #[test]
    fn test_discover_with_family_falls_back() {
        let path = discover_socket_with_family(&|_| None, Some("test-family"));
        assert_eq!(path.to_str(), Some("/tmp/biomeos/nestgate.sock"));
    }
}
