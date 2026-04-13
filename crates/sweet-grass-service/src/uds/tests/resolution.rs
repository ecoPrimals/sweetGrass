// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! DI-based socket resolution tests.

use std::path::PathBuf;

use super::super::*;

#[test]
fn di_explicit_socket_override() {
    let config = SocketConfig {
        explicit_socket: Some("/custom/path.sock".to_string()),
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/custom/path.sock")
    );
}

#[test]
fn di_biomeos_dir() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/sweetgrass.sock")
    );
}

#[test]
fn di_biomeos_dir_with_family() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        family_id: Some("alpha".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/sweetgrass-alpha.sock")
    );
}

#[test]
fn di_xdg_runtime() {
    let config = SocketConfig {
        xdg_runtime_dir: Some("/run/user/1000".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/user/1000/biomeos/sweetgrass.sock")
    );
}

#[test]
fn di_user_fallback() {
    let config = SocketConfig {
        user: Some("testuser".to_string()),
        ..Default::default()
    };
    let expected = std::env::temp_dir()
        .join("biomeos-testuser")
        .join("sweetgrass.sock");
    assert_eq!(resolve_socket_path_with(&config), expected);
}

#[test]
fn di_temp_fallback() {
    let config = SocketConfig::default();
    let path = resolve_socket_path_with(&config);
    assert!(path.to_string_lossy().contains("sweetgrass.sock"));
}

#[test]
fn di_custom_primal_name() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        primal_name: Some("mySweetGrass".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/mySweetGrass.sock")
    );
}

#[test]
fn di_family_id_in_temp_fallback() {
    let config = SocketConfig {
        family_id: Some("beta".to_string()),
        ..Default::default()
    };
    let path = resolve_socket_path_with(&config);
    assert!(path.to_string_lossy().contains("sweetgrass-beta.sock"));
}

#[test]
fn di_priority_explicit_overrides_all() {
    let config = SocketConfig {
        explicit_socket: Some("/my/explicit.sock".to_string()),
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        xdg_runtime_dir: Some("/run/user/1000".to_string()),
        family_id: Some("gamma".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/my/explicit.sock"),
        "explicit socket should override all other resolution"
    );
}
