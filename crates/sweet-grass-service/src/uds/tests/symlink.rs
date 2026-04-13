// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Cleanup and capability symlink tests.

use super::super::*;

// ==================== Cleanup tests ====================

#[test]
fn test_cleanup_socket_when_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("cleanup-test.sock");
    std::fs::write(&sock_path, "").expect("create socket file");
    assert!(sock_path.exists());
    cleanup_socket_at(&sock_path);
    assert!(!sock_path.exists());
}

#[test]
fn test_cleanup_socket_nonexistent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("nonexistent.sock");
    cleanup_socket_at(&sock_path);
}

// ==================== Capability symlink tests ====================

#[test]
fn test_create_capability_symlink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);

    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink(), "symlink should exist");
    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(
        target,
        std::path::PathBuf::from("sweetgrass.sock"),
        "symlink should be relative"
    );
}

#[test]
fn test_create_capability_symlink_with_family() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass-alpha.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);

    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());
    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(target, std::path::PathBuf::from("sweetgrass-alpha.sock"));
}

#[test]
fn test_create_capability_symlink_replaces_stale() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    let symlink_path = dir.path().join("provenance.sock");
    std::os::unix::fs::symlink("old-target.sock", &symlink_path).expect("create stale");

    create_capability_symlink(&sock_path);

    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(target, std::path::PathBuf::from("sweetgrass.sock"));
}

#[test]
fn test_cleanup_capability_symlink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);
    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());

    cleanup_capability_symlink(&sock_path);
    assert!(!symlink_path.exists());
    assert!(!symlink_path.is_symlink());
}

#[test]
fn test_cleanup_socket_at_removes_symlink_too() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);
    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());
    assert!(sock_path.exists());

    cleanup_socket_at(&sock_path);
    assert!(!sock_path.exists());
    assert!(!symlink_path.exists());
}

#[test]
fn test_cleanup_capability_symlink_nonexistent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    cleanup_capability_symlink(&sock_path);
}
