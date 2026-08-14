// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

mod autodetect;
mod domain;
mod env;
mod guard;
mod resolution;
mod roundtrip;
mod symlink;
