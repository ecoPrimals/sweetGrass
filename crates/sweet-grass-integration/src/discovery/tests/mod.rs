// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

pub use super::*;
pub use std::sync::Arc;
pub use std::time::Duration;

pub fn make_test_primal(name: &str, capabilities: Vec<Capability>) -> DiscoveredPrimal {
    // Use OS-allocated ports for test primals
    let [tarpc_port, rest_port] = crate::testing::allocate_test_ports::<2>();

    DiscoveredPrimal {
        instance_id: format!("{name}-instance"),
        name: name.to_string(),
        capabilities,
        tarpc_address: Some(format!("localhost:{tarpc_port}")),
        rest_address: Some(format!("localhost:{rest_port}")),
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    }
}

mod cached_discovery;
mod discovered_primal;
#[path = "extract_capabilities.rs"]
mod extract_capability_tests;
mod local_discovery;
mod registry;
mod service_info;
