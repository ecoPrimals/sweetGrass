// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

#[path = "tests/env.rs"]
mod env;
#[path = "tests/guard.rs"]
mod guard;
#[path = "tests/resolution.rs"]
mod resolution;
#[path = "tests/roundtrip.rs"]
mod roundtrip;
#[path = "tests/symlink.rs"]
mod symlink;
