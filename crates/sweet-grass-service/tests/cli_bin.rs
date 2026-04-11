// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! CLI integration tests for the `sweetgrass` binary.
//!
//! Exercises the non-server entry points (capabilities, socket) and
//! verifies that the Clap-derived CLI parsing rejects invalid inputs.

#![expect(clippy::expect_used, reason = "test file: expect is standard in tests")]

use std::process::Command;

fn cargo_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args([
        "run",
        "--quiet",
        "--all-features",
        "-p",
        "sweet-grass-service",
        "--bin",
        "sweetgrass",
        "--",
    ]);
    cmd
}

#[test]
fn cli_capabilities_succeeds() {
    let output = cargo_bin().arg("capabilities").output().expect("spawn");
    assert!(
        output.status.success(),
        "capabilities should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sweetgrass"),
        "should contain primal name: {stdout}"
    );
    assert!(
        stdout.contains("provenance"),
        "should list provenance capability: {stdout}"
    );
}

#[test]
fn cli_socket_succeeds() {
    let output = cargo_bin().arg("socket").output().expect("spawn");
    assert!(
        output.status.success(),
        "socket should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sweetgrass") && stdout.contains(".sock"),
        "should print socket path: {stdout}"
    );
}

#[test]
fn cli_version_succeeds() {
    let output = cargo_bin().arg("--version").output().expect("spawn");
    assert!(
        output.status.success(),
        "--version should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sweetgrass"),
        "should print name+version: {stdout}"
    );
}

#[test]
fn cli_help_succeeds() {
    let output = cargo_bin().arg("--help").output().expect("spawn");
    assert!(
        output.status.success(),
        "--help should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("server"), "should list server subcommand");
    assert!(stdout.contains("status"), "should list status subcommand");
    assert!(
        stdout.contains("capabilities"),
        "should list capabilities subcommand"
    );
    assert!(stdout.contains("socket"), "should list socket subcommand");
}

#[test]
fn cli_invalid_subcommand_fails() {
    let output = cargo_bin().arg("nonexistent").output().expect("spawn");
    assert!(!output.status.success(), "invalid subcommand should fail");
}

#[test]
fn cli_server_help_shows_port_flag() {
    let output = cargo_bin()
        .args(["server", "--help"])
        .output()
        .expect("spawn");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--port"),
        "server help should show --port flag: {stdout}"
    );
    assert!(
        stdout.contains("--http-address"),
        "server help should show --http-address flag: {stdout}"
    );
}
