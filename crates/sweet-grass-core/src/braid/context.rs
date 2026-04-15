// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! JSON-LD context for Braid semantic interpretation.
//!
//! Provides the `BraidContext` type and associated vocabulary URI resolution.
//! Vocabulary URIs are resolved from environment (DI-friendly) with safe
//! defaults for development and testing.

use indexmap::IndexMap;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

/// W3C PROV-O vocabulary namespace.
pub const PROV_VOCAB_URI: &str = "http://www.w3.org/ns/prov#";
/// W3C XML Schema namespace.
pub const XSD_VOCAB_URI: &str = "http://www.w3.org/2001/XMLSchema#";
/// W3C RDF Schema namespace.
pub const RDFS_VOCAB_URI: &str = "http://www.w3.org/2000/01/rdf-schema#";
/// Schema.org namespace.
pub const SCHEMA_VOCAB_URI: &str = "http://schema.org/";

/// Default ecoPrimals vocabulary namespace.
///
/// In production, override via `ECOP_VOCAB_URI` env var or capability-based
/// discovery. These defaults are safe for development and testing.
pub const DEFAULT_ECOP_VOCAB_URI: &str = "https://ecoprimals.io/vocab#";
/// Default ecoPrimals base URI.
///
/// In production, override via `ECOP_BASE_URI` env var or capability-based
/// discovery.
pub const DEFAULT_ECOP_BASE_URI: &str = "https://ecoprimals.io/";

/// Resolve the ecoPrimals vocabulary URI from environment or default.
#[must_use]
pub fn ecop_vocab_uri() -> String {
    ecop_vocab_uri_with_reader(|key| std::env::var(key).ok())
}

/// DI-friendly vocabulary URI resolution.
#[must_use]
pub fn ecop_vocab_uri_with_reader(reader: impl Fn(&str) -> Option<String>) -> String {
    reader("ECOP_VOCAB_URI").unwrap_or_else(|| DEFAULT_ECOP_VOCAB_URI.to_string())
}

/// Resolve the ecoPrimals base URI from environment or default.
#[must_use]
pub fn ecop_base_uri() -> String {
    ecop_base_uri_with_reader(|key| std::env::var(key).ok())
}

/// DI-friendly base URI resolution.
#[must_use]
pub fn ecop_base_uri_with_reader(reader: impl Fn(&str) -> Option<String>) -> String {
    reader("ECOP_BASE_URI").unwrap_or_else(|| DEFAULT_ECOP_BASE_URI.to_string())
}

/// JSON-LD context version — always 1.1 per W3C specification.
///
/// Avoids float representation issues by using a dedicated type that
/// serializes to the JSON number `1.1` and validates on deserialization.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonLdVersion;

impl Serialize for JsonLdVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(1.1)
    }
}

impl<'de> Deserialize<'de> for JsonLdVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f64::deserialize(deserializer)?;
        if (v - 1.1).abs() < 0.1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported JSON-LD version: {v} (expected 1.1)"
            )))
        }
    }
}

/// JSON-LD context for semantic interpretation.
///
/// Uses [`IndexMap`] for vocabulary imports to guarantee deterministic
/// serialization order — important for content-addressed hashing and
/// reproducible JSON-LD output.
///
/// **Serialization note**: `#[serde(flatten)]` is incompatible with binary
/// codecs such as bincode (tarpc). For human-readable formats (JSON), imports
/// remain flattened into the root object. For non-human-readable serializers,
/// `imports` is a normal nested field so lengths are known on the wire.
#[derive(Clone, Debug)]
pub struct BraidContext {
    /// Base context URL.
    pub base: String,

    /// JSON-LD version (always 1.1).
    pub version: JsonLdVersion,

    /// Vocabulary imports (insertion-ordered for deterministic serialization).
    pub imports: IndexMap<String, String>,
}

#[derive(Serialize)]
struct BraidContextSerBin<'a> {
    #[serde(rename = "@base")]
    base: &'a str,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    imports: &'a IndexMap<String, String>,
}

