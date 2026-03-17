// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! NDJSON streaming types for pipeline coordination.
//!
//! Aligned with `rhizoCrypt::streaming::StreamItem` for interoperable
//! NDJSON streaming across the provenance trio pipeline
//! (toadStool → rhizoCrypt → sweetGrass).
//!
//! Each line in the stream is a self-describing JSON object tagged by `type`.

use serde::{Deserialize, Serialize};

/// A single item in an NDJSON stream.
///
/// Uses `#[serde(tag = "type")]` so each JSON line self-identifies:
///
/// ```json
/// {"type":"Data","payload":{"braid_ref":"urn:braid:uuid:123"}}
/// {"type":"Progress","processed":42,"total":100}
/// {"type":"End"}
/// {"type":"Error","message":"timeout","recoverable":true}
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamItem {
    /// A data payload in the stream.
    Data {
        /// The payload content (schema depends on the method).
        payload: serde_json::Value,
    },
    /// Stream progress indicator.
    Progress {
        /// Items processed so far.
        processed: u64,
        /// Total items (if known).
        total: Option<u64>,
    },
    /// End-of-stream marker.
    End,
    /// Stream-level error (non-fatal; stream may continue if `recoverable`).
    Error {
        /// Error message.
        message: String,
        /// Whether the stream can continue after this error.
        recoverable: bool,
    },
}

impl StreamItem {
    /// Create a `Data` item from any serializable value.
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the value cannot be converted to JSON.
    pub fn data(value: &impl Serialize) -> Result<Self, serde_json::Error> {
        Ok(Self::Data {
            payload: serde_json::to_value(value)?,
        })
    }

    /// Create an `End` marker.
    #[must_use]
    pub const fn end() -> Self {
        Self::End
    }

    /// Create a `Progress` indicator.
    #[must_use]
    pub const fn progress(processed: u64, total: Option<u64>) -> Self {
        Self::Progress { processed, total }
    }

    /// Create a recoverable error.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a fatal (non-recoverable) error.
    #[must_use]
    pub fn fatal(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: false,
        }
    }

    /// Serialize to a single NDJSON line (no trailing newline).
    ///
    /// # Errors
    ///
    /// Returns a serialization error if the item cannot be serialized.
    pub fn to_ndjson_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Whether this is an `End` marker.
    #[must_use]
    pub const fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }

    /// Whether this is a `Data` item.
    #[must_use]
    pub const fn is_data(&self) -> bool {
        matches!(self, Self::Data { .. })
    }

    /// Whether this is a fatal error.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::Error {
                recoverable: false,
                ..
            }
        )
    }
}

/// Parse a single NDJSON line into a `StreamItem`.
///
/// # Errors
///
/// Returns an error if the line is not valid JSON or doesn't match `StreamItem`.
pub fn parse_ndjson_line(line: &str) -> Result<StreamItem, serde_json::Error> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn data_item_roundtrip() {
        let item = StreamItem::data(&serde_json::json!({"braid_ref": "urn:braid:uuid:123"}));
        let item = item.unwrap();
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_data());
        assert!(!parsed.is_end());
    }

    #[test]
    fn end_marker() {
        let item = StreamItem::end();
        let line = item.to_ndjson_line().unwrap();
        assert_eq!(line, r#"{"type":"End"}"#);
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(parsed.is_end());
    }

    #[test]
    fn progress_item() {
        let item = StreamItem::progress(42, Some(100));
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(matches!(
            parsed,
            StreamItem::Progress {
                processed: 42,
                total: Some(100)
            }
        ));
    }

    #[test]
    fn error_recoverable() {
        let item = StreamItem::error("timeout");
        assert!(!item.is_fatal());
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(matches!(
            parsed,
            StreamItem::Error {
                recoverable: true,
                ..
            }
        ));
    }

    #[test]
    fn error_fatal() {
        let item = StreamItem::fatal("connection lost");
        assert!(item.is_fatal());
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(matches!(
            parsed,
            StreamItem::Error {
                recoverable: false,
                ..
            }
        ));
    }

    #[test]
    fn progress_no_total() {
        let item = StreamItem::progress(10, None);
        let line = item.to_ndjson_line().unwrap();
        let parsed = parse_ndjson_line(&line).unwrap();
        assert!(matches!(
            parsed,
            StreamItem::Progress {
                processed: 10,
                total: None
            }
        ));
    }

    #[test]
    fn parse_invalid_json() {
        assert!(parse_ndjson_line("not json").is_err());
    }

    #[test]
    fn parse_with_whitespace() {
        let line = r#"  {"type":"End"}  "#;
        let parsed = parse_ndjson_line(line).unwrap();
        assert!(parsed.is_end());
    }
}
