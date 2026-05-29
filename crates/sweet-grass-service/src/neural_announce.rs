// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Neural API `primal.announce` — self-registration with biomeOS.
//!
//! On startup (after UDS socket is listening), sweetGrass sends a
//! `primal.announce` JSON-RPC call to biomeOS's neural-api socket.
//! This registers capabilities, cost hints, and latency estimates so
//! the Neural API routing layer can route `capability.call` dispatches
//! through sweetGrass for provenance/attribution/braid operations.
//!
//! Gracefully degrades: if biomeOS is unavailable, sweetGrass continues
//! in standalone mode. Re-announcement happens on version upgrade.
//!
//! Wire schema per `WAVE42_NEURAL_API_DEPLOYMENT_GUIDE.md`.

use std::path::PathBuf;

use tracing::{debug, info, warn};

use sweet_grass_core::niche;

/// Default family for neural-api socket resolution.
const DEFAULT_FAMILY: &str = "ecoPrimal";

/// Resolve the biomeOS neural-api socket path via tiered lookup.
///
/// 1. `$NEURAL_API_SOCKET` — explicit override
/// 2. `$BIOMEOS_SOCKET_DIR/neural-api-{family}.sock`
/// 3. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family}.sock`
/// 4. `{temp_dir}/biomeos/neural-api-{family}.sock`
fn resolve_neural_api_socket() -> Option<PathBuf> {
    resolve_neural_api_socket_with(&|key| std::env::var(key).ok())
}

/// DI-friendly variant for testing.
fn resolve_neural_api_socket_with(reader: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    if let Some(explicit) = reader("NEURAL_API_SOCKET") {
        let path = PathBuf::from(&explicit);
        if path.exists() {
            return Some(path);
        }
        debug!("NEURAL_API_SOCKET={explicit} does not exist");
    }

    let family = reader("ECOPRIMALS_FAMILY_ID")
        .or_else(|| reader("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|| DEFAULT_FAMILY.to_string());

    let socket_name = format!("neural-api-{family}.sock");

    if let Some(dir) = reader("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(dir).join(&socket_name);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(xdg) = reader("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(xdg).join("biomeos").join(&socket_name);
        if path.exists() {
            return Some(path);
        }
    }

    let fallback = std::env::temp_dir().join("biomeos").join(&socket_name);
    if fallback.exists() {
        return Some(fallback);
    }

    None
}

/// Build the `primal.announce` JSON-RPC payload.
///
/// Includes all registered methods, capability domains, signal tier,
/// cost hints, and latency estimates per Wave 43 blurb.
fn build_announce_payload(socket_path: &str, version: &str) -> serde_json::Value {
    let methods: Vec<&str> = niche::CAPABILITIES.to_vec();

    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": {
            "primal": niche::NICHE_ID,
            "socket": socket_path,
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
/// Called after the UDS socket is bound. Resolves the neural-api socket,
/// builds the payload, and sends a single JSON-RPC request. Gracefully
/// degrades if biomeOS is unavailable.
pub async fn announce_to_neural_api(own_socket_path: &str, version: &str) {
    let Some(neural_socket) = resolve_neural_api_socket() else {
        debug!(
            "Neural API socket not found — skipping primal.announce \
             (biomeOS not running or no NEURAL_API_SOCKET set)"
        );
        return;
    };

    let payload = build_announce_payload(own_socket_path, version);

    match send_jsonrpc_uds(&neural_socket, &payload).await {
        Ok(response) => {
            if let Some(result) = response.get("result") {
                info!(
                    neural_socket = %neural_socket.display(),
                    capabilities_registered = ?result.get("capabilities_registered"),
                    methods_registered = ?result.get("methods_registered"),
                    "primal.announce: registered with Neural API"
                );
            } else if let Some(error) = response.get("error") {
                warn!(
                    neural_socket = %neural_socket.display(),
                    error = %error,
                    "primal.announce: Neural API returned error (non-fatal)"
                );
            }
        },
        Err(e) => {
            debug!(
                neural_socket = %neural_socket.display(),
                error = %e,
                "primal.announce: failed to reach Neural API — standalone mode"
            );
        },
    }
}

/// Send a JSON-RPC request over UDS and read the response.
async fn send_jsonrpc_uds(
    socket_path: &std::path::Path,
    request: &serde_json::Value,
) -> Result<serde_json::Value, std::io::Error> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();

    let mut payload = serde_json::to_string(request).map_err(std::io::Error::other)?;
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    let bytes_read = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        buf_reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "neural-api response timeout"))?
    ?;

    if bytes_read == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "neural-api closed connection without response",
        ));
    }

    serde_json::from_str(response_line.trim()).map_err(std::io::Error::other)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn test_build_announce_payload_structure() {
        let payload = build_announce_payload("/tmp/biomeos/sweetgrass.sock", "0.7.37");
        let params = payload.get("params").unwrap();

        assert_eq!(params["primal"], "sweetgrass");
        assert_eq!(params["socket"], "/tmp/biomeos/sweetgrass.sock");
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
    fn test_build_announce_payload_pid() {
        let payload = build_announce_payload("/tmp/test.sock", "0.7.37");
        let pid = payload["params"]["pid"].as_u64().unwrap();
        assert_eq!(pid, u64::from(std::process::id()));
    }

    #[test]
    fn test_resolve_neural_api_socket_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("neural-test.sock");
        std::fs::write(&sock, "").unwrap();
        let sock_str = sock.to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            if key == "NEURAL_API_SOCKET" {
                Some(sock_str.clone())
            } else {
                None
            }
        };

        let result = resolve_neural_api_socket_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), sock);
    }

    #[test]
    fn test_resolve_neural_api_socket_xdg() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir(&biomeos_dir).unwrap();
        let sock = biomeos_dir.join("neural-api-testFamily.sock");
        std::fs::write(&sock, "").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();

        let reader = move |key: &str| -> Option<String> {
            match key {
                "XDG_RUNTIME_DIR" => Some(xdg.clone()),
                "ECOPRIMALS_FAMILY_ID" => Some("testFamily".to_string()),
                _ => None,
            }
        };

        let result = resolve_neural_api_socket_with(&reader);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), sock);
    }

    #[test]
    fn test_resolve_neural_api_socket_none_when_missing() {
        let reader = |_: &str| -> Option<String> { None };
        let result = resolve_neural_api_socket_with(&reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_announce_payload_method_count() {
        let payload = build_announce_payload("/tmp/test.sock", "0.7.37");
        let methods = payload["params"]["methods"].as_array().unwrap();
        assert_eq!(
            methods.len(),
            niche::CAPABILITIES.len(),
            "should include all registered capabilities"
        );
    }

    #[tokio::test]
    async fn test_announce_to_neural_api_graceful_when_no_socket() {
        announce_to_neural_api("/tmp/nonexistent.sock", "0.7.37").await;
    }
}