#[derive(Deserialize)]
struct BraidContextDeBin {
    #[serde(rename = "@base")]
    base: String,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    imports: IndexMap<String, String>,
}

#[derive(Deserialize)]
struct BraidContextFlat {
    #[serde(rename = "@base")]
    base: String,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    #[serde(flatten)]
    imports: IndexMap<String, String>,
}

impl Serialize for BraidContext {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_human_readable() {
            return BraidContextSerBin {
                base: &self.base,
                version: self.version,
                imports: &self.imports,
            }
            .serialize(serializer);
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("@base", &self.base)?;
        map.serialize_entry("@version", &self.version)?;
        for (k, v) in &self.imports {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for BraidContext {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if !deserializer.is_human_readable() {
            let b = BraidContextDeBin::deserialize(deserializer)?;
            return Ok(Self {
                base: b.base,
                version: b.version,
                imports: b.imports,
            });
        }

        let f = BraidContextFlat::deserialize(deserializer)?;
        Ok(Self {
            base: f.base,
            version: f.version,
            imports: f.imports,
        })
    }
}

impl Default for BraidContext {
    fn default() -> Self {
        let mut imports = IndexMap::new();
        imports.insert("prov".to_string(), PROV_VOCAB_URI.to_string());
        imports.insert("xsd".to_string(), XSD_VOCAB_URI.to_string());
        imports.insert("schema".to_string(), SCHEMA_VOCAB_URI.to_string());
        imports.insert("ecop".to_string(), ecop_vocab_uri());

        Self {
            base: ecop_base_uri(),
            version: JsonLdVersion,
            imports,
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn braid_context_bincode_roundtrip() {
        let ctx = BraidContext::default();
        let bytes = bincode::serialize(&ctx).expect("serialize");
        let decoded: BraidContext = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded.base, ctx.base);
        assert_eq!(decoded.imports.len(), ctx.imports.len());
    }

    #[test]
    fn json_ld_version_rejects_bad_version() {
        let result: Result<JsonLdVersion, _> = serde_json::from_str("2.0");
        let err = result
            .expect_err("2.0 must not deserialize as JSON-LD 1.1")
            .to_string();
        assert!(err.contains("unsupported JSON-LD version"), "{err}");
    }

    #[test]
    fn ecop_vocab_uri_default() {
        temp_env::with_vars([("ECOP_VOCAB_URI", None::<&str>)], || {
            assert_eq!(ecop_vocab_uri(), DEFAULT_ECOP_VOCAB_URI);
        });
    }

    #[test]
    fn ecop_vocab_uri_env_override() {
        temp_env::with_vars(
            [("ECOP_VOCAB_URI", Some("https://custom.io/vocab#"))],
            || {
                assert_eq!(ecop_vocab_uri(), "https://custom.io/vocab#");
            },
        );
    }

    #[test]
    fn ecop_base_uri_default() {
        temp_env::with_vars([("ECOP_BASE_URI", None::<&str>)], || {
            assert_eq!(ecop_base_uri(), DEFAULT_ECOP_BASE_URI);
        });
    }

    #[test]
    fn ecop_vocab_uri_with_reader_custom() {
        let uri = ecop_vocab_uri_with_reader(|key| {
            (key == "ECOP_VOCAB_URI").then(|| "https://test.io/vocab#".to_string())
        });
        assert_eq!(uri, "https://test.io/vocab#");
    }

    #[test]
    fn ecop_base_uri_with_reader_custom() {
        let uri = ecop_base_uri_with_reader(|key| {
            (key == "ECOP_BASE_URI").then(|| "https://test.io/".to_string())
        });
        assert_eq!(uri, "https://test.io/");
    }

    #[test]
    fn ecop_vocab_uri_with_reader_fallback() {
        let uri = ecop_vocab_uri_with_reader(|_| None);
        assert_eq!(uri, DEFAULT_ECOP_VOCAB_URI);
    }

    #[test]
    fn ecop_base_uri_with_reader_fallback() {
        let uri = ecop_base_uri_with_reader(|_| None);
        assert_eq!(uri, DEFAULT_ECOP_BASE_URI);
    }
}
