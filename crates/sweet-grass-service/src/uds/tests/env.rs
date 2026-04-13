// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Env-reading wrapper coverage: `resolve_family_id_from_env`,
//! `validate_insecure_guard`, `resolve_socket_path`, `cleanup_socket`.

use std::path::PathBuf;

use sweet_grass_core::primal_names::env_vars;

use super::super::*;

#[test]
fn resolve_family_id_sweetgrass_override() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("sweet-fam")),
            (env_vars::BIOMEOS_FAMILY_ID, Some("biome-fam")),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), Some("sweet-fam".to_string()));
        },
    );
}

#[test]
fn resolve_family_id_biomeos_fallback() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, Some("biome-fam")),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), Some("biome-fam".to_string()));
        },
    );
}

#[test]
fn resolve_family_id_generic_fallback() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(
                resolve_family_id_from_env(),
                Some("generic-fam".to_string())
            );
        },
    );
}

#[test]
fn resolve_family_id_none_when_all_absent() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn resolve_family_id_filters_empty_string() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("")),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn resolve_family_id_filters_default_string() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("default")),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn validate_insecure_guard_env_passes_when_no_family() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_INSECURE, Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn validate_insecure_guard_env_fails_when_family_and_insecure() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("test-family")),
            (env_vars::BIOMEOS_INSECURE, Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_err());
        },
    );
}

#[test]
fn validate_insecure_guard_env_passes_when_family_no_insecure() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("test-family")),
            (env_vars::BIOMEOS_INSECURE, None::<&str>),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn resolve_socket_path_env_reads_biomeos_dir() {
    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", None::<&str>),
            (env_vars::BIOMEOS_SOCKET_DIR, Some("/run/biomeos-env")),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            let path = resolve_socket_path(Some("myprimal"));
            assert_eq!(path, PathBuf::from("/run/biomeos-env/myprimal.sock"));
        },
    );
}

#[test]
fn cleanup_socket_resolves_and_cleans() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create");

    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", Some(sock_path.to_str().unwrap())),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            cleanup_socket();
            assert!(!sock_path.exists());
        },
    );
}
