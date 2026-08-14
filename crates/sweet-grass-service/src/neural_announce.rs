// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Neural API `primal.announce` — self-registration with biomeOS.
//!
//! On startup (after transport is listening), sweetGrass sends a
//! `primal.announce` JSON-RPC call to biomeOS's neural-api endpoint.
//! This registers capabilities, cost hints, and latency estimates so
//! the Neural API routing layer can route `capability.call` dispatches
//! through sweetGrass for provenance/attribution/braid operations.
//!
//! Transport-agnostic: resolves via [`TransportEndpoint`] — UDS on Unix,
//! TCP if configured via `NEURAL_API_ENDPOINT` env var. Gracefully
//! degrades if biomeOS is unavailable.
//!
//! Wire schema per `WAVE42_NEURAL_API_DEPLOYMENT_GUIDE.md`.

use std::path::PathBuf;
use std::time::Duration;

use sweet_grass_core::niche;
use sweet_grass_core::primal_names::env_vars;
use sweet_grass_core::transport::TransportEndpoint;
use tracing::{debug, info, warn};

use crate::transport_connect::send_jsonrpc;

/// Default family for neural-api socket resolution.
const DEFAULT_FAMILY: &str = "ecoPrimal";

/// Timeout for neural-api announce call (longer than standard probe).
///
/// Override via `SWEETGRASS_ANNOUNCE_TIMEOUT_MS` for tuning without recompilation.
fn announce_timeout() -> Duration {
    std::env::var("SWEETGRASS_ANNOUNCE_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map_or(Duration::from_secs(5), Duration::from_millis)
}

/// Resolve the biomeOS neural-api transport endpoint.
///
/// Resolution order:
/// 1. `$NEURAL_API_ENDPOINT` — explicit JSON endpoint (TCP, UDS, etc.)
/// 2. `$NEURAL_API_SOCKET` — explicit UDS path
/// 3. `$BIOMEOS_SOCKET_DIR/neural-api-{family}.sock`
/// 4. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
/// 5. `{temp_dir}/biomeos/neural-api-{family}.sock`
fn resolve_neural_api_endpoint() -> Option<TransportEndpoint> {
    resolve_neural_api_endpoint_with(&|key| std::env::var(key).ok())
}

/// DI-friendly variant for testing.
fn resolve_neural_api_endpoint_with(
    reader: &dyn Fn(&str) -> Option<String>,
) -> Option<TransportEndpoint> {
    if let Some(json) = reader("NEURAL_API_ENDPOINT") {
        if let Ok(ep) = sweet_grass_core::transport::parse_transport_endpoint(&json) {
            return Some(ep);
        }
        debug!("NEURAL_API_ENDPOINT={json} is not valid TransportEndpoint JSON");
    }

    if let Some(explicit) = reader(env_vars::NEURAL_API_SOCKET) {
        let path = PathBuf::from(&explicit);
        if path.exists() {
            return Some(TransportEndpoint::uds(explicit));
        }
        debug!("NEURAL_API_SOCKET={explicit} does not exist");
    }

    let family = reader(env_vars::ECOPRIMALS_FAMILY_ID)
        .or_else(|| reader(env_vars::BIOMEOS_FAMILY_ID))
        .unwrap_or_else(|| DEFAULT_FAMILY.to_string());

    let socket_name = format!("neural-api-{family}.sock");

    if let Some(dir) = reader(env_vars::BIOMEOS_SOCKET_DIR) {
        let path = PathBuf::from(&dir).join(&socket_name);
        if path.exists() {
            return Some(TransportEndpoint::uds(path.to_string_lossy()));
        }
    }

    if let Some(xdg) = reader(env_vars::XDG_RUNTIME_DIR) {
        let path = PathBuf::from(xdg)
            .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
            .join(&socket_name);
        if path.exists() {
            return Some(TransportEndpoint::uds(path.to_string_lossy()));
        }
    }

    let fallback = std::env::temp_dir()
        .join(sweet_grass_core::primal_names::paths::BIOMEOS_DIR)
        .join(&socket_name);
    if fallback.exists() {
        return Some(TransportEndpoint::uds(fallback.to_string_lossy()));
    }

    None
}

/// Build the `primal.announce` JSON-RPC payload.
///
/// Includes all registered methods, capability domains, signal tier,
/// cost hints, and latency estimates per Wave 43 blurb.
fn build_announce_payload(own_endpoint: &TransportEndpoint, version: &str) -> serde_json::Value {
    let methods: Vec<&str> = niche::CAPABILITIES.to_vec();

    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": {
            "primal": niche::NICHE_ID,
            "endpoint": own_endpoint,
            "pid": std::process::id(),
            "capabilities": ["provenance", "attribution", "braid"],
            "methods": methods,
            "signal_tiers": ["nest"],
            "cost_hints": {
                "provenance": 10.0,
                "attribution": 8.0,
                "braid": 12.0
            },
            "latency_estimates": {
                "provenance": 15,
                "attribution": 10,
                "braid": 20
            },
            "version": version,
            "attestation": null
        },
        "id": 1
    })
}

