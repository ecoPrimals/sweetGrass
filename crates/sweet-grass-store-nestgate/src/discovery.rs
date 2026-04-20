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

use sweet_grass_core::primal_names::{env_vars, paths};

/// Socket filename for `NestGate`.
const NESTGATE_SOCK: &str = "nestgate.sock";

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

    if let Some(dir) = reader(env_vars::BIOMEOS_SOCKET_DIR) {
        return PathBuf::from(dir).join(NESTGATE_SOCK);
    }

    if let Some(xdg) = reader(env_vars::XDG_RUNTIME_DIR) {
        return PathBuf::from(xdg)
            .join(paths::BIOMEOS_DIR)
            .join(NESTGATE_SOCK);
    }

    PathBuf::from(paths::DEFAULT_SOCKET_DIR).join(NESTGATE_SOCK)
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
        && let Some(dir) = reader(env_vars::BIOMEOS_SOCKET_DIR)
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
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
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

    #[test]
    fn test_discover_with_family_uses_family_socket_when_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let family_sock = dir.path().join("nestgate-my-family.sock");
        std::fs::write(&family_sock, b"").expect("create socket file");

        let dir_str = dir.path().to_string_lossy().to_string();
        let path = discover_socket_with_family(
            &|key| match key {
                "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
                _ => None,
            },
            Some("my-family"),
        );
        assert_eq!(path, family_sock);
    }

    #[test]
    fn test_discover_with_family_falls_back_when_socket_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_str = dir.path().to_string_lossy().to_string();
        let path = discover_socket_with_family(
            &|key| match key {
                "BIOMEOS_SOCKET_DIR" => Some(dir_str.clone()),
                _ => None,
            },
            Some("missing-family"),
        );
        assert_eq!(path, dir.path().join(NESTGATE_SOCK));
    }

    #[test]
    fn test_discover_with_family_none_uses_standard_chain() {
        let path = discover_socket_with_family(
            &|key| match key {
                "NESTGATE_SOCKET" => Some("/explicit/path.sock".to_string()),
                _ => None,
            },
            None,
        );
        assert_eq!(path.to_str(), Some("/explicit/path.sock"));
    }

    #[test]
    fn test_discover_with_family_no_biomeos_dir_ignores_family() {
        let path = discover_socket_with_family(
            &|key| match key {
                "XDG_RUNTIME_DIR" => Some("/run/user/1000".to_string()),
                _ => None,
            },
            Some("some-family"),
        );
        assert_eq!(path.to_str(), Some("/run/user/1000/biomeos/nestgate.sock"));
    }
}
