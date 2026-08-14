// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! G65 Protocol Negotiation — single-socket protocol selection.
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc at
//! connection time. Replaces C2 dual-socket as Phase 3 of cephalization.
//!
//! ## Wire Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! [Connection proceeds in selected protocol]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the first bytes are NOT `PROTOCOLS:`, the server routes via the
//! existing riboCipher detection path. Existing clients work unchanged.

use std::fmt;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info};

/// RPC protocol selector for G65 negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum IpcProtocol {
    /// JSON-RPC 2.0 — default, backward-compatible, human-readable.
    #[default]
    JsonRpc,
    /// tarpc — binary, type-safe, high-performance intra-gate composition.
    Tarpc,
}

impl fmt::Display for IpcProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.wire_name())
    }
}

impl IpcProtocol {
    /// Parse from wire name (case-insensitive).
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" => Some(Self::Tarpc),
            _ => None,
        }
    }

    /// Wire-format name for negotiation.
    #[must_use]
    pub const fn wire_name(&self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }

    /// All protocols this build supports (server preference order).
    #[must_use]
    pub fn supported() -> Vec<Self> {
        vec![Self::Tarpc, Self::JsonRpc]
    }
}

/// Select best protocol: first client preference that server also supports.
#[must_use]
pub fn select_protocol(
    client_supported: &[IpcProtocol],
    server_supported: &[IpcProtocol],
) -> IpcProtocol {
    for client_proto in client_supported {
        if server_supported.contains(client_proto) {
            return *client_proto;
        }
    }
    IpcProtocol::JsonRpc
}

/// Parse a `PROTOCOLS:` request line into supported protocols.
///
/// Expected format: `"PROTOCOLS: tarpc,jsonrpc\n"`
#[must_use]
pub fn parse_protocol_request(line: &str) -> Option<Vec<IpcProtocol>> {
    let trimmed = line.trim();
    let body = trimmed.strip_prefix("PROTOCOLS: ")?;
    let mut protocols = Vec::new();
    for name in body.split(',') {
        if let Some(p) = IpcProtocol::from_wire(name.trim()) {
            protocols.push(p);
        }
    }
    if protocols.is_empty() {
        return None;
    }
    Some(protocols)
}

/// Format a `PROTOCOL:` response line.
#[must_use]
pub fn format_protocol_response(selected: IpcProtocol) -> String {
    format!("PROTOCOL: {}\n", selected.wire_name())
}

/// Server-side negotiation: read the remainder of the `PROTOCOLS:` line
/// (first byte `P` already consumed by peek), select protocol, respond.
///
/// Returns the selected protocol. The caller should then route the
/// connection to the appropriate handler (tarpc or JSON-RPC).
///
/// # Errors
///
/// Returns `std::io::Error` on I/O failure or malformed negotiation line.
pub async fn negotiate_server_from_partial<S>(
    stream: &mut S,
    first_byte: u8,
) -> std::io::Result<IpcProtocol>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    line.push(char::from(first_byte));

    reader.read_line(&mut line).await?;

    let client_protocols = parse_protocol_request(&line).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid protocol negotiation line: {line:?}"),
        )
    })?;

    let server_supported = IpcProtocol::supported();
    let selected = select_protocol(&client_protocols, &server_supported);

    debug!(
        ?client_protocols,
        ?selected,
        "G65 protocol negotiation complete"
    );

    let response = format_protocol_response(selected);
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    info!(%selected, "G65 negotiated");
    Ok(selected)
}

/// Client-side negotiation: send supported protocols, read response.
///
/// # Errors
///
/// Returns `std::io::Error` on I/O failure or invalid server response.
pub async fn negotiate_client<S>(
    stream: &mut S,
    preferred: &[IpcProtocol],
) -> std::io::Result<IpcProtocol>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let names: Vec<&str> = preferred.iter().map(IpcProtocol::wire_name).collect();
    let request = format!("PROTOCOLS: {}\n", names.join(","));

    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let trimmed = response_line.trim();
    let proto_name = trimmed.strip_prefix("PROTOCOL: ").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid protocol response: {trimmed:?}"),
        )
    })?;

    IpcProtocol::from_wire(proto_name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unknown protocol: {proto_name:?}"),
        )
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test module")]
mod tests {
    use super::*;

    #[test]
    fn protocol_wire_names() {
        assert_eq!(IpcProtocol::JsonRpc.wire_name(), "jsonrpc");
        assert_eq!(IpcProtocol::Tarpc.wire_name(), "tarpc");
    }