/// Send `primal.announce` to biomeOS neural-api on startup.
///
/// Called after the transport is listening. Resolves the neural-api endpoint,
/// builds the payload, and sends a single JSON-RPC request. Gracefully
/// degrades if biomeOS is unavailable.
pub async fn announce_to_neural_api(own_endpoint: &TransportEndpoint, version: &str) {
    let Some(neural_endpoint) = resolve_neural_api_endpoint() else {
        debug!(
            "Neural API endpoint not found — skipping primal.announce \
             (biomeOS not running or no NEURAL_API_ENDPOINT/NEURAL_API_SOCKET set)"
        );
        return;
    };

    let payload = build_announce_payload(own_endpoint, version);

    match send_jsonrpc(&neural_endpoint, &payload, announce_timeout()).await {
        Ok(response) => {
            if let Some(result) = response.get("result") {
                info!(
                    neural_endpoint = %neural_endpoint,
                    capabilities_registered = ?result.get("capabilities_registered"),
                    methods_registered = ?result.get("methods_registered"),
                    "primal.announce: registered with Neural API"
                );
            } else if let Some(error) = response.get("error") {
                warn!(
                    neural_endpoint = %neural_endpoint,
                    error = %error,
                    "primal.announce: Neural API returned error (non-fatal)"
                );
            }
        },
        Err(e) => {
            debug!(
                neural_endpoint = %neural_endpoint,
                error = %e,
                "primal.announce: failed to reach Neural API — standalone mode"
            );
        },
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn test_build_announce_payload_structure() {
        let ep = TransportEndpoint::uds("/tmp/biomeos/sweetgrass.sock");
        let payload = build_announce_payload(&ep, "0.7.37");
        let params = payload.get("params").unwrap();

        assert_eq!(params["primal"], "sweetgrass");
        assert_eq!(params["endpoint"]["transport"], "uds");
        assert_eq!(params["endpoint"]["path"], "/tmp/biomeos/sweetgrass.sock");
        assert_eq!(params["version"], "0.7.37");

        let caps = params["capabilities"].as_array().unwrap();
        assert_eq!(caps.len(), 3);
        assert!(caps.contains(&serde_json::json!("provenance")));
        assert!(caps.contains(&serde_json::json!("attribution")));
        assert!(caps.contains(&serde_json::json!("braid")));

        let methods = params["methods"].as_array().unwrap();
        assert_eq!(methods.len(), niche::CAPABILITIES.len());
        assert!(methods.contains(&serde_json::json!("braid.create")));
        assert!(methods.contains(&serde_json::json!("attribution.witness")));

        let tiers = params["signal_tiers"].as_array().unwrap();
        assert_eq!(tiers, &[serde_json::json!("nest")]);

        let cost = params["cost_hints"].as_object().unwrap();
        assert!(cost.contains_key("provenance"));
        assert!(cost.contains_key("attribution"));
        assert!(cost.contains_key("braid"));

        let latency = params["latency_estimates"].as_object().unwrap();
        assert!(latency.contains_key("provenance"));
        assert!(latency.contains_key("attribution"));
        assert!(latency.contains_key("braid"));

        assert_eq!(payload["method"], "primal.announce");
        assert_eq!(payload["jsonrpc"], "2.0");
    }

    #[test]
    fn test_build_announce_payload_tcp_endpoint() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 9100);
        let payload = build_announce_payload(&ep, "0.7.61");
        let params = payload.get("params").unwrap();

        assert_eq!(params["endpoint"]["transport"], "tcp");
        assert_eq!(params["endpoint"]["host"], "127.0.0.1");
        assert_eq!(params["endpoint"]["port"], 9100);
    }

    #[test]
    fn test_build_announce_payload_pid() {
        let ep = TransportEndpoint::uds("/tmp/test.sock");
        let payload = build_announce_payload(&ep, "0.7.37");
        let pid = payload["params"]["pid"].as_u64().unwrap();
        assert_eq!(pid, u64::from(std::process::id()));
    }

    #[test]
    fn test_resolve_neural_api_endpoint_explicit_json() {
        let reader = |key: &str| -> Option<String> {
            if key == "NEURAL_API_ENDPOINT" {
                Some(r#"{"transport":"tcp","host":"10.0.0.1","port":7800}"#.to_string())
            } else {
                None
            }
        };
        let result = resolve_neural_api_endpoint_with(&reader);
        assert_eq!(result, Some(TransportEndpoint::tcp("10.0.0.1", 7800)));
    }

    #[test]
    fn test_resolve_neural_api_endpoint_explicit_socket() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("neural-test.sock");
        std::fs::write(&sock, "").unwrap();
        let sock_str = sock.to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            if key == env_vars::NEURAL_API_SOCKET {
                Some(sock_str.clone())
            } else {
                None
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap().transport_name(), "uds");
    }

    #[test]
    fn test_resolve_neural_api_endpoint_xdg() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("neural-api-testFamily.sock");
        std::fs::write(&sock, "").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                env_vars::XDG_RUNTIME_DIR => Some(xdg.clone()),
                env_vars::ECOPRIMALS_FAMILY_ID => Some("testFamily".to_string()),
                _ => None,
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap().transport_name(), "uds");
    }

    #[test]
    fn test_resolve_neural_api_endpoint_none_when_missing() {
        let reader = |_: &str| -> Option<String> { None };
        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_announce_payload_method_count() {
        let ep = TransportEndpoint::uds("/tmp/test.sock");
        let payload = build_announce_payload(&ep, "0.7.37");
        let methods = payload["params"]["methods"].as_array().unwrap();
        assert_eq!(
            methods.len(),
            niche::CAPABILITIES.len(),
            "should include all registered capabilities"
        );
    }

    #[tokio::test]
    async fn test_announce_to_neural_api_graceful_when_no_endpoint() {
        let ep = TransportEndpoint::uds("/tmp/nonexistent.sock");
        announce_to_neural_api(&ep, "0.7.37").await;
    }

    #[test]
    fn test_resolve_neural_api_endpoint_biomeos_socket_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("neural-api-myFamily.sock");
        std::fs::write(&sock, "").unwrap();
        let dir_str = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                env_vars::BIOMEOS_SOCKET_DIR => Some(dir_str.clone()),
                env_vars::ECOPRIMALS_FAMILY_ID => Some("myFamily".to_string()),
                _ => None,
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap().transport_name(), "uds");
    }

    #[test]
    fn test_resolve_neural_api_endpoint_explicit_missing_falls_through_to_xdg() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("neural-api-ecoPrimal.sock");
        std::fs::write(&sock, "").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                env_vars::NEURAL_API_SOCKET => Some("/tmp/nonexistent-fake.sock".to_string()),
                env_vars::XDG_RUNTIME_DIR => Some(xdg.clone()),
                _ => None,
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap().transport_name(), "uds");
    }

    #[test]
    fn test_resolve_neural_api_endpoint_biomeos_family_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("neural-api-altFamily.sock");
        std::fs::write(&sock, "").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                env_vars::BIOMEOS_FAMILY_ID => Some("altFamily".to_string()),
                env_vars::XDG_RUNTIME_DIR => Some(xdg.clone()),
                _ => None,
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
    }

    #[test]
    fn test_resolve_neural_api_endpoint_default_family() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("neural-api-ecoPrimal.sock");
        std::fs::write(&sock, "").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                env_vars::XDG_RUNTIME_DIR => Some(xdg.clone()),
                _ => None,
            }
        };

        let result = resolve_neural_api_endpoint_with(&reader);
        assert!(result.is_some());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_send_jsonrpc_via_transport_uds_mock() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("mock-neural.sock");

        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
        let sock_str = sock_path.to_string_lossy().to_string();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();
            let request_line = lines.next_line().await.unwrap().unwrap();
            let request: serde_json::Value = serde_json::from_str(&request_line).unwrap();
            assert_eq!(request["method"], "primal.announce");

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"capabilities_registered": 3, "methods_registered": 40},
                "id": 1
            });
            let mut resp_str = serde_json::to_string(&response).unwrap();
            resp_str.push('\n');
            writer.write_all(resp_str.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        });

        let endpoint = TransportEndpoint::uds(&sock_str);
        let ep = TransportEndpoint::uds("/tmp/test.sock");
        let payload = build_announce_payload(&ep, "0.7.59");
        let result = send_jsonrpc(&endpoint, &payload, announce_timeout()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["result"]["capabilities_registered"], 3);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_send_jsonrpc_via_transport_tcp_mock() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = tokio::io::split(stream);
            let mut buf = BufReader::new(reader);
            let mut line = String::new();
            buf.read_line(&mut line).await.unwrap();
            let request: serde_json::Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["method"], "primal.announce");

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"capabilities_registered": 3},
                "id": 1
            });
            let mut resp_str = serde_json::to_string(&response).unwrap();
            resp_str.push('\n');
            writer.write_all(resp_str.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        });

        let endpoint = TransportEndpoint::tcp("127.0.0.1", port);
        let ep = TransportEndpoint::tcp("127.0.0.1", 9100);
        let payload = build_announce_payload(&ep, "0.7.61");
        let result = send_jsonrpc(&endpoint, &payload, announce_timeout()).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["result"]["capabilities_registered"], 3);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn test_send_jsonrpc_connection_refused() {
        let endpoint = TransportEndpoint::tcp("127.0.0.1", 1);
        let payload = serde_json::json!({"jsonrpc": "2.0", "method": "test", "id": 1});
        let result = send_jsonrpc(&endpoint, &payload, announce_timeout()).await;
        assert!(result.is_err());
    }
}