    #[test]
    fn protocol_from_wire() {
        assert_eq!(
            IpcProtocol::from_wire("jsonrpc"),
            Some(IpcProtocol::JsonRpc)
        );
        assert_eq!(
            IpcProtocol::from_wire("json-rpc"),
            Some(IpcProtocol::JsonRpc)
        );
        assert_eq!(IpcProtocol::from_wire("TARPC"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::from_wire("tarpc"), Some(IpcProtocol::Tarpc));
        assert_eq!(IpcProtocol::from_wire("unknown"), None);
    }

    #[test]
    fn protocol_display() {
        assert_eq!(IpcProtocol::JsonRpc.to_string(), "jsonrpc");
        assert_eq!(IpcProtocol::Tarpc.to_string(), "tarpc");
    }

    #[test]
    fn select_protocol_client_preference_wins() {
        let client = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::Tarpc);
    }

    #[test]
    fn select_protocol_server_only_jsonrpc() {
        let client = &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc];
        let server = &[IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn select_protocol_no_common_falls_back() {
        let client = &[IpcProtocol::Tarpc];
        let server = &[IpcProtocol::JsonRpc];
        assert_eq!(select_protocol(client, server), IpcProtocol::JsonRpc);
    }

    #[test]
    fn parse_request_line() {
        let protos = parse_protocol_request("PROTOCOLS: tarpc,jsonrpc\n").unwrap();
        assert_eq!(protos, vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc]);
    }

    #[test]
    fn parse_request_single() {
        let protos = parse_protocol_request("PROTOCOLS: jsonrpc\n").unwrap();
        assert_eq!(protos, vec![IpcProtocol::JsonRpc]);
    }

    #[test]
    fn parse_request_invalid_prefix() {
        assert!(parse_protocol_request("NOTPROTOCOLS: jsonrpc\n").is_none());
    }

    #[test]
    fn parse_request_no_valid_protocols() {
        assert!(parse_protocol_request("PROTOCOLS: unknown\n").is_none());
    }

    #[test]
    fn format_response() {
        assert_eq!(
            format_protocol_response(IpcProtocol::Tarpc),
            "PROTOCOL: tarpc\n"
        );
        assert_eq!(
            format_protocol_response(IpcProtocol::JsonRpc),
            "PROTOCOL: jsonrpc\n"
        );
    }

    #[tokio::test]
    async fn negotiate_client_server_roundtrip() {
        let (client_stream, mut server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(&mut server_stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let client_protos = parse_protocol_request(&line).unwrap();
            let selected = select_protocol(&client_protos, &IpcProtocol::supported());
            let response = format_protocol_response(selected);
            server_stream.write_all(response.as_bytes()).await.unwrap();
            server_stream.flush().await.unwrap();
            selected
        });

        let mut client = client_stream;
        let client_result =
            negotiate_client(&mut client, &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc])
                .await
                .unwrap();

        let server_result = server_handle.await.unwrap();

        assert_eq!(client_result, IpcProtocol::Tarpc);
        assert_eq!(server_result, IpcProtocol::Tarpc);
    }

    #[tokio::test]
    async fn negotiate_server_from_partial_selects_tarpc() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(4096);

        let server_handle =
            tokio::spawn(
                async move { negotiate_server_from_partial(&mut server_stream, b'P').await },
            );

        client_stream
            .write_all(b"ROTOCOLS: tarpc,jsonrpc\n")
            .await
            .unwrap();
        client_stream.flush().await.unwrap();

        let mut response = String::new();
        let mut reader = BufReader::new(&mut client_stream);
        reader.read_line(&mut response).await.unwrap();
        assert_eq!(response.trim(), "PROTOCOL: tarpc");

        let selected = server_handle.await.unwrap().unwrap();
        assert_eq!(selected, IpcProtocol::Tarpc);
    }

    #[tokio::test]
    async fn negotiate_server_from_partial_jsonrpc_only() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(4096);

        let server_handle =
            tokio::spawn(
                async move { negotiate_server_from_partial(&mut server_stream, b'P').await },
            );

        client_stream
            .write_all(b"ROTOCOLS: jsonrpc\n")
            .await
            .unwrap();
        client_stream.flush().await.unwrap();

        let mut response = String::new();
        let mut reader = BufReader::new(&mut client_stream);
        reader.read_line(&mut response).await.unwrap();
        assert_eq!(response.trim(), "PROTOCOL: jsonrpc");

        let selected = server_handle.await.unwrap().unwrap();
        assert_eq!(selected, IpcProtocol::JsonRpc);
    }
}
